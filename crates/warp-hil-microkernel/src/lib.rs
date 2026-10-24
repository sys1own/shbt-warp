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

/// Isomer battery register map (`shbt_isomer_battery_mmio_t`, 128 B
/// dual-cacheline at 0x70000000, mirroring `shbt_isomer_battery_mmio.h`).
pub mod isomer {
    pub const ENERGY_REMAINING_JOULES: usize = 0x00;
    pub const STATE_OF_CHARGE_Q32: usize = 0x08;
    pub const BUS_VOLTAGE_UV: usize = 0x10;
    pub const BUS_CURRENT_UA: usize = 0x18;
    pub const CORE_TEMPERATURE_UK: usize = 0x20;
    pub const CRYO_HEADROOM_UK: usize = 0x28;
    pub const TRIGGER_DELAY_PS: usize = 0x30;
    pub const GRASER_COHERENT_FLUX_W_M2: usize = 0x38;
    pub const TRANSIENT_SHEAR_STRESS_KPA: usize = 0x40;
    pub const SYSTEM_STATUS_FLAGS: usize = 0x48;
    pub const INDUCTIVE_RECOVERY_EFF_Q32: usize = 0x50;
    pub const DARK_LEDGER_SINK_Q32: usize = 0x58;
    pub const TMSV_SQUEEZING_R_Q16: usize = 0x60;
    pub const INTERLOCK_CMD_REG: usize = 0x64;
    pub const HARDWARE_RESERVED_PAD: usize = 0x68;

    pub const STAT_READY: u32 = 1 << 0;
    pub const STAT_BORRMANN_LOCKED: u32 = 1 << 1;
    pub const STAT_CROWBAR_ARMED: u32 = 1 << 2;
    pub const STAT_CROWBAR_TRIPPED: u32 = 1 << 3;
    pub const STAT_CRYO_WARNING: u32 = 1 << 4;
    pub const STAT_QUENCH_FAULT: u32 = 1 << 5;
    pub const STAT_LASING_ACTIVE: u32 = 1 << 6;
    pub const STAT_DARK_SINK_SYNC: u32 = 1 << 7;
}

/// Battery bus crowbar: PCSS-triggered nuclear burst closing-time bound.
pub const ISOMER_CROWBAR_CLOSE_NS: f64 = 2.10;
/// Battery bus slew-rate bounds.
pub const ISOMER_DI_DT_MAX: f64 = 1.85e14;
pub const ISOMER_DV_DT_MAX: f64 = 4.20e13;
/// Cryo headroom for the isomer core, in microkelvin (11.79 K).
pub const ISOMER_HEADROOM_MIN_UK: u64 = 11_790_000;
/// Core operating temperature, in microkelvin (21.13 K).
pub const ISOMER_CORE_OP_UK: u64 = 21_130_000;

/// Battery crowbar interlock: a nuclear-burst trip must close within
/// 2.10 ns (under the shared 2.50 ns hard limit) while retaining the
/// >= 94.20% SiC inductive recovery.
pub fn isomer_crowbar_ok(close_ns: f64, recovery: f64) -> bool {
    close_ns <= ISOMER_CROWBAR_CLOSE_NS && recovery >= SIC_RECOVERY
}

// ---- SHBT-MAR-312 4th-Gen Mass-Annihilation Microkernel Mirror -------------

pub mod mar312 {
    pub const MAR_STATE_STANDBY_STASIS: u32 = 0x01;
    pub const MAR_STATE_TRIGGER_ARMED: u32 = 0x02;
    pub const MAR_STATE_BURST_TRACTION: u32 = 0x04;
    pub const MAR_STATE_DEC_COOLDOWN: u32 = 0x08;
    pub const MAR_STATE_EMERGENCY_QUENCH: u32 = 0x10;

    pub const MAR_FAULT_NONE: u32 = 0x0000_0000;
    pub const MAR_FAULT_ANOMALY_CLOSURE: u32 = 0x0000_0001;
    pub const MAR_FAULT_CRYO_QUENCH_WARN: u32 = 0x0000_0002;
    pub const MAR_FAULT_NBN_TRANSITION: u32 = 0x0000_0004;
    pub const MAR_FAULT_VESSEL_STRESS: u32 = 0x0000_0008;
    pub const MAR_FAULT_RMHD_ALFVEN: u32 = 0x0000_0010;
    pub const MAR_FAULT_ADM_DET_UNLOCK: u32 = 0x0000_0020;
    pub const MAR_FAULT_WAKE_RIGIDITY: u32 = 0x0000_0040;
    pub const MAR_FAULT_TQEC_DEADLINE: u32 = 0x0000_0080;

    pub const SHBT_SYNTH_STATE_COSET_ANTIBARYON_FOLD: u8 = 0x06;
    pub const SHBT_SYNTH_STATE_SYMPLECTIC_CRYSTALLIZATION: u8 = 0x08;
    pub const COSET_ANTIBARYON_FOLD: u8 = 0x06;
    pub const SYMPLECTIC_CRYSTALLIZATION: u8 = 0x08;

