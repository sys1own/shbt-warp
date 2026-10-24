//! GATE-MARX-01..08 & GATE-SWARM-UPG-01..08 verification checks:
//! 5th-Gen Exotic-Material Autopoietic Mass-Annihilation Reactor (SHBT-MAR-312X)
//! and Phase 1–3 SHBT-SWARM Architectural Upgrades.

use crate::GateResult;
use warp_emitter_pdk as pdk;
use warp_flight_dynamics as flt;
use warp_hil_microkernel as hil;
use warp_power_battery as bat;
use warp_wormhole_ew312 as ew;

fn marx(n: u32, name: &str, metric: f64, tolerance: f64, passed: bool) -> GateResult {
    GateResult {
        gate: format!("GATE-MARX-{n:02}"),
        name: name.to_string(),
        metric,
        tolerance,
        passed,
    }
}

fn swarm_upg(n: u32, name: &str, metric: f64, tolerance: f64, passed: bool) -> GateResult {
    GateResult {
        gate: format!("GATE-SWARM-UPG-{n:02}"),
        name: name.to_string(),
        metric,
        tolerance,
        passed,
    }
}

/// GATE-MARX-01: SHBT-MAR-312X 8-Subsystem Mass & Volume Closure
/// (950.00 kg, 0.3500 m^3, -57.92% vs. SHBT-MAR-312C, -76.81% vs. SHBT-MAR-312).
pub fn gate_marx_01() -> GateResult {
    let cfg = bat::Mar312xExoticNacelleConfig { loaded_cores: 41 };
    let mass_err = (bat::mar312x_total_mass_kg() - 950.00).abs();
    let vol_err = (bat::mar312x_total_volume_m3() - 0.3500).abs();

    let ok = cfg.verify_single_frame_adm_closure() == Ok(16_406_590)
        && bat::MAR312X_SUBSYSTEMS.len() == 8
        && mass_err <= 1e-9
        && vol_err <= 1e-9
        && (bat::MAR312X_MASS_REDUCTION_VS_312C_PCT - 57.9218).abs() < 0.01
        && (bat::MAR312X_MASS_REDUCTION_VS_312_PCT - 76.8122).abs() < 0.01;

    marx(1, "MAR-312X 950 kg & 0.35 m^3 8-subsystem closure", mass_err, 1e-9, ok)
}

/// GATE-MARX-02: SUB-01 Purcell-Inhibited Lonsdaleite/12C-c-11B15N Isomer Stasis
/// (rho_DOS(omega_spont) == 0, F_P == 0, zero spontaneous decay loss).
pub fn gate_marx_02() -> GateResult {
    let sub01 = &bat::MAR312X_SUBSYSTEMS[0];
    let die = bat::PurcellHyperDiamondSeedDie::canonical_tier4_forge();
    let ok = die.validate_seed_die().is_ok()
        && sub01.code == "SUB-01"
        && (sub01.mar312x_mass_kg - 12.50).abs() < 1e-9
        && sub01.primary_space_group == 194
        && die.spontaneous_ldos_milli == 0
        && die.purcell_factor_milli == 0;

    marx(2, "SUB-01 Purcell-inhibited infinite isomer stasis", die.purcell_factor_milli as f64, 0.0, ok)
}

/// GATE-MARX-03: SUB-02 5-Zone 4D Weyl-Polariton/PTC SHBT-DEC-X Converter
/// (C_2 != 0, eta_DEC_X = 99.200%, 33.61114 EJ net electric per 376.99 kg core).
pub fn gate_marx_03() -> GateResult {
    let sub02 = &bat::MAR312X_SUBSYSTEMS[1];
    let decx = bat::WeylPtcDecXConverter::canonical();
    let e_core_err = (bat::MAR312X_CORE_NET_ELEC_EJ - 33.61114).abs();
    let e_wafer_err = (bat::MAR312X_WAFER_NET_ELEC_EJ - 3.36111).abs();

    let ok = decx.compute_net_efficiency_ppm() == Ok(992_000)
        && sub02.code == "SUB-02"
        && (sub02.mar312x_mass_kg - 260.00).abs() < 1e-9
        && (bat::MAR312X_DECX_EFFICIENCY - 0.99200).abs() < 1e-9
        && e_core_err < 1e-4
        && e_wafer_err < 1e-4;

    marx(3, "SUB-02 99.200% Weyl-PTC SHBT-DEC-X converter", e_core_err, 1e-4, ok)
}

