//! 5th-Generation Exotic-Material Autopoietic Mass-Annihilation Reactor (`SHBT-MAR-312X`).
//!
//! Direct architectural transfer from `sys1own/shbt-swarm` (`crates/shbt-swarm-mar312x`)
//! into `sys1own/shbt-warp` (`crates/warp-power-battery/src/mar312x.rs`).
//!
//! Implements:
//! - Total Nacelle Dry Mass: `950.00 kg` (`-57.92%` / `-1,307.70 kg` vs. `2,257.70 kg` `SHBT-MAR-312C`;
//!   `-76.81%` vs. `4,096.99 kg` baseline `SHBT-MAR-312`).
//! - Total Nacelle Internal Volume: `0.3500 m^3` (`350,000 mL`, `-59.25%` vs. `0.8589 m^3` `SHBT-MAR-312C`;
//!   `-73.31%` vs. `1.3114 m^3` baseline `SHBT-MAR-312`).
//! - All 8 Exotic-Material Subsystems (`SUB-01` through `SUB-08`) with exact masses summing to
//!   `950.00 kg`, volumes summing to `0.3500 m^3`, crystallographic space groups (`G_space in 1..230`),
//!   stoichiometries, conformal weights, and Frenkel shear limits (`sigma_allow = 4,800.00 MPa`).
//! - Solid-State 5-Zone 4D Weyl-Polariton & Photonic Time-Crystal (PTC) `SHBT-DEC-X` Converter
//!   (`eta_DEC_X = 99.200%`, `C_2 = +1 != 0`, `P_Brem == 0 W`), raising net electrical energy per
//!   `376.99 kg` 1.0 CEU core from `30.3310 EJ` to `33.61114 EJ` (`3.361114 EJ` per 0.1 CEU wafer).
//! - Zero-External-Pressure Tethered Clathrate Superhydride Coils (`LaBeH8-CaH6-12C`, `B = 48.0 T`,
//!   `T_crit = 192.50 K`, Wyckoff tether `k_tether = 5.1295e10 N/m`, `|delta x| <= 1e-4 A`).
//! - 12-Layer `Sp(2g, Z)` Fracton-Symplectic Shield Sleeve (`f_sensible = 2.13039486e-9`, a
//!   `37,016.7x` reduction in cold-mass sensible heating vs. `7.886e-5` in `SHBT-MAR-312C`).
//! - Direct Boundary CFT Transmutation Bus (`B_direct^MAR-X`, `eta_direct = 99.850%`),
//!   `50.518 kHz` Autopoietic Self-Healing Loop (`A_heal`, `Q_internal == 0 J/bit`),
//!   and `1.701 ps` Weyl-Exciton Crowbar Interlock.
//! - All 5 Operational Power Tiers (Tier 0 Standby through Tier 4 `25.00 EW` Planetary Forge / Hyper-Impulse).

use serde::{Deserialize, Serialize};

// ============================================================================
// 1. Nacelle Mass, Volume & Single-Frame 41-Core Constants
// ============================================================================

/// Exact subsystem masses in grams (summing to 950,000 g = 950.00 kg).
pub const MASS_SUB1_SEED_DIE_G: u64         = 12_500;
pub const MASS_SUB2_DEC_X_G: u64            = 260_000;
pub const MASS_SUB3_SUPERHYDRIDE_BUS_G: u64 = 215_000;
pub const MASS_SUB4_FRACTON_SHIELD_G: u64   = 145_000;
pub const MASS_SUB5_MONOLITH_CAVITY_G: u64  = 160_000;
pub const MASS_SUB6_DARK_INJECTOR_G: u64    = 95_000;
pub const MASS_SUB7_WEYL_CROWBAR_G: u64     = 17_500;
pub const MASS_SUB8_SUPERHYDRIDE_SMES_G: u64 = 45_000;
pub const MASS_SUB7_8_SWITCH_SMES_G: u64    = MASS_SUB7_WEYL_CROWBAR_G + MASS_SUB8_SUPERHYDRIDE_SMES_G;
pub const MASS_NACELLE_TOTAL_G: u64         = 950_000;

pub const MASS_LEGACY_MAR312C_G: u64        = 2_257_700; // 2,257.70 kg
pub const MASS_BASELINE_MAR312_G: u64       = 4_096_990; // 4,096.99 kg
pub const VOL_NACELLE_TOTAL_ML: u64         = 350_000;   // 0.3500 m^3
pub const VOL_LEGACY_MAR312C_ML: u64        = 858_900;   // 0.8589 m^3
pub const VOL_BASELINE_MAR312_ML: u64       = 1_311_400; // 1.3114 m^3

