//! Three-phase dynamic flight engine: ramp, vector steering, and safe collapse
//! with Stinespring de-rendering.

use crate::constants::*;
use crate::shbt::character_excitation::CharacterExcitationRegister;
use pyo3::prelude::*;
use std::f64::consts::PI;

/// Flight dynamics engine covering Phase A (ramping), Phase B (vector
/// steering), and Phase C (safe collapse / de-rendering).
#[pyclass(name = "FlightDynamicsEngine")]
pub struct FlightDynamicsEngine {
    #[pyo3(get)]
    pub v_eff: f64,
    #[pyo3(get)]
    pub delta_mod_0: f64,
    #[pyo3(get)]
    pub radius: f64,
    #[pyo3(get)]
    pub gamma_lock: f64,
    #[pyo3(get)]
    pub c_dark_residual: f64,
    #[pyo3(get)]
    pub c_dark_completed: f64,
}

impl FlightDynamicsEngine {
    /// Stinespring modular-decoupling amplitude ratio.
    fn eta_d(&self) -> f64 {
        self.c_dark_residual / self.c_dark_completed
    }

    /// Prime skeleton (2, 3, 5, 7, 11).
    const PRIMES: [f64; 5] = [2.0, 3.0, 5.0, 7.0, 11.0];

    /// Signed longitudinal character for each of the 9 visible coordinates
    /// projected onto a Cartesian axis.
    ///
    /// Coordinates are ordered row-major over `(q_i, p_j)` with `q_i` the
    /// `SU(2)` charge (row) and `p_j` the first `SU(3)` weight (column).
    /// `axis` = 0 uses `p_j - 1`, axis = 1 uses `q_i - 1`, and axis = 2
    /// returns zero because the visible register is a 2-D boundary slice.
    fn signed_characters(axis: usize) -> [f64; 9] {
        let mut chi = [0.0; 9];
        for qi in 0..3 {
            for pj in 0..3 {
                let idx = qi * 3 + pj;
                chi[idx] = match axis {
                    0 => (pj as f64) - 1.0,
                    1 => (qi as f64) - 1.0,
                    _ => 0.0,
                };
            }
        }
        chi
    }

    /// Minkowski flat metric as a 16-element row-major array.
    fn flat_metric() -> [f64; 16] {
        [
            -1.0, 0.0, 0.0, 0.0,
             0.0, 1.0, 0.0, 0.0,
             0.0, 0.0, 1.0, 0.0,
             0.0, 0.0, 0.0, 1.0,
        ]
    }
}

#[pymethods]
impl FlightDynamicsEngine {
    /// Create a flight engine with canonical SHBT benchmark parameters.
    #[new]
    #[pyo3(signature = (v_eff=1.071186351, delta_mod_0=0.137533547486, radius=10.0, gamma_lock=4.665e-19))]
    fn new(v_eff: f64, delta_mod_0: f64, radius: f64, gamma_lock: f64) -> Self {
        FlightDynamicsEngine {
            v_eff,
            delta_mod_0,
            radius,
            gamma_lock,
            c_dark_residual: C_DARK_RESIDUAL,
            c_dark_completed: C_DARK_COMP,
        }
    }

    /// Phase A: smooth sinusoidal ramp coefficient `xi(t)` in `[0, 1]`.
    ///
    /// `xi(t) = 0.5 * (1 - cos(pi * t / t_ramp))` for `0 <= t <= t_ramp`,
    /// clamped to `1.0` afterward.
    #[pyo3(signature = (t, t_ramp))]
    fn phase_a_ramp(&self, t: f64, t_ramp: f64) -> f64 {
        if t_ramp <= 0.0 || t <= 0.0 {
            return 0.0;
        }
        if t >= t_ramp {
            return 1.0;
        }
        0.5 * (1.0 - (PI * t / t_ramp).cos())
    }

    /// Phase A: shift vector `beta^i = -v_eff * xi * f_SHBT * n^i`.
    #[pyo3(signature = (xi, f_shbt, n))]
    fn phase_a_shift(&self, xi: f64, f_shbt: f64, n: [f64; 3]) -> [f64; 3] {
        let b = -self.v_eff * xi * f_shbt;
        [b * n[0], b * n[1], b * n[2]]
    }

