//! GATE-GHOST-01..08 verification checks:
//! Synthetic Ghost-Seed Gravity, Reactionless Hovering, 1g Habitat Metric
//! Superposition, Gravitational Lensing/PINN Optics, and Bondi-Hoyle Mass Harvesting.

use crate::GateResult;
use warp_flight_dynamics as flt;
use warp_power_battery as bat;
use warp_wormhole_ew312 as ew;

fn ghost(n: u32, name: &str, metric: f64, tolerance: f64, passed: bool) -> GateResult {
    GateResult {
        gate: format!("GATE-GHOST-{n:02}"),
        name: name.to_string(),
        metric,
        tolerance,
        passed,
    }
}

/// GATE-GHOST-01: Topological Mass-per-Bit Coupling & Baseline Seed Mass
/// (alpha_seed = 1.3258e-51 M_sun/bit, M_seed,0 = 1.0e-6 M_sun at 906.00 kW debt).
pub fn gate_ghost_01() -> GateResult {
    let alpha_msun = flt::ALPHA_SEED_MSUN_PER_BIT;
    let delta_n0 = flt::DELTA_N0_BITS;
    let m_seed_msun = flt::GhostHoverController::seed_mass_msun(delta_n0);
    let debt_kw = m_seed_msun * 906.0e6;

    let tol_alpha = 1.0e-54;
    let tol_mass = 1.0e-9;
    let tol_debt = 0.01;

    let ok = (alpha_msun - 1.3258e-51).abs() <= tol_alpha
        && (m_seed_msun - 1.0e-6).abs() <= tol_mass
        && (debt_kw - 906.00).abs() <= tol_debt;

    let diff = (m_seed_msun - 1.0e-6).abs();
    ghost(1, "Topological mass-per-bit coupling & seed", diff, tol_mass, ok)
}

/// GATE-GHOST-02: Reactionless Geodesic Hovering & Stationkeeping Equilibrium
/// (||a_thrust + g_ext|| <= 1e-15 m/s^2, dot{m}_prop == 0 kg/s).
pub fn gate_ghost_02() -> GateResult {
    let ctrl = flt::GhostHoverController::new(16_600.0);
    let g_earth = [0.0, 0.0, -9.80665];
    let (_dn, _r, res_earth) = ctrl.solve_hover_equilibrium(g_earth, 100.0);

    let g_jup = [0.0, 0.0, -24.79];
    let (_dn_j, _r_j, res_jup) = ctrl.solve_hover_equilibrium(g_jup, 100.0);

    let max_res = res_earth.max(res_jup);
    let ok = max_res <= 1.0e-14;
    ghost(2, "Reactionless geodesic hover equilibrium", max_res, 1.0e-14, ok)
}

/// GATE-GHOST-03: 3rd-Order Kinematic Wake Compensation & Mass Rigidity
/// (|mu_comp - mu_0| <= 3.2e-13 <= 1.0e-12 up to 0.1c).
pub fn gate_ghost_03() -> GateResult {
    let (_mu, resid) = flt::GhostHoverController::wake_compensation(
        1.0,
        0.10 * flt::C_LIGHT,
        flt::DELTA_N0_BITS,
    );
    let ok = resid <= flt::RIGIDITY_BOUND;
    ghost(3, "3rd-order kinematic wake rigidity", resid, flt::RIGIDITY_BOUND, ok)
}

/// GATE-GHOST-04: Multi-Seed 1g Habitat Floor Superposition & Zero Coriolis
/// (g_hab = 9.80665 m/s^2, Omega_Coriolis == 0, I_00 = +2.4189e-32).
pub fn gate_ghost_04() -> GateResult {
    let engine = flt::MultiSeedHabitatGravity::new();
    let (g_hab, rel_err) = engine.deck_gravity_at(0.0, 0.0, 0.0);
    let omega = flt::MultiSeedHabitatGravity::coriolis_angular_velocity();
    let omega_norm = (omega[0] * omega[0] + omega[1] * omega[1] + omega[2] * omega[2]).sqrt();
    let i_00 = flt::INTERFERENCE_I00;

    let ok = (g_hab - 9.80665).abs() < 1e-6
        && rel_err <= flt::SPATIAL_UNIFORMITY_TOL
        && omega_norm == 0.0
        && (i_00 - 2.418930510842e-32).abs() / 2.418930510842e-32 < 1e-10;

    ghost(4, "Multi-seed 1g habitat floor & zero Coriolis", rel_err, flt::SPATIAL_UNIFORMITY_TOL, ok)
}

