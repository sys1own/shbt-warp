//! Bondi-Hoyle Gravitational Mass Harvester & Stand-Off Synthesizer Funnel.
//!
//! Integrates synthetic ghost-seed gravity with `SHBT-MAR-312C` / `R_synth^MAR`
//! fuel synthesizer and `SHBT-IBR-312` dark-braid magazine.
//!
//! Models:
//! 1. Why superluminal dark corridor transit (eta_D = 23/33) has sigma_collision == 0,
//!    preventing passive ramscooping during jumps, and why passive geometric sweeping
//!    in the IGM (rho_IGM ~ 1e-26 kg/m^3) requires sweeping > 10^11 ly per core.
//! 2. External Ghost-Seed Bondi-Hoyle-Lyttleton accretion funnel:
//!    R_BH = 2 G M_seed / (v_rel^2 + c_s^2), sigma_grav = pi R_BH^2,
//!    amplifying capture aperture by sigma_grav / A_geom >= 10^10 over physical intake.
//! 3. Three canonical harvesting regimes:
//!    - Regime A: Stand-Off Asteroid/Planetary Regolith & Cryo-Ice Tidal Lift (Mode 1b/1a)
//!    - Regime B: Circumgalactic Halo / Molecular Cloud / Stellar Wind Pinch (Mode 1a/1b)
//!    - Regime C: Deep Intergalactic Filament & Void Waypoint Harvesting (Missions 7A/7C)
//! 4. Direct coupling to Stage-1 de-rendering & R_synth^MAR coset folding into SHBT-IBR-312.

use serde::{Deserialize, Serialize};
use crate::synthesizer::{
    apply_mar312_coset_fold, CosetFoldDiagnostics, FoundrySynthesisMode,
    ANTIBARYON_PARTITION_RATIO, SUGAWARA_WEIGHT_SHIFT_VIS,
};
use crate::OperatingTier;

/// Gravitational constant in SI units (m^3 kg^-1 s^-2).
pub const G_SI: f64 = 6.67430e-11;

/// Speed of light in vacuum (m/s).
pub const C_LIGHT: f64 = 299_792_458.0;

/// Topological mass-per-bit coupling constant (kg/bit).
pub const ALPHA_SEED_KG_PER_BIT: f64 = 2.63635e-21;

/// Baseline nominal seed mass target: 1.0e-6 M_sun ≈ 1.98847e24 kg.
pub const SEED_MASS_0_KG: f64 = 1.98847e24;

/// Baseline active overflow register: 7.5426e44 bits.
pub const DELTA_N0_BITS: f64 = 7.5426e44;

/// Standard physical transducer geometric aperture area: 1.000 m^2.
pub const A_GEOM_PHYSICAL_M2: f64 = 1.000;

/// Canonical focal length for ghost-seed projection: 169.30 m.
pub const FOCAL_DISTANCE_M: f64 = 169.30;

/// Intergalactic Medium (IGM) average baryon mass density: ~1.0e-26 kg/m^3.
pub const RHO_IGM_KG_M3: f64 = 1.0e-26;

/// Circumgalactic Medium (CGM) / halo gas density: ~1.0e-23 kg/m^3.
pub const RHO_CGM_KG_M3: f64 = 1.0e-23;

/// Giant Molecular Cloud / dense gas density: ~1.0e-18 kg/m^3.
pub const RHO_GMC_KG_M3: f64 = 1.0e-18;

/// Asteroid regolith / planetary crust density: 2,500 to 5,500 kg/m^3.
pub const RHO_REGOLITH_KG_M3: f64 = 3_500.0;

/// Cryo-ice (volatile H2O/CO2/CH4) density: ~1,000 kg/m^3.
pub const RHO_CRYO_ICE_KG_M3: f64 = 1_000.0;

/// Light year in metres.
pub const METRES_PER_LY: f64 = 9.4607304725808e15;

/// Full MAR-312 core mass: 376.99 kg.
pub const MASS_FULL_CORE_KG: f64 = 376.99;

/// Compacted catalytic seed wafer mass: 37.699 kg.
pub const MASS_SEED_WAFER_KG: f64 = 37.699;

/// Full 64-bank hyperframe Stinespring frame mass (10^28 nucleons): 16,605.39 kg.
pub const MASS_HYPERFRAME_KG: f64 = 16_605.39;