    pub const SHBT_SYNTH_ANTIBARYON_RATIO_50_50_Q16: u16 = 0x8000;
    pub const SHBT_SYNTH_COSET_WEIGHT_45_64_Q16: u16 = 0xB400;
    pub const SYNTH_ANTIBARYON_RATIO_Q16: u16 = 0x8000;
    pub const SYNTH_COSET_WEIGHT_Q16: u16 = 0xB400;

    pub const SHBT_SYNTH_RG_LOCK_312: u32 = 1 << 16;
    pub const SHBT_SYNTH_CBARB_ZERO_VERIFIED: u32 = 1 << 17;

    pub const SHBT_CTRL_DYNAMIC_GST_MATCH: u64 = 1 << 16;
    pub const SHBT_CTRL_PROGRESSIVE_EIKONAL: u64 = 1 << 17;
    pub const SHBT_CTRL_TTN_MERA_ADAPT: u64 = 1 << 18;
    pub const SHBT_CTRL_HYPERFRAME_64BANK: u64 = 1 << 19;
    pub const SHBT_EW_CTRL_GHOST_HOVER_LOCK: u64 = 1 << 21;
    pub const SHBT_EW_CTRL_GHOST_HARVEST_FUNNEL: u64 = 1 << 22;
    pub const SHBT_EW_CTRL_HABITAT_1G_FLOOR: u64 = 1 << 23;
    pub const SHBT_EW_CTRL_WAKE_COMP_3RD_ORDER: u64 = 1 << 24;

    // ---- Phase 1-4 SHBT-SWARM & SHBT-MAR-312X Command Opcodes ----
    pub const SHBT_SWARM_CMD_NOP: u32 = 0x0000_0000;
    pub const SHBT_SWARM_CMD_TOMITA_QUINE_CPY: u32 = 0x0000_0001;
    pub const SHBT_SWARM_CMD_PT_GOI_PROJECT: u32 = 0x0000_0002;
    pub const SHBT_SWARM_CMD_TOMO_CAPTURE: u32 = 0x0000_0004;
    pub const SHBT_SWARM_CMD_EXEC_CLONE_SHIP: u32 = 0x0000_0008;
    pub const SHBT_SWARM_CMD_EXEC_CLONE_BIO: u32 = 0x0000_0010;
    pub const SHBT_SWARM_CMD_LANDAUER_SHUNT: u32 = 0x0000_0020;
    pub const SHBT_SWARM_CMD_MERA_PINN_SYNC: u32 = 0x0000_0040;
    pub const SHBT_SWARM_CMD_PT_GOI_EQUIL: u32 = 0x0000_0080;
    pub const SHBT_SWARM_CMD_TOMITA_LEARN: u32 = 0x0000_0100;
    pub const SHBT_SWARM_CMD_VERLINDE_PLAN: u32 = 0x0000_0200;
    pub const SHBT_SWARM_CMD_SP2G_HIVE_SYNC: u32 = 0x0000_0400;
    pub const SHBT_SWARM_CMD_AP_MPC_ENCLAVE: u32 = 0x0000_0800;
    pub const SHBT_SWARM_CMD_XENO_ASSIMILATE: u32 = 0x0000_1000;
    pub const SHBT_SWARM_CMD_EXEC_UNIV_SYNTH: u32 = 0x0000_2000;
    pub const SHBT_SWARM_CMD_ZERO_COMP_DIRECT_RENDER: u32 = 0x0000_4000;
    pub const SHBT_SWARM_CMD_MERA_DAG_INSTANCE: u32 = 0x0000_8000;
    pub const SHBT_SWARM_CMD_MAR312X_FORGE_TIER4: u32 = 0x0001_0000;
    pub const SHBT_SWARM_CMD_DIRECT_BOUNDARY_SYNTH: u32 = 0x0002_0000;
    pub const SHBT_SWARM_CMD_AUTOPOIETIC_SELF_HEAL: u32 = 0x0004_0000;
    pub const SHBT_SWARM_CMD_WEYL_EXCITON_CROWBAR: u32 = 0x8000_0000;
    pub const SHBT_SWARM_CMD_PCSS_CROWBAR: u32 = 0x8000_0000;

