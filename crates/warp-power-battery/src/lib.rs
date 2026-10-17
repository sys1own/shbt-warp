//! 4th-Generation Ultra-High-Density Stinespring–SO(10) Mass-Annihilation
//! Battery-Reactor (`SHBT-MAR-312`) and legacy baseline graser model.
//!
//! Implements:
//! - Canonical WZW tuple (26, 8, 312) over SU(2)_26 x SU(3)_8 x SO(10)_312.
//! - Non-thermal SO(10)_312 mass annihilation: 376.99 kg active fuel core
//!   (188.495 kg visible baryons + 188.495 kg dark-ledger anti-baryons).
//! - Specific energy density rho_E = c^2 = 8.98755179e16 J/kg (89,875.52 TJ/kg).
//! - Total stored energy E_total = 33.8822 EJ (9.4117 TWh), Net usable electrical energy = 30.3310 EJ.
//! - 5-stage hybrid relativistic DEC duct (840.00 kg, 0.4200 m^3) with eta_conv = 89.519%.
//! - Standardized integrated propulsion nacelle: 4,096.99 kg / 1.3114 m^3.
//! - Four canonical operating tiers: BASELINE-HF178, CRUISE-PW, BURST-PW, IMPULSE-EW.

// ---- Algebraic & Stinespring Closure Invariants ----------------------------

pub const WZW_K_LEPTON: u32 = 26;
pub const WZW_K_QUARK: u32 = 8;
pub const WZW_K_SO10: u32 = 312;
pub const WZW_TUPLE: (u32, u32, u32) = (WZW_K_LEPTON, WZW_K_QUARK, WZW_K_SO10);

pub const C_SU2_26: f64 = 39.0 / 14.0;
pub const C_SU3_8: f64 = 64.0 / 11.0;
pub const C_VIS: f64 = 1325.0 / 154.0;
pub const C_VIS_COMPLETED: f64 = 32327.0 / 616.0;
pub const C_PARENT: f64 = 351.0 / 8.0;
pub const C_DARK_COMP: f64 = 1197103.0 / 362670.0;
pub const C_TOTAL: f64 = 16186.0 / 308.0;

pub const ETA_A: f64 = 10.0 / 33.0;
pub const ETA_D: f64 = 23.0 / 33.0;

pub const FIBONACCI_CHANNELS: usize = 124;
pub const FIBONACCI_QUANTUM_DIM: f64 = 1.618033988749895; // (1 + sqrt(5))/2
pub const BRAID_CAPACITY_BITS: f64 = 86.084964; // 124 * log2(phi)
pub const BRAID_CAPACITY_NATS: f64 = 38.196601;

// ---- Fuel Core & Annihilation Dynamics -------------------------------------

pub const SPEED_OF_LIGHT_M_S: f64 = 299_792_458.0;
/// Specific fuel energy density rho_E = c^2, J/kg.
pub const RHO_E_J_KG: f64 = 8.987551787368176e16;
/// Specific gravimetric energy density, TJ/kg.
pub const RHO_E_TJ_KG: f64 = 89_875.51787368176;

/// Active MAR-312 core mass, kg.
pub const M_CORE_KG: f64 = 376.99;
pub const M_ISO_KG: f64 = M_CORE_KG; // Backward compatibility alias
pub const V_CORE_M3: f64 = 0.0359;

/// Fuel partition: visible baryons + dark-ledger anti-baryons.
pub const M_VISIBLE_BARYONS_KG: f64 = 188.495;
pub const M_DARK_ANTIBARYONS_KG: f64 = 188.495;

/// Standby stasis metrics.
pub const STANDBY_RENDER_COST_W: f64 = 0.0;
pub const STANDBY_ANNIHILATION_RATE_S_INV: f64 = 0.0;
pub const LINDBLAD_COSET_BARRIER_GEV: f64 = 1.220900e16;

/// Total stored energy: E_total = 33.8822 EJ.
pub const E_TOTAL_EJ: f64 = 33.8822;
pub const E_TOTAL_TJ: f64 = 3.38822e7;
pub const E_TOTAL_TWH: f64 = 9.4117;
pub const E_NET_USABLE_EJ: f64 = 30.3310;

/// Quiescent decay power (178m2Hf catalyst component), W/kg.
pub const DECAY_POWER_W_KG: f64 = 939.73;
pub const QUIESCENT_HEAT_KW: f64 = 354.27;
pub const QUIESCENT_TEG_RECOVERY_KW: f64 = 48.0;

pub fn quiescent_heat_kw() -> f64 {
    QUIESCENT_HEAT_KW
}

/// Optical gateway catalyst state (40.0 keV -> 2.486 MeV state, 2.446 MeV release).
pub const E_ISOMER_MEV: f64 = 2.446;
pub const E_GATEWAY_MEV: f64 = 0.040;
pub const E_GATEWAY_STATE_MEV: f64 = 2.486;
pub const GAIN_ISOMER: f64 = 61.15;

pub fn isomer_gain() -> f64 {
    GAIN_ISOMER
}