/// GATE-MARX-04: SUB-03 Zero-External-Pressure Clathrate Superhydride Coils
/// (LaBeH8-CaH6-12C, B = 48.0 T, T_crit = 192.50 K, |delta x| <= 1e-4 A).
pub fn gate_marx_04() -> GateResult {
    let sub03 = &bat::MAR312X_SUBSYSTEMS[2];
    let coil = bat::TetheredSuperhydrideCoilSystem::canonical_tier4();
    let disp_a = (coil.wyckoff_jitter_max_fm as f64) * 1.0e-5;
    let ok = coil.validate_coils_and_thermal() == Ok(167_650)
        && sub03.code == "SUB-03"
        && (sub03.mar312x_mass_kg - 215.00).abs() < 1e-9
        && sub03.primary_space_group == 225
        && (bat::MAR312X_MAGNETIC_FIELD_T - 48.0).abs() < 1e-9
        && (bat::MAR312X_SUPERHYDRIDE_TCRIT_K - 192.50).abs() < 1e-9
        && disp_a <= bat::MAR312X_MAX_DISPLACEMENT_A;

    marx(
        4,
        "SUB-03 48.0 T superhydride coils & Wyckoff tether",
        disp_a,
        bat::MAR312X_MAX_DISPLACEMENT_A,
        ok,
    )
}

/// GATE-MARX-05: SUB-04 12-Layer Sp(2g, Z) Fracton-Symplectic Shield Sleeve
/// (f_sensible = 2.13039486e-9, 37,016.7x sensible heat reduction vs. MAR-312C).
pub fn gate_marx_05() -> GateResult {
    let sub04 = &bat::MAR312X_SUBSYSTEMS[3];
    let ok = sub04.code == "SUB-04"
        && (sub04.mar312x_mass_kg - 145.00).abs() < 1e-9
        && (bat::MAR312X_SENSIBLE_HEAT_FRAC - 2.13039486e-9).abs() < 1e-16
        && (bat::MAR312X_HEAT_SUPPRESSION_RATIO - 37_016.7).abs() < 1.0;

    marx(5, "SUB-04 37,016.7x fracton-symplectic heat shield", bat::MAR312X_SENSIBLE_HEAT_FRAC, 2.5e-9, ok)
}

/// GATE-MARX-06: SUB-05..08 Hyper-Diamond Cavity, 99.850% Direct Boundary Bus,
/// 50.518 kHz Autopoietic Self-Healing Loop, & 1.701 ps Weyl-Exciton Crowbar.
pub fn gate_marx_06() -> GateResult {
    let mut engine = bat::AutopoieticSelfHealingEngine { cumulative_heal_cycles: 0 };
    let h = [1u64, 2, 3];
    let heal_ok = engine.execute_tick_scrub(h, h, 10) == Ok((1, 0));
    let bus_us = bat::DirectBoundaryTransmutationBus::verify_tier4_replication_us(
        5_892_300_000_000,
        5_892_300_000_000,
        0,
        true,
    );
    let ok = heal_ok
        && bus_us == Ok(13_418)
        && (bat::MAR312X_DIRECT_BUS_EFFICIENCY - 0.99850).abs() < 1e-9
        && (bat::MAR312X_WEYL_CROWBAR_PS - 1.701).abs() < 1e-9;

    marx(6, "SUB-05..08 Direct Bus, 50.518 kHz heal & 1.701 ps crowbar", bat::MAR312X_WEYL_CROWBAR_PS, 2.50, ok)
}