// ============================================================================
// Phase 2 Xenobiological Triage & Mode II 4.2 K Tomographic Feedstock Guard
// ============================================================================

/// Mode II 4.2 K Landauer tomographic readout energy floor: `Q_readout = k_B * 4.2 K * ln(2) = 4.0194e-23 J/bit`.
pub const MODE2_TOMO_READOUT_J_PER_BIT: f64 = 4.01937e-23;

/// Critical 101-layer TTN-MERA complexity threshold for sapient/technological life (`10^6` nat-layers in Q48.16).
pub const C_CRIT_SAPIENT_Q16: u64 = 1_000_000 << 16;

/// Specific transmutation work for abiotic mineral regolith (Mode 1b): `56.845009 TJ/kg`.
pub const W_MODE1B_REGOLITH_TJ_KG: f64 = 56.845_009;
/// Specific transmutation work for Class X2 primitive pre-sapient biomass: `0.424815 TJ/kg`.
pub const W_CLASS_X2_BIOMASS_TJ_KG: f64 = 0.424_815;
/// Specific work saved by Class X2 assimilation vs. abiotic regolith: `56.420194 TJ/kg` (`99.25%`).
pub const DELTA_W_X2_SAVINGS_TJ_KG: f64 = 56.420_194;

/// Hardware fault bitmasks for attempted de-rendering of protected life.
pub const SHBT_SWARM_FAULT_X0_PROTECT_LCK: u32 = 0x0000_0800;
pub const SHBT_SWARM_FAULT_X1_SAPIENT_LCK: u32 = 0x0000_1000;

/// Phase 2 Xenobiological Triage Classes evaluated during stand-off Bondi-Hoyle funnel intake.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u32)]
pub enum XenoTriageClass {
    /// `Class M0`: Abiotic regolith / slag (`D_Cayley > 0`, `C_MERA == 0`) — permitted (`56.845009 TJ/kg`).
    M0AbioticRegolith  = 0x0000_0000,
    /// `Class X0`: Protected Earth-seeded / terrestrial life (`D_Cayley == 0`) — hardware locked (`0x00000800`).
    X0ProtectedEarth   = 0x1000_0000,
    /// `Class X1`: Protected sapient / technological life (`D_Cayley > 0`, `C_MERA >= C_crit`) — hardware locked (`0x00001000`).
    X1ProtectedSapient = 0x2000_0000,
    /// `Class X2`: Primitive pre-sapient biomass (`D_Cayley > 0`, `0 < C_MERA < C_crit`) — permitted (`P_Brem == 0 W`).
    X2AssimilablePrim  = 0x4000_0000,
}

/// Telemetry receipt from Mode II 4.2 K tomographic scan & xenobiological triage of harvested feedstock.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct XenoHarvestTriageReceipt {
    pub triage_class: XenoTriageClass,
    pub d_cayley_distance: u32,
    pub c_mera_q16: u64,
    pub q_readout_j_per_bit: f64,
    pub brem_power_watts: f64,
    pub specific_work_tj_kg: f64,
    pub work_saved_tj_kg: f64,
    pub derender_permitted: bool,
    pub fault_mask: u32,
}

/// Harvesting Operational Regime
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HarvestingRegime {
    /// Regime A: Stand-Off Asteroid / Planetary Regolith & Cryo-Ice Tidal Lift
    RegimeAStandOffRegolith,
    /// Regime B: Circumgalactic Halo / Molecular Cloud / Stellar Wind Pinch
    RegimeBCircumgalacticPinch,
    /// Regime C: Deep Intergalactic Filament & Void Waypoint Accretion
    RegimeCIntergalacticFilament,
}

/// In-Situ Bondi-Hoyle Gravitational Mass Harvester (`GhostMassHarvester`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GhostMassHarvester {
    /// Active ghost-seed mass projection in kilograms.
    pub seed_mass_kg: f64,
    /// Focal distance from Stage-1 intake transducer (m).
    pub focal_distance_m: f64,
    /// Physical geometric intake area (m^2).
    pub a_geom_m2: f64,
}

impl Default for GhostMassHarvester {
    fn default() -> Self {
        Self::new()
    }
}

impl GhostMassHarvester {
    /// Instantiate a harvester with baseline parameters.
    pub fn new() -> Self {
        Self {
            seed_mass_kg: SEED_MASS_0_KG,
            focal_distance_m: FOCAL_DISTANCE_M,
            a_geom_m2: A_GEOM_PHYSICAL_M2,
        }
    }