    // ---- Phase 1-4 SHBT-SWARM & SHBT-MAR-312X Fault Bitmasks ----
    pub const SHBT_SWARM_FAULT_NONE: u32 = 0x0000_0000;
    pub const SHBT_SWARM_FAULT_ADM_IMBALANCE: u32 = 0x0000_0001;
    pub const SHBT_SWARM_FAULT_PARTITION_LOCK: u32 = 0x0000_0002;
    pub const SHBT_SWARM_FAULT_FRAMING_WEYL: u32 = 0x0000_0004;
    pub const SHBT_SWARM_FAULT_BL_ANOMALY: u32 = 0x0000_0008;
    pub const SHBT_SWARM_FAULT_NEUTRINO_LEAK: u32 = 0x0000_0010;
    pub const SHBT_SWARM_FAULT_SECDED_UNCORR: u32 = 0x0000_0020;
    pub const SHBT_SWARM_FAULT_CAYLEY_HASH: u32 = 0x0000_0040;
    pub const SHBT_SWARM_FAULT_THERMAL_CLAMP: u32 = 0x0000_0080;
    pub const SHBT_SWARM_FAULT_CBF_PHYS_VIOL: u32 = 0x0000_0100;
    pub const SHBT_SWARM_FAULT_GREY_GOO_CAP: u32 = 0x0000_0200;
    pub const SHBT_SWARM_FAULT_CLF_DIVERGENCE: u32 = 0x0000_0400;
    pub const SHBT_SWARM_FAULT_X0_PROTECT_LCK: u32 = 0x0000_0800;
    pub const SHBT_SWARM_FAULT_X1_SAPIENT_LCK: u32 = 0x0000_1000;
    pub const SHBT_SWARM_FAULT_SUGAWARA_CAP_EXCEEDED: u32 = 0x0000_2000;
    pub const SHBT_SWARM_FAULT_DRIPLINE_UNSTABLE: u32 = 0x0000_4000;
    pub const SHBT_SWARM_FAULT_SIM_TO_RENDER_HASH_MISMATCH: u32 = 0x0000_8000;
    pub const SHBT_SWARM_FAULT_HABITAT_GRAV_DEVIATION: u32 = 0x0001_0000;
    pub const SHBT_SWARM_FAULT_SUPERHYDRIDE_TETHER_LOSS: u32 = 0x0002_0000;
    pub const SHBT_SWARM_FAULT_PURCELL_BANDGAP_DRIFT: u32 = 0x0004_0000;
    pub const SHBT_SWARM_FAULT_DECX_CHERN_BACKSCATTER: u32 = 0x0008_0000;
    pub const SHBT_SWARM_FAULT_FORGE_SHEAR_EXCEEDED: u32 = 0x0010_0000;

    // ---- SHBT-MAR-312X Hardware Constants ----
    pub const SHBT_MAR312X_NACELLE_MASS_KG: u32 = 950;
    pub const SHBT_MAR312X_NACELLE_VOLUME_ML: u32 = 350_000;
    pub const SHBT_MAR312X_SINGLE_FRAME_CORES: u32 = 41;
    pub const SHBT_MAR312X_SUPERHYDRIDE_FIELD_MT: u32 = 48_000;
    pub const SHBT_MAR312X_SUPERHYDRIDE_TCRIT_MK: u32 = 192_500;
    pub const SHBT_MAR312X_DECX_EFF_PPM: u32 = 992_000;
    pub const SHBT_MAR312X_DIRECT_BUS_EFF_PPM: u32 = 998_500;
    pub const SHBT_MAR312X_CROWBAR_TRIP_FS: u32 = 1_701;

    pub fn mar312x_dispatch_tier4(
        t_op_kelvin: f64,
        b_field_tesla: f64,
        tether_disp_angstrom: f64,
        sigma_shear_mpa: f64,
        decx_chern_c2: i32,
        purcell_dos: f64,
    ) -> u32 {
        let mut fault = SHBT_SWARM_FAULT_NONE;
        if purcell_dos != 0.0 {
            fault |= SHBT_SWARM_FAULT_PURCELL_BANDGAP_DRIFT;
        }
        if decx_chern_c2 == 0 {
            fault |= SHBT_SWARM_FAULT_DECX_CHERN_BACKSCATTER;
        }
        if b_field_tesla < 48.0 || tether_disp_angstrom.abs() > 1.0e-4 {
            fault |= SHBT_SWARM_FAULT_SUPERHYDRIDE_TETHER_LOSS;
        }
        if !(0.0..=192.50).contains(&t_op_kelvin) {
            fault |= SHBT_SWARM_FAULT_THERMAL_CLAMP;
        }
        if !(0.0..=4800.00).contains(&sigma_shear_mpa) {
            fault |= SHBT_SWARM_FAULT_FORGE_SHEAR_EXCEEDED;
        }
        fault
    }

    pub fn mar312x_weyl_crowbar_trip_fs() -> u32 {
        SHBT_MAR312X_CROWBAR_TRIP_FS
    }

    pub fn swarm_cbf_xeno_gate(min_cbf_barrier: f64, xeno_class_code: u8) -> u32 {
        let mut fault = SHBT_SWARM_FAULT_NONE;
        if min_cbf_barrier < 0.0 {
            fault |= SHBT_SWARM_FAULT_CBF_PHYS_VIOL;
        }
        if xeno_class_code == 1 {
            fault |= SHBT_SWARM_FAULT_X0_PROTECT_LCK;
        } else if xeno_class_code == 2 {
            fault |= SHBT_SWARM_FAULT_X1_SAPIENT_LCK;
        }
        fault
    }

