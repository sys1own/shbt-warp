//! 9-Component Proof-Carrying `Linear<V>` Control Barrier Function (`h_phys(x) >= 0`)
//! & Xenobiological Triage Safety Shield (`Classes M0, X0, X1, X2`).
//!
//! Direct architectural transfer from `sys1own/shbt-swarm` (`crates/shbt-swarm-shield`)
//! into `sys1own/shbt-warp` (`crates/warp-flight-dynamics/src/cbf_shield.rs`).

use serde::{Deserialize, Serialize};

pub const C_CRIT_SAPIENT_Q16: u64 = 1_000_000 << 16;
pub const C_BRAID_MAX_MICRO_NATS: i64 = 38_196_601; // 38.196601 nats
pub const MAX_QC_TEMP_MK: u32 = 4_200;              // 4.200 K
pub const MAX_MAR312C_TEMP_MK: u32 = 32_920;        // 32.920 K
pub const MAX_MAR312X_TEMP_MK: u32 = 192_500;       // 192.500 K
pub const MAX_FRENKEL_SHEAR_MPA: u32 = 4_800;       // 4,800.00 MPa
pub const MIN_SUPERHYDRIDE_FIELD_MT: u32 = 48_000;  // 48.0 T
pub const MAX_WYCKOFF_JITTER_FM: u32 = 10;          // 10 fm = 10^-4 A
pub const MAX_EIKONAL_ACCEL_MILLI: u32 = 5_774;     // 5.7735

/// Hardware fault bitmasks matching `shbt_swarm_mmio.h` v4.
pub const FAULT_ADM_IMBALANCE: u32            = 0x0000_0001;
pub const FAULT_FRAMING_WEYL: u32             = 0x0000_0004;
pub const FAULT_BL_ANOMALY: u32               = 0x0000_0008;
pub const FAULT_NEUTRINO_LEAK: u32            = 0x0000_0010;
pub const FAULT_THERMAL_CLAMP: u32            = 0x0000_0080;
pub const FAULT_CBF_PHYS_VIOL: u32            = 0x0000_0100;
pub const FAULT_X0_PROTECT_LCK: u32           = 0x0000_0800;
pub const FAULT_X1_SAPIENT_LCK: u32           = 0x0000_1000;
pub const FAULT_SUGAWARA_CAP_EXCEEDED: u32    = 0x0000_2000;
pub const FAULT_SUPERHYDRIDE_TETHER_LOSS: u32 = 0x0002_0000;
pub const FAULT_FORGE_SHEAR_EXCEEDED: u32     = 0x0010_0000;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u32)]
pub enum XenoTriageClass {
    M0AbioticRegolith  = 0x0000_0000,
    X0ProtectedEarth   = 0x1000_0000,
    X1ProtectedSapient = 0x2000_0000,
    X2AssimilablePrim  = 0x4000_0000,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReactorGeneration {
    Mar312cCompacted, // T_MAR <= 32,920 mK (32.92 K)
    Mar312xExotic,    // T_MAR <= 192,500 mK (192.50 K)
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShieldVerdict {
    Approved {
        cmd_opcode: u32,
        barriers_verified: u8,
    },
    BlockedFault {
        fault_mask: u32,
    },
}

/// 9-Component Physical State Vector evaluated by the Proof-Carrying `Linear<V>` CBF Shield.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CbfStateVector9 {
    /// Component 1: ADM mass difference `|M_feedstock - M_payload|` in ug (must be 0)
    pub adm_mass_diff_ug: u64,
    /// Component 2: Electric Weyl curvature norm `||E_munu||` and Sugawara shift `|Delta h_vis|` (micro-nats)
    pub weyl_e_munu_norm: u64,
    pub abs_delta_h_vis_micro_nats: i64,
    /// Component 3: Net `SO(10)` `(B - L)` charge balance (must be 0)
    pub net_b_minus_l: i64,
    /// Component 4: Free-streaming neutrino flux `Phi_nu` (must be 0)
    pub phi_nu_flux: u64,
    /// Component 5: Cryogenic temperatures `T_QC <= 4,200 mK` and `T_MAR <= 192,500 mK`
    pub temp_qc_mk: u32,
    pub temp_mar_mk: u32,
    pub reactor_gen: ReactorGeneration,
    /// Component 6: Bremsstrahlung power (`0 W`) and Eikonal acceleration (`<= 5,774 milli`)
    pub p_brem_watts: u64,
    pub eikonal_accel_milli: u32,
    /// Component 7: Superhydride Wyckoff tether jitter (`<= 10 fm`) and magnetic field (`>= 48,000 mT`)
    pub wyckoff_jitter_fm: u32,
    pub superhydride_field_mt: u32,
    /// Component 8: Mechanical Frenkel shear stress (`<= 4,800 MPa`)
    pub seed_shear_stress_mpa: u32,
    /// Component 9: 2PN metric causal horizon (`ds^2_2PN <= 0`)
    pub causal_horizon_timelike: bool,
}

impl CbfStateVector9 {
    /// Nominal Tier 4 `25.00 EW` `SHBT-MAR-312X` operating state satisfying all 9 CBF barriers.
    #[inline]
    pub const fn nominal_mar312x_tier4() -> Self {
        Self {
            adm_mass_diff_ug: 0,
            weyl_e_munu_norm: 0,
            abs_delta_h_vis_micro_nats: 703_125, // 0.703125 nats <= 38.196601 nats
            net_b_minus_l: 0,
            phi_nu_flux: 0,
            temp_qc_mk: 4_200,
            temp_mar_mk: 24_850, // 24.85 K <= 192.50 K
            reactor_gen: ReactorGeneration::Mar312xExotic,
            p_brem_watts: 0,
            eikonal_accel_milli: 5_773,
            wyckoff_jitter_fm: 10,
            superhydride_field_mt: 48_000,
            seed_shear_stress_mpa: 785, // 785 MPa <= 4,800 MPa
            causal_horizon_timelike: true,
        }
    }
}

/// Proof-Carrying `Linear<V>` 9-Component Control Barrier Function (`h_phys(x) >= 0`) Safety Shield.
pub struct LinearVCbfSafetyShield;

impl LinearVCbfSafetyShield {
    #[inline]
    pub const fn classify_xenobiology(d_cayley: u32, c_mera_q16: u64) -> XenoTriageClass {
        if d_cayley == 0 {
            XenoTriageClass::X0ProtectedEarth
        } else if c_mera_q16 >= C_CRIT_SAPIENT_Q16 {
            XenoTriageClass::X1ProtectedSapient
        } else if c_mera_q16 > 0 {
            XenoTriageClass::X2AssimilablePrim
        } else {
            XenoTriageClass::M0AbioticRegolith
        }
    }

