//! Phase 3 Universal Arbitrary Matter Folding (`R_univ^SWARM[Theta(x)]`),
//! Dynamic Sugawara Framing Compensator, 230-Space-Group Kohn-Sham Lattice Projector,
//! and Sub-Millisecond Direct Boundary Bus (`24.9625 EW`) Fuel Core Breeding.
//!
//! Direct architectural transfer from `sys1own/shbt-swarm` (`crates/shbt-swarm-foundry`
//! & `crates/shbt-swarm-architect`) into `sys1own/shbt-warp` (`crates/warp-power-battery/src/universal_foundry.rs`).

use serde::{Deserialize, Serialize};
use crate::mar312x::{
    FAULT_ADM_IMBALANCE, FAULT_BL_ANOMALY, MAR312X_DIRECT_BUS_EFFICIENCY, MAR312X_SUBSYSTEMS,
    Mar312xSubsystemSpec,
};
use crate::synthesizer::FoundrySynthesisMode;

/// Exact Q32.32 fixed-point representation of C_braid = 100 / phi^2 = 38.19660112501052 nats.
/// 38.19660112501052 * 2^32 = 164_053_152_611
pub const C_BRAID_MAX_Q32: i64 = 164_053_152_611;
pub const FIB_BRAID_CHANNELS: i64 = 124;

/// 346-slot continuous Dynkin weight partition across `su(2)_26 x su(3)_8 x so(10)_312`.
pub const DYNKIN_SLOTS_SU2_26: u16 = 27;
pub const DYNKIN_SLOTS_SU3_8: u16 = 45;
pub const DYNKIN_SLOTS_SO10_312: u16 = 274;
pub const DYNKIN_SLOTS_TOTAL: u16 = DYNKIN_SLOTS_SU2_26 + DYNKIN_SLOTS_SU3_8 + DYNKIN_SLOTS_SO10_312;

const _: () = assert!(DYNKIN_SLOTS_TOTAL == 346);

/// Affine Kac-Moody integrability bounds for su(2)_26 and su(3)_8.
pub const SU2_LEVEL_K2: u8 = 26; // 2*j <= 26 => j <= 13
pub const SU3_LEVEL_K3: u8 = 8;  // lambda_1 + lambda_2 <= 8

/// Sugawara denominators 2*(k_a + g_a^v) for su(2)_26, su(3)_8, and so(10)_312.
pub const SUGAWARA_DENOM_SU2: u32 = 56;
pub const SUGAWARA_DENOM_SU3: u32 = 22;
pub const SUGAWARA_DENOM_SO10: u32 = 640;

/// Tier 4 Planetary Forge / Hyper-Impulse Direct Boundary Bus power (`25.00 EW * 0.9985 = 24.9625 EW`).
pub const TIER4_GROSS_POWER_EW: f64 = 25.00;
pub const TIER4_DIRECT_BUS_POWER_EW: f64 = TIER4_GROSS_POWER_EW * MAR312X_DIRECT_BUS_EFFICIENCY; // 24.9625 EW
pub const TIER4_DIRECT_BUS_POWER_W: f64 = 24.9625e18;

/// Sub-millisecond fuel core (376.99 kg, 1.0 CEU) breeding times on the 24.9625 EW Direct Boundary Bus.
pub const MODE1C_DIRECT_BUS_BREED_US: f64 = 11.397_095_6; // ~11.40 us (0.2845 PJ / 24.9625 EW)
pub const MODE1A_DIRECT_BUS_BREED_US: f64 = 85.007_511_3; // ~85.01 us (2.1220 PJ / 24.9625 EW)
pub const MODE1B_DIRECT_BUS_BREED_US: f64 = 858.487_731_6; // ~858.5 us (21.4300 PJ / 24.9625 EW)

/// Fault bitmasks matching `shbt_swarm_mmio.h` v3/v4.
pub const FAULT_FRAMING_WEYL: u32                = 0x0000_0004;
pub const FAULT_NEUTRINO_LEAK: u32               = 0x0000_0010;
pub const FAULT_CBF_PHYS_VIOL: u32               = 0x0000_0100;
pub const FAULT_SUGAWARA_CAP_EXCEEDED: u32       = 0x0000_2000;
pub const FAULT_DRIPLINE_UNSTABLE: u32           = 0x0000_4000;
pub const FAULT_SIM_TO_RENDER_HASH_MISMATCH: u32 = 0x0000_8000;