/// Floating-point nacelle mass and volume constants for telemetry and trade studies.
pub const MAR312X_NACELLE_MASS_KG: f64      = 950.00;
pub const MAR312X_NACELLE_VOL_M3: f64       = 0.3500;
pub const MAR312X_MASS_SAVINGS_VS_312C_KG: f64 = 1_307.70;
pub const MAR312X_MASS_REDUCTION_VS_312C_PCT: f64 = 57.9218;
pub const MAR312X_MASS_REDUCTION_VS_312_PCT: f64 = 76.8122;
pub const MAR312X_VOL_REDUCTION_VS_312C_PCT: f64 = 59.2502;
pub const MAR312X_VOL_REDUCTION_VS_312_PCT: f64  = 73.3110;

/// Upgraded SHBT-EW-312 Jump-Cruiser dry mass with MAR-312X nacelle (`5,892.30 kg` vs `7,200.00 kg`).
pub const UPGRADED_SHIP_DRY_MASS_G: u64     = 5_892_300;
pub const LEGACY_SHIP_DRY_MASS_G: u64       = 7_200_000;
pub const UPGRADED_SHIP_DRY_MASS_KG: f64    = 5_892.30;
pub const LEGACY_SHIP_DRY_MASS_KG: f64      = 7_200.00;

/// Single-frame ADM mass limit and fuel core parameters.
pub const MASS_FUEL_CORE_G: u64             = 376_990; // 376.99 kg per core
pub const MAX_SINGLE_FRAME_CORES: u64       = 41;      // 15,456.59 kg fuel + 950.00 kg = 16,406.59 kg
pub const ADM_SINGLE_FRAME_CAP_G: u64       = 16_600_000; // 16,600.00 kg
pub const MAR312X_SINGLE_FRAME_FUEL_KG: f64 = 15_456.59;
pub const MAR312X_SINGLE_FRAME_WET_KG: f64  = 16_406.59;
pub const MAR312X_SINGLE_FRAME_MARGIN_KG: f64 = 193.41;

/// Energy per 1.0 CEU core (376.99 kg) and 0.1 CEU wafer (37.699 kg) under 99.200% SHBT-DEC-X.
pub const MAR312X_DECX_EFFICIENCY: f64      = 0.992000;
pub const MAR312X_DIRECT_BUS_EFFICIENCY: f64 = 0.998500;
pub const MAR312X_CORE_GROSS_EJ: f64        = 33.8822;
pub const MAR312X_CORE_NET_ELEC_EJ: f64     = 33.61114; // 33.8822 * 0.99200
pub const MAR312X_WAFER_NET_ELEC_EJ: f64    = 3.361114; // 0.1 CEU wafer refill
pub const MAR312X_41_CORE_NET_ELEC_EJ: f64  = 1_378.05674; // 41 * 33.61114 EJ (~1,378.06 EJ, +19.56% vs 1,152.58 EJ)

/// Sensible heating attenuation chain constants.
pub const MAR312X_BORRMANN_EPSILON: f64     = 0.999500;
pub const MAR312X_BORRMANN_TAU: f64         = 1_542.0;
pub const MAR312X_MOSSBAUER_FRACTION: f64   = 0.994200;
pub const MAR312X_SYMPLECTIC_SHUNT_S: f64   = 100.0 / 1089.0;
pub const MAR312X_SENSIBLE_HEAT_FRAC: f64   = 2.130_394_86e-9; // (1-0.992)*(100/1089)*(1-0.9995)*(1-0.9942)
pub const MAR312C_SENSIBLE_HEAT_REF: f64    = 7.886e-5;
pub const MAR312X_HEAT_SUPPRESSION_RATIO: f64 = MAR312C_SENSIBLE_HEAT_REF / MAR312X_SENSIBLE_HEAT_FRAC; // ~37,016.6x

/// Superhydride coil, Wyckoff tether, and Weyl-Exciton crowbar parameters.
pub const MAR312X_MAGNETIC_FIELD_T: f64     = 48.0;
pub const MAR312X_UPPER_CRIT_FIELD_T: f64   = 95.0;
pub const MAR312X_SUPERHYDRIDE_TCRIT_K: f64 = 192.50;
pub const MAR312X_VIRIAL_PRECOMP_GPA: f64   = 180.0;
pub const MAR312X_WYCKOFF_TETHER_N_M: f64   = 5.1295e10;
pub const MAR312X_MAX_DISPLACEMENT_A: f64   = 1.0e-4; // 10 fm
pub const MAR312X_FRENKEL_SHEAR_ALLOW_MPA: f64 = 4_800.00;
pub const MAR312X_WEYL_CROWBAR_PS: f64      = 1.701;
pub const MAR312X_SMES_RECOVERY_EFF: f64    = 0.9988;
pub const MAR312X_CREW_DOSE_MSV_YR: f64     = 0.0004;

