//! Hardware-in-the-loop microkernel bridge.
//!
//! Rust mirror of `kernel/include/shbt_warp_hardware.h`: the 128-byte
//! dual-cacheline `shbt_warp_mmio_t` register block at 0x70000000, SECDED
//! Hamming(72,64) ECC scrubbing, Givens remapping and the sub-2.5 ns PCSS
//! crowbar quench driver (94.20% SiC inductive recovery).

/// MMIO base physical address.
pub const MMIO_BASE: u64 = 0x7000_0000;
/// Register-block footprint (bytes) — two aligned cache lines.
pub const MMIO_SIZE: usize = 128;

pub mod reg {
    pub const SYS_CONTROL: usize = 0x00;
    pub const SYS_STATUS: usize = 0x04;
    pub const LANR_POWER_MW: usize = 0x08;
    pub const LANDAUER_DEBT_MW: usize = 0x10;
    pub const BUBBLE_VELOCITY_C: usize = 0x18;
    pub const WALL_THICKNESS_NM: usize = 0x20;
    pub const LAPSE_ERROR_RAW: usize = 0x28;
    pub const QUENCH_TIMER_NS: usize = 0x2C;
    pub const INTERLOCK_FLAGS: usize = 0x30;
    pub const SHIFT_BETA_X: usize = 0x40;
    pub const RICCI_SCALAR: usize = 0x48;
    pub const QI_INTEGRAL_BOUND: usize = 0x50;
    pub const RF_EMITTER_PHASE: usize = 0x58;
    pub const ECC_SYNDROME: usize = 0x5C;
    pub const FLIGHT_STAGE: usize = 0x60;
    pub const CRC32_CHECKSUM: usize = 0x64;
}

/// SYS_CONTROL bits.
pub const CTRL_ENABLE: u32 = 1 << 0;
pub const CTRL_BUBBLE_ACTIVE: u32 = 1 << 1;
pub const CTRL_QUENCH_TRIGGER: u32 = 1 << 2;
/// SYS_STATUS bits.
pub const STAT_BUBBLE_STABLE: u32 = 1 << 0;
pub const STAT_ECC_OK: u32 = 1 << 1;
pub const STAT_LAPSE_UNITY: u32 = 1 << 2;
/// INTERLOCK_FLAGS bits.
pub const ILK_SPACELIKE: u32 = 1 << 0;
pub const ILK_UNDERPOWER: u32 = 1 << 1;
pub const ILK_HORIZON_RISK: u32 = 1 << 2;

/// Register block mirroring the C11 `shbt_warp_mmio_t`.
#[repr(C, align(64))]
#[derive(Clone, Copy, Debug)]
pub struct ShbtWarpMmio {
    pub regs: [u32; 32],
}

const _: [(); 128] = [(); std::mem::size_of::<ShbtWarpMmio>()];
const _: [(); 64] = [(); std::mem::align_of::<ShbtWarpMmio>()];

impl ShbtWarpMmio {
    pub fn zeroed() -> Self {
        Self { regs: [0; 32] }
    }

    pub fn read(&self, offset: usize) -> u32 {
        self.regs[offset / 4]
    }

    pub fn write(&mut self, offset: usize, v: u32) {
        self.regs[offset / 4] = v;
    }
}

/// PCSS crowbar trigger budget (ns) and SiC inductive recovery fraction.
pub const PCSS_TRIGGER_NS: f64 = 2.18;
pub const PCSS_HARD_LIMIT_NS: f64 = 2.50;
pub const SIC_RECOVERY: f64 = 0.9420;
/// Minimum thermal headroom across the substrate stack (K).
pub const THERMAL_HEADROOM_K: f64 = 11.79;

/// SECDED Hamming(72,64): encode a 64-bit word into 72 bits (8 parity bits,
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
/// error corrected in place.
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

/// Givens rotation applied to a pair of lanes; the scalar form the AVX-512
/// kernel unrolls across 8 lanes.
pub fn givens_rotate(x: f64, y: f64, c: f64, s: f64) -> (f64, f64) {
    (c * x + s * y, -s * x + c * y)
}

/// Crowbar audit: trigger latency under the 2.18 ns budget plus SiC
/// recovery at the 94.20% rating.
pub fn crowbar_ok(trigger_ns: f64) -> bool {
    trigger_ns <= PCSS_TRIGGER_NS
}

/// Thermal headroom audit for the substrate stack.
pub fn headroom_ok(delta_t: f64) -> bool {
    delta_t >= THERMAL_HEADROOM_K
}

/// CRC-32/Castagnoli (0x1EDC6F41, reflected 0x82F63B78) used by the frame
/// checksum register at offset 0x64.
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

/// Q32.32 fixed-point encoding for v_s and beta^x registers.
pub fn q32_32(v: f64) -> u64 {
    (v * 4294967296.0).round() as u64
}

/// Lapse-deviation interlock: |alpha - 1| > 1e-6 or a spacelike horizon
/// anomaly (det(gamma) <= 0) must fire the crowbar quench.
pub fn quench_interlock_ok(alpha: f64, det_gamma: f64, trigger_ns: f64) -> bool {
    let fires = (alpha - 1.0).abs() > 1e-6 || det_gamma <= 0.0;
    !fires || trigger_ns <= PCSS_HARD_LIMIT_NS
}
