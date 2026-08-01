//! Hardware-in-the-Loop (HIL) safety monitor for real-time warp-field audits.

use pyo3::prelude::*;

/// Real-time safety monitor with configurable Gram-eigenvalue threshold.
#[pyclass(name = "SafetyMonitor")]
pub struct SafetyMonitor {
    #[pyo3(get)]
    pub min_gram_threshold: f64,
}

#[pymethods]
impl SafetyMonitor {
    /// Create a new monitor.  The default `min_gram_threshold` is `0.35`.
    #[new]
    #[pyo3(signature = (min_gram_threshold=0.350000))]
    fn new(min_gram_threshold: f64) -> Self {
        SafetyMonitor { min_gram_threshold }
    }

    /// Audit a single HIL step.
    ///
    /// Inputs:
    /// - `min_gram_eig`: smallest Gram-matrix eigenvalue observed in the step.
    /// - `max_det_err`: largest Lorentzian determinant residual observed.
    /// - `max_info_density`: maximum local information density (bits / logical
    ///   cell) reported by the boundary register.
    /// - `budget_limit`: operational upper bound for `max_info_density`.
    ///
    /// Returns `"STATUS_NOMINAL_PASS"` when all checks are within bounds.
    /// Otherwise returns an emergency trigger identifier describing the first
    /// violated limit.
    #[pyo3(signature = (min_gram_eig, max_det_err, max_info_density, budget_limit))]
    fn audit_hil_step(
        &self,
        min_gram_eig: f64,
        max_det_err: f64,
        max_info_density: f64,
        budget_limit: f64,
    ) -> PyResult<String> {
        const DET_TOL: f64 = 1.0e-12;

        if max_det_err > DET_TOL {
            return Ok("EMERGENCY_DETERMINANT_VIOLATION".to_string());
        }

        if min_gram_eig < self.min_gram_threshold {
            return Ok("EMERGENCY_GRAM_EIGENVALUE".to_string());
        }

        if max_info_density > budget_limit {
            return Ok("EMERGENCY_INFORMATION_DENSITY".to_string());
        }

        Ok("STATUS_NOMINAL_PASS".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nominal_step_passes() {
        let monitor = SafetyMonitor::new(0.350000);
        let status = monitor
            .audit_hil_step(0.5, 1.0e-13, 1.0e60, 1.0e70)
            .unwrap();
        assert_eq!(status, "STATUS_NOMINAL_PASS");
    }

    #[test]
    fn determinant_violation_triggers() {
        let monitor = SafetyMonitor::new(0.350000);
        let status = monitor.audit_hil_step(0.5, 1.0e-11, 1.0e60, 1.0e70).unwrap();
        assert_eq!(status, "EMERGENCY_DETERMINANT_VIOLATION");
    }

    #[test]
    fn gram_eigenvalue_violation_triggers() {
        let monitor = SafetyMonitor::new(0.350000);
        let status = monitor.audit_hil_step(0.34, 1.0e-13, 1.0e60, 1.0e70).unwrap();
        assert_eq!(status, "EMERGENCY_GRAM_EIGENVALUE");
    }

    #[test]
    fn information_density_violation_triggers() {
        let monitor = SafetyMonitor::new(0.350000);
        let status = monitor.audit_hil_step(0.5, 1.0e-13, 1.0e80, 1.0e70).unwrap();
        assert_eq!(status, "EMERGENCY_INFORMATION_DENSITY");
    }
}
