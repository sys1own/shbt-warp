//! Coherent graser-discharged nuclear isomer battery model for `shbt-warp`.
//!
//! Implements the warp2.txt specification: ^178m2Hf isomer energy storage,
//! K-selection bypass via a 40 keV intermediate gateway state, Borrmann
//! anomalous transmission graser cavity dynamics, the 3-stage relativistic
//! direct energy conversion stack, quiescent/burst thermal-hydraulics, and
//! the standardized nacelle mass/volume/shielding budget.

// ---- ^178m2Hf isomer storage ----------------------------------------------

/// Isomer excitation energy, MeV.
pub const E_ISOMER_MEV: f64 = 2.446;
/// Intermediate gateway excitation energy above the isomeric state, MeV (40.0 keV).
pub const E_GATEWAY_MEV: f64 = 0.040;
/// Half-life, years.
pub const HALF_LIFE_Y: f64 = 31.0;
/// Molar mass, g/mol.
pub const MOLAR_MASS_G: f64 = 177.94;
/// Gravimetric energy density, TJ/kg (E_x * N_A / M_mol).
pub const RHO_E_TJ_KG: f64 = 1.32631;
/// Specific quiescent decay power, W/kg.
pub const DECAY_POWER_W_KG: f64 = 939.73;
/// Active isomer core mass, kg.
pub const M_ISO_KG: f64 = 376.99;
/// Total stored energy, TJ (138.89 GWh).
pub const E_TOTAL_TJ: f64 = 500.0;
/// Trigger flux threshold for stimulated de-excitation, W/m^2.
pub const TRIGGER_FLUX_MIN_W_M2: f64 = 1.85e16;
/// K-selection forbiddenness degree (|ΔK| = 8 or 16 → ν = 6 empirical).
pub const K_FORBIDDENNESS_NU: u32 = 6;

/// Trigger energy multiplication factor G_isomer = E_released / E_trigger.
pub fn isomer_gain() -> f64 {
    E_ISOMER_MEV / E_GATEWAY_MEV
}

/// Continuous quiescent decay heating of the full core, kW.
pub fn quiescent_heat_kw() -> f64 {
    M_ISO_KG * DECAY_POWER_W_KG / 1000.0
}

/// Thermoelectric recovery reserved for standby avionics, kW.
pub const QUIESCENT_TEG_RECOVERY_KW: f64 = 48.0;

// ---- Graser cavity / Borrmann anomalous transmission ----------------------

/// Mössbauer zero-phonon fraction at T ≤ 21.13 K.
pub const MOSSBAUER_FRACTION_MIN: f64 = 0.74;
/// Operating core temperature, K (matches the shared 21.13 K cryo baseline).
pub const T_OP_K: f64 = 21.13;
/// Borrmann suppression parameter ε_B.
pub const BORRMANN_EPSILON: f64 = 0.985;
/// Effective non-nuclear attenuation under Borrmann suppression, cm^-1 at 574 keV.
pub const MU_LOSS_EFF_CM1: f64 = 0.18;
/// Threshold inversion density for small-signal gain, cm^-3 (η_inv ≈ 1.63%).
pub const N_INV_CRIT_CM3: f64 = 7.35e20;
/// Prompt discharge pulse duration, ns.
pub const PULSE_NS: f64 = 10.0;

// ---- 3-stage relativistic direct energy conversion -------------------------

/// Stage 1: forward Compton recoil (interleaved W/Ta 250 nm foils, Be collectors).
pub const ETA_STAGE1: f64 = 0.264;
/// Stage 2: pair induction core (W foam + pulsed REBCO HTS pickup coils).
pub const ETA_STAGE2: f64 = 0.121;
/// Stage 3: 8-stage graded Be electrostatic retarding grids.
pub const ETA_STAGE3: f64 = 0.073;
/// Forward ejection half-angle limit, degrees.
pub const COMPTON_CONE_DEG_MAX: f64 = 18.0;
/// Regulated DC output window, kV.
pub const BUS_VOLTAGE_MIN_KV: f64 = 15.0;
pub const BUS_VOLTAGE_MAX_KV: f64 = 400.0;

/// Combined conversion efficiency η_conv = η1 + η2 + η3.
pub fn conversion_efficiency() -> f64 {
    ETA_STAGE1 + ETA_STAGE2 + ETA_STAGE3
}

// ---- PCSS optical crowbar --------------------------------------------------

/// Optical crowbar closing time bound, ns.
pub const CROWBAR_CLOSE_NS: f64 = 2.10;
/// Current slew-rate bound, A/s.
pub const DI_DT_MAX: f64 = 1.85e14;
/// Voltage slew-rate bound, V/s.
pub const DV_DT_MAX: f64 = 4.20e13;
/// Inductive energy recovery fraction into the cryogenic LC tank.
pub const INDUCTIVE_RECOVERY: f64 = 0.9420;

// ---- Thermo-elastic shock / cryogenics -------------------------------------

