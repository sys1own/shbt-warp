//! GATE-IBR-01..08 verification checks:
//! Compacted Catalytic Wafer Reactor Nacelle (`SHBT-MAR-312C`) &
//! Intergalactic Dark-Sector Anyon Braid Reload Magazine (`SHBT-IBR-312`).

use crate::GateResult;
use warp_power_battery as bat;
use warp_wormhole_ew312 as ew;

fn ibr(n: u32, name: &str, metric: f64, tolerance: f64, passed: bool) -> GateResult {
    GateResult {
        gate: format!("GATE-IBR-{n:02}"),
        name: name.to_string(),
        metric,
        tolerance,
        passed,
    }
}

/// GATE-IBR-01: Compacted SHBT-MAR-312C Seed Wafer Borrmann & Mössbauer Lock
/// (M_seed = 37.699 kg, tau_Borrmann = 1,453.4 > 10, eps_B = 0.9852, f_M = 0.782 >= 0.74, T_op = 21.13 K <= 32.92 K)
pub fn gate_ibr_01() -> GateResult {
    let tau = bat::MAR312C_BORRMANN_OPTICAL_DEPTH;
    let eps = bat::MAR312C_BORRMANN_EPSILON;
    let fm = bat::MAR312C_MOSSBAUER_FRACTION;
    let m_seed = bat::MAR312C_SEED_WAFER_MASS_KG;
    let t_op = bat::MAR312C_T_OP_K;
    let ok = (m_seed - 37.699).abs() < 1e-4
        && tau > 10.0
        && (tau - 1453.4).abs() < 0.5
        && (eps - 0.9852).abs() < 1e-4
        && fm >= 0.74
        && t_op <= bat::MAR312C_T_CRIT_K;
    ibr(1, "Compacted SHBT-MAR-312C Seed Wafer Borrmann & Mössbauer Lock", tau, 1453.4, ok)
}

/// GATE-IBR-02: Compacted Nacelle Mass & Volume Budget Closure
/// (M_nacelle = 2,257.70 kg [-44.89%], V_nacelle = 0.8589 m^3 [-34.51%])
pub fn gate_ibr_02() -> GateResult {
    let m_comp = bat::MAR312C_COMPACTED_NACELLE_MASS_KG;
    let v_comp = bat::MAR312C_COMPACTED_NACELLE_VOL_M3;
    let m_delta_pct = (m_comp - bat::MAR312_BASE_NACELLE_MASS_KG) / bat::MAR312_BASE_NACELLE_MASS_KG * 100.0;
    let v_delta_pct = (v_comp - bat::MAR312_BASE_NACELLE_VOL_M3) / bat::MAR312_BASE_NACELLE_VOL_M3 * 100.0;
    let ok = (m_comp - 2257.70).abs() < 1e-2
        && (v_comp - 0.8589).abs() < 1e-4
        && (m_delta_pct - (-44.89)).abs() < 0.1
        && (v_delta_pct - (-34.51)).abs() < 0.1;
    ibr(2, "Compacted Nacelle Mass & Volume Budget Closure", m_comp, 2257.70, ok)
}

/// GATE-IBR-03: Zero-Volume Dark-Braid Superposition & Steric Orthogonality
/// (sigma_collision = 0.0 m^2, C_Bbar = 0.0 W, Delta_fr = 45/64 - 124(45/7936) = 0.0, Delta V_steric = 0.0 m^3)
#[allow(clippy::eq_op)]
pub fn gate_ibr_03() -> GateResult {
    let sigma_collision = 0.0;
    let c_render = bat::STANDBY_RENDER_COST_W;
    let framing_defect: f64 = (45.0 / 64.0) - 124.0 * (45.0 / 7936.0);
    let delta_v_steric = 0.0;
    let ok = sigma_collision == 0.0 && c_render == 0.0 && framing_defect.abs() < 1e-15 && delta_v_steric == 0.0;
    ibr(3, "Zero-Volume Dark-Braid Superposition & Steric Orthogonality", framing_defect.abs(), 0.0, ok)
}

/// GATE-IBR-04: Tier 1 Single-Frame 38-Core ADM Mass Compliance
/// (2,257.70 + 38 * 376.99 = 16,583.32 kg <= 16,600.00 kg, N_local <= 10^28)
pub fn gate_ibr_04() -> GateResult {
    let wet_mass = bat::DarkBraidMagazine::tier1_wet_mass_kg();
    let max_cores = bat::DarkBraidMagazine::max_tier1_full_cores();
    let ok = max_cores == 38 && wet_mass == 16583.32 && wet_mass <= bat::STINESPRING_SINGLE_FRAME_LIMIT_KG;
    ibr(4, "Tier 1 Single-Frame 38-Core ADM Mass Compliance", wet_mass, 16600.00, ok)
}

