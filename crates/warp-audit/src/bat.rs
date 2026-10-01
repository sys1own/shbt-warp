//! GATE-BAT-01..08 isomer-battery verification checks (warp2.txt):
//! coherent graser-discharged ^178m2Hf nuclear isomer battery — energy
//! density, graser gain, Borrmann suppression, DEC conversion, PCSS
//! crowbar, nacelle stress/cryo headroom and total mass budget.

use crate::GateResult;
use warp_flight_dynamics as flt;
use warp_hil_microkernel as hil;
use warp_power_battery as bat;

fn b(n: u32, name: &str, metric: f64, tolerance: f64, passed: bool) -> GateResult {
    GateResult {
        gate: format!("GATE-BAT-{n:02}"),
        name: name.to_string(),
        metric,
        tolerance,
        passed,
    }
}

/// GATE-BAT-01: gravimetric energy density rho_E >= 1.326 TJ/kg.
pub fn gate_bat_01() -> GateResult {
    b(1, "rho_E >= 1.326 TJ/kg", bat::RHO_E_TJ_KG, 1.326,
        bat::RHO_E_TJ_KG >= 1.326 && bat::RHO_E_TJ_KG <= 1.327)
}

/// GATE-BAT-02: graser gain G = E_iso / E_gateway >= 60.0 (61.15).
pub fn gate_bat_02() -> GateResult {
    let g = bat::isomer_gain();
    b(2, "G >= 60.0", g, 60.0, g >= 60.0 && (g - 61.15).abs() <= 0.01)
}

/// GATE-BAT-03: Borrmann mode suppression epsilon_B >= 0.980 (0.985).
pub fn gate_bat_03() -> GateResult {
    b(3, "eps_B >= 0.980", bat::BORRMANN_EPSILON, 0.980,
        bat::BORRMANN_EPSILON >= 0.980 && bat::BORRMANN_EPSILON <= 0.999)
}

/// GATE-BAT-04: staged DEC conversion efficiency eta_conv >= 45.0% (45.8%).
pub fn gate_bat_04() -> GateResult {
    let e = bat::conversion_efficiency() * 100.0;
    let staged = (1.0 - bat::ETA_STAGE1) * (1.0 - bat::ETA_STAGE2) * (1.0 - bat::ETA_STAGE3);
    let net = bat::ETA_STAGE1 + bat::ETA_STAGE2 + bat::ETA_STAGE3;
    b(4, "eta_conv >= 45.0%", e, 45.0, e >= 45.0 && staged > 0.0 && net <= 1.0)
}

/// GATE-BAT-05: PCSS crowbar closes <= 2.10 ns with >= 94.20% recovery.
pub fn gate_bat_05() -> GateResult {
    let ok = hil::isomer_crowbar_ok(bat::CROWBAR_CLOSE_NS, bat::INDUCTIVE_RECOVERY)
        && bat::CROWBAR_CLOSE_NS < hil::PCSS_HARD_LIMIT_NS;
    b(5, "PCSS <= 2.10 ns / rec >= 94.20%", bat::CROWBAR_CLOSE_NS, 2.10,
        ok && bat::INDUCTIVE_RECOVERY >= 0.9420)
}

/// GATE-BAT-06: transient shear stress sigma <= 125.0 MPa (124.60).
pub fn gate_bat_06() -> GateResult {
    b(6, "sigma <= 125.0 MPa", bat::SIGMA_MAX_MPA, 125.0,
        bat::SIGMA_MAX_MPA <= 125.0 && bat::SIGMA_MAX_MPA < bat::SIGMA_YIELD_MPA)
}

/// GATE-BAT-07: cryo headroom T_quench - T_op >= 11.79 K.
pub fn gate_bat_07() -> GateResult {
    let h = bat::cryo_headroom_k();
    b(7, "dT >= 11.79 K", h, 11.79, h >= 11.79 && (h - 11.79).abs() <= 0.01)
}

/// GATE-BAT-08: nacelle mass budget M <= 4250.0 kg (4199.98).
pub fn gate_bat_08() -> GateResult {
    let m = bat::nacelle_mass_kg();
    let budget = flt::burst_budget_ok();
    b(8, "M_nacelle <= 4250.0 kg", m, 4250.0,
        m <= 4250.0 && (m - bat::M_NACELLE_KG).abs() <= 0.01 && budget)
}

pub fn run_all_bat() -> Vec<GateResult> {
    vec![
        gate_bat_01(), gate_bat_02(), gate_bat_03(), gate_bat_04(),
        gate_bat_05(), gate_bat_06(), gate_bat_07(), gate_bat_08(),
    ]
}