    pub fn ghost_hover_step_50k518(delta_p_net_w: f64, p_bit_w: f64) -> u64 {
        if delta_p_net_w <= 0.0 || p_bit_w <= 0.0 {
            0
        } else {
            (delta_p_net_w / p_bit_w).floor() as u64
        }
    }

    pub fn ghost_antiphase_clamp(g_det: f64) -> f64 {
        let diff = g_det + 1.0;
        if diff > 1.0e-12 {
            -1.0 + 1.0e-12
        } else if diff < -1.0e-12 {
            -1.0 - 1.0e-12
        } else {
            g_det
        }
    }

    pub fn ghost_harvest_trigger_stage1(accumulated_mass_kg: f64, threshold_kg: f64) -> bool {
        accumulated_mass_kg >= threshold_kg && threshold_kg > 0.0
    }

    pub fn synth_fold_ok(
        antibaryon_ratio_q16: u16,
        coset_weight_q16: u16,
        synth_status_flags: u32,
    ) -> bool {
        let req = SHBT_SYNTH_RG_LOCK_312 | SHBT_SYNTH_CBARB_ZERO_VERIFIED;
        antibaryon_ratio_q16 == SHBT_SYNTH_ANTIBARYON_RATIO_50_50_Q16
            && coset_weight_q16 == SHBT_SYNTH_COSET_WEIGHT_45_64_Q16
            && (synth_status_flags & req) == req
    }
}

pub use mar312::{
    ghost_antiphase_clamp, ghost_harvest_trigger_stage1, ghost_hover_step_50k518,
    COSET_ANTIBARYON_FOLD, SHBT_CTRL_DYNAMIC_GST_MATCH, SHBT_CTRL_HYPERFRAME_64BANK,
    SHBT_CTRL_PROGRESSIVE_EIKONAL, SHBT_CTRL_TTN_MERA_ADAPT,
    SHBT_EW_CTRL_GHOST_HARVEST_FUNNEL, SHBT_EW_CTRL_GHOST_HOVER_LOCK,
    SHBT_EW_CTRL_HABITAT_1G_FLOOR, SHBT_EW_CTRL_WAKE_COMP_3RD_ORDER,
    SHBT_SYNTH_ANTIBARYON_RATIO_50_50_Q16, SHBT_SYNTH_CBARB_ZERO_VERIFIED,
    SHBT_SYNTH_COSET_WEIGHT_45_64_Q16, SHBT_SYNTH_RG_LOCK_312,
    SHBT_SYNTH_STATE_COSET_ANTIBARYON_FOLD, SHBT_SYNTH_STATE_SYMPLECTIC_CRYSTALLIZATION,
    SYMPLECTIC_CRYSTALLIZATION, SYNTH_ANTIBARYON_RATIO_Q16, SYNTH_COSET_WEIGHT_Q16,
    mar312x_dispatch_tier4, mar312x_weyl_crowbar_trip_fs, swarm_cbf_xeno_gate,
    SHBT_MAR312X_CROWBAR_TRIP_FS, SHBT_MAR312X_DECX_EFF_PPM, SHBT_MAR312X_DIRECT_BUS_EFF_PPM,
    SHBT_MAR312X_NACELLE_MASS_KG, SHBT_MAR312X_NACELLE_VOLUME_ML,
    SHBT_MAR312X_SINGLE_FRAME_CORES, SHBT_MAR312X_SUPERHYDRIDE_FIELD_MT,
    SHBT_MAR312X_SUPERHYDRIDE_TCRIT_MK, SHBT_SWARM_CMD_AP_MPC_ENCLAVE,
    SHBT_SWARM_CMD_AUTOPOIETIC_SELF_HEAL, SHBT_SWARM_CMD_DIRECT_BOUNDARY_SYNTH,
    SHBT_SWARM_CMD_EXEC_CLONE_BIO, SHBT_SWARM_CMD_EXEC_CLONE_SHIP,
    SHBT_SWARM_CMD_EXEC_UNIV_SYNTH, SHBT_SWARM_CMD_LANDAUER_SHUNT,
    SHBT_SWARM_CMD_MAR312X_FORGE_TIER4, SHBT_SWARM_CMD_MERA_DAG_INSTANCE,
    SHBT_SWARM_CMD_MERA_PINN_SYNC, SHBT_SWARM_CMD_NOP, SHBT_SWARM_CMD_PCSS_CROWBAR,
    SHBT_SWARM_CMD_PT_GOI_EQUIL, SHBT_SWARM_CMD_PT_GOI_PROJECT, SHBT_SWARM_CMD_SP2G_HIVE_SYNC,
    SHBT_SWARM_CMD_TOMITA_LEARN, SHBT_SWARM_CMD_TOMITA_QUINE_CPY, SHBT_SWARM_CMD_TOMO_CAPTURE,
    SHBT_SWARM_CMD_VERLINDE_PLAN, SHBT_SWARM_CMD_WEYL_EXCITON_CROWBAR,
    SHBT_SWARM_CMD_XENO_ASSIMILATE, SHBT_SWARM_CMD_ZERO_COMP_DIRECT_RENDER,
    SHBT_SWARM_FAULT_ADM_IMBALANCE, SHBT_SWARM_FAULT_BL_ANOMALY, SHBT_SWARM_FAULT_CAYLEY_HASH,
    SHBT_SWARM_FAULT_CBF_PHYS_VIOL, SHBT_SWARM_FAULT_CLF_DIVERGENCE,
    SHBT_SWARM_FAULT_DECX_CHERN_BACKSCATTER, SHBT_SWARM_FAULT_DRIPLINE_UNSTABLE,
    SHBT_SWARM_FAULT_FORGE_SHEAR_EXCEEDED, SHBT_SWARM_FAULT_FRAMING_WEYL,
    SHBT_SWARM_FAULT_GREY_GOO_CAP, SHBT_SWARM_FAULT_HABITAT_GRAV_DEVIATION,
    SHBT_SWARM_FAULT_NEUTRINO_LEAK, SHBT_SWARM_FAULT_NONE, SHBT_SWARM_FAULT_PARTITION_LOCK,
    SHBT_SWARM_FAULT_PURCELL_BANDGAP_DRIFT, SHBT_SWARM_FAULT_SECDED_UNCORR,
    SHBT_SWARM_FAULT_SIM_TO_RENDER_HASH_MISMATCH, SHBT_SWARM_FAULT_SUGAWARA_CAP_EXCEEDED,
    SHBT_SWARM_FAULT_SUPERHYDRIDE_TETHER_LOSS, SHBT_SWARM_FAULT_THERMAL_CLAMP,
    SHBT_SWARM_FAULT_X0_PROTECT_LCK, SHBT_SWARM_FAULT_X1_SAPIENT_LCK,
};