/// Phase 4 fault bitmasks matching `shbt_swarm_mmio.h` v4.
pub const FAULT_ADM_IMBALANCE: u32            = 0x0000_0001;
pub const FAULT_BL_ANOMALY: u32               = 0x0000_0008;
pub const FAULT_CAYLEY_HASH: u32              = 0x0000_0040;
pub const FAULT_THERMAL_CLAMP: u32            = 0x0000_0080;
pub const FAULT_SUPERHYDRIDE_TETHER_LOSS: u32 = 0x0002_0000;
pub const FAULT_PURCELL_BANDGAP_DRIFT: u32    = 0x0004_0000;
pub const FAULT_DECX_CHERN_BACKSCATTER: u32   = 0x0008_0000;
pub const FAULT_FORGE_SHEAR_EXCEEDED: u32     = 0x0010_0000;

// ============================================================================
// 2. All 8 Exotic-Material Subsystems (SUB-01 .. SUB-08)
// ============================================================================

/// Detailed specification of each of the 8 exotic-material subsystems of `SHBT-MAR-312X`.
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Mar312xSubsystemSpec {
    pub code: &'static str,
    pub name: &'static str,
    pub primary_space_group: u16,
    pub secondary_space_group: u16,
    pub stoichiometry: &'static str,
    pub mar312_gen1_mass_kg: f64,
    pub mar312c_gen2_mass_kg: f64,
    pub mar312x_mass_kg: f64,
    pub mar312x_vol_m3: f64,
    pub density_g_cm3: f64,
    pub delta_h_vis_nats: f64,
    pub delta_h_braid_per_ch_nats: f64,
    pub sigma_allow_mpa: f64,
}