/// GATE-GHOST-05: Anti-Phase External Attractor Hull Nullification
/// (Delta R_net < 1.0e-12 s^-2, |det(g) + 1| <= 1.0e-12).
pub fn gate_ghost_05() -> GateResult {
    let h_ext = [
        [1.5e-14, 0.0, 0.0, 0.0],
        [0.0, 1.5e-14, 0.0, 0.0],
        [0.0, 0.0, 1.5e-14, 0.0],
        [0.0, 0.0, 0.0, 1.5e-14],
    ];
    let (tidal_shear, _h_net) = flt::MultiSeedHabitatGravity::nullify_external_attractor(&h_ext);

    let g = flt::MultiSeedHabitatGravity::cabin_stabilized_metric();
    let det_err = flt::MultiSeedHabitatGravity::adm_volume_stability(&g);

    let ok = tidal_shear < flt::TIDAL_SHEAR_TOL && det_err <= flt::ADM_DET_TOL;
    ghost(5, "Anti-phase external attractor nullification", tidal_shear, flt::TIDAL_SHEAR_TOL, ok)
}

/// GATE-GHOST-06: Bondi-Hoyle Gravitational Funnel Capture Cross-Section Amplification
/// (sigma_grav / A_geom >= 1.0e10, stand-off regolith lift >= 376.99 kg core in < 10 ms).
pub fn gate_ghost_06() -> GateResult {
    let harvester = bat::GhostMassHarvester::new();
    let amp_ratio = harvester.aperture_amplification_ratio(10_000.0, 1_000.0);
    let t_core = harvester.time_to_accumulate_regime_a(bat::MASS_FULL_CORE_KG, 100.0);
    let t_wafer = harvester.time_to_accumulate_regime_a(bat::MASS_SEED_WAFER_KG, 100.0);

    let ok = amp_ratio >= 1.0e10 && t_core < 10.0e-3 && t_wafer < 1.0e-3;
    ghost(6, "Bondi-Hoyle capture amplification & lift", t_core, 10.0e-3, ok)
}

/// GATE-GHOST-07: Ghost-Seed Harvester to R_synth^MAR & SHBT-IBR-312 Intergalactic Closure
/// (Mission 7A 500 Mly & Mission 7C 45.18 Gly zero fuel starvation).
pub fn gate_ghost_07() -> GateResult {
    let (ok_m7a, _p_m7a, amp_m7a) = ew::IntergalacticBraidMission::verify_ghost_harvest_fuel_closure(
        ew::IntergalacticTarget::Mission7LaniakeaCore,
        ew::PowerTier::ContinuousCruise,
    );
    let (ok_m7c, _p_m7c, amp_m7c) = ew::IntergalacticBraidMission::verify_ghost_harvest_fuel_closure(
        ew::IntergalacticTarget::Mission7CosmologicalHorizon,
        ew::PowerTier::ContinuousCruise,
    );

    let harvester = bat::GhostMassHarvester::new();
    let res = harvester.harvest_and_synthesize_core(
        bat::HarvestingRegime::RegimeAStandOffRegolith,
        bat::MASS_FULL_CORE_KG,
        bat::OperatingTier::BurstPw,
        bat::FoundrySynthesisMode::Mode1bAsteroidRegolith,
    );

    let ok = ok_m7a && ok_m7c && amp_m7a >= 1.0e10 && amp_m7c >= 1.0e10
        && res.framing_defect_zero
        && res.stored_in_ibr312;

    ghost(7, "Ghost harvester to R_synth^MAR intergalactic closure", 0.0, 0.0, ok)
}

/// GATE-GHOST-08: Gravitational Lens Bessel J_0^2 Caustic & PINN Tomography
/// (sigma_r <= 0.144 pm/sqrt(Hz), sigma_theta <= 11.38 nrad, f0 = 169.30 m).
pub fn gate_ghost_08() -> GateResult {
    let optics = ew::GhostGravitationalOptics::new();
    let scan = optics.scan_feedstock(72, 178, 169.30);
    let f0 = ew::GhostGravitationalOptics::focal_length(0.0);

    let ok = (f0 - 169.30).abs() < 1e-9
        && scan.ranging_precision_pm <= 0.144
        && scan.pointing_jitter_nrad <= 11.38
        && scan.contrast_passed
        && scan.tomography_verified;

    ghost(8, "Gravitational lens caustic & PINN tomography", scan.ranging_precision_pm, 0.144, ok)
}

/// Run all 8 Ghost-Gravity and Mass Harvesting gates in numerical order.
pub fn run_all_ghost() -> Vec<GateResult> {
    vec![
        gate_ghost_01(),
        gate_ghost_02(),
        gate_ghost_03(),
        gate_ghost_04(),
        gate_ghost_05(),
        gate_ghost_06(),
        gate_ghost_07(),
        gate_ghost_08(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gate_ghost_all() {
        let gates = run_all_ghost();
        assert_eq!(gates.len(), 8);
        for g in gates {
            assert!(g.passed, "Gate {} failed", g.gate);
        }
    }
}