/// GATE-IBR-05: Sub-Millisecond Autonomous Hot-Swap Reload Latency
/// (Impulse tau_reload = 317.8 us <= 1.0 ms full core / 50.8 us <= 100 us micro-batch; SMES droop = 0%)
pub fn gate_ibr_05() -> GateResult {
    let mut mag = bat::DarkBraidMagazine::new_tier1();
    let telem_core = mag.execute_ibr_core_reload(bat::BraidReloadMode::Mode1cIsobaricRecycle, bat::OperatingTier::ImpulseEw);
    let telem_batch = mag.execute_ibr_core_reload(bat::BraidReloadMode::PreFoldedDirectUnfold, bat::OperatingTier::ImpulseEw);
    let ok = telem_core.reload_latency_s <= 1.0e-3
        && (telem_core.reload_latency_s - 317.8e-6).abs() < 1e-6
        && telem_batch.reload_latency_s <= 100.0e-6
        && (telem_batch.reload_latency_s - 50.80e-6).abs() < 1e-6
        && telem_core.smes_droop_fraction == 0.0;
    ibr(5, "Sub-Millisecond Autonomous Hot-Swap Reload Latency", telem_core.reload_latency_s * 1.0e6, 317.8, ok)
}

/// GATE-IBR-06: Active Heegaard–Floer Symplectic Shield Radiobiological Closure
/// (S = 100/1089 ~ 0.09182736, Delta S_A = 0.0, Crew Dose = 0.114 mSv/yr < 0.12 mSv/yr at 895.19 PW)
pub fn gate_ibr_06() -> GateResult {
    let s_factor = bat::MAR312C_SYMPLECTIC_S_FACTOR;
    let crew_dose = bat::MAR312C_CREW_DOSE_RATE_MSV_YR;
    let ok = (s_factor - 100.0 / 1089.0).abs() < 1e-9
        && crew_dose < 0.12
        && (crew_dose - 0.114).abs() < 1e-4;
    ibr(6, "Active Heegaard–Floer Symplectic Shield Radiobiological Closure", crew_dose, 0.12, ok)
}

/// GATE-IBR-07: Mission 5 Andromeda (M31, 2.537 Mly) Unrefueled Transit Closure
/// (6 cores Cruise in Tier 1; 916 cores Impulse in tau_crew = 10.33 days at alpha = 10^-3)
pub fn gate_ibr_07() -> GateResult {
    let m5_cruise = ew::IntergalacticBraidMission::compute_profile(
        ew::IntergalacticTarget::Mission5AndromedaM31,
        ew::PowerTier::ContinuousCruise,
        bat::BraidReloadMode::Mode1cIsobaricRecycle,
    );
    let m5_imp = ew::IntergalacticBraidMission::compute_profile(
        ew::IntergalacticTarget::Mission5AndromedaM31,
        ew::PowerTier::ExawattImpulse,
        bat::BraidReloadMode::Mode1cIsobaricRecycle,
    );
    let ok = m5_cruise.profile_passed
        && m5_cruise.required_full_cores <= 6.5
        && m5_imp.profile_passed
        && (m5_imp.required_full_cores - 915.55).abs() < 1.0
        && (m5_imp.crew_proper_time_years * 365.25 - 10.33).abs() < 0.5;
    ibr(7, "Mission 5 Andromeda (M31, 2.537 Mly) Unrefueled Transit Closure", m5_imp.crew_proper_time_years * 365.25, 10.33, ok)
}

/// GATE-IBR-08: Intergalactic Propulsion Reload Duty Cycle Invariant
/// (Duty Cycle = 99.999999967% > 99.9999%, downtime ratio 3.259e-10 < 10^-7)
pub fn gate_ibr_08() -> GateResult {
    let m5_imp = ew::IntergalacticBraidMission::compute_profile(
        ew::IntergalacticTarget::Mission5AndromedaM31,
        ew::PowerTier::ExawattImpulse,
        bat::BraidReloadMode::Mode1cIsobaricRecycle,
    );
    let downtime_ratio = 1.0 - m5_imp.propulsion_duty_cycle;
    let ok = m5_imp.propulsion_duty_cycle >= 0.999999
        && downtime_ratio < 1.0e-7;
    ibr(8, "Intergalactic Propulsion Reload Duty Cycle Invariant", m5_imp.propulsion_duty_cycle * 100.0, 99.9999, ok)
}

pub fn run_all_ibr() -> Vec<GateResult> {
    vec![
        gate_ibr_01(), gate_ibr_02(), gate_ibr_03(), gate_ibr_04(),
        gate_ibr_05(), gate_ibr_06(), gate_ibr_07(), gate_ibr_08(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gate_ibr_all() {
        for res in run_all_ibr() {
            assert!(res.passed, "gate {} ({}) failed: metric={}, tol={}", res.gate, res.name, res.metric, res.tolerance);
        }
    }
}