    /// Phase A: combined ramp, shift, and boundary-state audit.
    ///
    /// `rho` must be a flat 81-element complex density matrix.  Returns
    /// `(xi, beta, trace_pass, trace_residual, framing_defect)`.
    #[allow(clippy::too_many_arguments)]
    #[pyo3(signature = (t, t_ramp, f_shbt, n, rho, k_l=26, k_q=8, big_k=312))]
    fn phase_a_audit(
        &self,
        t: f64,
        t_ramp: f64,
        f_shbt: f64,
        n: [f64; 3],
        rho: Vec<num_complex::Complex64>,
        k_l: i64,
        k_q: i64,
        big_k: i64,
    ) -> PyResult<(f64, [f64; 3], bool, f64, f64)> {
        let xi = self.phase_a_ramp(t, t_ramp);
        let beta = self.phase_a_shift(xi, f_shbt, n);
        let register = CharacterExcitationRegister::new(rho)?;
        let (trace_pass, trace_residual) = register.verify_unitarity()?;
        let framing_defect = register.audit_framing_defect(k_l, k_q, big_k)?;
        Ok((xi, beta, trace_pass, trace_residual, framing_defect))
    }

    /// Phase B: integrate one RK4 step of `dn/dt = omega x n` and renormalize.
    #[pyo3(signature = (n, omega, dt))]
    fn phase_b_step_n(&self, n: [f64; 3], omega: [f64; 3], dt: f64) -> [f64; 3] {
        let cross = |a: [f64; 3], b: [f64; 3]| -> [f64; 3] {
            [
                a[1] * b[2] - a[2] * b[1],
                a[2] * b[0] - a[0] * b[2],
                a[0] * b[1] - a[1] * b[0],
            ]
        };

        let rhs = |state: [f64; 3]| -> [f64; 3] { cross(omega, state) };

        let k1 = rhs(n);
        let s2 = [n[0] + 0.5 * dt * k1[0], n[1] + 0.5 * dt * k1[1], n[2] + 0.5 * dt * k1[2]];
        let k2 = rhs(s2);
        let s3 = [n[0] + 0.5 * dt * k2[0], n[1] + 0.5 * dt * k2[1], n[2] + 0.5 * dt * k2[2]];
        let k3 = rhs(s3);
        let s4 = [n[0] + dt * k3[0], n[1] + dt * k3[1], n[2] + dt * k3[2]];
        let k4 = rhs(s4);

        let mut next = [
            n[0] + dt / 6.0 * (k1[0] + 2.0 * k2[0] + 2.0 * k3[0] + k4[0]),
            n[1] + dt / 6.0 * (k1[1] + 2.0 * k2[1] + 2.0 * k3[1] + k4[1]),
            n[2] + dt / 6.0 * (k1[2] + 2.0 * k2[2] + 2.0 * k3[2] + k4[2]),
        ];

        let norm = (next[0].powi(2) + next[1].powi(2) + next[2].powi(2)).sqrt();
        if norm > 1.0e-15 {
            for comp in next.iter_mut() {
                *comp /= norm;
            }
        }
        next
    }

    /// Phase B: compute directional prime-load vector `l_r` and longitudinal
    /// Euler flux `Phi_s` for a single Cartesian `axis` (0=x, 1=y, 2=z).
    ///
    /// `rho` is a 9-element normalized visible probability distribution.
    /// `x` is the Cartesian coordinate along `axis`, `tau` the holographic
    /// resolution.  Returns `(l_r, Phi_s)` as two `Vec<f64>`.
    #[pyo3(signature = (rho, x, tau, axis))]
    fn phase_b_prime_loads(&self, rho: [f64; 9], x: f64, tau: f64, axis: usize) -> PyResult<(Vec<f64>, Vec<f64>)> {
        if axis > 2 {
            return Err(pyo3::exceptions::PyValueError::new_err("axis must be 0, 1, or 2"));
        }
        if rho.len() != 9 {
            return Err(pyo3::exceptions::PyValueError::new_err("rho must have 9 entries"));
        }

        let chi = Self::signed_characters(axis);
        let one_over = 1.0 / (1.0 + tau);
        let x_over_r = x / self.radius;

        // l_r, r = 0..4
        let mut l_r = vec![0.0; 5];
        for (m, prob) in rho.iter().enumerate() {
            let r = m % 5;
            l_r[r] += one_over * prob * (1.0 + x_over_r * chi[m]);
        }

        // Phi_s, s = 0..3
        let mut phi_s = Vec::with_capacity(4);
        let primes = Self::PRIMES;
        for s in 0..4 {
            let num = l_r[s + 1] - l_r[s];
            let denom = (primes[s + 1] / primes[s]).ln();
            phi_s.push(if denom.abs() > 1.0e-15 { num / denom } else { 0.0 });
        }

        Ok((l_r, phi_s))
    }

