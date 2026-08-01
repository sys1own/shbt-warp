//! Boundary emitter-array hardware synthesis and noise sensitivity auditor.

use pyo3::prelude::*;
use std::f64::consts::PI;

/// Controller that maps boundary character states to physical emitter-array
/// element phases and RF drive signals for the `SU(2)_{26} x SU(3)_8`
/// register.
#[pyclass(name = "EmitterArrayController")]
pub struct EmitterArrayController {
    #[pyo3(get)]
    pub k_l: i64,
    #[pyo3(get)]
    pub k_q: i64,
    #[pyo3(get)]
    pub big_k: i64,
    #[pyo3(get)]
    pub visible_central_charge: f64,
}

impl EmitterArrayController {
    /// C_vis / 24, used in the modular T-phase correction.
    fn c_vis_over_24(&self) -> f64 {
        self.visible_central_charge / 24.0
    }
}

#[pymethods]
impl EmitterArrayController {
    /// Create a controller with the canonical SHBT visible-sector parameters.
    #[new]
    #[pyo3(signature = (k_l=26, k_q=8, big_k=312))]
    fn new(k_l: i64, k_q: i64, big_k: i64) -> Self {
        EmitterArrayController {
            k_l,
            k_q,
            big_k,
            visible_central_charge: 1325.0 / 154.0,
        }
    }

    /// Visible conformal dimension for the `(q_i, p_j, r_j)` charge/weight entry.
    ///
    /// $$ h_{ij} = \frac{q_i (q_i + 2)}{4 (k_l + 2)} + \frac{p_j^2 + r_j^2 + p_j r_j + 3 p_j + 3 r_j}{3 (k_q + 3)} $$
    #[pyo3(signature = (q_i, p_j, r_j))]
    fn conformal_dimension(&self, q_i: f64, p_j: f64, r_j: f64) -> f64 {
        let su2_term = q_i * (q_i + 2.0) / (4.0 * (self.k_l as f64 + 2.0));
        let su3_term = (p_j.powi(2) + r_j.powi(2) + p_j * r_j + 3.0 * p_j + 3.0 * r_j)
            / (3.0 * (self.k_q as f64 + 3.0));
        su2_term + su3_term
    }

    /// Compute the emitter-array phase command for a single element.
    ///
    /// $$ \theta_k(t) = \theta_{local} [q_i + \nu(w_j)]
    ///     + 2\pi \theta_{local} \left( h_{ij} - \frac{c_{vis}}{24} \right) $$
    #[pyo3(signature = (theta_local, q_i, weight_grade, h_ij))]
    fn compute_emitter_phase(&self, theta_local: f64, q_i: f64, weight_grade: f64, h_ij: f64) -> f64 {
        let char_phase = theta_local * (q_i + weight_grade);
        let modular_phase = 2.0 * PI * theta_local * (h_ij - self.c_vis_over_24());
        char_phase + modular_phase
    }

    /// Synthesize the RF control voltage for an emitter array element.
    ///
    /// $$ V_k(t) = V_0 \cos(\omega_{carrier} t + \theta_k + \phi_{cal}) $$
    #[pyo3(signature = (v0, omega_carrier, t, theta_k, phi_cal))]
    fn synthesize_rf_signal(&self, v0: f64, omega_carrier: f64, t: f64, theta_k: f64, phi_cal: f64) -> f64 {
        v0 * (omega_carrier * t + theta_k + phi_cal).cos()
    }
}

/// Auditor for hardware noise and sensitivity limits that would corrupt the
/// phase-locked boundary state.
#[pyclass(name = "HardwareNoiseAuditor")]
pub struct HardwareNoiseAuditor;

#[pymethods]
impl HardwareNoiseAuditor {
    #[new]
    fn new() -> Self {
        HardwareNoiseAuditor
    }

    /// Audit phase jitter against the limit required to keep the population
    /// displacement residual below `1e-6`.
    ///
    /// The allowed phase-noise standard deviation is `5.05e-5 rad`
    /// (approximately `0.0029 deg`).
    #[pyo3(signature = (sigma_theta))]
    fn audit_phase_jitter(&self, sigma_theta: f64) -> PyResult<(bool, f64)> {
        const PHASE_JITTER_LIMIT_RAD: f64 = 5.05e-5;
        Ok((sigma_theta <= PHASE_JITTER_LIMIT_RAD, sigma_theta))
    }