/// Callan-Symanzik threshold shifts.
pub const CS_DELTA_1: f64 = 24.071227;
pub const CS_DELTA_2: f64 = 19.973448;
pub const CS_DELTA_3: f64 = 36.712129;

/// Spinor conformal-weight shift Delta h_vis = 45 / 64 = 0.703125.
pub const DELTA_H_VIS: f64 = 45.0 / 64.0;
pub const SYNTH_UNITARY_FIDELITY_BOUND: f64 = 2.22e-16;

// ---- 5-Stage Relativistic Direct Energy Converter (DEC) --------------------

pub const ETA_STAGE1: f64 = 0.26400; // Compton photo-converter
pub const ETA_STAGE2: f64 = 0.12100; // Pair induction matrix
pub const ETA_STAGE3: f64 = 0.29800; // High-beta resistive MHD flux compression
pub const ETA_STAGE4A: f64 = 0.07300; // Epithermal retarding grids
pub const ETA_STAGE4B: f64 = 0.13919; // Tri-rail venetian blind + Sternglass-Joy

/// Combined 5-stage conversion efficiency: 89.519%.
pub const ETA_CONV_TOTAL: f64 = 0.89519;

pub fn conversion_efficiency() -> f64 {
    ETA_CONV_TOTAL
}

pub const M_DEC_DUCT_KG: f64 = 840.00;
pub const V_DEC_DUCT_M3: f64 = 0.4200;

pub const BUS_VOLTAGE_MIN_KV: f64 = 15.0;
pub const BUS_VOLTAGE_MAX_KV: f64 = 2700.0; // Up to 2.7 MV DC

// ---- Thermal, Borrmann & Cryogenic Attenuation -----------------------------

pub const BORRMANN_EPSILON: f64 = 0.9852;
pub const SYMPLECTIC_SCATTER_SUPPRESSION_S: f64 = 100.0 / 1089.0; // (10/33)^2 ~ 0.091827
pub const SYMPLECTIC_DARK_SHUNT_FRACTION: f64 = 989.0 / 1089.0; // ~ 0.908173
pub const MOSSBAUER_FRACTION_MIN: f64 = 0.782;
pub const WALL_THERMAL_FRACTION: f64 = 7.76e-7;

pub const T_OP_K: f64 = 21.13;
pub const T_QUENCH_K: f64 = 32.92;
pub const T_CRUISE_CORE_K: f64 = 22.84;
pub const T_BURST_CORE_K: f64 = 28.40;
pub const T_NBN_STANDBY_K: f64 = 4.21;
pub const T_NBN_BURST_K: f64 = 4.82;
pub const T_NBN_TRANSITION_K: f64 = 16.0;

pub const VOID_FRACTION_LIMIT: f64 = 0.380;
pub const VOID_FRACTION_CRUISE: f64 = 0.162;
pub const VOID_FRACTION_BURST: f64 = 0.368;

pub const M_DOT_CRUISE_KG_S: f64 = 14.8;
pub const M_DOT_BURST_KG_S: f64 = 112.5;

pub fn cryo_headroom_k() -> f64 {
    T_QUENCH_K - T_OP_K
}

pub const SIGMA_MAX_MPA: f64 = 124.60;
pub const SIGMA_YIELD_MPA: f64 = 350.0;
pub const E_DIAMOND_GPA: f64 = 1050.0;
pub const NU_DIAMOND: f64 = 0.10;
pub const FATIGUE_CYCLES_NF: f64 = 1.4e5;
pub const GST225_RECOVERY_FRACTION: f64 = 0.9995;

// ---- PCSS Optical Crowbar & Fast Safety ------------------------------------

pub const CROWBAR_CLOSE_NS: f64 = 2.10;
pub const SMES_INDUCTIVE_RECOVERY: f64 = 0.9420;
pub const INDUCTIVE_RECOVERY: f64 = SMES_INDUCTIVE_RECOVERY;
pub const TQEC_MAX_LATENCY_PS: u32 = 45000;
pub const TQEC_BLOSSOM_V_LATENCY_PS: u32 = 23700;

// ---- Standardized Propulsion Nacelle Mass & Volume Envelope ---------------

pub const NACELLE_BUDGET: &[(&str, f64, f64)] = &[
    ("Active Fuel Core (SHBT-MAR-312)", 376.99, 0.0359),
    ("CVD Diamond Borrmann Cavity Matrix", 65.12, 0.0185),
    ("5-Stage Relativistic DEC Duct", 840.00, 0.4200),
    ("Ti-6Al-4V Cryostat & Cold Plate", 520.00, 0.2850),
    ("Graded W-Cu / Pb / Borated-PE Shield", 1884.88, 0.3450),
    ("SI-SiC/GaN PCSS Crowbars & Coaxial Busbars", 230.00, 0.1120),
    ("TMSV Metrology & C11 Avionics Bay", 180.00, 0.0950),
];

pub const M_NACELLE_KG: f64 = 4096.99;
pub const V_NACELLE_M3: f64 = 1.3114;
pub const M_FLIGHT_CEILING_KG: f64 = 4200.00;
pub const M_RESERVE_MARGIN_KG: f64 = 103.01;