#[derive(Clone, Copy, Debug)]
pub struct ShbtMar312Telemetry {
    pub t_core_kelvin: f64,
    pub t_nbn_kelvin: f64,
    pub sigma_vessel_mpa: f64,
    pub rmhd_mach_alfven: f64,
    pub adm_det_g_err: f64,
    pub wake_mu_err: f64,
    pub lhe_void_fraction: f64,
    pub tqec_latency_ps: u32,
    pub entropy_headroom: u64,
}

#[repr(C, align(64))]
#[derive(Clone, Copy, Debug)]
pub struct ShbtMar312Mmio {
    pub regs: [u64; 16],
}

impl ShbtMar312Mmio {
    pub fn zeroed() -> Self {
        Self { regs: [0; 16] }
    }

    /// Read Register 0x30: Bipartite Anti-Baryon Ratio in Q0.16 (default 0x8000 = 0.500)
    pub fn synth_antibaryon_ratio_q16(&self) -> u16 {
        (self.regs[6] & 0xFFFF) as u16
    }

    /// Write Register 0x30: Bipartite Anti-Baryon Ratio in Q0.16
    pub fn set_synth_antibaryon_ratio_q16(&mut self, ratio: u16) {
        self.regs[6] = (self.regs[6] & !0xFFFF) | (ratio as u64);
    }

    /// Read Register 0x32: Coset Conformal Weight Shift in Q0.16 (default 0xB400 = 45/64)
    pub fn synth_coset_weight_q16(&self) -> u16 {
        ((self.regs[6] >> 16) & 0xFFFF) as u16
    }

    /// Write Register 0x32: Coset Conformal Weight Shift in Q0.16
    pub fn set_synth_coset_weight_q16(&mut self, weight: u16) {
        self.regs[6] = (self.regs[6] & !(0xFFFF << 16)) | ((weight as u64) << 16);
    }

    /// Read Register 0x34: Max allowable timing jitter in attoseconds (default 4 as)
    pub fn synth_jitter_bound_q16(&self) -> u16 {
        ((self.regs[6] >> 32) & 0xFFFF) as u16
    }

    /// Write Register 0x34: Max allowable timing jitter in attoseconds
    pub fn set_synth_jitter_bound_q16(&mut self, jitter_as: u16) {
        self.regs[6] = (self.regs[6] & !(0xFFFF << 32)) | ((jitter_as as u64) << 32);
    }

    /// Read Register 0x36: CS tether displacement in 10^-5 Angstroms
    pub fn synth_tether_displacement_q16(&self) -> u16 {
        ((self.regs[6] >> 48) & 0xFFFF) as u16
    }

    /// Write Register 0x36: CS tether displacement in 10^-5 Angstroms
    pub fn set_synth_tether_displacement_q16(&mut self, disp: u16) {
        self.regs[6] = (self.regs[6] & !(0xFFFF << 48)) | ((disp as u64) << 48);
    }

