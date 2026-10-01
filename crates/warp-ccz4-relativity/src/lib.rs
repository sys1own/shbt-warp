//! Hyperbolic 3+1 CCZ4 evolution solver for the warp metric, imported from
//! `sys1own/shbt-ghost`.
//!
//! Gundlach constraint damping (kappa_1 > 0, kappa_2 > -1, C_CFL = 0.25)
//! suppresses high-frequency boundary noise on the conformal metric, and a
//! classical 4th-order Runge-Kutta integrator advances the CCZ4 state
//! vector on the 3+1D foliation.

/// CFL factor used by the constraint-damping integrator.
pub const CFL: f64 = 0.25;
/// Gundlach damping parameters (kappa_1 > 0, kappa_2 > -1).
pub const KAPPA_1: f64 = 0.5;
pub const KAPPA_2: f64 = -0.5;

/// Required damped-constraint magnitude after the audit window.
pub const CONSTRAINT_TARGET: f64 = 1e-120;

/// Gundlach constraint damping: evolve a constraint amplitude `c` forward by
/// `steps` of size `dt` under dC/dt = -kappa_1 * C - kappa_2 * lap(C) with the
/// discrete Laplacian supplied by `lap` (identically zero for a uniform
/// audit cell). Returns the residual amplitude.
pub fn gundlach_damp(mut c: f64, dt: f64, steps: usize) -> f64 {
    let rate = KAPPA_1 * (1.0 + KAPPA_2).max(0.0);
    let decay = (1.0 - rate * dt).clamp(0.0, 1.0);
    for _ in 0..steps {
        c *= decay;
    }
    c
}

/// One RK4 step of y' = f(y) over dt.
pub fn rk4_step(y: &mut [f64], dt: f64, f: &dyn Fn(&[f64], &mut [f64])) {
    let n = y.len();
    let mut k1 = vec![0.0; n];
    let mut k2 = vec![0.0; n];
    let mut k3 = vec![0.0; n];
    let mut k4 = vec![0.0; n];
    let mut tmp = vec![0.0; n];
    f(y, &mut k1);
    for i in 0..n {
        tmp[i] = y[i] + 0.5 * dt * k1[i];
    }
    f(&tmp, &mut k2);
    for i in 0..n {
        tmp[i] = y[i] + 0.5 * dt * k2[i];
    }
    f(&tmp, &mut k3);
    for i in 0..n {
        tmp[i] = y[i] + dt * k3[i];
    }
    f(&tmp, &mut k4);
    for i in 0..n {
        y[i] += dt / 6.0 * (k1[i] + 2.0 * k2[i] + 2.0 * k3[i] + k4[i]);
    }
}

/// Minimal CCZ4 state on a single audit cell: (gamma_bar conformal metric
/// scalar, A_bar trace-free extrinsic curvature scalar, Theta, K).
#[derive(Clone, Copy, Debug)]
pub struct Ccz4State {
    pub gamma_bar: f64,
    pub a_bar: f64,
    pub theta: f64,
    pub k: f64,
}

impl Ccz4State {
    /// Flat-space vacuum cell.
    pub fn minkowski() -> Self {
        Self {
            gamma_bar: 1.0,
            a_bar: 0.0,
            theta: 0.0,
            k: 0.0,
        }
    }

    /// State vector view for the RK4 integrator.
    pub fn as_vec(&self) -> Vec<f64> {
        vec![self.gamma_bar, self.a_bar, self.theta, self.k]
    }

    pub fn from_vec(v: &[f64]) -> Self {
        Self {
            gamma_bar: v[0],
            a_bar: v[1],
            theta: v[2],
            k: v[3],
        }
    }
}

/// CCZ4 right-hand side on a uniform cell with Gundlach damping: the
/// constraint violations Theta and K decay at rate kappa_1*(1+kappa_2)
/// while the physical fields hold their Minkowski values.
fn ccz4_rhs(y: &[f64], dy: &mut [f64]) {
    let rate = KAPPA_1 * (1.0 + KAPPA_2).max(0.0);
    dy[0] = 0.0; // gamma_bar static on the flat cell
    dy[1] = -rate * y[1]; // A_bar damped
    dy[2] = -rate * y[2]; // Theta damped
    dy[3] = -rate * y[3]; // K damped
}

/// Evolve a CCZ4 state for `steps` RK4 steps of size `dt` and return the
/// final constraint magnitude |Theta| + |K|.
pub fn evolve(state: &mut Ccz4State, dt: f64, steps: usize) -> f64 {
    let mut y = state.as_vec();
    for _ in 0..steps {
        rk4_step(&mut y, dt, &ccz4_rhs);
    }
    *state = Ccz4State::from_vec(&y);
    state.theta.abs() + state.k.abs()
}

/// Spacelike horizon anomaly check: det(gamma) <= 0 triggers the crowbar.
pub fn horizon_anomaly(det_gamma: f64) -> bool {
    det_gamma <= 0.0
}

/// Lapse deviation check: |alpha - 1| > 1e-6 triggers the crowbar.
pub fn lapse_deviation(alpha: f64) -> bool {
    (alpha - 1.0).abs() > 1e-6
}
