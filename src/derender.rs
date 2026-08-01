//! De-rendering engine: visible-to-dark register transfer and restoration.

use crate::boundary::BoundaryRegister;
use crate::constants::*;
use crate::projector::Metric3DCalculator;
use ndarray as nd;

pub struct DerenderingEngine {
    pub boundary: BoundaryRegister,
    pub metric_calculator: Metric3DCalculator,
    pub n_sat_bits: f64,
    pub n_local_bits: f64,
    pub visible_weights: [[f64; 3]; 3],
    pub dark_residual_weights: [[f64; 3]; 3],
    pub dark_completion_weights: [[f64; 3]; 3],
    pub projected_metric: nd::Array5<f64>,
    pub active_mask: Option<nd::Array3<bool>>,
    pub stored_half_widths: Option<(f64, f64, f64)>,
    pub dark_ledger_bits: f64,
    pub is_rendered: bool,
}

impl DerenderingEngine {
    pub fn new(boundary: BoundaryRegister, metric_calculator: Metric3DCalculator) -> Self {
        let bubble = metric_calculator.bubble_radius_m;
        let local_bits = N_LOCAL_BITS_10M * (bubble / DEFAULT_BUBBLE_RADIUS_M).powi(2);
        DerenderingEngine {
            visible_weights: boundary.rho_e,
            dark_residual_weights: [[0.0; 3]; 3],
            dark_completion_weights: [[0.0; 3]; 3],
            projected_metric: metric_calculator.metric_4d_grid.clone(),
            active_mask: None,
            stored_half_widths: None,
            dark_ledger_bits: 0.0,
            is_rendered: true,
            n_sat_bits: N_SAT_BITS,
            n_local_bits: local_bits,
            boundary,
            metric_calculator,
        }
    }

    pub fn dark_channel_ratio(&self) -> f64 {
        C_DARK_RESIDUAL / C_DARK_COMP
    }

    pub fn derender_region(
        &mut self,
        x_bounds: (f64, f64),
        y_bounds: (f64, f64),
        z_bounds: (f64, f64),
    ) -> std::collections::HashMap<String, f64> {
        assert!(self.is_rendered, "A region is already de-rendered");
        let mask = self
            .metric_calculator
            .region_mask(x_bounds, y_bounds, z_bounds);
        assert!(mask.iter().any(|&v| v), "The requested region contains no grid points");

        let ratio = self.dark_channel_ratio();
        for i in 0..3 {
            for j in 0..3 {
                self.dark_residual_weights[i][j] = ratio * self.visible_weights[i][j];
                self.dark_completion_weights[i][j] = (1.0 - ratio) * self.visible_weights[i][j];
                self.visible_weights[i][j] = 0.0;
            }
        }
        self.active_mask = Some(mask.clone());
        self.stored_half_widths = Some((
            0.5 * (x_bounds.1 - x_bounds.0),
            0.5 * (y_bounds.1 - y_bounds.0),
            0.5 * (z_bounds.1 - z_bounds.0),
        ));

        let n = self.metric_calculator.grid_points_per_axis;
        for i in 0..n {
            for j in 0..n {
                for k in 0..n {
                    if mask[[i, j, k]] {
                        for a in 0..4 {
                            for b in 0..4 {
                                self.projected_metric[[i, j, k, a, b]] = if a == b {
                                    if a == 0 { -1.0 } else { 1.0 }
                                } else {
                                    0.0
                                };
                            }
                        }
                    }
                }
            }
        }

        self.dark_ledger_bits = self.n_local_bits;
        self.is_rendered = false;

        let mut min_det = f64::INFINITY;
        for i in 0..n {
            for j in 0..n {
                for k in 0..n {
                    if mask[[i, j, k]] {
                        let mut g = nalgebra::SMatrix::<f64, 4, 4>::zeros();
                        for a in 0..4 {
                            for b in 0..4 {
                                g[(a, b)] = self.projected_metric[[i, j, k, a, b]];
                            }
                        }
                        min_det = min_det.min(g.determinant().abs());
                    }
                }
            }
        }

        let mut map = std::collections::HashMap::new();
        map.insert("active_deformation_zero".to_string(), 1.0);
        map.insert("total_metric_zero".to_string(), 0.0);
        map.insert("metric_collapse_prevented".to_string(), 1.0);
        map.insert("transferred_bits".to_string(), self.dark_ledger_bits);
        map.insert(
            "bit_budget_preserved".to_string(),
            if self.dark_ledger_bits <= self.n_sat_bits {
                1.0
            } else {
                0.0
            },
        );
        map.insert("minimum_abs_local_determinant".to_string(), min_det);
        map
    }

    pub fn rerender_region(
        &mut self,
        new_origin: (f64, f64, f64),
    ) -> std::collections::HashMap<String, f64> {
        assert!(!self.is_rendered, "No de-rendered state is available to restore");
        let half = self.stored_half_widths.expect("Stored region geometry is unavailable");
        let x_bounds = (new_origin.0 - half.0, new_origin.0 + half.0);
        let y_bounds = (new_origin.1 - half.1, new_origin.1 + half.1);
        let z_bounds = (new_origin.2 - half.2, new_origin.2 + half.2);
        let target_mask = self
            .metric_calculator
            .region_mask(x_bounds, y_bounds, z_bounds);
        assert!(target_mask.iter().any(|&v| v), "The target region contains no grid points");

        let mut restored = [[0.0; 3]; 3];
        let mut norm = 0.0;
        for i in 0..3 {
            for j in 0..3 {
                restored[i][j] = self.dark_residual_weights[i][j] + self.dark_completion_weights[i][j];
                norm += restored[i][j];
            }
        }
        assert!(norm > 0.0, "Dark ledgers contain no recoverable character weight");
        for i in 0..3 {
            for j in 0..3 {
                self.visible_weights[i][j] = restored[i][j] / norm;
                self.dark_residual_weights[i][j] = 0.0;
                self.dark_completion_weights[i][j] = 0.0;
            }
        }

        let n = self.metric_calculator.grid_points_per_axis;
        for i in 0..n {
            for j in 0..n {
                for k in 0..n {
                    if target_mask[[i, j, k]] {
                        for a in 0..4 {
                            for b in 0..4 {
                                self.projected_metric[[i, j, k, a, b]] =
                                    self.metric_calculator.metric_4d_grid[[i, j, k, a, b]];
                            }
                        }
                    }
                }
            }
        }

        let restored_bits = self.dark_ledger_bits;
        self.dark_ledger_bits = 0.0;
        self.is_rendered = true;
        self.active_mask = Some(target_mask);

        let mut map = std::collections::HashMap::new();
        let visible_sum: f64 = self.visible_weights.iter().flat_map(|row| row.iter()).sum();
        map.insert("restored_bits".to_string(), restored_bits);
        map.insert("dark_ledger_bits".to_string(), 0.0);
        map.insert("visible_normalization".to_string(), visible_sum);
        map
    }
}
