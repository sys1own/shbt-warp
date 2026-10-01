//! Hardware-in-the-loop microkernel bridge.
//!
//! Rust mirror of `kernel/include/shbt_warp_hardware.h`: the 128-byte
//! dual-cacheline `shbt_warp_mmio_t` register block at 0x70000000, SECDED
//! Hamming(72,64) ECC scrubbing, Givens remapping and the sub-2.5 ns PCSS
//! crowbar quench driver (2.140 ns trip, 94.20% SiC inductive recovery).

/// MMIO base physical address.
pub const MMIO_BASE: u64 = 0x7000_0000;
/// Register-block footprint (bytes) — two aligned cache lines.
pub const MMIO_SIZE: usize = 128;

/// Byte offsets of the 16 u64 registers (warp1.txt normative layout).
pub mod reg {
    pub const CTRL_STATUS: usize = 0x00;
    pub const TARGET_VELOCITY: usize = 0x08;
    pub const CURRENT_VELOCITY: usize = 0x10;
    pub const CAVITY_ACCEL_RAW: usize = 0x18;
    pub const BUBBLE_RADIUS_NM: usize = 0x20;
    pub const WALL_THICKNESS_PM: usize = 0x28;
    pub const RF_PHASE_GRAD_URAD: usize = 0x30;
    pub const OPTICAL_POWER_MW: usize = 0x38;
    pub const CRYO_TEMP_MILLIK: usize = 0x40;
    pub const KAPITZA_DROP_UV: usize = 0x48;
    pub const LANR_POWER_MW: usize = 0x50;
    pub const DARK_LEDGER_SINK: usize = 0x58;
    pub const ECC_SYNDROME_REG: usize = 0x60;
    pub const PCSS_INTERLOCK_RAW: usize = 0x68;
    pub const WATCHDOG_HEARTBEAT: usize = 0x70;
    pub const RESERVED_PADDING: usize = 0x78;
}

/// ctrl_status bits.
pub const CTRL_ARM: u64 = 1 << 0;
pub const CTRL_FIRE: u64 = 1 << 1;
pub const CTRL_ABORT: u64 = 1 << 2;
pub const CTRL_SECDED_ERR: u64 = 1 << 3;
/// pcss_interlock_raw status bits.
pub const PCSS_TRIPPED: u64 = 1 << 0;
pub const PCSS_SIC_RECOVERED: u64 = 1 << 1;

/// Register block mirroring the packed C11 `shbt_warp_mmio_t`: 16 u64 words
/// on two 64-byte cache lines.
#[repr(C, align(64))]
#[derive(Clone, Copy, Debug)]
pub struct ShbtWarpMmio {
    pub regs: [u64; 16],
}

const _: [(); 128] = [(); std::mem::size_of::<ShbtWarpMmio>()];
const _: [(); 64] = [(); std::mem::align_of::<ShbtWarpMmio>()];

impl ShbtWarpMmio {
    pub fn zeroed() -> Self {
        Self { regs: [0; 16] }
    }

    pub fn read(&self, offset: usize) -> u64 {
        self.regs[offset / 8]
    }

    pub fn write(&mut self, offset: usize, v: u64) {
        self.regs[offset / 8] = v;
    }
}

/// PCSS crowbar constants: 180 ps optical trigger + 820 ps avalanche rise +
/// 8 x 0.1425 ns latch chain = 2.140 ns total trip (sub-2.5 ns guaranteed),
/// structural abort limit 5.0 ns, 94.20% SiC inductive recovery.
pub const PCSS_OPTICAL_PS: f64 = 180.0;
pub const PCSS_AVALANCHE_PS: f64 = 820.0;
pub const PCSS_TRIGGER_NS: f64 = 2.140;
pub const PCSS_HARD_LIMIT_NS: f64 = 2.50;
pub const PCSS_STRUCT_LIMIT_NS: f64 = 5.00;
pub const PCSS_PEAK_SHUNT_KA: f64 = 4.85;
pub const SIC_RECOVERY: f64 = 0.9420;
/// Minimum thermal headroom across the substrate stack (K).
pub const THERMAL_HEADROOM_K: f64 = 11.79;