/// Representative material entry in the Phase 3 Universal Matter Folding lookup table.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MaterialSynthesisDescriptor {
    pub name: &'static str,
    pub space_group_id: u16,
    pub secondary_space_group_id: u16,
    pub dynkin_slot_range_a: (u16, u16),
    pub dynkin_slot_range_b: (u16, u16),
    pub dark_fraction_eta_b_ppm: u32,
    pub delta_h_vis_micro_nats: i64,
    pub delta_h_braid_per_ch_pico_nats: i64,
    pub net_framing_mod_1_micro_nats: i64,
}

/// Canonical 9-entry lookup table of representative materials synthesized by `R_univ^SWARM`.
pub const REPRESENTATIVE_MATERIALS: [MaterialSynthesisDescriptor; 9] = [
    MaterialSynthesisDescriptor {
        name: "Homochiral CHNOPS Tissue (From Class X2)",
        space_group_id: 0,
        secondary_space_group_id: 0,
        dynkin_slot_range_a: (1, 12),
        dynkin_slot_range_b: (73, 120),
        dark_fraction_eta_b_ppm: 0,
        delta_h_vis_micro_nats: 84_210,
        delta_h_braid_per_ch_pico_nats: -679_113,
        net_framing_mod_1_micro_nats: 0,
    },
    MaterialSynthesisDescriptor {
        name: "Amorphous Fused Silica Glass (a-SiO2)",
        space_group_id: 0,
        secondary_space_group_id: 0,
        dynkin_slot_range_a: (28, 40),
        dynkin_slot_range_b: (121, 156),
        dark_fraction_eta_b_ppm: 0,
        delta_h_vis_micro_nats: 192_500,
        delta_h_braid_per_ch_pico_nats: -1_552_419,
        net_framing_mod_1_micro_nats: 0,
    },
    MaterialSynthesisDescriptor {
        name: "Legacy SHBT-MAR-312C Seed Wafer",
        space_group_id: 191,
        secondary_space_group_id: 227,
        dynkin_slot_range_a: (1, 27),
        dynkin_slot_range_b: (73, 168),
        dark_fraction_eta_b_ppm: 500_000,
        delta_h_vis_micro_nats: 703_125,
        delta_h_braid_per_ch_pico_nats: -5_670_363,
        net_framing_mod_1_micro_nats: 0,
    },
    MaterialSynthesisDescriptor {
        name: "Isotopically Pure 12C CVD Diamond",
        space_group_id: 227,
        secondary_space_group_id: 227,
        dynkin_slot_range_a: (28, 36),
        dynkin_slot_range_b: (301, 318),
        dark_fraction_eta_b_ppm: 0,
        delta_h_vis_micro_nats: 2_418_750,
        delta_h_braid_per_ch_pico_nats: -19_506_048,
        net_framing_mod_1_micro_nats: 0,
    },
    MaterialSynthesisDescriptor {
        name: "Dislocation-Free Ti-6Al-4V Hull/Frame",
        space_group_id: 194,
        secondary_space_group_id: 229,
        dynkin_slot_range_a: (28, 52),
        dynkin_slot_range_b: (169, 210),
        dark_fraction_eta_b_ppm: 0,
        delta_h_vis_micro_nats: 6_842_105,
        delta_h_braid_per_ch_pico_nats: -55_178_266,
        net_framing_mod_1_micro_nats: 0,
    },
    MaterialSynthesisDescriptor {
        name: "52-Hexamer InP/InGaAsP Photonic Core",
        space_group_id: 216,
        secondary_space_group_id: 216,
        dynkin_slot_range_a: (1, 12),
        dynkin_slot_range_b: (331, 346),
        dark_fraction_eta_b_ppm: 0,
        delta_h_vis_micro_nats: 14_625_000,
        delta_h_braid_per_ch_pico_nats: -117_943_548,
        net_framing_mod_1_micro_nats: 0,
    },
    MaterialSynthesisDescriptor {
        name: "REBCO (18.5 T) Superconducting Tape",
        space_group_id: 47,
        secondary_space_group_id: 47,
        dynkin_slot_range_a: (63, 67),
        dynkin_slot_range_b: (249, 276),
        dark_fraction_eta_b_ppm: 0,
        delta_h_vis_micro_nats: 22_910_448,
        delta_h_braid_per_ch_pico_nats: -184_761_677,
        net_framing_mod_1_micro_nats: 0,
    },
    MaterialSynthesisDescriptor {
        name: "shbt-ghost 1g Metric Foundation Tile",
        space_group_id: 186,
        secondary_space_group_id: 186,
        dynkin_slot_range_a: (53, 62),
        dynkin_slot_range_b: (211, 248),
        dark_fraction_eta_b_ppm: 303_030,
        delta_h_vis_micro_nats: 34_851_200,
        delta_h_braid_per_ch_pico_nats: -281_058_065,
        net_framing_mod_1_micro_nats: 0,
    },
    MaterialSynthesisDescriptor {
        name: "Maximum Admissible Single-Frame Limit",
        space_group_id: 230,
        secondary_space_group_id: 230,
        dynkin_slot_range_a: (1, 72),
        dynkin_slot_range_b: (73, 346),
        dark_fraction_eta_b_ppm: 1_000_000,
        delta_h_vis_micro_nats: 38_196_601,
        delta_h_braid_per_ch_pico_nats: -308_037_106,
        net_framing_mod_1_micro_nats: 0,
    },
];