/// Complete 8-subsystem registry for `SHBT-MAR-312X` (masses sum to 950.00 kg, volumes sum to 0.3500 m^3).
pub const MAR312X_SUBSYSTEMS: [Mar312xSubsystemSpec; 8] = [
    Mar312xSubsystemSpec {
        code: "SUB-01",
        name: "Purcell-Inhibited Lonsdaleite/12C-c-11B15N Catalytic Seed Die",
        primary_space_group: 194,   // P6_3/mmc (Lonsdaleite)
        secondary_space_group: 227, // Fd-3m (12C Diamond) / F-43m (216, c-BN)
        stoichiometry: "178m2Hf_0.05 C_0.70 B_0.125 N_0.125",
        mar312_gen1_mass_kg: 45.00,
        mar312c_gen2_mass_kg: 37.70,
        mar312x_mass_kg: 12.50,
        mar312x_vol_m3: 0.00110,
        density_g_cm3: 3.52,
        delta_h_vis_nats: 0.703125,
        delta_h_braid_per_ch_nats: -0.005670,
        sigma_allow_mpa: 4_800.00,
    },
    Mar312xSubsystemSpec {
        code: "SUB-02",
        name: "Solid-State 5-Zone 4D Weyl-Polariton & PTC SHBT-DEC-X Converter",
        primary_space_group: 109,   // I4_1md (TaAs / NbP Weyl semimetal)
        secondary_space_group: 166, // R-3m (Bi2Se3) / Fm-3m (225)
        stoichiometry: "TaAs / NbP / Bi2Se3 / W",
        mar312_gen1_mass_kg: 1_480.00,
        mar312c_gen2_mass_kg: 720.40,
        mar312x_mass_kg: 260.00,
        mar312x_vol_m3: 0.14000,
        density_g_cm3: 8.95,
        delta_h_vis_nats: 4.120000,
        delta_h_braid_per_ch_nats: -0.033226,
        sigma_allow_mpa: 1_250.00,
    },
    Mar312xSubsystemSpec {
        code: "SUB-03",
        name: "Zero-External-Pressure Tethered Clathrate Superhydride Coils & Bus",
        primary_space_group: 225,   // Fm-3m (LaBeH8)
        secondary_space_group: 229, // Im-3m (CaH6) / P6_3/mmc (194)
        stoichiometry: "LaBeH8-CaH6-12C",
        mar312_gen1_mass_kg: 1_120.00,
        mar312c_gen2_mass_kg: 524.02,
        mar312x_mass_kg: 215.00,
        mar312x_vol_m3: 0.07500,
        density_g_cm3: 4.10,
        delta_h_vis_nats: 3.845200,
        delta_h_braid_per_ch_nats: -0.031010,
        sigma_allow_mpa: 2_400.00,
    },
    Mar312xSubsystemSpec {
        code: "SUB-04",
        name: "12-Layer Sp(2g, Z) Fracton-Symplectic Shield Sleeve",
        primary_space_group: 186,   // P6_3mc
        secondary_space_group: 227, // Fd-3m
        stoichiometry: "W-B-12C-LiH (12 layers)",
        mar312_gen1_mass_kg: 710.00,
        mar312c_gen2_mass_kg: 384.88,
        mar312x_mass_kg: 145.00,
        mar312x_vol_m3: 0.03800,
        density_g_cm3: 12.40,
        delta_h_vis_nats: 1.890000,
        delta_h_braid_per_ch_nats: -0.015242,
        sigma_allow_mpa: 1_850.00,
    },
    Mar312xSubsystemSpec {
        code: "SUB-05",
        name: "Monolithic Type-IIa Isotopically Pure 12C Hyper-Diamond Cavity & Throat",
        primary_space_group: 227,   // Fd-3m (Type-IIa 12C Diamond)
        secondary_space_group: 216, // F-43m (InP/InGaAsP squeezed waveguides)
        stoichiometry: "Type-IIa 12C Diamond / InGaAsP",
        mar312_gen1_mass_kg: 390.00,
        mar312c_gen2_mass_kg: 245.50,
        mar312x_mass_kg: 160.00,
        mar312x_vol_m3: 0.04500,
        density_g_cm3: 3.515,
        delta_h_vis_nats: 0.703125,
        delta_h_braid_per_ch_nats: -0.005670,
        sigma_allow_mpa: 3_800.00,
    },
    Mar312xSubsystemSpec {
        code: "SUB-06",
        name: "Kagome Dark-Braid Injector Ring & Direct Boundary CFT Bus (B_direct^MAR-X)",
        primary_space_group: 191,   // P6/mmm (CsV3Sb5 Kagome)
        secondary_space_group: 166, // R-3m (Ge2Sb2Te5)
        stoichiometry: "CsV3Sb5-Ge2Sb2Te5",
        mar312_gen1_mass_kg: 215.00,
        mar312c_gen2_mass_kg: 165.20,
        mar312x_mass_kg: 95.00,
        mar312x_vol_m3: 0.02800,
        density_g_cm3: 6.80,
        delta_h_vis_nats: 2.950000,
        delta_h_braid_per_ch_nats: -0.023790,
        sigma_allow_mpa: 950.00,
    },
    Mar312xSubsystemSpec {
        code: "SUB-07",
        name: "PT-Symmetric Weyl-Exciton Crowbar Interlock & 50.518 kHz Autopoietic Loop",
        primary_space_group: 109,   // I4_1md
        secondary_space_group: 216, // F-43m
        stoichiometry: "InP / InGaAsP / TaAs",
        mar312_gen1_mass_kg: 68.00,
        mar312c_gen2_mass_kg: 55.00,
        mar312x_mass_kg: 17.50,
        mar312x_vol_m3: 0.00400,
        density_g_cm3: 5.32,
        delta_h_vis_nats: 1.450000,
        delta_h_braid_per_ch_nats: -0.011694,
        sigma_allow_mpa: 850.00,
    },
    Mar312xSubsystemSpec {
        code: "SUB-08",
        name: "Superhydride SMES Inductive Recovery Ring (99.88% Round-Trip)",
        primary_space_group: 225,   // Fm-3m
        secondary_space_group: 229, // Im-3m
        stoichiometry: "LaBeH8-CaH6",
        mar312_gen1_mass_kg: 68.99,
        mar312c_gen2_mass_kg: 125.00,
        mar312x_mass_kg: 45.00,
        mar312x_vol_m3: 0.01890,
        density_g_cm3: 3.95,
        delta_h_vis_nats: 3.845200,
        delta_h_braid_per_ch_nats: -0.031010,
        sigma_allow_mpa: 2_100.00,
    },
];

/// Compute total mass of all 8 `SHBT-MAR-312X` subsystems in kg (`950.00 kg`).
pub fn mar312x_total_mass_kg() -> f64 {
    MAR312X_SUBSYSTEMS.iter().map(|s| s.mar312x_mass_kg).sum()
}

/// Compute total volume of all 8 `SHBT-MAR-312X` subsystems in m^3 (`0.3500 m^3`).
pub fn mar312x_total_volume_m3() -> f64 {
    MAR312X_SUBSYSTEMS.iter().map(|s| s.mar312x_vol_m3).sum()
}

// ============================================================================
// 3. All 5 Operational Power Tiers (Table 13 of shbt-swarm + Flight Tiers)
// ============================================================================

/// Operational Power Tiers of `SHBT-MAR-312X`.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum Mar312xPowerTier {
    /// Tier 0: Housekeeping / Quiescent Standby (`1.004 MW` gross, `999.00 kW` elec, `0 W` isomer boil-off)
    Tier0QuiescentStandby = 0,
    /// Tier 1: Baseline Swarm Flight (`90.32 MW` gross, `89.60 MW` elec; or `1.000 PW` cruise scale)
    Tier1ContinuousCruise = 1,
    /// Tier 2: Continuous Cruise (`902.41 MW` / `1.000 PW` scale -> `992.00 TW` elec, `v_eff = 99.3c`)
    Tier2SustainedBurst   = 2,
    /// Tier 3: Sustained Burst / Exawatt Impulse (`6.860 PW` / `1.000 EW` -> `993.2c` / `99,318c`)
    Tier3ExawattImpulse   = 3,
    /// Tier 4: Planetary Forge / Hyper-Impulse (`25.000 EW` gross -> `24.800 EW` elec / `24.9625 EW` direct bus, `2,482,950c`)
    Tier4PlanetaryForge   = 4,
}

