//! Fefferman–Graham slice projectors and 3-D metric calculators.

use crate::constants::*;
use ndarray as nd;
use nalgebra as na;

/// One-dimensional FG slice projector along the x-axis.
pub struct FGSliceProjector {
    pub delta_mod: f64,
    pub bubble_radius_m: f64,
    pub domain_radius_m: f64,
    pub grid_points: usize,
    pub wall_steepness_per_m: f64,
    pub speed_of_light_m_s: f64,
    pub x_m: nd::Array1<f64>,
    pub r_s_m: nd::Array1<f64>,
    pub shape: nd::Array1<f64>,
    pub shape_gradient_per_m: nd::Array1<f64>,
    pub beta_m_s: nd::Array1<f64>,
    pub beta_gradient_per_s: nd::Array1<f64>,
    pub lorentzian_metrics: nd::Array3<f64>,
    pub gram_metrics: nd::Array3<f64>,
}

impl FGSliceProjector {
    pub fn new(
        delta_mod: f64,
        bubble_radius_m: f64,
        domain_radius_m: f64,
        grid_points: usize,
        wall_steepness_per_m: f64,
    ) -> Self {
        assert!(bubble_radius_m > 0.0, "bubble_radius_m must be positive");
        assert!(domain_radius_m > bubble_radius_m, "domain_radius_m must exceed bubble_radius_m");
        let mut n = grid_points;
        if n < 5 {
            n = 5;
        }
        if n % 2 == 0 {
            n += 1;
        }
        assert!(wall_steepness_per_m > 0.0, "wall_steepness_per_m must be positive");

        let x_m = nd::Array1::linspace(-domain_radius_m, domain_radius_m, n);
        let r_s_m = x_m.mapv(|x| x.abs());
        let shape = Self::alcubierre_shape(&r_s_m, bubble_radius_m, wall_steepness_per_m);
        let shape_gradient = Self::alcubierre_shape_gradient(&x_m, bubble_radius_m, wall_steepness_per_m);
        let v_eff_c = (delta_mod / 2.0).exp();
        let v_eff_m_s = LIGHT_SPEED_M_S * v_eff_c;
        let beta_m_s = shape.mapv(|f| -v_eff_m_s * f);
        let beta_gradient = shape_gradient.mapv(|g| -v_eff_m_s * g);

        let (lorentzian, gram) = Self::build_metrics(&beta_m_s);

        FGSliceProjector {
            delta_mod,
            bubble_radius_m,
            domain_radius_m,
            grid_points: n,
            wall_steepness_per_m,
            speed_of_light_m_s: LIGHT_SPEED_M_S,
            x_m,
            r_s_m,
            shape,
            shape_gradient_per_m: shape_gradient,
            beta_m_s,
            beta_gradient_per_s: beta_gradient,
            lorentzian_metrics: lorentzian,
            gram_metrics: gram,
        }
    }

    pub fn v_eff_c(&self) -> f64 {
        (self.delta_mod / 2.0).exp()
    }

    fn alcubierre_shape(r_s: &nd::Array1<f64>, radius: f64, sigma: f64) -> nd::Array1<f64> {
        let denom = 2.0 * (sigma * radius).tanh();
        r_s.mapv(|r| ((sigma * (r + radius)).tanh() - (sigma * (r - radius)).tanh()) / denom)
    }

    fn alcubierre_shape_gradient(x: &nd::Array1<f64>, radius: f64, sigma: f64) -> nd::Array1<f64> {
        let denom = 2.0 * (sigma * radius).tanh();
        let mut grad = nd::Array1::zeros(x.len());
        for (i, &xv) in x.iter().enumerate() {
            let r = xv.abs();
            let deriv_r = sigma
                * ((sigma * (r + radius)).cosh().powi(-2)
                    - (sigma * (r - radius)).cosh().powi(-2))
                / denom;
            grad[i] = if xv.abs() < f64::EPSILON {
                0.0
            } else {
                deriv_r * xv.signum()
            };
        }
        grad
    }

    fn build_metrics(beta_m_s: &nd::Array1<f64>) -> (nd::Array3<f64>, nd::Array3<f64>) {
        let n = beta_m_s.len();
        let mut lorentzian = nd::Array3::zeros((n, 4, 4));
        let mut gram = nd::Array3::zeros((n, 4, 4));
        for (i, &beta) in beta_m_s.iter().enumerate() {
            let beta_c = beta / LIGHT_SPEED_M_S;
            lorentzian[[i, 0, 0]] = -1.0 + beta_c * beta_c;
            lorentzian[[i, 0, 1]] = beta_c;
            lorentzian[[i, 1, 0]] = beta_c;
            lorentzian[[i, 1, 1]] = 1.0;
            lorentzian[[i, 2, 2]] = 1.0;
            lorentzian[[i, 3, 3]] = 1.0;

            gram[[i, 0, 0]] = 1.0 + beta_c * beta_c;
            gram[[i, 0, 1]] = beta_c;
            gram[[i, 1, 0]] = beta_c;
            gram[[i, 1, 1]] = 1.0;
            gram[[i, 2, 2]] = 1.0;
            gram[[i, 3, 3]] = 1.0;
        }
        (lorentzian, gram)
    }