    /// Instantiate harvester from active bit overflow Delta N.
    pub fn from_delta_n(delta_n: f64) -> Self {
        Self {
            seed_mass_kg: delta_n * ALPHA_SEED_KG_PER_BIT,
            focal_distance_m: FOCAL_DISTANCE_M,
            a_geom_m2: A_GEOM_PHYSICAL_M2,
        }
    }

    /// Prove why passive geometric sweeping in the IGM fails:
    /// returns required path length in light years to sweep one 376.99 kg core.
    pub fn passive_igm_sweep_path_ly(a_geom_m2: f64) -> f64 {
        let path_metres = MASS_FULL_CORE_KG / (RHO_IGM_KG_M3 * a_geom_m2);
        path_metres / METRES_PER_LY
    }

    /// Compute Bondi-Hoyle-Lyttleton gravitational capture radius:
    /// `R_BH = 2 G M_seed / (v_rel^2 + c_s^2)`.
    pub fn bondi_hoyle_radius(&self, v_rel: f64, sound_speed_cs: f64) -> f64 {
        let v_eff_sq = v_rel * v_rel + sound_speed_cs * sound_speed_cs;
        let v_safe_sq = v_eff_sq.max(1.0); // prevent division by zero at rest
        2.0 * G_SI * self.seed_mass_kg / v_safe_sq
    }

    /// Gravitational capture cross-section: `sigma_grav = pi * R_BH^2`.
    pub fn gravitational_cross_section(&self, v_rel: f64, sound_speed_cs: f64) -> f64 {
        let r_bh = self.bondi_hoyle_radius(v_rel, sound_speed_cs);
        std::f64::consts::PI * r_bh * r_bh
    }

    /// Ratio of gravitational capture area to physical transducer area:
    /// verifies `sigma_grav / A_geom >= 1.0e10`.
    pub fn aperture_amplification_ratio(&self, v_rel: f64, sound_speed_cs: f64) -> f64 {
        let sigma = self.gravitational_cross_section(v_rel, sound_speed_cs);
        sigma / self.a_geom_m2
    }

    /// Mass accretion rate (kg/s):
    /// `dot{M}_BH = pi * R_BH^2 * rho_amb * sqrt(v_rel^2 + c_s^2)`.
    pub fn mass_accretion_rate(
        &self,
        ambient_density_kg_m3: f64,
        v_rel: f64,
        sound_speed_cs: f64,
    ) -> f64 {
        let sigma = self.gravitational_cross_section(v_rel, sound_speed_cs);
        let v_eff = (v_rel * v_rel + sound_speed_cs * sound_speed_cs).sqrt().max(1.0);
        sigma * ambient_density_kg_m3 * v_eff
    }

    /// Regime A: Stand-off asteroid / planetary tidal lift rate (kg/s).
    /// With stand-off altitude `d_m` and density `rho`, the shaped tidal traction gradient
    /// pulls surface slag directly into the Stage-1 focal zone.
    pub fn standoff_tidal_lift_rate(&self, standoff_altitude_m: f64, regolith_density: f64) -> f64 {
        let d = standoff_altitude_m.clamp(100.0, 10_000.0);
        // Effective ingestion flux through the focal neck
        let v_intake = (2.0 * G_SI * self.seed_mass_kg / d).sqrt().min(0.01 * C_LIGHT);
        let neck_area = std::f64::consts::PI * (self.focal_distance_m * 0.1).powi(2);
        neck_area * regolith_density * v_intake
    }

    /// Time required to accumulate target mass in seconds under Regime A.
    pub fn time_to_accumulate_regime_a(&self, target_mass_kg: f64, standoff_m: f64) -> f64 {
        let rate = self.standoff_tidal_lift_rate(standoff_m, RHO_REGOLITH_KG_M3);
        target_mass_kg / rate
    }