    /// Audit thermal decoherence: noise temperature must not exceed `15.4 mK`
    /// and the decoherence rate must not exceed `1.2e-4 s^-1`.
    ///
    /// Returns `(passed, worst_ratio)`, where `worst_ratio` is the maximum of
    /// `T_N / 15.4 mK` and `gamma_dec / 1.2e-4 s^-1`.  A value `<= 1.0` means
    /// the hardware is within specification.
    #[pyo3(signature = (temp_kelvin, gamma_dec))]
    fn audit_thermal_decoherence(&self, temp_kelvin: f64, gamma_dec: f64) -> PyResult<(bool, f64)> {
        const TEMP_LIMIT_K: f64 = 15.4e-3;
        const GAMMA_LIMIT_S_INV: f64 = 1.2e-4;
        let temp_ratio = temp_kelvin / TEMP_LIMIT_K;
        let gamma_ratio = gamma_dec / GAMMA_LIMIT_S_INV;
        let worst = temp_ratio.max(gamma_ratio);
        Ok((worst <= 1.0, worst))
    }

    /// Enforce exact integer lock of the branch parameters.
    ///
    /// Only `(delta_k_l, delta_k_q, delta_big_k) == (0, 0, 0)` yields a zero
    /// framing defect.
    #[pyo3(signature = (delta_kl, delta_kq, delta_bigk))]
    fn audit_level_integer_lock(&self, delta_kl: i64, delta_kq: i64, delta_bigk: i64) -> PyResult<bool> {
        Ok(delta_kl == 0 && delta_kq == 0 && delta_bigk == 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::PI;

    #[test]
    fn canonical_visible_central_charge() {
        let controller = EmitterArrayController::new(26, 8, 312);
        assert!((controller.visible_central_charge - 1325.0 / 154.0).abs() < 1.0e-12);
    }

    #[test]
    fn conformal_dimension_for_origin() {
        let controller = EmitterArrayController::new(26, 8, 312);
        let h = controller.conformal_dimension(0.0, 0.0, 0.0);
        // q_i = 0 gives su2 term 0; p_j = r_j = 0 gives su3 term 0.
        assert!(h.abs() < 1.0e-15);
    }

    #[test]
    fn emitter_phase_matches_formula() {
        let controller = EmitterArrayController::new(26, 8, 312);
        let q_i = 1.0;
        let p_j = 0.0;
        let r_j = 1.0;
        let weight_grade = p_j + r_j;
        let h = controller.conformal_dimension(q_i, p_j, r_j);
        let theta_local = 0.421;
        let theta_k = controller.compute_emitter_phase(theta_local, q_i, weight_grade, h);

        let expected = theta_local * (q_i + weight_grade)
            + 2.0 * PI * theta_local * (h - controller.visible_central_charge / 24.0);
        assert!((theta_k - expected).abs() < 1.0e-15);
    }

    #[test]
    fn rf_signal_matches_cosine() {
        let controller = EmitterArrayController::new(26, 8, 312);
        let v0 = 1.0;
        let omega = 2.0 * PI * 1.0e9;
        let t = 1.0e-9;
        let theta_k = 0.1;
        let phi_cal = 0.05;
        let v = controller.synthesize_rf_signal(v0, omega, t, theta_k, phi_cal);
        let expected = (omega * t + theta_k + phi_cal).cos();
        assert!((v - expected).abs() < 1.0e-15);
    }

    #[test]
    fn phase_jitter_audit() {
        let auditor = HardwareNoiseAuditor::new();
        let (pass, residual) = auditor.audit_phase_jitter(5.0e-5).unwrap();
        assert!(pass);
        assert!((residual - 5.0e-5).abs() < 1.0e-18);

        let (pass, _) = auditor.audit_phase_jitter(5.1e-5).unwrap();
        assert!(!pass);
    }

    #[test]
    fn thermal_decoherence_audit() {
        let auditor = HardwareNoiseAuditor::new();
        let (pass, ratio) = auditor.audit_thermal_decoherence(15.4e-3, 1.2e-4).unwrap();
        assert!(pass);
        assert!((ratio - 1.0).abs() < 1.0e-12);

        let (pass, ratio) = auditor.audit_thermal_decoherence(16.0e-3, 1.0e-4).unwrap();
        assert!(!pass);
        assert!(ratio > 1.0);
    }

    #[test]
    fn integer_lock_audit() {
        let auditor = HardwareNoiseAuditor::new();
        assert!(auditor.audit_level_integer_lock(0, 0, 0).unwrap());
        assert!(!auditor.audit_level_integer_lock(0, 0, 1).unwrap());
        assert!(!auditor.audit_level_integer_lock(0, 1, 0).unwrap());
        assert!(!auditor.audit_level_integer_lock(1, 0, 0).unwrap());
    }
}