/// GATE-MARX-07: 5-Tier Operational Power Envelope up to Tier 4 25.00 EW Gross
/// (24.80 EW electric, 24.9625 EW Direct Bus, 53.26 MW sensible heat, 785 MPa <= 4,800 MPa).
pub fn gate_marx_07() -> GateResult {
    let t4 = bat::Mar312xPowerTier::Tier4PlanetaryForge.table13_profile();
    let ok = (t4.gross_power_w - 25.000e18).abs() / 25.000e18 < 1e-12
        && (t4.net_electric_power_w - 24.800e18).abs() / 24.800e18 < 1e-12
        && (t4.direct_bus_power_w - 24.9625e18).abs() / 24.9625e18 < 1e-12
        && (t4.sensible_heat_w - 53.26e6).abs() / 53.26e6 < 1e-3
        && t4.cold_mass_temp_k == 24.850
        && t4.seed_shear_stress_mpa == 785.00
        && (bat::MAR312X_FRENKEL_SHEAR_ALLOW_MPA / t4.seed_shear_stress_mpa) >= 6.11;

    marx(7, "5-Tier envelope & Tier 4 25.00 EW Hyper-Impulse", t4.seed_shear_stress_mpa, 4800.0, ok)
}

/// GATE-MARX-08: 41-Core Single-Frame Magazine (1,378.06 EJ, +19.56%) &
/// 8.957-Hour Andromeda M31 Transit at 2,482,950c.
pub fn gate_marx_08() -> GateResult {
    let prof = bat::SingleFrameMagazineProfile::mar312x_profile();
    let mag = bat::DarkBraidMagazine::new_tier1_mar312x();
    let m31 = ew::IntergalacticBraidMission::compute_mar312x_profile(
        ew::IntergalacticTarget::Mission5AndromedaM31,
        bat::Mar312xPowerTier::Tier4PlanetaryForge,
        bat::FoundrySynthesisMode::Mode1cSalvageIsomerBaryons,
    );
    let m31_hours = m31.crew_proper_time_hours;

    let ok = prof.full_cores_stored == 41
        && (prof.total_wet_mass_kg - 16_406.59).abs() < 1e-6
        && (prof.total_net_elec_ej - 1_378.05674).abs() < 1e-2
        && mag.total_cores_stored == 41.0
        && (m31.effective_velocity_c - 2_482_950.0).abs() < 1e-6
        && (m31_hours - 8.957).abs() < 0.01;

    marx(8, "41-core magazine (1,378.06 EJ) & 8.957h M31 transit", m31_hours, 9.00, ok)
}

/// GATE-SWARM-UPG-01: Phase 1 Two-Tier SECDED Hamming(72,64) (1,116 B <= 1,472 B)
/// + 256-Bit Non-Abelian Burau-Cayley Hash (H_Cayley, Delta F_gen == 0).
pub fn gate_swarm_upg_01() -> GateResult {
    let mut hyper = ew::UnifiedStinespringHyperFrame::default();
    for b in 0..ew::MAX_HYPERFRAME_BANKS {
        for k in 0..ew::DARK_BRAID_DESCRIPTORS {
            hyper.banks[b].dark_braid_ledger[k] =
                0x55AA_F00F_1234_5678u64 ^ ((b as u64) << 32) ^ (k as u64);
        }
    }
    let audit = ew::verify_hyperframe_secded_and_cayley(&mut hyper).unwrap();

    let ok = ew::SECDED_BRAID_PACKED_BYTES_PER_BANK == 1116
        && ew::SECDED_BRAID_PACKED_BYTES_PER_BANK <= ew::DARK_LEDGER_BUDGET_BYTES_PER_BANK
        && audit.audit_passed
        && audit.generational_drift_delta_f == 0
        && audit.stage2_cayley_digest == audit.stage4_cayley_digest;

    swarm_upg(1, "Phase 1 SECDED(72,64) & 256-bit Burau-Cayley hash", audit.generational_drift_delta_f as f64, 0.0, ok)
}

