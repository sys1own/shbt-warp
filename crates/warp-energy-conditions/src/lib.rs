//! Continuous point-wise energy-condition auditing on the 3+1 warp
//! foliation plus Ford-Roman Quantum Inequality stress sampling, balanced
//! against the topological dark-ledger sink (eta_D = 23/33).
//!
//! Ported from `sys1own/shbt-exotic` (exotic-energy-conditions) and the
//! legacy `src/stress_energy.rs` NEC/WEC sampler.
//!
//! Ford-Roman Lorentzian sampling:
//!   int <T_mn n^m n^n> tau0 / (pi (tau^2 + tau0^2)) dt >= -C / tau0^4
//! with C = 3 / (32 pi^2).

/// Ford-Roman bound constant C = 3 / (32 pi^2).
pub const FORD_ROMAN_C: f64 = 3.0 / (32.0 * std::f64::consts::PI * std::f64::consts::PI);

/// Planck sampling timescale floor (s) below which the QI does not apply.
pub const TAU_PLANCK_S: f64 = 5.39e-44;

/// Dark-ledger sink fractions (topological NEC-violation budget).
/// 124 Fibonacci braid descriptors in the dark ledger and their aggregate
/// backreaction coefficient C_braid = sum_k Re(1 - exp(i 2 pi F_{k-1} /
/// F_{k+2})) ~= 38.196601.
pub const BRAID_DESCRIPTORS: usize = 124;
pub const C_BRAID: f64 = 38.196601;

/// Topological entanglement entropy floor: S_top = ln sqrt(2 + phi).
pub fn topological_entropy() -> f64 {
    let phi = (1.0 + 5.0f64.sqrt()) / 2.0;
    (2.0 + phi).sqrt().ln()
}

/// Braid phase-continuity sum: sum_k arg(B_k) mod 2pi ~ 0.
pub fn braid_phase_sum() -> f64 {
    let mut f_prev = 1.0f64;
    let mut f_cur = 1.0f64;
    let mut acc = 0.0;
    for _ in 0..BRAID_DESCRIPTORS {
        let f_next = f_prev + f_cur;
        let arg = 2.0 * std::f64::consts::PI * f_prev / f_next;
        acc += arg.rem_euclid(2.0 * std::f64::consts::PI) - std::f64::consts::PI;
        f_prev = f_cur;
        f_cur = f_next;
    }
    acc.rem_euclid(2.0 * std::f64::consts::PI) - std::f64::consts::PI
}

/// Eulerian energy density in the warp wall (warp1.txt):
/// rho = -v_s^2 exp(Delta_mod) / (32 pi) * (y^2+z^2)/r^2 * (df/dr)^2.
/// Always <= 0 on the wall annulus.
pub fn eulerian_rho(v_s: f64, delta_mod: f64, angular: f64, dfdr: f64) -> f64 {
    -v_s * v_s * delta_mod.exp() / (32.0 * std::f64::consts::PI)
        * angular.max(0.0)
        * dfdr * dfdr
}

/// Net Ford-Roman margin including the dark-ledger topological
/// backreaction: eta_D * C_braid hbar / (pi tau0^4) term keeps
/// I_Q^net >= -3/(32 pi^2 tau0^4) down to the Planck scale.
pub fn qi_net_margin(tau0: f64, rho_bare_abs: f64, hbar: f64) -> f64 {
    let eta_a = ETA_A_NUM as f64 / PARTITION_DEN as f64;
    let eta_d = ETA_D_NUM as f64 / PARTITION_DEN as f64;
    let bound = -FORD_ROMAN_C / tau0.powi(4);
    let net = -eta_a * rho_bare_abs + eta_d * C_BRAID * hbar / (std::f64::consts::PI * tau0.powi(4));
    net - bound
}

pub const ETA_D_NUM: u64 = 23;
pub const ETA_A_NUM: u64 = 10;
pub const PARTITION_DEN: u64 = 33;

/// Diagonal coordinate stress tensor on one foliation cell, in the
/// orthonormal Eulerian frame: T = diag(rho, p_r, p_t, p_t).
#[derive(Clone, Copy, Debug)]
pub struct StressTensor {
    /// Energy density rho.
    pub rho: f64,
    /// Radial pressure p_r.
    pub p_r: f64,
    /// Transverse pressure p_t.
    pub p_t: f64,
}

impl StressTensor {
    /// Minkowski vacuum.
    pub fn vacuum() -> Self {
        Self { rho: 0.0, p_r: 0.0, p_t: 0.0 }
    }