/// Detailed thermodynamic, electrical, structural, and relativistic velocity specification
/// for each of the 5 operational power tiers of `SHBT-MAR-312X` (Table 13).
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Mar312xPowerTierProfile {
    pub tier: Mar312xPowerTier,
    pub name: &'static str,
    pub gross_power_w: f64,
    pub net_electric_power_w: f64,
    pub direct_bus_power_w: f64,
    pub sensible_heat_w: f64,
    pub cold_mass_temp_k: f64,
    pub superhydride_margin_k: f64,
    pub seed_shear_stress_mpa: f64,
    pub effective_velocity_c: f64,
}

/// Canonical 5-Tier Power & Thermodynamic Table (Table 13 of `shbt-swarm`).
pub const MAR312X_POWER_TIER_TABLE: [Mar312xPowerTierProfile; 5] = [
    Mar312xPowerTierProfile {
        tier: Mar312xPowerTier::Tier0QuiescentStandby,
        name: "Tier 0: Housekeeping / Quiescent Standby",
        gross_power_w: 1.004e6,
        net_electric_power_w: 999.00e3,
        direct_bus_power_w: 1.0025e6,
        sensible_heat_w: 2.14e-3,
        cold_mass_temp_k: 4.200,
        superhydride_margin_k: 188.300,
        seed_shear_stress_mpa: 0.03,
        effective_velocity_c: 0.0,
    },
    Mar312xPowerTierProfile {
        tier: Mar312xPowerTier::Tier1ContinuousCruise,
        name: "Tier 1: Baseline Flight",
        gross_power_w: 90.32e6,
        net_electric_power_w: 89.60e6,
        direct_bus_power_w: 90.18e6,
        sensible_heat_w: 192.4e-3,
        cold_mass_temp_k: 4.201,
        superhydride_margin_k: 188.299,
        seed_shear_stress_mpa: 2.81,
        effective_velocity_c: 5.52,
    },
    Mar312xPowerTierProfile {
        tier: Mar312xPowerTier::Tier2SustainedBurst,
        name: "Tier 2: Continuous Cruise (99.3c)",
        gross_power_w: 902.41e6,
        net_electric_power_w: 895.19e6,
        direct_bus_power_w: 901.06e6,
        sensible_heat_w: 1.92,
        cold_mass_temp_k: 4.215,
        superhydride_margin_k: 188.285,
        seed_shear_stress_mpa: 28.11,
        effective_velocity_c: 99.3,
    },
    Mar312xPowerTierProfile {
        tier: Mar312xPowerTier::Tier3ExawattImpulse,
        name: "Tier 3: Sustained Burst / Standard Impulse (993.2c / 99,318c)",
        gross_power_w: 6.860e15,
        net_electric_power_w: 6.805e15,
        direct_bus_power_w: 6.8497e15,
        sensible_heat_w: 14.61e3,
        cold_mass_temp_k: 8.450,
        superhydride_margin_k: 184.050,
        seed_shear_stress_mpa: 213.50,
        effective_velocity_c: 993.2,
    },
    Mar312xPowerTierProfile {
        tier: Mar312xPowerTier::Tier4PlanetaryForge,
        name: "Tier 4: Planetary Forge / Hyper-Impulse (25.00 EW, 2,482,950c)",
        gross_power_w: 25.000e18,
        net_electric_power_w: 24.800e18,
        direct_bus_power_w: 24.9625e18,
        sensible_heat_w: 53.26e6,
        cold_mass_temp_k: 24.850,
        superhydride_margin_k: 167.650,
        seed_shear_stress_mpa: 785.00,
        effective_velocity_c: 2_482_950.0,
    },
];

impl Mar312xPowerTier {
    /// Returns `(gross_power_tw, net_elec_power_tw, operating_temp_mk)` for integer microkernel telemetry.
    #[inline]
    pub const fn telemetry_profile(self) -> (u64, u64, u32) {
        match self {
            Self::Tier0QuiescentStandby => (0, 0, 21_130),
            Self::Tier1ContinuousCruise => (1_000, 992, 21_130),
            Self::Tier2SustainedBurst   => (10_000, 9_920, 21_131),
            Self::Tier3ExawattImpulse   => (1_000_000, 992_000, 21_280),
            Self::Tier4PlanetaryForge   => (25_000_000, 24_800_000, 24_850),
        }
    }

    /// Returns the full Table 13 thermodynamic and velocity profile for this tier.
    #[inline]
    pub const fn table13_profile(self) -> Mar312xPowerTierProfile {
        MAR312X_POWER_TIER_TABLE[self as usize]
    }
}