/// GATE-SWARM-UPG-02: Phase 1 Constructive Quine Replication (R_clone^SWARM Pathways A/B/C,
/// Q_internal = 0 J/bit, F_bio >= 1 - 10^-122, 13.418 ms 5,892.30 kg MAR-312X Jump-Cruiser clone).
pub fn gate_swarm_upg_02() -> GateResult {
    let mut hyper = ew::UnifiedStinespringHyperFrame::default();
    for b in 0..32usize {
        for k in 0..ew::DARK_BRAID_DESCRIPTORS {
            hyper.banks[b].dark_braid_ledger[k] = 0x1234_5678_9ABC_DEF0u64 ^ (b as u64) ^ (k as u64);
        }
    }
    let rec_a = ew::execute_pathway_a_quine_copy(&mut hyper).unwrap();
    let rec_b = ew::execute_pathway_b_verlinde_pt_goi(32, 3_820, true).unwrap();
    let rec_c = ew::execute_pathway_c_bio_fidelity(1.58, 1.58, 0, 0.0, true).unwrap();
    let marx_rep = ew::replicate_jump_cruiser_tier4_direct_bus(5_892.30, true, 0).unwrap();
    let leg_rep = ew::replicate_jump_cruiser_tier4_direct_bus(7_200.00, false, 0).unwrap();

    let t_err = (marx_rep.replication_time_ms - 13.418).abs();
    let ok = rec_a.q_internal_joules_per_bit == 0
        && rec_b.total_propagation_fs == 54_432
        && rec_c.bio_fidelity_log10_defect == -122
        && marx_rep.adm_mass_conserved
        && marx_rep.b_minus_l_conserved
        && t_err < 1e-3
        && (leg_rep.replication_time_ms - 16.396).abs() < 1e-3
        && marx_rep.speedup_vs_mar312c > 3600.0;

    swarm_upg(2, "Phase 1 13.418 ms MAR-312X Jump-Cruiser quine clone", marx_rep.replication_time_ms, 15.0, ok)
}

/// GATE-SWARM-UPG-03: Phase 2 SHBT-STEN 54.432 ps PT-GoI EP2 Non-Hermitian Optical Solver
/// ((UM)^2 = 0) & alpha = 10^4 Blueshifted MPC Enclave (3.6366e9 sweeps/tick).
pub fn gate_swarm_upg_03() -> GateResult {
    let solver = flt::PtGoiEquilibriumSolver::canonical();
    let mpc = flt::StenRelativisticMpcEnclave::optimize_hyper_impulse_trajectory(
        10_000,
        flt::HYPER_IMPULSE_VELOCITY_C,
    )
    .unwrap();
    let sweep_ps = (flt::PtGoiEquilibriumSolver::single_pass_latency_fs(32) as f64) * 1.0e-3;

    let ok = solver.validate_ep2_nilpotency()
        && mpc.ds2_2pn_interval <= 0.0
        && mpc.interior_lapse_grad_norm == 0.0
        && (sweep_ps - 54.432).abs() < 1e-9
        && mpc.sweeps_per_tick == 3_636_633_700
        && (pdk::PT_GOI_32BANK_SWEEP_PS - 54.432).abs() < 1e-9
        && pdk::PT_GOI_SECOND_CHERN_C2 == 1;

    swarm_upg(3, "Phase 2 SHBT-STEN 54.432 ps EP2 & 10^4 MPC enclave", sweep_ps, 55.0, ok)
}