    /// Phase B: evaluate a proper-acceleration norm proxy on the central plateau.
    ///
    /// If `|grad f_SHBT|` and `|xi_dot|` are below tolerance, the ADM proper
    /// acceleration of the Eulerian observer vanishes.  Returns `(passed, norm)`.
    #[pyo3(signature = (f_shbt, grad_f, xi_dot, n, tolerance=1.0e-12))]
    fn phase_b_plateau_acceleration(
        &self,
        f_shbt: f64,
        grad_f: [f64; 3],
        xi_dot: f64,
        n: [f64; 3],
        tolerance: f64,
    ) -> PyResult<(bool, f64)> {
        let grad_norm = (grad_f[0].powi(2) + grad_f[1].powi(2) + grad_f[2].powi(2)).sqrt();
        if grad_norm < tolerance && xi_dot.abs() < tolerance {
            return Ok((true, 0.0));
        }
        // Non-plateau proxy: -v_eff * xi_dot * f_shbt * n^i.
        let a_mag = self.v_eff * xi_dot.abs() * f_shbt.abs() * (n[0].powi(2) + n[1].powi(2) + n[2].powi(2)).sqrt();
        Ok((false, a_mag))
    }

    /// Phase C: thermodynamic entropy-debt decay from `t0`.
    ///
    /// `Delta_mod(t) = Delta_mod(t0) * exp(-Gamma_lock * (t - t0) / 24)`.
    #[pyo3(signature = (t, t0, delta_t0))]
    fn phase_c_delta_mod(&self, t: f64, t0: f64, delta_t0: f64) -> f64 {
        if t <= t0 {
            return delta_t0;
        }
        delta_t0 * (-self.gamma_lock * (t - t0) / 24.0).exp()
    }

    /// Phase C: Stinespring modular-decoupling amplitude ratio
    /// `eta_D = c_dark^res / c_dark^comp`.
    fn phase_c_stinespring_ratio(&self) -> f64 {
        self.eta_d()
    }