    pub fn audit(&self, tolerance: f64) -> std::collections::HashMap<String, f64> {
        let mut min_abs_det = f64::INFINITY;
        let mut det_error: f64 = 0.0;
        let mut min_abs_lorentzian_ev = f64::INFINITY;
        let mut min_gram_ev = f64::INFINITY;

        for i in 0..self.grid_points {
            let mut g = na::SMatrix::<f64, 4, 4>::zeros();
            for a in 0..4 {
                for b in 0..4 {
                    g[(a, b)] = self.lorentzian_metrics[[i, a, b]];
                }
            }
            let det = g.determinant();
            min_abs_det = min_abs_det.min(det.abs());
            det_error = det_error.max((det + 1.0).abs());
            let ev = na::linalg::SymmetricEigen::new(g).eigenvalues;
            for v in ev.iter() {
                min_abs_lorentzian_ev = min_abs_lorentzian_ev.min(v.abs());
            }

            let mut gr = na::SMatrix::<f64, 4, 4>::zeros();
            for a in 0..4 {
                for b in 0..4 {
                    gr[(a, b)] = self.gram_metrics[[i, a, b]];
                }
            }
            let gev = na::linalg::SymmetricEigen::new(gr).eigenvalues;
            for v in gev.iter() {
                min_gram_ev = min_gram_ev.min(*v);
            }
        }

        let passed = min_abs_det > tolerance
            && min_abs_lorentzian_ev > tolerance
            && min_gram_ev > tolerance
            && det_error <= 100.0 * tolerance;

        let mut map = std::collections::HashMap::new();
        map.insert("passed".to_string(), if passed { 1.0 } else { 0.0 });
        map.insert("minimum_abs_determinant".to_string(), min_abs_det);
        map.insert("minimum_abs_lorentzian_eigenvalue".to_string(), min_abs_lorentzian_ev);
        map.insert("minimum_gram_eigenvalue".to_string(), min_gram_ev);
        map.insert("determinant_error".to_string(), det_error);
        map
    }
}

/// Three-dimensional Cartesian metric calculator.
#[derive(Clone)]
pub struct Metric3DCalculator {
    pub bubble_radius_m: f64,
    pub domain_radius_m: f64,
    pub grid_points_per_axis: usize,
    pub wall_steepness_per_m: f64,
    pub delta_mod: f64,
    pub x_m: nd::Array1<f64>,
    pub y_m: nd::Array1<f64>,
    pub z_m: nd::Array1<f64>,
    pub radius_m: nd::Array3<f64>,
    pub shape: nd::Array3<f64>,
    pub beta_over_c: nd::Array3<f64>,
    pub metric_4d_grid: nd::Array5<f64>,
    pub spatial_metric_grid: nd::Array5<f64>,
    pub gram_metric_grid: nd::Array5<f64>,
}

impl Metric3DCalculator {
    pub fn new(
        bubble_radius_m: f64,
        domain_radius_m: f64,
        grid_points_per_axis: usize,
        wall_steepness_per_m: f64,
        delta_mod: f64,
    ) -> Self {
        assert!(bubble_radius_m > 0.0, "bubble_radius_m must be positive");
        assert!(domain_radius_m > bubble_radius_m, "domain_radius_m must exceed bubble_radius_m");
        let mut n = grid_points_per_axis;
        if n < 3 {
            n = 3;
        }
        if n % 2 == 0 {
            n += 1;
        }
        assert!(wall_steepness_per_m > 0.0, "wall_steepness_per_m must be positive");

        let axis = nd::Array1::linspace(-domain_radius_m, domain_radius_m, n);
        let mut radius = nd::Array3::zeros((n, n, n));
        for i in 0..n {
            for j in 0..n {
                for k in 0..n {
                    radius[[i, j, k]] = (axis[i].powi(2) + axis[j].powi(2) + axis[k].powi(2)).sqrt();
                }
            }
        }

        let shape = Self::shape_function(&radius, bubble_radius_m, wall_steepness_per_m);
        let v_eff_c = (delta_mod / 2.0).exp();
        let beta = shape.mapv(|f| -v_eff_c * f);

        let (metric, spatial, gram) = Self::build_metric_tensors(&beta);

        Metric3DCalculator {
            bubble_radius_m,
            domain_radius_m,
            grid_points_per_axis: n,
            wall_steepness_per_m,
            delta_mod,
            x_m: axis.clone(),
            y_m: axis.clone(),
            z_m: axis,
            radius_m: radius,
            shape,
            beta_over_c: beta,
            metric_4d_grid: metric,
            spatial_metric_grid: spatial,
            gram_metric_grid: gram,
        }
    }