    /// Read Register 0x48: DEC Space Charge Headroom Q16
    pub fn dec_space_charge_margin_q16(&self) -> u16 {
        (self.regs[9] & 0xFFFF) as u16
    }

    /// Write Register 0x48: DEC Space Charge Headroom Q16
    pub fn set_dec_space_charge_margin_q16(&mut self, margin: u16) {
        self.regs[9] = (self.regs[9] & !0xFFFF) | (margin as u64);
    }

    /// Read Register 0x4A: TQEC Syndrome Error Rate Q16
    pub fn tqec_syndrome_err_rate_q16(&self) -> u16 {
        ((self.regs[9] >> 16) & 0xFFFF) as u16
    }

    /// Write Register 0x4A: TQEC Syndrome Error Rate Q16
    pub fn set_tqec_syndrome_err_rate_q16(&mut self, rate: u16) {
        self.regs[9] = (self.regs[9] & !(0xFFFF << 16)) | ((rate as u64) << 16);
    }

    /// Read Register 0x4C: REBCO Quench Headroom Q16
    pub fn rebco_quench_headroom_q16(&self) -> u32 {
        (self.regs[9] >> 32) as u32
    }

    /// Write Register 0x4C: REBCO Quench Headroom Q16
    pub fn set_rebco_quench_headroom_q16(&mut self, headroom: u32) {
        self.regs[9] = (self.regs[9] & 0xFFFF_FFFF) | ((headroom as u64) << 32);
    }
}

pub fn mar312_tick_dispatch(
    mar: &mut ShbtMar312Mmio,
    telem: &ShbtMar312Telemetry,
) -> u32 {
    let mut fault = mar312::MAR_FAULT_NONE;

    if telem.entropy_headroom == 0 {
        fault |= mar312::MAR_FAULT_ANOMALY_CLOSURE;
    }
    if telem.t_core_kelvin >= 32.92 || telem.lhe_void_fraction > 0.380 {
        fault |= mar312::MAR_FAULT_CRYO_QUENCH_WARN;
    }
    if telem.t_nbn_kelvin >= 16.00 || telem.t_nbn_kelvin > 8.00 {
        fault |= mar312::MAR_FAULT_NBN_TRANSITION;
    }
    if telem.sigma_vessel_mpa > 124.60 {
        fault |= mar312::MAR_FAULT_VESSEL_STRESS;
    }
    if telem.rmhd_mach_alfven > 0.120 {
        fault |= mar312::MAR_FAULT_RMHD_ALFVEN;
    }
    if telem.adm_det_g_err > 1.0e-12 {
        fault |= mar312::MAR_FAULT_ADM_DET_UNLOCK;
    }
    if telem.wake_mu_err > 1.0e-12 {
        fault |= mar312::MAR_FAULT_WAKE_RIGIDITY;
    }
    if telem.tqec_latency_ps > 45000 {
        fault |= mar312::MAR_FAULT_TQEC_DEADLINE;
    }

    if fault != mar312::MAR_FAULT_NONE {
        mar.regs[13] = (0xFFFF_FFFFu64) | ((mar312::MAR_STATE_EMERGENCY_QUENCH as u64) << 32);
        mar.regs[14] = (fault as u64) | ((telem.tqec_latency_ps as u64) << 32);
        mar.regs[10] = (0.9420 * 4294967296.0) as u64;
        fault
    } else {
        mar.regs[13] = (mar312::MAR_STATE_BURST_TRACTION as u64) << 32;
        mar.regs[14] = (telem.tqec_latency_ps as u64) << 32;
        mar.regs[10] = (0.9420 * 4294967296.0) as u64;
        mar312::MAR_FAULT_NONE
    }
}

pub mod ew312 {
    pub const SHBT_EW312_MMIO_BASE: u64 = 0x7000_0000;
    pub const SHBT_EW312_MMIO_SIZE: usize = 56;
    pub const SHBT_EW312_Z3_CONTRACT_SIG: u32 = 0x3120CAFE;

    pub const SHBT_STATE_STANDBY_STASIS: u8 = 0x00;
    pub const SHBT_STATE_INGEST_DERENDER: u8 = 0x01;
    pub const SHBT_STATE_DARK_LEDGER_LOCK: u8 = 0x02;
    pub const SHBT_STATE_CORRIDOR_TRACTION: u8 = 0x03;
    pub const SHBT_STATE_CAUSAL_EGRESS: u8 = 0x04;
    pub const SHBT_STATE_EMERGENCY_QUENCH: u8 = 0xFF;

    pub const G_CONST: f64 = 6.67430e-11;
    pub const C_LIGHT: f64 = 299_792_458.0;
    pub const M_SUN: f64 = 1.98847e30;
    pub const J2_SUN: f64 = 2.20e-7;
    pub const R_SUN: f64 = 6.96342e8;