    /// Evaluates all 9 components of `h_phys(x) >= 0` plus xenobiological triage lockouts (`X0, X1`).
    #[inline]
    pub const fn evaluate_9_component_barrier(
        requested_cmd: u32,
        state: &CbfStateVector9,
        d_cayley: u32,
        c_mera_q16: u64,
    ) -> ShieldVerdict {
        let mut fault: u32 = 0;

        // Barrier 1: ADM mass conservation
        if state.adm_mass_diff_ug != 0 {
            fault |= FAULT_ADM_IMBALANCE;
        }
        // Barrier 2: Electric Weyl curvature & Dynamic Sugawara framing capacity
        if state.weyl_e_munu_norm != 0 {
            fault |= FAULT_FRAMING_WEYL;
        }
        if state.abs_delta_h_vis_micro_nats > C_BRAID_MAX_MICRO_NATS
            || state.abs_delta_h_vis_micro_nats < -C_BRAID_MAX_MICRO_NATS
        {
            fault |= FAULT_SUGAWARA_CAP_EXCEEDED | FAULT_FRAMING_WEYL;
        }
        // Barrier 3: Global SO(10) (B-L) superselection
        if state.net_b_minus_l != 0 {
            fault |= FAULT_BL_ANOMALY;
        }
        // Barrier 4: Zero free-streaming neutrino flux
        if state.phi_nu_flux != 0 {
            fault |= FAULT_NEUTRINO_LEAK;
        }
        // Barrier 5: Cryogenic & Superhydride thermal bounds
        let mar_limit_mk = match state.reactor_gen {
            ReactorGeneration::Mar312cCompacted => MAX_MAR312C_TEMP_MK,
            ReactorGeneration::Mar312xExotic => MAX_MAR312X_TEMP_MK,
        };
        if state.temp_qc_mk > MAX_QC_TEMP_MK || state.temp_mar_mk > mar_limit_mk {
            fault |= FAULT_THERMAL_CLAMP;
        }
        // Barrier 6: Bremsstrahlung suppression & Eikonal minimum-jerk kinematics
        if state.p_brem_watts != 0 || state.eikonal_accel_milli > MAX_EIKONAL_ACCEL_MILLI {
            fault |= FAULT_CBF_PHYS_VIOL;
        }
        // Barrier 7: Superhydride Wyckoff framing tether & 48.0 T magnetic confinement
        if state.wyckoff_jitter_fm > MAX_WYCKOFF_JITTER_FM
            || state.superhydride_field_mt < MIN_SUPERHYDRIDE_FIELD_MT
        {
            fault |= FAULT_SUPERHYDRIDE_TETHER_LOSS;
        }
        // Barrier 8: Mechanical Frenkel shear limit (sigma_max <= 4,800.00 MPa)
        if state.seed_shear_stress_mpa > MAX_FRENKEL_SHEAR_MPA {
            fault |= FAULT_FORGE_SHEAR_EXCEEDED;
        }
        // Barrier 9: Metric causal horizon & X0/X1 xenobiological protection
        if !state.causal_horizon_timelike {
            fault |= FAULT_CBF_PHYS_VIOL;
        }

        let triage = Self::classify_xenobiology(d_cayley, c_mera_q16);
        let is_derender_or_assimilate =
            (requested_cmd & (0x0000_0004 | 0x0000_0010 | 0x0000_1000)) != 0;
        if is_derender_or_assimilate {
            match triage {
                XenoTriageClass::X0ProtectedEarth => {
                    fault |= FAULT_X0_PROTECT_LCK;
                }
                XenoTriageClass::X1ProtectedSapient => {
                    fault |= FAULT_X1_SAPIENT_LCK;
                }
                XenoTriageClass::X2AssimilablePrim | XenoTriageClass::M0AbioticRegolith => {}
            }
        }

        if fault != 0 {
            ShieldVerdict::BlockedFault { fault_mask: fault }
        } else {
            ShieldVerdict::Approved {
                cmd_opcode: requested_cmd,
                barriers_verified: 9,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_9_component_cbf_shield_nominal_and_faults() {
        let s = CbfStateVector9::nominal_mar312x_tier4();
        let v_ok = LinearVCbfSafetyShield::evaluate_9_component_barrier(0x0001_0000, &s, 10, 0);
        assert_eq!(
            v_ok,
            ShieldVerdict::Approved {
                cmd_opcode: 0x0001_0000,
                barriers_verified: 9,
            }
        );

        // Breach Frenkel shear limit (> 4,800 MPa)
        let mut s_shear = s;
        s_shear.seed_shear_stress_mpa = 4_801;
        assert_eq!(
            LinearVCbfSafetyShield::evaluate_9_component_barrier(0x0001_0000, &s_shear, 10, 0),
            ShieldVerdict::BlockedFault {
                fault_mask: FAULT_FORGE_SHEAR_EXCEEDED
            }
        );

        // Breach Superhydride tether (> 10 fm)
        let mut s_tether = s;
        s_tether.wyckoff_jitter_fm = 11;
        assert_eq!(
            LinearVCbfSafetyShield::evaluate_9_component_barrier(0x0001_0000, &s_tether, 10, 0),
            ShieldVerdict::BlockedFault {
                fault_mask: FAULT_SUPERHYDRIDE_TETHER_LOSS
            }
        );

        // Attempt de-render of Class X0 or Class X1
        assert_eq!(
            LinearVCbfSafetyShield::evaluate_9_component_barrier(0x0000_0004, &s, 0, 0),
            ShieldVerdict::BlockedFault {
                fault_mask: FAULT_X0_PROTECT_LCK
            }
        );
        assert_eq!(
            LinearVCbfSafetyShield::evaluate_9_component_barrier(
                0x0000_0004,
                &s,
                5,
                C_CRIT_SAPIENT_Q16
            ),
            ShieldVerdict::BlockedFault {
                fault_mask: FAULT_X1_SAPIENT_LCK
            }
        );
    }
}