    pub fn coordinates(&self) -> (&nd::Array1<f64>, &nd::Array1<f64>, &nd::Array1<f64>) {
        (&self.x_m, &self.y_m, &self.z_m)
    }

    pub fn spacing(&self) -> f64 {
        2.0 * self.domain_radius_m / (self.grid_points_per_axis as f64 - 1.0)
    }

    fn shape_function(radius: &nd::Array3<f64>, radius_value: f64, sigma: f64) -> nd::Array3<f64> {
        let denom = 2.0 * (sigma * radius_value).tanh();
        radius.mapv(|r| ((sigma * (r + radius_value)).tanh() - (sigma * (r - radius_value)).tanh()) / denom)
    }

    fn build_metric_tensors(
        beta: &nd::Array3<f64>,
    ) -> (nd::Array5<f64>, nd::Array5<f64>, nd::Array5<f64>) {
        let shape = beta.raw_dim();
        let n = shape[0];
        let mut metric = nd::Array5::zeros((n, n, n, 4, 4));
        let mut spatial = nd::Array5::zeros((n, n, n, 3, 3));
        let mut gram = nd::Array5::zeros((n, n, n, 4, 4));

        for i in 0..n {
            for j in 0..n {
                for k in 0..n {
                    let b = beta[[i, j, k]];
                    metric[[i, j, k, 0, 0]] = -1.0 + b * b;
                    metric[[i, j, k, 0, 1]] = b;
                    metric[[i, j, k, 1, 0]] = b;
                    metric[[i, j, k, 1, 1]] = 1.0;
                    metric[[i, j, k, 2, 2]] = 1.0;
                    metric[[i, j, k, 3, 3]] = 1.0;

                    spatial[[i, j, k, 0, 0]] = 1.0;
                    spatial[[i, j, k, 1, 1]] = 1.0;
                    spatial[[i, j, k, 2, 2]] = 1.0;

                    gram[[i, j, k, 0, 0]] = 1.0 + b * b;
                    gram[[i, j, k, 0, 1]] = b;
                    gram[[i, j, k, 1, 0]] = b;
                    gram[[i, j, k, 1, 1]] = 1.0;
                    gram[[i, j, k, 2, 2]] = 1.0;
                    gram[[i, j, k, 3, 3]] = 1.0;
                }
            }
        }
        (metric, spatial, gram)
    }

    pub fn region_mask(
        &self,
        x_bounds: (f64, f64),
        y_bounds: (f64, f64),
        z_bounds: (f64, f64),
    ) -> nd::Array3<bool> {
        assert!(x_bounds.0 <= x_bounds.1, "x bounds must be ordered");
        assert!(y_bounds.0 <= y_bounds.1, "y bounds must be ordered");
        assert!(z_bounds.0 <= z_bounds.1, "z bounds must be ordered");
        let n = self.grid_points_per_axis;
        nd::Array3::from_shape_fn((n, n, n), |(i, j, k)| {
            self.x_m[i] >= x_bounds.0
                && self.x_m[i] <= x_bounds.1
                && self.y_m[j] >= y_bounds.0
                && self.y_m[j] <= y_bounds.1
                && self.z_m[k] >= z_bounds.0
                && self.z_m[k] <= z_bounds.1
        })
    }

    pub fn audit(&self, tolerance: f64) -> std::collections::HashMap<String, f64> {
        let n = self.grid_points_per_axis;
        let mut min_abs_det = f64::INFINITY;
        let mut det_error: f64 = 0.0;
        let mut min_gram_ev = f64::INFINITY;

        for i in 0..n {
            for j in 0..n {
                for k in 0..n {
                    let mut g = na::SMatrix::<f64, 4, 4>::zeros();
                    for a in 0..4 {
                        for b in 0..4 {
                            g[(a, b)] = self.metric_4d_grid[[i, j, k, a, b]];
                        }
                    }
                    let det = g.determinant();
                    min_abs_det = min_abs_det.min(det.abs());
                    det_error = det_error.max((det + 1.0).abs());

                    let mut gr = na::SMatrix::<f64, 4, 4>::zeros();
                    for a in 0..4 {
                        for b in 0..4 {
                            gr[(a, b)] = self.gram_metric_grid[[i, j, k, a, b]];
                        }
                    }
                    let gev = na::linalg::SymmetricEigen::new(gr).eigenvalues;
                    for v in gev.iter() {
                        min_gram_ev = min_gram_ev.min(*v);
                    }
                }
            }
        }

        let passed = det_error <= 100.0 * tolerance && min_abs_det > tolerance && min_gram_ev > tolerance;
        let mut map = std::collections::HashMap::new();
        map.insert("passed".to_string(), if passed { 1.0 } else { 0.0 });
        map.insert("determinant_error".to_string(), det_error);
        map.insert("minimum_abs_determinant".to_string(), min_abs_det);
        map.insert("minimum_gram_eigenvalue".to_string(), min_gram_ev);
        map
    }
}