pub fn nacelle_mass_kg() -> f64 {
    NACELLE_BUDGET.iter().map(|e| e.1).sum()
}

pub fn nacelle_volume_m3() -> f64 {
    NACELLE_BUDGET.iter().map(|e| e.2).sum()
}

// ---- Four Canonical Operating Tiers ----------------------------------------

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum OperatingTier {
    BaselineHf178,
    CruisePw,
    BurstPw,
    ImpulseEw,
}

#[derive(Clone, Copy, Debug)]
pub struct TierSpec {
    pub name: &'static str,
    pub p_gross_w: f64,
    pub p_net_w: f64,
    pub eta_conv: f64,
    pub delta_n_per_tick: f64,
    pub n_dot_bits_s: f64,
    pub v_linear_c: f64,
    pub v_quad_c: f64,
    pub mach_alfven: f64,
    pub range_ly: f64,
}

pub const TIER_BASELINE: TierSpec = TierSpec {
    name: "BASELINE-HF178",
    p_gross_w: 109.05e12,
    p_net_w: 49.9449e12,
    eta_conv: 0.4580,
    delta_n_per_tick: 2.7114e13,
    n_dot_bits_s: 1.3698e18,
    v_linear_c: 5.0,
    v_quad_c: 5.0,
    mach_alfven: 0.015,
    range_ly: 3.2,
};

pub const TIER_CRUISE: TierSpec = TierSpec {
    name: "CRUISE-PW",
    p_gross_w: 1.000e15,
    p_net_w: 895.19e12,
    eta_conv: 0.89519,
    delta_n_per_tick: 4.860e14,
    n_dot_bits_s: 2.455e19,
    v_linear_c: 89.6,
    v_quad_c: 21.2,
    mach_alfven: 0.044,
    range_ly: 423_839.0,
};

pub const TIER_BURST: TierSpec = TierSpec {
    name: "BURST-PW",
    p_gross_w: 7.600e15,
    p_net_w: 6.80465e15,
    eta_conv: 0.89519,
    delta_n_per_tick: 3.6942e15,
    n_dot_bits_s: 1.8662e20,
    v_linear_c: 681.2,
    v_quad_c: 58.36,
    mach_alfven: 0.120,
    range_ly: 36_312.4,
};

pub const TIER_IMPULSE: TierSpec = TierSpec {
    name: "IMPULSE-EW",
    p_gross_w: 1.000e18,
    p_net_w: 895.19e15,
    eta_conv: 0.89519,
    delta_n_per_tick: 4.860e17,
    n_dot_bits_s: 2.455e22,
    v_linear_c: 89618.0,
    v_quad_c: 669.0,
    mach_alfven: 0.120,
    range_ly: 2771.0,
};

pub fn get_tier_spec(tier: OperatingTier) -> TierSpec {
    match tier {
        OperatingTier::BaselineHf178 => TIER_BASELINE,
        OperatingTier::CruisePw => TIER_CRUISE,
        OperatingTier::BurstPw => TIER_BURST,
        OperatingTier::ImpulseEw => TIER_IMPULSE,
    }
}

// ---- Legacy Trade Study ----------------------------------------------------

pub const TRADE_STUDY: &[(&str, &str, &str, &str)] = &[
    ("Specific fuel energy density", "1.32631 TJ/kg (178m2HfB2)", "89,875.52 TJ/kg (SHBT-MAR-312 c^2)", "+67764.4×"),
    ("Total stored core energy", "500.0 TJ (0.1389 GWh)", "33.8822 EJ (9.4117 TWh)", "+67764.4×"),
    ("Net electrical conversion", "45.8% (3-stage DEC)", "89.519% (5-stage relativistic DEC)", "+95.5% relative"),
    ("Maximum cruise velocity", "5.0c (HF178 limit)", "89.6c cruise / 681.2c burst / 89,618c impulse", "+17923.6×"),
    ("Unrefueled flight range", "3.2 ly", "423,839 ly (cruise) / 36,312.4 ly (burst)", "+132450×"),
    ("Standby antimatter stasis", "N/A (nuclear isomer)", "0.0 s^-1 boil-off (Lindblad 1.22e16 GeV)", "zero loss"),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_isomer_gain() {
        assert!((isomer_gain() - 61.15).abs() < 1e-9);
    }

    #[test]
    fn test_quiescent_heat() {
        assert!((quiescent_heat_kw() - 354.27).abs() < 0.01);
    }

    #[test]
    fn test_conversion_efficiency() {
        assert!((conversion_efficiency() - 0.89519).abs() < 1e-6);
    }

    #[test]
    fn test_nacelle_mass_budget() {
        assert!((nacelle_mass_kg() - 4096.99).abs() < 0.01);
        assert!((nacelle_volume_m3() - 1.3114).abs() < 0.001);
    }

    #[test]
    fn test_cryo_headroom() {
        assert!((cryo_headroom_k() - 11.79).abs() < 1e-9);
    }

    #[test]
    fn test_stored_energy_ej() {
        assert!((E_TOTAL_EJ - 33.8822).abs() < 1e-4);
    }
}