/// WP-1.1: Dynamic Sugawara Conformal Weight Shift & 124-Channel Fibonacci Compensator.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DynamicSugawaraCompensator {
    pub delta_h_vis_q32: i64,
    pub delta_h_braid_per_ch_q32: i64,
    pub residual_framing_q32: i64,
}

impl DynamicSugawaraCompensator {
    /// Computes the per-channel Fibonacci braid compensation shift -Delta h_vis / 124
    /// and verifies |Delta h_vis| <= C_braid = 100 / phi^2.
    #[inline]
    pub const fn compute_and_verify(delta_h_vis_q32: i64) -> Result<Self, u32> {
        let abs_h = if delta_h_vis_q32 < 0 { -delta_h_vis_q32 } else { delta_h_vis_q32 };
        if abs_h > C_BRAID_MAX_Q32 {
            return Err(FAULT_SUGAWARA_CAP_EXCEEDED | FAULT_FRAMING_WEYL);
        }
        let per_ch = -(delta_h_vis_q32 / FIB_BRAID_CHANNELS);
        let residual = 0i64;
        Ok(Self {
            delta_h_vis_q32,
            delta_h_braid_per_ch_q32: per_ch,
            residual_framing_q32: residual,
        })
    }
}

/// WP-1.2: Universal Kohn-Sham & Chern-Simons Spatial Lattice Projector across G_space in [0, 230].
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct KohnShamSpaceGroupProjector {
    pub space_group_id: u16,          // 0 = Amorphous/Chiral-Bio, 1..=230 = 3D Space Groups
    pub tether_disp_max_fm: u32,      // <= 10 fm (10^-4 Angstrom)
    pub gst225_impedance_krayl: u32,  // 1,000 ..= 100,000 kRayl (1..100 MRayl)
    pub brem_power_watts: u64,        // Strictly 0 W at E_KS = 352.48 eV
}

impl KohnShamSpaceGroupProjector {
    pub const TETHER_SPRING_N_PER_M: u64 = 51_295_000_000; // 5.1295e10 N/m
    pub const KS_SCREENING_ENERGY_MEV_MILLI: u32 = 352_480; // 352.48 eV in milli-eV
    pub const TF_SCREENING_LENGTH_MA: u32 = 482; // 0.482 Angstrom in milli-Angstrom

    #[inline]
    pub const fn validate_projector(&self) -> Result<(), u32> {
        if self.space_group_id > 230 {
            return Err(FAULT_CBF_PHYS_VIOL);
        }
        if self.tether_disp_max_fm > 10 || self.brem_power_watts != 0 {
            return Err(FAULT_CBF_PHYS_VIOL);
        }
        if self.gst225_impedance_krayl < 1_000 || self.gst225_impedance_krayl > 100_000 {
            return Err(FAULT_CBF_PHYS_VIOL);
        }
        Ok(())
    }
}

/// WP-2.1: Zero-Compute Simulation-to-Reality Isomorphism Bridge (`Delta C_post-sim == 0`).
pub struct ZeroComputeSimToRenderBridge;

impl ZeroComputeSimToRenderBridge {
    /// Verifies that the converged MERA-PINN UV boundary banks (Banks 32..63) match the
    /// transceiver frame layout and Burau-Cayley digest with zero post-sim compilation cycles.
    #[inline]
    pub const fn verify_zero_compute_direct_render(
        mera_uv_cayley_hash: [u64; 3],
        transceiver_frame_cayley_hash: [u64; 3],
        mera_uv_frame_bytes: usize,
    ) -> Result<u64, u32> {
        if mera_uv_frame_bytes != 2112 {
            return Err(FAULT_SIM_TO_RENDER_HASH_MISMATCH);
        }
        if mera_uv_cayley_hash[0] != transceiver_frame_cayley_hash[0]
            || mera_uv_cayley_hash[1] != transceiver_frame_cayley_hash[1]
            || mera_uv_cayley_hash[2] != transceiver_frame_cayley_hash[2]
        {
            return Err(FAULT_SIM_TO_RENDER_HASH_MISMATCH);
        }
        // Returns strictly 0 post-simulation compute cycles (Delta C_post-sim == 0)
        Ok(0)
    }
}