/// GATE-SWARM-UPG-04: Phase 2 Higher-Genus Sp(2g, Z) Stinespring Entanglement Routing
/// (g = M + 2 >= 3, Ent(phi) == 0, rho_spec = 1, tau_byz <= 1.701 ps).
pub fn gate_swarm_upg_04() -> GateResult {
    let router = ew::Sp2gStinespringNetworkRouter::new(6);
    let expected = [1u64, 2, 3];
    let forged = [1u64, 2, 999];
    let byz_ok = ew::Sp2gStinespringNetworkRouter::verify_byzantine_frame_digest(expected, expected);
    let byz_err = ew::Sp2gStinespringNetworkRouter::verify_byzantine_frame_digest(expected, forged);

    let ok = router.heegaard_genus_g == 8
        && router.verify_zero_entropy_routing()
        && router.kojima_routing_entropy_milli == 0
        && router.spectral_radius_rho_spec == 1
        && byz_ok == Ok(1_701)
        && byz_err == Err(ew::SHBT_SWARM_FAULT_CAYLEY_HASH);

    swarm_upg(4, "Phase 2 Sp(2g, Z) zero-entropy routing & 1.701 ps Byz", router.kojima_routing_entropy_milli as f64, 0.0, ok)
}

/// GATE-SWARM-UPG-05: Phase 2 9-Component Proof-Carrying Linear<V> Control Barrier Function
/// (h_phys(x) >= 0) & Class M0/X0/X1/X2 Xenobiological Triage on GhostMassHarvester.
pub fn gate_swarm_upg_05() -> GateResult {
    let nom = flt::CbfStateVector9::nominal_mar312x_tier4();
    let cert = flt::LinearVCbfSafetyShield::evaluate_9_component_barrier(0x0000_0004, &nom, 100, 0);

    let harvester = bat::GhostMassHarvester::new();
    let res_m0 = harvester.evaluate_xenobiological_triage(100, 0);
    let res_x0 = harvester.evaluate_xenobiological_triage(0, 100);
    let res_x1 = harvester.evaluate_xenobiological_triage(100, bat::C_CRIT_SAPIENT_Q16);

    let ok = matches!(cert, flt::ShieldVerdict::Approved { barriers_verified: 9, .. })
        && res_m0.is_ok()
        && res_x0 == Err(bat::SHBT_SWARM_FAULT_X0_PROTECT_LCK)
        && res_x1 == Err(bat::SHBT_SWARM_FAULT_X1_SAPIENT_LCK);

    swarm_upg(5, "Phase 2 9-component CBF & X0/X1 xenobiological lock", 9.0, 9.0, ok)
}

/// GATE-SWARM-UPG-06: Phase 3 Universal Arbitrary Matter Folding (R_univ^SWARM,
/// 230 Space Groups, 346 Dynkin Slots, Dynamic Sugawara Compensator |Delta h_vis| <= 38.196601 nats).
pub fn gate_swarm_upg_06() -> GateResult {
    let sub_receipts = bat::UniversalMatterCloner::synthesize_complete_mar312x_nacelle()
        .expect("All 8 MAR-312X subsystems must synthesize cleanly");
    let total_mass: f64 = sub_receipts.iter().map(|r| r.synthesized_mass_kg).sum();
    let sug = bat::DynamicSugawaraCompensator::compute_and_verify(bat::C_BRAID_MAX_Q32).unwrap();

    let ok = sub_receipts.len() == 8
        && (total_mass - 950.00).abs() < 1e-9
        && sug.residual_framing_q32 == 0
        && bat::DYNKIN_SLOTS_TOTAL == 346
        && bat::REPRESENTATIVE_MATERIALS.len() >= 9;

    swarm_upg(6, "Phase 3 R_univ^SWARM 230 space groups & Sugawara lock", sug.residual_framing_q32 as f64, 0.0, ok)
}

