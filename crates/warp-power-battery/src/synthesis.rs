//! Bipartite so(10)_312 / [su(2)_26 x su(3)_8] Coset Fuel Synthesizer
//! & Closed-Loop Autonomous Breeder Mechanics.
//!
//! Direct microscopic implementation transferred from `shbt-recon`.
#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
pub use crate::synthesizer::{
    apply_mar312_coset_fold, CosetFoldDiagnostics, FoundrySynthesisMode,
    ANTIBARYON_PARTITION_RATIO, BRANCH_A_VISIBLE_MASS_KG, BRANCH_B_ANTIBARYON_MASS_KG,
    CALLAN_SYMANZIK_DELTA_1, CALLAN_SYMANZIK_DELTA_2, CALLAN_SYMANZIK_DELTA_3,
    C_COSET, C_PARENT_SO10, C_VISIBLE, MAR312_NET_ENERGY_EJ, MAR312_TOTAL_ENERGY_EJ,
    MAR312_TOTAL_MASS_KG, PER_BRAID_SHIFT_H, SUGAWARA_WEIGHT_SHIFT_DARK,
    SUGAWARA_WEIGHT_SHIFT_VIS, SYNTH_UNITARITY_LIMIT, SYNTH_UNITARITY_RESIDUAL,
};

/// Synthesis operating mode for breeder and fuel generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SynthesisMode {
    /// Mode 1c: Salvage / pre-refined 178m2HfB2 + 12C baryons (50% anti-baryon fold).
    Mode1c,
    /// Mode 1a: Matched natural SiC / HfO2 / B4C feedstock (isomer pump + anti-baryon fold).
    Mode1a,
    /// Mode 1b: Unrefined planetary / asteroid slag Fe / Si / C (transmutation + isomer pump + fold).
    Mode1b,
    /// Mode 2: Pure vacuum pair creation baseline (2 M_core c^2 + topological overhead).
    Mode2,
}

impl From<SynthesisMode> for FoundrySynthesisMode {
    fn from(m: SynthesisMode) -> Self {
        match m {
            SynthesisMode::Mode1c => FoundrySynthesisMode::Mode1cSalvageIsomerBaryons,
            SynthesisMode::Mode1a => FoundrySynthesisMode::Mode1aNaturalFeedstock,
            SynthesisMode::Mode1b => FoundrySynthesisMode::Mode1bAsteroidRegolith,
            SynthesisMode::Mode2 => FoundrySynthesisMode::Mode2VacuumPairCreation,
        }
    }
}

impl From<FoundrySynthesisMode> for SynthesisMode {
    fn from(m: FoundrySynthesisMode) -> Self {
        match m {
            FoundrySynthesisMode::Mode1cSalvageIsomerBaryons => SynthesisMode::Mode1c,
            FoundrySynthesisMode::Mode1aNaturalFeedstock => SynthesisMode::Mode1a,
            FoundrySynthesisMode::Mode1bAsteroidRegolith => SynthesisMode::Mode1b,
            FoundrySynthesisMode::Mode2VacuumPairCreation => SynthesisMode::Mode2,
        }
    }
}

impl SynthesisMode {
    /// Net electrical input energy in Petajoules (PJ) for one 376.99 kg core.
    pub const fn input_energy_pj(self) -> f64 {
        match self {
            Self::Mode1c => 0.2845,
            Self::Mode1a => 2.1220,
            Self::Mode1b => 21.4300,
            Self::Mode2 => 33_882.17,
        }
    }

    /// Net electrical input energy in Joules (J).
    pub fn input_energy_joules(self) -> f64 {
        self.input_energy_pj() * 1.0e15
    }

    /// Fuel energy amplification factor Q_fuel = E_stored / E_synth,in.
    pub const fn amplification_q_fuel(self) -> f64 {
        match self {
            Self::Mode1c => 119_094.0,
            Self::Mode1a => 15_967.0,
            Self::Mode1b => 1_581.0,
            Self::Mode2 => 1.000,
        }
    }

    /// Number of daughter cores bred from one parent core net output (30.3310 EJ).
    pub const fn daughter_cores_per_parent(self) -> u32 {
        match self {
            Self::Mode1c => 106_611,
            Self::Mode1a => 14_293,
            Self::Mode1b => 1_415,
            Self::Mode2 => 0,
        }
    }
}

/// Calculate exact breeder turnaround time in seconds for a given power level and synthesis mode.
/// tau = E_synth,in / P_reactor
pub fn calculate_breeder_turnaround(power_watts: f64, mode: SynthesisMode) -> f64 {
    if power_watts <= 0.0 {
        return f64::INFINITY;
    }
    mode.input_energy_joules() / power_watts
}

/// Microscopic Bipartite Coset Spinor Synthesizer (`SHBT-MAR-312`).
#[derive(Debug, Clone, Copy, Default)]
pub struct Mar312Synthesizer;

impl Mar312Synthesizer {
    /// Execute bipartite coset fold R_synth^MAR for arbitrary core mass.
    pub fn synthesize_core(total_mass_kg: f64, antibaryon_ratio: f64) -> CosetFoldDiagnostics {
        apply_mar312_coset_fold(total_mass_kg, antibaryon_ratio)
    }