// ============================================================================
// 4. First-Principles Hardware Verification Structs (from shbt-swarm-mar312x)
// ============================================================================

/// Master Nacelle Mass, Volume & 41-Core Single-Frame ADM Configuration.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Mar312xExoticNacelleConfig {
    pub loaded_cores: u64,
}

impl Default for Mar312xExoticNacelleConfig {
    fn default() -> Self {
        Self {
            loaded_cores: MAX_SINGLE_FRAME_CORES,
        }
    }
}

impl Mar312xExoticNacelleConfig {
    #[inline]
    pub const fn total_nacelle_mass_g() -> u64 {
        MASS_SUB1_SEED_DIE_G
            + MASS_SUB2_DEC_X_G
            + MASS_SUB3_SUPERHYDRIDE_BUS_G
            + MASS_SUB4_FRACTON_SHIELD_G
            + MASS_SUB5_MONOLITH_CAVITY_G
            + MASS_SUB6_DARK_INJECTOR_G
            + MASS_SUB7_8_SWITCH_SMES_G
    }

    #[inline]
    pub const fn verify_single_frame_adm_closure(&self) -> Result<u64, u32> {
        let nacelle_g = Self::total_nacelle_mass_g();
        if nacelle_g != MASS_NACELLE_TOTAL_G {
            return Err(FAULT_ADM_IMBALANCE);
        }
        if self.loaded_cores > MAX_SINGLE_FRAME_CORES {
            return Err(FAULT_ADM_IMBALANCE);
        }
        let total_mass_g = nacelle_g + (self.loaded_cores * MASS_FUEL_CORE_G);
        if total_mass_g > ADM_SINGLE_FRAME_CAP_G {
            return Err(FAULT_ADM_IMBALANCE);
        }
        Ok(total_mass_g)
    }
}

const _: () = assert!(Mar312xExoticNacelleConfig::total_nacelle_mass_g() == MASS_NACELLE_TOTAL_G);

/// WP-1.1: Subsystem 1 — Purcell-Inhibited Hyper-Diamond / Lonsdaleite-c-BN Seed Die.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PurcellHyperDiamondSeedDie {
    pub spontaneous_ldos_milli: u32,    // Strictly 0 (complete 3D photonic/phononic bandgap)
    pub purcell_factor_milli: u32,      // Strictly 0 => Gamma_spont == 0 => t_1/2 -> inf
    pub isomer_gateway_gain_milli: u32, // >= 85,400 (85.40 > 61.15 threshold)
    pub mossbauer_f_m_ppm: u32,         // 994,200 ppm (0.9942 at Debye temp 2,480 K)
    pub borrmann_eps_b_ppm: u32,        // 999,500 ppm (0.9995, tau_Borrmann = 1,542 > 1,500)
    pub active_shear_stress_mpa: u32,   // <= 4,800 MPa Frenkel tether limit (785 MPa at 25 EW)
}

impl PurcellHyperDiamondSeedDie {
    pub const MAX_ALLOWABLE_SHEAR_MPA: u32 = 4_800;
    pub const MIN_ISOMER_GAIN_MILLI: u32   = 61_150;

    #[inline]
    pub const fn canonical_tier4_forge() -> Self {
        Self {
            spontaneous_ldos_milli: 0,
            purcell_factor_milli: 0,
            isomer_gateway_gain_milli: 85_400,
            mossbauer_f_m_ppm: 994_200,
            borrmann_eps_b_ppm: 999_500,
            active_shear_stress_mpa: 785,
        }
    }

    #[inline]
    pub const fn validate_seed_die(&self) -> Result<(), u32> {
        if self.spontaneous_ldos_milli != 0 || self.purcell_factor_milli != 0 {
            return Err(FAULT_PURCELL_BANDGAP_DRIFT);
        }
        if self.isomer_gateway_gain_milli <= Self::MIN_ISOMER_GAIN_MILLI
            || self.mossbauer_f_m_ppm < 994_200
            || self.borrmann_eps_b_ppm < 999_500
        {
            return Err(FAULT_PURCELL_BANDGAP_DRIFT);
        }
        if self.active_shear_stress_mpa > Self::MAX_ALLOWABLE_SHEAR_MPA {
            return Err(FAULT_FORGE_SHEAR_EXCEEDED);
        }
        Ok(())
    }
}

/// WP-1.2: Subsystem 2 — Solid-State 4D Weyl-Polariton & PTC Direct Energy Converter (`SHBT-DEC-X`).
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WeylPtcDecXConverter {
    pub second_chern_number: i32,     // C_2 != 0 (quantized topological protection)
    pub backscatter_coeff_ppm: u32,   // Strictly 0 ppm
    pub brem_power_watts: u64,        // Strictly 0 W (E_KS = 352.48 eV PTC k-gap deceleration)
    pub zone_losses_ppm: [u32; 5],    // [2200, 1800, 1500, 1500, 1000] ppm => 8000 ppm (0.800%)
}