    /// Warp bubble boundary sample: negative energy density `rho_neg`
    /// screened by the QI budget, with anisotropic wall pressures.
    pub fn warp_wall(rho_neg: f64) -> Self {
        Self { rho: rho_neg, p_r: rho_neg.abs(), p_t: -rho_neg.abs() }
    }
}

/// Weak Energy Condition: T_mn u^m u^n >= 0 for every timelike u.
pub fn wec(t: &StressTensor) -> bool {
    t.rho >= 0.0 && t.rho + t.p_r >= 0.0 && t.rho + t.p_t >= 0.0
}

/// Null Energy Condition: T_mn k^m k^n >= 0 for every null k.
pub fn nec(t: &StressTensor) -> bool {
    t.rho + t.p_r >= 0.0 && t.rho + t.p_t >= 0.0
}

/// Strong Energy Condition: (T_mn - 1/2 T g_mn) u^m u^n >= 0.
pub fn sec(t: &StressTensor) -> bool {
    t.rho + t.p_r + 2.0 * t.p_t >= 0.0 && nec(t)
}

/// Dominant Energy Condition: energy flux is non-spacelike, |p_i| <= rho.
pub fn dec(t: &StressTensor) -> bool {
    t.rho >= 0.0
        && t.p_r.abs() <= t.rho + f64::EPSILON * t.rho.abs().max(1.0)
        && t.p_t.abs() <= t.rho + f64::EPSILON * t.rho.abs().max(1.0)
}

/// Classical energy-condition verdict for one cell.
#[derive(Clone, Copy, Debug)]
pub struct ConditionAudit {
    pub wec: bool,
    pub nec: bool,
    pub sec: bool,
    pub dec: bool,
}

pub fn audit_classical(t: &StressTensor) -> ConditionAudit {
    ConditionAudit {
        wec: wec(t),
        nec: nec(t),
        sec: sec(t),
        dec: dec(t),
    }
}

/// Lorentzian sampling kernel g(tau; tau0) = tau0 / (pi (tau^2 + tau0^2)).
pub fn lorentzian_kernel(tau: f64, tau0: f64) -> f64 {
    tau0 / (std::f64::consts::PI * (tau * tau + tau0 * tau0))
}

/// Ford-Roman QI: sample the energy-density profile `rho(tau)` over a
/// Lorentzian window of width `tau0` and compare against -C / tau0^4.
pub fn ford_roman_integral(rho_samples: &[f64], dt: f64, tau0: f64) -> f64 {
    let n = rho_samples.len() as f64;
    rho_samples
        .iter()
        .enumerate()
        .map(|(k, r)| {
            let tau = (k as f64 - (n - 1.0) / 2.0) * dt;
            *r * lorentzian_kernel(tau, tau0) * dt
        })
        .sum()
}

/// The Ford-Roman lower bound for sampling width `tau0`: -C / tau0^4.
pub fn ford_roman_bound(tau0: f64) -> f64 {
    -FORD_ROMAN_C / tau0.powi(4)
}

/// QI compliance check: the sampled integral must meet the bound whenever
/// `tau0 >= tau_planck`.
pub fn qi_compliant(rho_samples: &[f64], dt: f64, tau0: f64) -> bool {
    if tau0 < TAU_PLANCK_S {
        return true;
    }
    ford_roman_integral(rho_samples, dt, tau0) >= ford_roman_bound(tau0)
}

/// Negative-energy warp-wall audit: a constant density `rho_neg` sustained
/// over duration `t_dur` must satisfy rho_neg * t_dur^4 >= -C.
pub fn warp_wall_qi(rho_neg: f64, t_dur: f64) -> bool {
    rho_neg * t_dur.powi(4) >= -FORD_ROMAN_C
}

/// Dark-ledger sink capacity: the fraction eta_D = 23/33 of the boundary
/// character budget available to absorb negative-energy wall stress.
pub fn dark_ledger_fraction() -> f64 {
    ETA_D_NUM as f64 / PARTITION_DEN as f64
}

/// Sink-capacity audit: a wall stress of magnitude |rho_neg| distributed
/// uniformly across the dark ledger must stay inside the eta_D budget.
pub fn sink_capacity_ok(rho_neg: f64, total_budget: f64) -> bool {
    rho_neg.abs() <= dark_ledger_fraction() * total_budget
}

/// Count NEC violations across a foliation profile (wall skin only: the
/// interior cabin profile is the Minkowski vacuum).
pub fn foliation_nec_violations(profile: &[StressTensor]) -> usize {
    profile.iter().filter(|t| !nec(t)).count()
}
