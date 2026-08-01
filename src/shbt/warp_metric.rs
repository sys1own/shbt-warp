//! 3+1D ADM metric grid evaluation for the SHBT warp bubble.

use nalgebra as na;
use pyo3::exceptions::{PyRuntimeError, PyValueError};
use pyo3::prelude::*;

/// Fefferman-Graham slice projector that evaluates the 3+1D ADM metric on a
/// Cartesian grid from a supplied shape field and a unit normal vector.
#[pyclass(name = "FGSliceProjector")]
pub struct FGSliceProjector {
    #[pyo3(get)]
    pub v_eff: f64,
    #[pyo3(get)]
    pub delta_mod: f64,
    #[pyo3(get)]
    pub radius: f64,
}

#[pymethods]
impl FGSliceProjector {
    /// Create a new projector with the canonical SHBT benchmark parameters.
    #[new]
    #[pyo3(signature = (v_eff=1.071186351, delta_mod=0.137533547486, radius=10.0))]
    fn new(v_eff: f64, delta_mod: f64, radius: f64) -> Self {
        FGSliceProjector {
            v_eff,
            delta_mod,
            radius,
        }
    }

    /// Evaluate the 4x4 covariant ADM metric on a `dim_x * dim_y * dim_z` grid.
    ///
    /// The metric is returned as a flat `Vec<f64>` with 16 entries per cell,
    /// row-major ordered as `g_00, g_01, ..., g_33`.  Cells are iterated in
    /// row-major order `(i, j, k)` where `i` is the slowest index and `k` the
    /// fastest: `idx = i * dim_y * dim_z + j * dim_z + k`.
    ///
    /// The ADM ansatz is:
    ///   alpha = 1.0
    ///   gamma_ij = delta_ij
    ///   beta^i = -v_eff * xi * f_SHBT * n^i
    ///
    /// with covariant components
    ///   g_00 = -alpha^2 + gamma_ij beta^i beta^j
    ///   g_0i = g_i0 = gamma_ij beta^j
    ///   g_ij = gamma_ij.
    ///
    /// Every cell is checked for `det(g_mu_nu) = -1.0`; if any cell violates
    /// this to within `1e-9` the call raises a Python `RuntimeError`.
    #[pyo3(signature = (xi, n_vec, shape_field, dim_x, dim_y, dim_z))]
    fn evaluate_adm_grid(
        &self,
        xi: f64,
        n_vec: [f64; 3],
        shape_field: Vec<f64>,
        dim_x: usize,
        dim_y: usize,
        dim_z: usize,
    ) -> PyResult<Vec<f64>> {
        let expected_len = dim_x * dim_y * dim_z;
        if shape_field.len() != expected_len {
            return Err(PyValueError::new_err(format!(
                "shape_field length {} does not match grid dimensions {} x {} x {} = {}",
                shape_field.len(), dim_x, dim_y, dim_z, expected_len
            )));
        }

        let n_norm = (n_vec[0].powi(2) + n_vec[1].powi(2) + n_vec[2].powi(2)).sqrt();
        let n = if n_norm > 1e-15 {
            [n_vec[0] / n_norm, n_vec[1] / n_norm, n_vec[2] / n_norm]
        } else {
            n_vec
        };

        let mut grid_metrics = Vec::with_capacity(expected_len * 16);

        for i in 0..dim_x {
            for j in 0..dim_y {
                for k in 0..dim_z {
                    let idx = (i * dim_y + j) * dim_z + k;
                    let f_shbt = shape_field[idx];
                    let b = -self.v_eff * xi * f_shbt;
                    let beta = [b * n[0], b * n[1], b * n[2]];
                    let beta_sq = beta[0].powi(2) + beta[1].powi(2) + beta[2].powi(2);

                    let g00 = -1.0 + beta_sq;
                    let g01 = beta[0];
                    let g02 = beta[1];
                    let g03 = beta[2];

                    let cell = [
                        g00, g01, g02, g03,
                        g01, 1.0, 0.0, 0.0,
                        g02, 0.0, 1.0, 0.0,
                        g03, 0.0, 0.0, 1.0,
                    ];

                    let g = na::SMatrix::<f64, 4, 4>::from_row_slice(&cell);
                    let det = g.determinant();
                    if (det + 1.0).abs() > 1.0e-9 {
                        return Err(PyRuntimeError::new_err(format!(
                            "ADM metric determinant violated at cell ({},{},{}): det = {}, expected -1.0",
                            i, j, k, det
                        )));
                    }

                    grid_metrics.extend_from_slice(&cell);
                }
            }
        }

        Ok(grid_metrics)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adm_grid_determinant_and_shape() {
        let projector = FGSliceProjector::new(1.071186351, 0.137533547486, 10.0);
        let dim_x = 3;
        let dim_y = 2;
        let dim_z = 2;
        let n_cells = dim_x * dim_y * dim_z;
        let shape_field = vec![0.5; n_cells];
        let n_vec = [1.0, 0.0, 0.0];
        let xi = 1.0;

        let metrics = projector
            .evaluate_adm_grid(xi, n_vec, shape_field, dim_x, dim_y, dim_z)
            .unwrap();

        assert_eq!(metrics.len(), n_cells * 16);

        // Spot-check a single cell.
        let b = -projector.v_eff * xi * 0.5;
        let g00 = -1.0 + b * b;
        let cell = &metrics[0..16];
        assert!((cell[0] - g00).abs() < 1.0e-12);
        assert!((cell[1] - b).abs() < 1.0e-12);
        assert!((cell[5] - 1.0).abs() < 1.0e-12);
        assert!((cell[10] - 1.0).abs() < 1.0e-12);
        assert!((cell[15] - 1.0).abs() < 1.0e-12);
    }

    #[test]
    fn adm_grid_rejects_mismatched_shape_field() {
        let projector = FGSliceProjector::new(1.071186351, 0.137533547486, 10.0);
        let shape_field = vec![0.5; 4];
        let result = projector.evaluate_adm_grid(1.0, [1.0, 0.0, 0.0], shape_field, 2, 2, 2);
        assert!(result.is_err());
    }
}
