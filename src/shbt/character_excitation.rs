//! Boundary character excitation register for the visible
//! `SU(2)_{26} x SU(3)_8` sector.

use num_complex::Complex64;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

/// A 9x9 complex density matrix representing the phase-locked visible
/// character excitation state.
#[pyclass(name = "CharacterExcitationRegister")]
pub struct CharacterExcitationRegister {
    data: Vec<Complex64>,
}

impl CharacterExcitationRegister {
    fn trace(&self) -> Complex64 {
        let mut tr = Complex64::new(0.0, 0.0);
        for i in 0..9 {
            tr += self.data[i * 9 + i];
        }
        tr
    }
}

#[pymethods]
impl CharacterExcitationRegister {
    /// Construct a register from a flat 81-element list of `Complex64` values.
    #[new]
    pub fn new(data: Vec<Complex64>) -> PyResult<Self> {
        if data.len() != 81 {
            return Err(PyValueError::new_err(format!(
                "density matrix must contain 81 entries for a 9x9 matrix, got {}",
                data.len()
            )));
        }
        Ok(CharacterExcitationRegister { data })
    }

    /// Verify that the matrix has unit trace to the required precision.
    ///
    /// Returns a `(passed, residual)` tuple where `passed` is `true` when
    /// `|Re(Tr(rho)) - 1.0| + |Im(Tr(rho))| < 1e-14`.
    pub fn verify_unitarity(&self) -> PyResult<(bool, f64)> {
        let tr = self.trace();
        let residual = (tr.re - 1.0).abs() + tr.im.abs();
        Ok((residual < 1.0e-14, residual))
    }

    /// Evaluate the scalar framing defect for the integer branch
    /// `(k_l, k_q, big_k)`.
    ///
    /// `Delta_fr = max( ||K / (2 k_l)||_Z, ||K / (3 k_q)||_Z )`,
    /// where `||x||_Z` is the distance from `x` to the nearest integer.
    pub fn audit_framing_defect(&self, k_l: i64, k_q: i64, big_k: i64) -> PyResult<f64> {
        if k_l == 0 || k_q == 0 {
            return Err(PyValueError::new_err("k_l and k_q must be non-zero"));
        }
        let frac_l = big_k as f64 / (2.0 * k_l as f64);
        let frac_q = big_k as f64 / (3.0 * k_q as f64);
        let dist_l = (frac_l - frac_l.round()).abs();
        let dist_q = (frac_q - frac_q.round()).abs();
        Ok(dist_l.max(dist_q))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unit_density_matrix_passes_unitarity() {
        let mut data = vec![Complex64::new(0.0, 0.0); 81];
        for i in 0..9 {
            data[i * 9 + i] = Complex64::new(1.0 / 9.0, 0.0);
        }
        let reg = CharacterExcitationRegister::new(data).unwrap();
        let (passed, residual) = reg.verify_unitarity().unwrap();
        assert!(passed);
        assert!(residual < 1.0e-14);
    }

    #[test]
    fn non_unit_trace_fails_unitarity() {
        let mut data = vec![Complex64::new(0.0, 0.0); 81];
        for i in 0..9 {
            data[i * 9 + i] = Complex64::new(1.0 / 8.0, 0.0);
        }
        let reg = CharacterExcitationRegister::new(data).unwrap();
        let (passed, residual) = reg.verify_unitarity().unwrap();
        assert!(!passed);
        assert!(residual > 1.0e-3);
    }

    #[test]
    fn canonical_branch_has_zero_framing_defect() {
        let mut data = vec![Complex64::new(0.0, 0.0); 81];
        for i in 0..9 {
            data[i * 9 + i] = Complex64::new(1.0 / 9.0, 0.0);
        }
        let reg = CharacterExcitationRegister::new(data).unwrap();
        let delta_fr = reg.audit_framing_defect(26, 8, 312).unwrap();
        assert!(delta_fr.abs() < 1.0e-15);
    }
}