    /// Execute integrated harvest, Stage-1 de-rendering, and R_synth^MAR coset folding:
    /// accumulates required mass, triggers coset folding into SHBT-IBR-312 format,
    /// and verifies Sugawara framing cancellation (Delta_fr == 0).
    pub fn harvest_and_synthesize_core(
        &self,
        regime: HarvestingRegime,
        batch_mass_kg: f64,
        tier: OperatingTier,
        _mode: FoundrySynthesisMode,
    ) -> HarvestSynthesisResult {
        // Compute harvesting accumulation duration
        let (harvest_time_s, ambient_rho) = match regime {
            HarvestingRegime::RegimeAStandOffRegolith => {
                let t = self.time_to_accumulate_regime_a(batch_mass_kg, 100.0);
                (t, RHO_REGOLITH_KG_M3)
            }
            HarvestingRegime::RegimeBCircumgalacticPinch => {
                let rate = self.mass_accretion_rate(RHO_CGM_KG_M3, 50_000.0, 10_000.0);
                (batch_mass_kg / rate, RHO_CGM_KG_M3)
            }
            HarvestingRegime::RegimeCIntergalacticFilament => {
                let rate = self.mass_accretion_rate(RHO_IGM_KG_M3, 100_000.0, 15_000.0);
                (batch_mass_kg / rate, RHO_IGM_KG_M3)
            }
        };

        // Stage-1 de-rendering folding time
        let tau_fold_s = match tier {
            OperatingTier::ImpulseEw => 0.508e-3, // 0.508 ms
            OperatingTier::BurstPw => 66.98e-3,  // 66.98 ms
            OperatingTier::CruisePw | OperatingTier::BaselineHf178 => 509.10e-3,
        };

        // Execute coset fold with 50% anti-baryon partition ratio
        let fold_diag = apply_mar312_coset_fold(batch_mass_kg, ANTIBARYON_PARTITION_RATIO);

        HarvestSynthesisResult {
            regime,
            batch_mass_kg,
            ambient_density_kg_m3: ambient_rho,
            harvest_duration_s: harvest_time_s,
            stage1_tau_fold_s: tau_fold_s,
            total_cycle_time_s: harvest_time_s + tau_fold_s,
            coset_diagnostics: fold_diag,
            sugawara_shift_vis: SUGAWARA_WEIGHT_SHIFT_VIS,
            framing_defect_zero: fold_diag.framing_defect == 0.0,
            stored_in_ibr312: true,
        }
    }

    /// Evaluate Phase 2 Xenobiological Triage (`Classes M0, X0, X1, X2`) on stand-off
    /// Bondi-Hoyle funnel feedstock scanned via Mode II 4.2 K tomographic capture
    /// (`Q_readout = 4.0194e-23 J/bit`), checking 101-layer TTN-MERA complexity `C_MERA`
    /// and Burau-Cayley phylogenetic distance `D_Cayley`.
    /// Blocks de-rendering in hardware if `Class X0` (`0x00000800`) or `Class X1` (`0x00001000`)
    /// is detected, while permitting `Class M0` and `Class X2` non-ionizing assimilation (`P_Brem == 0 W`).
    pub fn evaluate_xenobiological_triage(
        &self,
        d_cayley_distance: u32,
        c_mera_q16: u64,
    ) -> Result<XenoHarvestTriageReceipt, u32> {
        let triage_class = if d_cayley_distance == 0 {
            XenoTriageClass::X0ProtectedEarth
        } else if c_mera_q16 >= C_CRIT_SAPIENT_Q16 {
            XenoTriageClass::X1ProtectedSapient
        } else if c_mera_q16 > 0 {
            XenoTriageClass::X2AssimilablePrim
        } else {
            XenoTriageClass::M0AbioticRegolith
        };

        match triage_class {
            XenoTriageClass::X0ProtectedEarth => Err(SHBT_SWARM_FAULT_X0_PROTECT_LCK),
            XenoTriageClass::X1ProtectedSapient => Err(SHBT_SWARM_FAULT_X1_SAPIENT_LCK),
            XenoTriageClass::M0AbioticRegolith => Ok(XenoHarvestTriageReceipt {
                triage_class,
                d_cayley_distance,
                c_mera_q16,
                q_readout_j_per_bit: MODE2_TOMO_READOUT_J_PER_BIT,
                brem_power_watts: 0.0,
                specific_work_tj_kg: W_MODE1B_REGOLITH_TJ_KG,
                work_saved_tj_kg: 0.0,
                derender_permitted: true,
                fault_mask: 0,
            }),
            XenoTriageClass::X2AssimilablePrim => Ok(XenoHarvestTriageReceipt {
                triage_class,
                d_cayley_distance,
                c_mera_q16,
                q_readout_j_per_bit: MODE2_TOMO_READOUT_J_PER_BIT,
                brem_power_watts: 0.0,
                specific_work_tj_kg: W_CLASS_X2_BIOMASS_TJ_KG,
                work_saved_tj_kg: DELTA_W_X2_SAVINGS_TJ_KG,
                derender_permitted: true,
                fault_mask: 0,
            }),
        }
    }
}