/// Receipt for synthesizing or autopoietically repairing a `SHBT-MAR-312X` subsystem or `376.99 kg` core.
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UniversalSynthesisReceipt {
    pub target_name: &'static str,
    pub space_group_id: u16,
    pub synthesized_mass_kg: f64,
    pub delta_h_vis_nats: f64,
    pub delta_h_braid_per_ch_nats: f64,
    pub framing_defect_nats: f64,
    pub wyckoff_jitter_fm: u32,
    pub post_sim_compile_flops: u64,
    pub q_internal_joules_per_bit: u64,
}

/// WP-1.1 & WP-1.3: Universal Arbitrary Matter Folding Superoperator (`R_univ^SWARM`).
pub struct UniversalMatterCloner;

impl UniversalMatterCloner {
    /// Validates affine integrability (nuclear drip-line lock), Sugawara framing capacity,
    /// and relativistic ADM / (B-L) / Q_EM / neutrino conservation before committing synthesis.
    #[allow(clippy::too_many_arguments)]
    pub const fn verify_universal_synthesis(
        two_j_su2: u8,
        lambda1_su3: u8,
        lambda2_su3: u8,
        delta_h_vis_q32: i64,
        feedstock_mass_ug: u64,
        payload_mass_ug: u64,
        net_b_minus_l: i64,
        net_q_em: i64,
        free_neutrino_flux: u64,
        projector: &KohnShamSpaceGroupProjector,
    ) -> Result<DynamicSugawaraCompensator, u32> {
        let mut fault: u32 = 0;

        // Boundary 1: Affine integrability & nuclear drip-line lock
        if two_j_su2 > SU2_LEVEL_K2 || (lambda1_su3.saturating_add(lambda2_su3)) > SU3_LEVEL_K3 {
            fault |= FAULT_DRIPLINE_UNSTABLE;
        }

        // Boundary 3: ADM mass, (B-L), Q_EM, and neutrino-free conservation
        if feedstock_mass_ug != payload_mass_ug {
            fault |= FAULT_ADM_IMBALANCE;
        }
        if net_b_minus_l != 0 || net_q_em != 0 {
            fault |= FAULT_BL_ANOMALY;
        }
        if free_neutrino_flux != 0 {
            fault |= FAULT_NEUTRINO_LEAK;
        }

        if let Err(proj_fault) = projector.validate_projector() {
            fault |= proj_fault;
        }

        // Boundary 2: Dynamic Sugawara capacity check
        match DynamicSugawaraCompensator::compute_and_verify(delta_h_vis_q32) {
            Ok(comp) if fault == 0 => Ok(comp),
            Ok(_) => Err(fault),
            Err(sug_fault) => Err(fault | sug_fault),
        }
    }

    /// Synthesize or autopoietically repair any of the 8 exotic subsystems (`SUB-01..08`) of `SHBT-MAR-312X`.
    pub fn synthesize_or_repair_mar312x_subsystem(
        sub: &Mar312xSubsystemSpec,
    ) -> Result<UniversalSynthesisReceipt, u32> {
        let proj = KohnShamSpaceGroupProjector {
            space_group_id: sub.primary_space_group,
            tether_disp_max_fm: 10,
            gst225_impedance_krayl: 35_000,
            brem_power_watts: 0,
        };
        let delta_q32 = (sub.delta_h_vis_nats * (1u64 << 32) as f64) as i64;
        let mass_ug = (sub.mar312x_mass_kg * 1.0e9) as u64;
        let _comp = Self::verify_universal_synthesis(
            16,
            4,
            4,
            delta_q32,
            mass_ug,
            mass_ug,
            0,
            0,
            0,
            &proj,
        )?;
        Ok(UniversalSynthesisReceipt {
            target_name: sub.name,
            space_group_id: sub.primary_space_group,
            synthesized_mass_kg: sub.mar312x_mass_kg,
            delta_h_vis_nats: sub.delta_h_vis_nats,
            delta_h_braid_per_ch_nats: -sub.delta_h_vis_nats / 124.0,
            framing_defect_nats: 0.0,
            wyckoff_jitter_fm: 10,
            post_sim_compile_flops: 0,
            q_internal_joules_per_bit: 0,
        })
    }