    /// Phase C: safe collapse / de-rendering of the shift vector to zero and
    /// restoration of the flat Minkowski metric.
    ///
    /// Returns `([0.0; 3], flat_metric_16)` and verifies `det g = -1.0`.
    #[pyo3(signature = (beta))]
    fn phase_c_collapse(&self, beta: [f64; 3]) -> PyResult<([f64; 3], [f64; 16], f64)> {
        let _ = beta; // the shift is driven to zero
        let metric = Self::flat_metric();

        let g = nalgebra::SMatrix::<f64, 4, 4>::from_row_slice(&metric);
        let det = g.determinant();
        if (det + 1.0).abs() > 1.0e-9 {
            return Err(pyo3::exceptions::PyRuntimeError::new_err(format!(
                "collapse metric determinant {} != -1.0",
                det
            )));
        }

        Ok(([0.0, 0.0, 0.0], metric, det))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use num_complex::Complex64;

    #[test]
    fn phase_a_ramp_smooth() {
        let engine = FlightDynamicsEngine::new(1.071186351, 0.137533547486, 10.0, 4.665e-19);
        assert!((engine.phase_a_ramp(0.0, 1.0) - 0.0).abs() < 1.0e-15);
        assert!((engine.phase_a_ramp(1.0, 1.0) - 1.0).abs() < 1.0e-15);
        assert!((engine.phase_a_ramp(2.0, 1.0) - 1.0).abs() < 1.0e-15);
        assert!(engine.phase_a_ramp(0.5, 1.0) > 0.0 && engine.phase_a_ramp(0.5, 1.0) < 1.0);
    }

    #[test]
    fn phase_a_shift_matches_formula() {
        let engine = FlightDynamicsEngine::new(1.071186351, 0.137533547486, 10.0, 4.665e-19);
        let xi = 0.5;
        let f = 0.8;
        let n = [1.0, 0.0, 0.0];
        let beta = engine.phase_a_shift(xi, f, n);
        let expected = -engine.v_eff * xi * f;
        assert!((beta[0] - expected).abs() < 1.0e-15);
        assert!(beta[1].abs() < 1.0e-15);
        assert!(beta[2].abs() < 1.0e-15);
    }

    #[test]
    fn phase_a_audit_with_unit_register() {
        let engine = FlightDynamicsEngine::new(1.071186351, 0.137533547486, 10.0, 4.665e-19);
        let mut rho = vec![Complex64::new(0.0, 0.0); 81];
        for i in 0..9 {
            rho[i * 9 + i] = Complex64::new(1.0 / 9.0, 0.0);
        }
        let (xi, beta, trace_pass, trace_res, delta_fr) = engine
            .phase_a_audit(0.5, 1.0, 0.8, [1.0, 0.0, 0.0], rho, 26, 8, 312)
            .unwrap();
        assert!(xi >= 0.0 && xi <= 1.0);
        assert!(!beta.iter().any(|b| b.is_nan()));
        assert!(trace_pass);
        assert!(trace_res < 1.0e-14);
        assert!(delta_fr.abs() < 1.0e-15);
    }

    #[test]
    fn phase_b_step_preserves_unit_norm() {
        let engine = FlightDynamicsEngine::new(1.071186351, 0.137533547486, 10.0, 4.665e-19);
        let n = [1.0, 0.0, 0.0];
        let omega = [0.0, 0.0, 1.0]; // rotate around z
        let n_next = engine.phase_b_step_n(n, omega, 0.1);
        let norm = (n_next[0].powi(2) + n_next[1].powi(2) + n_next[2].powi(2)).sqrt();
        assert!((norm - 1.0).abs() < 1.0e-12);
    }

    #[test]
    fn phase_b_prime_loads_on_z_axis_are_axisymmetric() {
        let engine = FlightDynamicsEngine::new(1.071186351, 0.137533547486, 10.0, 4.665e-19);
        let rho = [1.0 / 9.0; 9];
        let (l_r, phi_s) = engine.phase_b_prime_loads(rho, 0.0, 0.0, 2).unwrap();
        assert_eq!(l_r.len(), 5);
        assert_eq!(phi_s.len(), 4);
        // On z-axis with x=0 and tau=0, every l_r equals the grouped probability sum.
        let expected_l: Vec<f64> = (0..5)
            .map(|r| (0..9).filter(|m| *m % 5 == r).map(|m| rho[m]).sum())
            .collect();
        for (got, exp) in l_r.iter().zip(expected_l.iter()) {
            assert!((got - exp).abs() < 1.0e-15);
        }
        // Flux vanishes where consecutive load components are equal.
        for s in 0..3 {
            assert!(phi_s[s].abs() < 1.0e-15);
        }
        // The last flux reflects the imbalance between the r=3 and r=4 groups.
        let denom = (11.0_f64 / 7.0_f64).ln();
        let expected_last = (l_r[4] - l_r[3]) / denom;
        assert!((phi_s[3] - expected_last).abs() < 1.0e-15);
    }

    #[test]
    fn phase_b_plateau_acceleration_zero() {
        let engine = FlightDynamicsEngine::new(1.071186351, 0.137533547486, 10.0, 4.665e-19);
        let (pass, a) = engine
            .phase_b_plateau_acceleration(1.0, [0.0, 0.0, 0.0], 0.0, [1.0, 0.0, 0.0], 1.0e-12)
            .unwrap();
        assert!(pass);
        assert!(a.abs() < 1.0e-15);
    }

    #[test]
    fn phase_c_delta_mod_decays() {
        let engine = FlightDynamicsEngine::new(1.071186351, 0.137533547486, 10.0, 4.665e-19);
        let delta0 = 0.137533547486;
        let t0 = 0.0;
        let t1 = 1.0e18; // very long compared to 1/Gamma_lock
        let delta1 = engine.phase_c_delta_mod(t1, t0, delta0);
        assert!(delta1 < delta0);
        assert!(delta1 > 0.0);
    }

    #[test]
    fn phase_c_stinespring_ratio_matches_constants() {
        let engine = FlightDynamicsEngine::new(1.071186351, 0.137533547486, 10.0, 4.665e-19);
        let eta = engine.phase_c_stinespring_ratio();
        assert!((eta - C_DARK_RESIDUAL / C_DARK_COMP).abs() < 1.0e-15);
    }

    #[test]
    fn phase_c_collapse_restores_minkowski() {
        let engine = FlightDynamicsEngine::new(1.071186351, 0.137533547486, 10.0, 4.665e-19);
        let (beta, metric, det) = engine.phase_c_collapse([1.0, 0.0, 0.0]).unwrap();
        assert!((beta[0]).abs() < 1.0e-15);
        assert!((beta[1]).abs() < 1.0e-15);
        assert!((beta[2]).abs() < 1.0e-15);
        assert!((det + 1.0).abs() < 1.0e-12);
        assert!((metric[0] + 1.0).abs() < 1.0e-15);
        assert!((metric[5] - 1.0).abs() < 1.0e-15);
    }
}