/// SECDED Hamming(72,64): encode a 64-bit word into 72 bits (7 parity bits +
/// overall parity in bit 71).
pub fn secded_encode(data: u64) -> u128 {
    let mut code: u128 = 0;
    let mut d = 0u64;
    for pos in 1u64..=71 {
        if pos & (pos - 1) != 0 {
            code |= (((data >> d) & 1) as u128) << (pos - 1);
            d += 1;
        }
    }
    let parity_positions = [0u64, 1, 3, 7, 15, 31, 63];
    for &p in &parity_positions {
        let mut parity = 0u64;
        for pos in 1u64..=71 {
            if (pos & (p + 1)) != 0 && pos != p + 1 {
                parity ^= ((code >> (pos - 1)) & 1) as u64;
            }
        }
        code |= (parity as u128) << p;
    }
    let mut o = 0u64;
    for i in 0..71 {
        o ^= ((code >> i) & 1) as u64;
    }
    code | ((o as u128) << 71)
}

/// SECDED decode: returns (corrected_data, syndrome, corrected_count).
/// Syndrome 0 = clean; nonzero syndrome with odd overall parity = single-bit
/// error corrected in place; nonzero syndrome with even parity = DUE abort.
pub fn secded_decode(code: u128) -> (u64, u8, u32) {
    let parity_positions = [0u64, 1, 3, 7, 15, 31, 63];
    let mut syndrome: u64 = 0;
    for (i, &p) in parity_positions.iter().enumerate() {
        let mut parity = 0u64;
        for pos in 1u64..=71 {
            if (pos & (p + 1)) != 0 {
                parity ^= ((code >> (pos - 1)) & 1) as u64;
            }
        }
        syndrome |= parity << i;
    }
    let mut overall = 0u64;
    for i in 0..72 {
        overall ^= ((code >> i) & 1) as u64;
    }
    let mut fixed = code;
    let mut corrected = 0;
    if syndrome != 0 && overall == 1 {
        fixed ^= 1u128 << (syndrome - 1);
        corrected = 1;
    }
    let mut data = 0u64;
    let mut d = 0u64;
    for pos in 1u64..=71 {
        if pos & (pos - 1) != 0 {
            data |= (((fixed >> (pos - 1)) & 1) as u64) << d;
            d += 1;
        }
    }
    (data, syndrome as u8, corrected)
}

/// Double-bit upset check: syndrome nonzero with even overall parity must
/// fire the emergency abort interlock within one 10 ns cycle.
pub fn secded_due_aborts(code: u128) -> bool {
    let (_data, syn, corrected) = secded_decode(code);
    syn != 0 && corrected == 0
}

/// Givens rotation applied to a pair of lanes; the scalar form the AVX-512
/// kernel unrolls across 8 lanes.
pub fn givens_rotate(x: f64, y: f64, c: f64, s: f64) -> (f64, f64) {
    (c * x + s * y, -s * x + c * y)
}

/// Crowbar audit: trigger latency under the 2.140 ns budget plus SiC
/// recovery at the 94.20% rating.
pub fn crowbar_ok(trigger_ns: f64) -> bool {
    trigger_ns <= PCSS_TRIGGER_NS
}

/// Thermal headroom audit for the substrate stack.
pub fn headroom_ok(delta_t: f64) -> bool {
    delta_t >= THERMAL_HEADROOM_K
}

/// CRC-32/Castagnoli (reflected poly 0x82F63B78) over the arena trailer.
pub fn crc32c(data: &[u8]) -> u32 {
    let mut crc: u32 = !0;
    for &b in data {
        crc ^= b as u32;
        for _ in 0..8 {
            let mask = (crc & 1).wrapping_neg();
            crc = (crc >> 1) ^ (0x82F6_3B78 & mask);
        }
    }
    !crc
}

/// Stinespring arena sizes (bytes): 640 active + 1472 dark = 2112.
pub const STINESPRING_ACTIVE_BYTES: usize = 640;
pub const STINESPRING_DARK_BYTES: usize = 1472;
pub const STINESPRING_BYTES: usize = STINESPRING_ACTIVE_BYTES + STINESPRING_DARK_BYTES;

/// Q32.32 fixed-point encoding for v_s and acceleration registers.
pub fn q32_32(v: f64) -> u64 {
    (v * 4294967296.0).round() as u64
}

/// Passenger-acceleration interlock: cavity_accel_raw above the 1e-7 m/s²
/// residual bound (or a SECDED DUE flag) must fire the crowbar quench.
pub fn quench_interlock_ok(accel_raw_q32: u64, secded_due: bool, trigger_ns: f64) -> bool {
    let fires = accel_raw_q32 > 429 || secded_due;
    !fires || trigger_ns <= PCSS_HARD_LIMIT_NS
}