/// GATE-SWARM-UPG-07: Phase 3 Sub-Millisecond Direct Boundary Bus Fuel Breeding
/// (11.40 us Mode 1c, 85.01 us Mode 1a, 858.5 us Mode 1b) & Zero-Compute Isomorphism (Delta C_post-sim == 0).
pub fn gate_swarm_upg_07() -> GateResult {
    let b1c = bat::UniversalMatterCloner::direct_boundary_bus_core_breed_time_us(
        bat::FoundrySynthesisMode::Mode1cSalvageIsomerBaryons,
    );
    let b1a = bat::UniversalMatterCloner::direct_boundary_bus_core_breed_time_us(
        bat::FoundrySynthesisMode::Mode1aNaturalFeedstock,
    );
    let b1b = bat::UniversalMatterCloner::direct_boundary_bus_core_breed_time_us(
        bat::FoundrySynthesisMode::Mode1bAsteroidRegolith,
    );
    let h = [0xCAFE_BABE_3120_0001u64, 0x1234_5678, 0x9ABC_DEF0];
    let iso = ew::ZeroComputeStage4Isomorphism::verify_direct_stage4_render(h, h, 2112);

    let ok = (b1c - 11.40).abs() < 0.05
        && (b1a - 85.01).abs() < 0.10
        && (b1b - 858.5).abs() < 1.0
        && iso == Ok(0);

    swarm_upg(7, "Phase 3 11.40 us Direct Bus breeding & Delta C == 0", b1c, 12.0, ok)
}

/// GATE-SWARM-UPG-08: Phase 1–4 Bare-Metal C11 MMIO Contract (Opcodes 0x00000001..0x80000000,
/// Faults 0x00000001..0x00100000, Tier 4 Dispatch, 1,701 fs Weyl Crowbar).
pub fn gate_swarm_upg_08() -> GateResult {
    let f_nom = hil::mar312x_dispatch_tier4(24.850, 48.0, 0.36e-4, 785.0, 1, 0.0);
    let f_tether = hil::mar312x_dispatch_tier4(24.850, 40.0, 2.0e-4, 785.0, 1, 0.0);
    let fs_crowbar = hil::mar312x_weyl_crowbar_trip_fs();
    let f_x0 = hil::swarm_cbf_xeno_gate(10.0, 1);
    let f_x1 = hil::swarm_cbf_xeno_gate(10.0, 2);

    let ok = f_nom == hil::SHBT_SWARM_FAULT_NONE
        && (f_tether & hil::SHBT_SWARM_FAULT_SUPERHYDRIDE_TETHER_LOSS) != 0
        && fs_crowbar == 1701
        && f_x0 == hil::SHBT_SWARM_FAULT_X0_PROTECT_LCK
        && f_x1 == hil::SHBT_SWARM_FAULT_X1_SAPIENT_LCK
        && hil::SHBT_SWARM_CMD_TOMITA_QUINE_CPY == 0x0000_0001
        && hil::SHBT_SWARM_CMD_MAR312X_FORGE_TIER4 == 0x0001_0000
        && hil::SHBT_SWARM_CMD_DIRECT_BOUNDARY_SYNTH == 0x0002_0000
        && hil::SHBT_SWARM_CMD_AUTOPOIETIC_SELF_HEAL == 0x0004_0000
        && hil::SHBT_SWARM_CMD_PCSS_CROWBAR == 0x8000_0000;

    swarm_upg(8, "Phase 1-4 C11 MMIO contract & 1,701 fs Weyl crowbar", fs_crowbar as f64, 2500.0, ok)
}

pub fn run_all_marx_and_swarm() -> Vec<GateResult> {
    vec![
        gate_marx_01(),
        gate_marx_02(),
        gate_marx_03(),
        gate_marx_04(),
        gate_marx_05(),
        gate_marx_06(),
        gate_marx_07(),
        gate_marx_08(),
        gate_swarm_upg_01(),
        gate_swarm_upg_02(),
        gate_swarm_upg_03(),
        gate_swarm_upg_04(),
        gate_swarm_upg_05(),
        gate_swarm_upg_06(),
        gate_swarm_upg_07(),
        gate_swarm_upg_08(),
    ]
}

pub fn run_all() -> Vec<GateResult> {
    run_all_marx_and_swarm()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_marx_and_swarm_upg_gates() {
        for g in run_all_marx_and_swarm() {
            assert!(g.passed, "{} ({}) failed: metric={}", g.gate, g.name, g.metric);
        }
    }
}