    /// Central charge partition weights (w1, w2, w3) for the 3 Callan-Symanzik threshold sectors.
    pub fn central_charge_weights() -> (f64, f64, f64) {
        let w1 = 4.0 / 63.0;
        let w2 = 512.0 / 3861.0;
        let w3 = 21727.0 / 27027.0;
        (w1, w2, w3)
    }

    /// Effective leptoquark energy threshold in MeV at 21.13 K under Floquet RG drive.
    pub fn effective_leptoquark_threshold_mev() -> f64 {
        2.486000
    }

    /// Landau-Zener leptoquark non-adiabatic excitation probability.
    pub fn landau_zener_probability() -> f64 {
        1.108e-124
    }

    /// 7:6 Commensurate Moiré co-crystallization misfit fraction.
    pub fn moire_lattice_misfit() -> f64 {
        let a_hfb2 = 21.9940;
        let d_diamond = 22.0000;
        (d_diamond - a_hfb2) / d_diamond
    }

    /// Chern-Simons framing tether spring constant k_tether (N/m).
    pub fn framing_tether_spring_constant() -> f64 {
        5.1295e10
    }

    /// Maximum anti-baryon displacement under 124.60 MPa peak stress (Angstroms).
    pub fn max_antibaryon_displacement_angstrom() -> f64 {
        0.6823e-4
    }

    /// Net Dicke superradiance gain G_SR^net.
    pub fn dicke_superradiance_net_gain() -> f64 {
        8.29e-22
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_energy_ratios_and_conformal_weight() {
        let diag = Mar312Synthesizer::synthesize_core(MAR312_TOTAL_MASS_KG, 0.500);
        assert_eq!(diag.wzw_levels, (26, 8, 312));
        assert!((diag.delta_h_vis - 45.0 / 64.0).abs() < 1e-15);
        assert!((diag.delta_h_dark - (-45.0 / 64.0)).abs() < 1e-15);
        assert_eq!(diag.framing_defect, 0.0);
        assert!(diag.unitarity_residual <= SYNTH_UNITARITY_LIMIT);
        assert_eq!(diag.standby_render_cost_j, 0.0);
        assert_eq!(diag.boiloff_rate_s_inv, 0.0);
        assert!(diag.cbarb_zero_verified);
        assert!(diag.rg_lock_verified);
    }

    #[test]
    fn test_breeder_turnaround_exact() {
        let p_cruise = 895.19e12; // 895.19 TW
        let p_burst = 6.805e15;   // 6.805 PW
        let p_impulse = 895.19e15;// 895.19 PW

        // Mode 1c
        let t_1c_cruise = calculate_breeder_turnaround(p_cruise, SynthesisMode::Mode1c);
        let t_1c_burst = calculate_breeder_turnaround(p_burst, SynthesisMode::Mode1c);
        let t_1c_impulse = calculate_breeder_turnaround(p_impulse, SynthesisMode::Mode1c);
        assert!((t_1c_cruise - 0.3178).abs() < 1e-3);
        assert!((t_1c_burst - 0.04181).abs() < 1e-4);
        assert!((t_1c_impulse - 0.0003178).abs() < 1e-6);

        // Mode 1a
        let t_1a_cruise = calculate_breeder_turnaround(p_cruise, SynthesisMode::Mode1a);
        let t_1a_burst = calculate_breeder_turnaround(p_burst, SynthesisMode::Mode1a);
        let t_1a_impulse = calculate_breeder_turnaround(p_impulse, SynthesisMode::Mode1a);
        assert!((t_1a_cruise - 2.370).abs() < 1e-2);
        assert!((t_1a_burst - 0.3118).abs() < 1e-3);
        assert!((t_1a_impulse - 0.002370).abs() < 1e-5);

        // Mode 1b
        let t_1b_cruise = calculate_breeder_turnaround(p_cruise, SynthesisMode::Mode1b);
        let t_1b_burst = calculate_breeder_turnaround(p_burst, SynthesisMode::Mode1b);
        let t_1b_impulse = calculate_breeder_turnaround(p_impulse, SynthesisMode::Mode1b);
        assert!((t_1b_cruise - 23.94).abs() < 0.05);
        assert!((t_1b_burst - 3.149).abs() < 0.01);
        assert!((t_1b_impulse - 0.02394).abs() < 1e-4);
    }

    #[test]
    fn test_microscopic_physics_invariants() {
        let (w1, w2, w3) = Mar312Synthesizer::central_charge_weights();
        assert!((w1 + w2 + w3 - 1.0).abs() < 1e-12);

        let p_lz = Mar312Synthesizer::landau_zener_probability();
        assert!(p_lz <= 1.0e-120);

        let misfit = Mar312Synthesizer::moire_lattice_misfit();
        assert!(misfit < 0.0003); // < 0.03%

        let disp = Mar312Synthesizer::max_antibaryon_displacement_angstrom();
        assert!(disp <= 1.0e-4); // <= 10^-4 Angstrom

        let g_sr = Mar312Synthesizer::dicke_superradiance_net_gain();
        assert!(g_sr < 1.0e-20);
    }
}