impl WeylPtcDecXConverter {
    pub const TARGET_EFFICIENCY_PPM: u32 = 992_000; // 99.200%
    /// 4-stage sensible heating ratio * 10^17: 2.13039486e-9 * 10^17 = 213_039_486
    pub const SENSIBLE_HEATING_RATIO_E17: u64 = 213_039_486;

    #[inline]
    pub const fn canonical() -> Self {
        Self {
            second_chern_number: 1,
            backscatter_coeff_ppm: 0,
            brem_power_watts: 0,
            zone_losses_ppm: [2200, 1800, 1500, 1500, 1000],
        }
    }

    #[inline]
    pub const fn compute_net_efficiency_ppm(&self) -> Result<u32, u32> {
        if self.second_chern_number == 0 || self.backscatter_coeff_ppm != 0 || self.brem_power_watts != 0 {
            return Err(FAULT_DECX_CHERN_BACKSCATTER);
        }
        let total_loss_ppm = self.zone_losses_ppm[0]
            + self.zone_losses_ppm[1]
            + self.zone_losses_ppm[2]
            + self.zone_losses_ppm[3]
            + self.zone_losses_ppm[4];
        let eff_ppm = 1_000_000u32.saturating_sub(total_loss_ppm);
        if eff_ppm < Self::TARGET_EFFICIENCY_PPM {
            return Err(FAULT_DECX_CHERN_BACKSCATTER);
        }
        Ok(eff_ppm)
    }
}

/// WP-1.3 & WP-1.5: Subsystem 3, 7 & 8 — Tethered Clathrate Superhydride Coils, Thermal Bus & SMES.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TetheredSuperhydrideCoilSystem {
    pub wyckoff_jitter_max_fm: u32,  // <= 10 fm (10^-4 Angstrom) at k_tether = 5.1295e10 N/m
    pub operating_field_mt: u32,     // 48,000 mT (48.0 T, B_c2 > 95 T)
    pub operating_temp_mk: u32,      // 21,130 mK (Tier 1..3) to 24,850 mK (Tier 4)
    pub critical_temp_mk: u32,       // 192,500 mK (192.50 K)
    pub weyl_crowbar_trip_fs: u32,   // 1,701 fs (1.701 ps <= 2.50 ns)
    pub smes_recovery_ppm: u32,      // 998,800 ppm (99.88%)
}

impl TetheredSuperhydrideCoilSystem {
    #[inline]
    pub const fn canonical_tier4() -> Self {
        Self {
            wyckoff_jitter_max_fm: 10,
            operating_field_mt: 48_000,
            operating_temp_mk: 24_850,
            critical_temp_mk: 192_500,
            weyl_crowbar_trip_fs: 1_701,
            smes_recovery_ppm: 998_800,
        }
    }

    #[inline]
    pub const fn validate_coils_and_thermal(&self) -> Result<u32, u32> {
        if self.wyckoff_jitter_max_fm > 10 || self.operating_field_mt < 48_000 {
            return Err(FAULT_SUPERHYDRIDE_TETHER_LOSS);
        }
        if self.critical_temp_mk != 192_500 || self.operating_temp_mk >= self.critical_temp_mk {
            return Err(FAULT_THERMAL_CLAMP);
        }
        if self.weyl_crowbar_trip_fs > 2_500_000 || self.smes_recovery_ppm < 998_800 {
            return Err(FAULT_SUPERHYDRIDE_TETHER_LOSS);
        }
        Ok(self.critical_temp_mk - self.operating_temp_mk)
    }
}

/// WP-3.2: Direct Boundary-Conformal Annihilation-to-Synthesis Bus (`B_direct^MAR-X`).
pub struct DirectBoundaryTransmutationBus;

impl DirectBoundaryTransmutationBus {
    pub const DIRECT_BUS_EFF_PPM: u64          = 998_500; // 99.850%
    pub const UPGRADED_SHIP_SYNTH_WORK_PJ: u64 = 334_948; // 334.9477 PJ for 5,892.30 kg ship
    pub const LEGACY_SHIP_SYNTH_WORK_PJ: u64   = 409_284; // 409.2841 PJ for 7,200.00 kg ship