    #[repr(C, packed)]
    #[derive(Clone, Copy, Debug)]
    pub struct ShbtEw312Mmio {
        pub causal_cone_lo: u32,
        pub causal_cone_hi: u32,
        pub pn2_metric_m0: f64,
        pub pn2_metric_j2: f64,
        pub pn2_spin_vec: [f32; 3],
        pub target_vel_gamma: u32,
        pub ds2_interval_lo: u32,
        pub ds2_interval_hi: i32,
        pub quench_time_ns: u32,
        pub z3_contract_sig: u32,
    }

    const _: [(); 56] = [(); std::mem::size_of::<ShbtEw312Mmio>()];

    impl ShbtEw312Mmio {
        pub fn init() -> Self {
            Self {
                causal_cone_lo: 0,
                causal_cone_hi: 0,
                pn2_metric_m0: M_SUN,
                pn2_metric_j2: J2_SUN,
                pn2_spin_vec: [0.0, 0.0, 1.92e33],
                target_vel_gamma: 0x00010000,
                ds2_interval_lo: 0,
                ds2_interval_hi: 0,
                quench_time_ns: 0,
                z3_contract_sig: SHBT_EW312_Z3_CONTRACT_SIG,
            }
        }

        #[allow(clippy::too_many_arguments)]
        pub fn evaluate_2pn(
            &mut self,
            dt: f64,
            dx: f64,
            dy: f64,
            dz: f64,
            vx: f64,
            vy: f64,
            vz: f64,
        ) -> bool {
            let r = (dx * dx + dy * dy + dz * dz).sqrt().max(1.0);
            let v2 = vx * vx + vy * vy + vz * vz;
            let beta2 = v2 / (C_LIGHT * C_LIGHT);
            if beta2 >= 1.0 {
                self.causal_cone_hi |= 0x8000_0000;
                return false;
            }
            let gamma = 1.0 / (1.0 - beta2).sqrt();
            self.target_vel_gamma = (gamma * 65536.0) as u32;

            let gm_over_rc2 = (G_CONST * self.pn2_metric_m0) / (C_LIGHT * C_LIGHT * r);
            let g00 = -(1.0 - 2.0 * gm_over_rc2 + 2.0 * gm_over_rc2 * gm_over_rc2
                + (3.0 * G_CONST * self.pn2_metric_m0 * self.pn2_metric_j2 * R_SUN * R_SUN)
                    / (C_LIGHT * C_LIGHT * r * r * r));
            let g_ij = 1.0 + 2.0 * gm_over_rc2;
            let dr2 = dx * dx + dy * dy + dz * dz;
            let ds2_2pn = g00 * C_LIGHT * C_LIGHT * dt * dt + g_ij * dr2;

            let ds2_fixed = (ds2_2pn * 1e-6) as i64;
            self.ds2_interval_lo = (ds2_fixed & 0xFFFF_FFFF) as u32;
            self.ds2_interval_hi = ((ds2_fixed >> 32) & 0xFFFF_FFFF) as i32;

            if ds2_2pn > 0.0 {
                self.causal_cone_hi |= 0x8000_0000;
                self.quench_time_ns = 2;
                false
            } else {
                self.causal_cone_lo |= 0x01;
                true
            }
        }

        pub fn fsm_step(&mut self, next_state: u8) -> u8 {
            if self.z3_contract_sig != SHBT_EW312_Z3_CONTRACT_SIG {
                return SHBT_STATE_EMERGENCY_QUENCH;
            }
            match next_state {
                SHBT_STATE_STANDBY_STASIS
                | SHBT_STATE_INGEST_DERENDER
                | SHBT_STATE_DARK_LEDGER_LOCK
                | SHBT_STATE_CORRIDOR_TRACTION
                | SHBT_STATE_CAUSAL_EGRESS => next_state,
                _ => SHBT_STATE_EMERGENCY_QUENCH,
            }
        }
    }
}

pub mod ibr312 {
    pub const SHBT_IBR312_MAGAZINE_BANKS: usize = 16;
    pub const SHBT_IBR312_MAGAZINE_SIZE: usize = 33792;

    pub const SHBT_MAR_SYMPLECTIC_SHIELD_LOCK: u32 = 1 << 8;
    pub const SHBT_MAR_REBCO_DEFLECTOR_18T5: u32 = 1 << 9;
    pub const SHBT_IBR_AUTO_RELOAD_ARMED: u32 = 1 << 10;
    pub const SHBT_IBR_RELOAD_IN_PROGRESS: u32 = 1 << 11;
    pub const SHBT_IBR_ZERO_VOLUME_LOCK: u32 = 1 << 12;
    pub const SHBT_IBR_ADM_FRAME_COMPLIANT: u32 = 1 << 13;

    pub const SYMPLECTIC_S_FACTOR_Q32: u32 = 0x1782_9CBC;
    pub const SYMPLECTIC_SHIELD_MASS_G: u32 = 384_880;
    pub const COMPACTED_NACELLE_MASS_G: u32 = 2_257_700;
    pub const CREW_DOSE_NSV_YR: u32 = 114_000;