/// Peak transient thermo-elastic shear stress, MPa.
pub const SIGMA_MAX_MPA: f64 = 124.60;
/// CVD diamond dynamic yield limit, MPa.
pub const SIGMA_YIELD_MPA: f64 = 350.0;
/// Young's modulus of the CVD diamond heat spreaders, GPa.
pub const E_DIAMOND_GPA: f64 = 1050.0;
/// Poisson's ratio of the CVD diamond spreaders.
pub const NU_DIAMOND: f64 = 0.10;
/// Thermal expansion coefficient, K^-1.
pub const ALPHA_TH: f64 = 1.0e-6;
/// Quarter-wave sapphire/aerogel matching layer thickness, nm.
pub const MATCH_LAYER_NM: f64 = 6.395;
/// HTS busbar quench threshold, K.
pub const T_QUENCH_K: f64 = 32.92;
/// Cruise mass flow of two-phase helium, kg/s.
pub const M_DOT_CRUISE_KG_S: f64 = 14.8;
/// Burst mass flow of two-phase helium, kg/s.
pub const M_DOT_BURST_KG_S: f64 = 112.5;

/// Cryogenic headroom ΔT = T_quench − T_op, K.
pub fn cryo_headroom_k() -> f64 {
    T_QUENCH_K - T_OP_K
}

/// Thermo-elastic stress coefficient β_th = E·α_th/(1−2ν), GPa.
pub fn thermo_elastic_beta() -> f64 {
    E_DIAMOND_GPA * ALPHA_TH / (1.0 - 2.0 * NU_DIAMOND)
}

// ---- Flight-stage burst energy budget --------------------------------------

/// Stage 2 subluminal ramp 0 → 0.95c: duration, energy, peak burst power.
pub const STAGE2_T_S: f64 = 10.0;
pub const STAGE2_E_TJ: f64 = 12.50;
pub const STAGE2_P_BURST_TW: f64 = 2.34;
/// Stage 3 superluminal cruise 2.0c → 5.0c: duration, energy, peak burst power.
pub const STAGE3_T_S: f64 = 5.0;
pub const STAGE3_E_TJ: f64 = 290.80;
pub const STAGE3_P_BURST_TW: f64 = 109.05;
/// Total staged energy demand, TJ.
pub fn staged_energy_tj() -> f64 {
    STAGE2_E_TJ + STAGE3_E_TJ
}

// ---- Standardized nacelle engineering budget --------------------------------

/// (component, mass kg, volume m³). Volumes of the truss are folded into the
/// outer envelope; the integrated assembly totals 4199.98 kg / 1.3326 m³.
pub const NACELLE_BUDGET: &[(&str, f64, f64)] = &[
    ("Active ^178m2Hf core", 376.99, 0.0283),
    ("Primary graser cavity (CVD diamond/Borrmann)", 145.20, 0.0413),
    ("Direct energy converter (W-Ta/Be/HTS)", 412.50, 0.1870),
    ("Primary gamma shield (Pb)", 2075.98, 0.1831),
    ("Neutron/scatter shield (5% BPE)", 520.91, 0.5209),
    ("PCSS crowbar & interlocks", 68.40, 0.0220),
    ("Cryostat + 350 L LHe dewar", 385.00, 0.3500),
    ("Nacelle support structure (C-C truss)", 215.00, 0.0),
];
/// Integrated nacelle totals.
pub const M_NACELLE_KG: f64 = 4199.98;
pub const V_NACELLE_M3: f64 = 1.3326;
/// Outer envelope, m.
pub const NACELLE_DIAMETER_M: f64 = 1.30;
pub const NACELLE_LENGTH_M: f64 = 1.45;
/// Lead TVL at 574 keV, cm.
pub const LEAD_TVL_CM: f64 = 1.35;

/// Sum of itemized nacelle masses, kg.
pub fn nacelle_mass_kg() -> f64 {
    NACELLE_BUDGET.iter().map(|e| e.1).sum()
}

// ---- Comparative trade study vs. baseline C-band linac graser ---------------

/// (metric, linac baseline, isomer battery, delta). Values per warp2.txt.
pub const TRADE_STUDY: &[(&str, &str, &str, &str)] = &[
    ("Gravimetric energy density", "0.045 GJ/kg (system level, PFN)", "119.05 GJ/kg (shielded nacelle)", "+2645×"),
    ("Volumetric energy density", "0.012 GJ/L", "375.21 GJ/L (shielded envelope)", "+31267×"),
    ("Priming power requirement", "450 MW continuous linac RF feed", "12.5 kW pulsed X-ray seed", "−36000×"),
    ("Pulse ramp latency", "1.20 ms (klystron rise time)", "≤ 2.10 ns (PCSS-triggered)", "571000× faster"),
    ("Quiescent parasitic load", "18 MW standby", "354.27 kW decay cooling (48 kW TEG recovered)", "parasitic reduced"),
    ("Peak output power", "1.20 TW", "109.05 TW", "+90.8×"),
    ("Conversion efficiency", "28.5% indirect thermal/MHD", "45.8% direct relativistic Compton/PE", "+60.7% relative"),
    ("Continuous flight range", "< 0.1 ly (external recharge)", "≈ 3.2 ly (500.0 TJ autonomous)", "deep-space capable"),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn isomer_gain_61_15() {
        assert!((isomer_gain() - 61.15).abs() < 1e-9);
    }

    #[test]
    fn quiescent_354_27kw() {
        assert!((quiescent_heat_kw() - 354.27).abs() < 0.01);
    }

    #[test]
    fn conversion_45_8() {
        assert!((conversion_efficiency() - 0.458).abs() < 1e-12);
    }

    #[test]
    fn nacelle_mass_budget() {
        assert!((nacelle_mass_kg() - M_NACELLE_KG).abs() < 0.01);
    }

    #[test]
    fn headroom_11_79() {
        assert!((cryo_headroom_k() - 11.79).abs() < 1e-9);
    }
}