    /// Synthesize or autopoietically repair all 8 subsystems (`950.00 kg` total) of `SHBT-MAR-312X`.
    pub fn synthesize_complete_mar312x_nacelle() -> Result<[UniversalSynthesisReceipt; 8], u32> {
        Ok([
            Self::synthesize_or_repair_mar312x_subsystem(&MAR312X_SUBSYSTEMS[0])?,
            Self::synthesize_or_repair_mar312x_subsystem(&MAR312X_SUBSYSTEMS[1])?,
            Self::synthesize_or_repair_mar312x_subsystem(&MAR312X_SUBSYSTEMS[2])?,
            Self::synthesize_or_repair_mar312x_subsystem(&MAR312X_SUBSYSTEMS[3])?,
            Self::synthesize_or_repair_mar312x_subsystem(&MAR312X_SUBSYSTEMS[4])?,
            Self::synthesize_or_repair_mar312x_subsystem(&MAR312X_SUBSYSTEMS[5])?,
            Self::synthesize_or_repair_mar312x_subsystem(&MAR312X_SUBSYSTEMS[6])?,
            Self::synthesize_or_repair_mar312x_subsystem(&MAR312X_SUBSYSTEMS[7])?,
        ])
    }

    /// Compute sub-millisecond turnaround time in microseconds to breed a `376.99 kg` (1.0 CEU)
    /// fuel core on the Tier 4 `24.9625 EW` Direct Boundary Bus (`B_direct^MAR-X`, `99.850%`).
    /// - Mode 1c (`0.2845 PJ`): `11.40 us`
    /// - Mode 1a (`2.1220 PJ`): `85.01 us`
    /// - Mode 1b (`21.4300 PJ` raw slag): `858.5 us`
    pub fn direct_boundary_bus_core_breed_time_us(mode: FoundrySynthesisMode) -> f64 {
        let energy_j = mode.input_energy_pj() * 1.0e15;
        (energy_j / TIER4_DIRECT_BUS_POWER_W) * 1.0e6
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dynamic_sugawara_compensator_and_9_materials() {
        let ok = DynamicSugawaraCompensator::compute_and_verify(C_BRAID_MAX_Q32).unwrap();
        assert_eq!(ok.residual_framing_q32, 0);

        let err = DynamicSugawaraCompensator::compute_and_verify(C_BRAID_MAX_Q32 + 1).unwrap_err();
        assert_eq!(err, FAULT_SUGAWARA_CAP_EXCEEDED | FAULT_FRAMING_WEYL);

        for mat in &REPRESENTATIVE_MATERIALS {
            let proj = KohnShamSpaceGroupProjector {
                space_group_id: mat.space_group_id,
                tether_disp_max_fm: 10,
                gst225_impedance_krayl: 27_300,
                brem_power_watts: 0,
            };
            let delta_q32 = (mat.delta_h_vis_micro_nats << 32) / 1_000_000;
            let res = UniversalMatterCloner::verify_universal_synthesis(
                16,
                4,
                4,
                delta_q32,
                5_892_300_000_000,
                5_892_300_000_000,
                0,
                0,
                0,
                &proj,
            );
            assert!(res.is_ok(), "Material {} failed synthesis check", mat.name);
        }
    }

    #[test]
    fn test_synthesize_mar312x_nacelle_and_sub_ms_core_breeding() {
        let receipts = UniversalMatterCloner::synthesize_complete_mar312x_nacelle().unwrap();
        let total_mass: f64 = receipts.iter().map(|r| r.synthesized_mass_kg).sum();
        assert!((total_mass - 950.00).abs() < 1e-9);
        assert!(receipts.iter().all(|r| r.post_sim_compile_flops == 0 && r.q_internal_joules_per_bit == 0));

        let t_1c = UniversalMatterCloner::direct_boundary_bus_core_breed_time_us(
            FoundrySynthesisMode::Mode1cSalvageIsomerBaryons,
        );
        let t_1a = UniversalMatterCloner::direct_boundary_bus_core_breed_time_us(
            FoundrySynthesisMode::Mode1aNaturalFeedstock,
        );
        let t_1b = UniversalMatterCloner::direct_boundary_bus_core_breed_time_us(
            FoundrySynthesisMode::Mode1bAsteroidRegolith,
        );
        assert!((t_1c - 11.40).abs() < 0.01);
        assert!((t_1a - 85.01).abs() < 0.01);
        assert!((t_1b - 858.49).abs() < 0.02);
    }
}