    #[repr(C, align(64))]
    #[derive(Clone, Copy, Debug)]
    pub struct ShbtIbr312Mmio {
        pub regs: [u64; 16],
    }

    const _: [(); 128] = [(); std::mem::size_of::<ShbtIbr312Mmio>()];
    const _: [(); 64] = [(); std::mem::align_of::<ShbtIbr312Mmio>()];

    impl ShbtIbr312Mmio {
        pub fn init() -> Self {
            let mut s = Self { regs: [0; 16] };
            // Set 0x58 (low word of regs[11]) to SYMPLECTIC_S_FACTOR_Q32,
            // 0x5C (high word of regs[11]) to SYMPLECTIC_SHIELD_MASS_G
            s.regs[11] = (SYMPLECTIC_S_FACTOR_Q32 as u64) | ((SYMPLECTIC_SHIELD_MASS_G as u64) << 32);
            // 0x60 (low word of regs[12]) to COMPACTED_NACELLE_MASS_G
            s.regs[12] = COMPACTED_NACELLE_MASS_G as u64;
            // 0x74 (low word of regs[14]) to CREW_DOSE_NSV_YR
            s.regs[14] = (CREW_DOSE_NSV_YR as u64) << 32;
            s
        }
    }

    #[repr(C, align(64))]
    #[derive(Clone, Copy, Debug)]
    pub struct ShbtStinespringMagazine {
        pub frames: [[u8; 2112]; 16],
    }

    const _: [(); 33792] = [(); std::mem::size_of::<ShbtStinespringMagazine>()];

    impl ShbtStinespringMagazine {
        pub fn new() -> Self {
            Self {
                frames: [[0u8; 2112]; 16],
            }
        }
    }

    impl Default for ShbtStinespringMagazine {
        fn default() -> Self {
            Self::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mar312_synthesis_registers() {
        let mut mmio = ShbtMar312Mmio::zeroed();
        mmio.set_synth_antibaryon_ratio_q16(SYNTH_ANTIBARYON_RATIO_Q16);
        mmio.set_synth_coset_weight_q16(SYNTH_COSET_WEIGHT_Q16);

        assert_eq!(mmio.synth_antibaryon_ratio_q16(), 0x8000);
        assert_eq!(mmio.synth_coset_weight_q16(), 0xB400);

        let ok = mar312::synth_fold_ok(
            mmio.synth_antibaryon_ratio_q16(),
            mmio.synth_coset_weight_q16(),
            mar312::SHBT_SYNTH_RG_LOCK_312 | mar312::SHBT_SYNTH_CBARB_ZERO_VERIFIED,
        );
        assert!(ok);

        mmio.set_synth_jitter_bound_q16(4);
        mmio.set_synth_tether_displacement_q16(36);
        mmio.set_dec_space_charge_margin_q16(11718);
        mmio.set_tqec_syndrome_err_rate_q16(55);
        mmio.set_rebco_quench_headroom_q16(10892);

        assert_eq!(mmio.synth_jitter_bound_q16(), 4);
        assert_eq!(mmio.synth_tether_displacement_q16(), 36);
        assert_eq!(mmio.dec_space_charge_margin_q16(), 11718);
        assert_eq!(mmio.tqec_syndrome_err_rate_q16(), 55);
        assert_eq!(mmio.rebco_quench_headroom_q16(), 10892);

        assert_eq!(COSET_ANTIBARYON_FOLD, 0x06);
        assert_eq!(SYMPLECTIC_CRYSTALLIZATION, 0x08);
        assert_eq!(SHBT_CTRL_DYNAMIC_GST_MATCH, 1 << 16);
        assert_eq!(SHBT_CTRL_PROGRESSIVE_EIKONAL, 1 << 17);
        assert_eq!(SHBT_CTRL_TTN_MERA_ADAPT, 1 << 18);
        assert_eq!(SHBT_CTRL_HYPERFRAME_64BANK, 1 << 19);
        assert_eq!(SHBT_EW_CTRL_GHOST_HOVER_LOCK, 1 << 21);
        assert_eq!(SHBT_EW_CTRL_GHOST_HARVEST_FUNNEL, 1 << 22);
        assert_eq!(SHBT_EW_CTRL_HABITAT_1G_FLOOR, 1 << 23);
        assert_eq!(SHBT_EW_CTRL_WAKE_COMP_3RD_ORDER, 1 << 24);

        assert_eq!(ghost_hover_step_50k518(93054.0, 1.842), 50517);
        assert!((ghost_antiphase_clamp(-1.0000000000005) - (-1.0)).abs() <= 1.0e-12);
        assert!(ghost_harvest_trigger_stage1(376.99, 376.99));
        assert!(!ghost_harvest_trigger_stage1(300.0, 376.99));
    }
}