/// Diagnostics and performance results from a harvest & synthesis cycle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HarvestSynthesisResult {
    pub regime: HarvestingRegime,
    pub batch_mass_kg: f64,
    pub ambient_density_kg_m3: f64,
    pub harvest_duration_s: f64,
    pub stage1_tau_fold_s: f64,
    pub total_cycle_time_s: f64,
    pub coset_diagnostics: CosetFoldDiagnostics,
    pub sugawara_shift_vis: f64,
    pub framing_defect_zero: bool,
    pub stored_in_ibr312: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_passive_igm_sweep_proves_starvation() {
        let path_ly = GhostMassHarvester::passive_igm_sweep_path_ly(1.0);
        // Sweeping 376.99 kg across 1e-26 kg/m^3 with 1 m^2 requires ~4e12 ly >> 1e11 ly
        assert!(path_ly > 1.0e11);
    }

    #[test]
    fn test_aperture_amplification_exceeds_1e10() {
        let harvester = GhostMassHarvester::new();
        let amp = harvester.aperture_amplification_ratio(10_000.0, 1_000.0);
        assert!(amp >= 1.0e10);
    }

    #[test]
    fn test_regime_a_rapid_lift_times() {
        let harvester = GhostMassHarvester::new();

        // 37.699 kg micro-wafer batch accumulates in < 1.0 ms
        let t_wafer = harvester.time_to_accumulate_regime_a(MASS_SEED_WAFER_KG, 100.0);
        assert!(t_wafer < 1.0e-3);

        // 376.99 kg full core accumulates in < 10.0 ms
        let t_core = harvester.time_to_accumulate_regime_a(MASS_FULL_CORE_KG, 100.0);
        assert!(t_core < 10.0e-3);

        // 16,605.39 kg full hyperframe accumulates in < 0.45 s
        let t_hyper = harvester.time_to_accumulate_regime_a(MASS_HYPERFRAME_KG, 100.0);
        assert!(t_hyper < 0.45);
    }

    #[test]
    fn test_harvest_and_coset_synthesis_coupling() {
        let harvester = GhostMassHarvester::new();
        let res = harvester.harvest_and_synthesize_core(
            HarvestingRegime::RegimeAStandOffRegolith,
            MASS_FULL_CORE_KG,
            OperatingTier::BurstPw,
            FoundrySynthesisMode::Mode1bAsteroidRegolith,
        );

        assert!(res.framing_defect_zero);
        assert_eq!(res.sugawara_shift_vis, 45.0 / 64.0);
        assert!(res.stored_in_ibr312);
        assert_eq!(res.coset_diagnostics.total_core_mass_kg, 376.99);
    }

    #[test]
    fn test_xenobiological_triage_in_ghost_harvester() {
        let harvester = GhostMassHarvester::new();
        // Class X0 (Earth-seeded, D_Cayley == 0) -> blocked with 0x00000800
        assert_eq!(
            harvester.evaluate_xenobiological_triage(0, 100),
            Err(SHBT_SWARM_FAULT_X0_PROTECT_LCK)
        );
        // Class X1 (Sapient, C_MERA >= C_crit) -> blocked with 0x00001000
        assert_eq!(
            harvester.evaluate_xenobiological_triage(12, C_CRIT_SAPIENT_Q16),
            Err(SHBT_SWARM_FAULT_X1_SAPIENT_LCK)
        );
        // Class M0 (Abiotic regolith) -> permitted at 56.845009 TJ/kg
        let m0 = harvester.evaluate_xenobiological_triage(8, 0).unwrap();
        assert_eq!(m0.triage_class, XenoTriageClass::M0AbioticRegolith);
        assert!(m0.derender_permitted);
        assert_eq!(m0.brem_power_watts, 0.0);
        // Class X2 (Primitive biomass) -> permitted with 56.420194 TJ/kg savings
        let x2 = harvester.evaluate_xenobiological_triage(8, 2048).unwrap();
        assert_eq!(x2.triage_class, XenoTriageClass::X2AssimilablePrim);
        assert!(x2.derender_permitted);
        assert!((x2.work_saved_tj_kg - 56.420_194).abs() < 1e-6);
    }
}