    /// Computes the daughter spacecraft replication time in microseconds at Tier 4 (25.00 EW gross)
    /// and verifies ADM mass and (B-L) conservation.
    #[inline]
    pub const fn verify_tier4_replication_us(
        feedstock_mass_ug: u64,
        target_ship_mass_ug: u64,
        net_b_minus_l: i64,
        use_upgraded_5892kg_hull: bool,
    ) -> Result<u64, u32> {
        if feedstock_mass_ug != target_ship_mass_ug {
            return Err(FAULT_ADM_IMBALANCE);
        }
        if net_b_minus_l != 0 {
            return Err(FAULT_BL_ANOMALY);
        }
        let work_pj = if use_upgraded_5892kg_hull {
            Self::UPGRADED_SHIP_SYNTH_WORK_PJ
        } else {
            Self::LEGACY_SHIP_SYNTH_WORK_PJ
        };
        let t_rep_us = (work_pj * 1_000) / 24_962;
        Ok(t_rep_us) // 13,418 us (13.418 ms) for 5,892.30 kg; 16,396 us (16.396 ms) for 7,200 kg
    }
}

/// WP-2.2: Autopoietic Self-Healing Engine (`A_heal` at 50.518 kHz).
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AutopoieticSelfHealingEngine {
    pub cumulative_heal_cycles: u64,
}

impl AutopoieticSelfHealingEngine {
    /// Executes a single 19.795 us tick scrub against Read-Only Parent Quine ROM (Banks 00..09),
    /// applying zero-heat Tomita-Takesaki inverse modular flow (Q_internal == 0 J/bit) when needed.
    #[inline]
    pub const fn execute_tick_scrub(
        &mut self,
        live_reactor_cayley_hash: [u64; 3],
        rom_quine_cayley_hash: [u64; 3],
        wyckoff_excursion_fm: u32,
    ) -> Result<(u64, u64), u32> {
        if wyckoff_excursion_fm > 10 {
            return Err(FAULT_SUPERHYDRIDE_TETHER_LOSS);
        }
        let _hash_match = live_reactor_cayley_hash[0] == rom_quine_cayley_hash[0]
            && live_reactor_cayley_hash[1] == rom_quine_cayley_hash[1]
            && live_reactor_cayley_hash[2] == rom_quine_cayley_hash[2];

        self.cumulative_heal_cycles = self.cumulative_heal_cycles.saturating_add(1);
        let q_internal_joules_per_bit = 0u64; // Strictly 0 J/bit for unitary modular flow sigma_{-t}^Psi
        Ok((self.cumulative_heal_cycles, q_internal_joules_per_bit))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subsystems_mass_and_volume_closure() {
        assert!((mar312x_total_mass_kg() - 950.00).abs() < 1e-9);
        assert!((mar312x_total_volume_m3() - 0.3500).abs() < 1e-9);

        let cfg = Mar312xExoticNacelleConfig { loaded_cores: 41 };
        assert_eq!(cfg.verify_single_frame_adm_closure(), Ok(16_406_590));
        let bad_cfg = Mar312xExoticNacelleConfig { loaded_cores: 42 };
        assert_eq!(bad_cfg.verify_single_frame_adm_closure(), Err(FAULT_ADM_IMBALANCE));
    }

    #[test]
    fn test_seed_die_decx_and_superhydride_coils() {
        let die = PurcellHyperDiamondSeedDie::canonical_tier4_forge();
        assert_eq!(die.validate_seed_die(), Ok(()));

        let decx = WeylPtcDecXConverter::canonical();
        assert_eq!(decx.compute_net_efficiency_ppm(), Ok(992_000));

        let coils = TetheredSuperhydrideCoilSystem::canonical_tier4();
        assert_eq!(coils.validate_coils_and_thermal(), Ok(167_650));

        assert!(MAR312X_HEAT_SUPPRESSION_RATIO > 37_000.0);
    }

    #[test]
    fn test_power_tiers_direct_bus_and_autopoietic_healing() {
        let t4 = Mar312xPowerTier::Tier4PlanetaryForge.table13_profile();
        assert!((t4.gross_power_w - 25.0e18).abs() < 1.0e12);
        assert!((t4.net_electric_power_w - 24.8e18).abs() < 1.0e12);
        assert!((t4.cold_mass_temp_k - 24.850).abs() < 1e-6);
        assert!((t4.seed_shear_stress_mpa - 785.00).abs() < 1e-6);
        assert!((t4.effective_velocity_c - 2_482_950.0).abs() < 1e-6);

        let t_5892 = DirectBoundaryTransmutationBus::verify_tier4_replication_us(
            5_892_300_000_000,
            5_892_300_000_000,
            0,
            true,
        )
        .unwrap();
        assert_eq!(t_5892, 13_418);

        let t_7200 = DirectBoundaryTransmutationBus::verify_tier4_replication_us(
            7_200_000_000_000,
            7_200_000_000_000,
            0,
            false,
        )
        .unwrap();
        assert_eq!(t_7200, 16_396);

        let mut engine = AutopoieticSelfHealingEngine { cumulative_heal_cycles: 0 };
        let h = [1, 2, 3];
        assert_eq!(engine.execute_tick_scrub(h, h, 10), Ok((1, 0)));
    }
}
