//! Causal observer: comoving Minkowski plateau and operational power.

use crate::constants::*;
use crate::projector::FGSliceProjector;
use nalgebra as na;
use ndarray as nd;
use std::collections::HashMap;

pub struct CausalObserver {
    pub projector: FGSliceProjector,
}

impl CausalObserver {
    pub fn new(projector: FGSliceProjector) -> Self {
        CausalObserver { projector }
    }

    pub fn power_requirement_mw(radius_m: f64, delta_mod: f64) -> f64 {
        assert!(radius_m > 0.0, "radius_m must be positive");
        let power_watts = LIGHT_SPEED_M_S.powi(5) / GRAVITATIONAL_CONSTANT_SI
            * delta_mod.abs()
            / (24.0 * std::f64::consts::PI)
            * (power_scale_radius_m() / radius_m).powi(2);
        power_watts / 1.0e6
    }

    fn christoffel_at_index(&self, index: usize) -> [na::SMatrix<f64, 4, 4>; 4] {
        let metric_view = self.projector.lorentzian_metrics.slice(nd::s![index, .., ..]);
        let mut g = na::SMatrix::<f64, 4, 4>::zeros();
        for a in 0..4 {
            for b in 0..4 {
                g[(a, b)] = metric_view[[a, b]];
            }
        }
        let inv = g.try_inverse().expect("metric must be invertible");
        let beta_c = self.projector.beta_m_s[index] / LIGHT_SPEED_M_S;
        let beta_g = self.projector.beta_gradient_per_s[index] / LIGHT_SPEED_M_S;

        let mut metric_gradient = na::SMatrix::<f64, 4, 4>::zeros();
        metric_gradient[(0, 0)] = 2.0 * beta_c * beta_g;
        metric_gradient[(0, 1)] = beta_g;
        metric_gradient[(1, 0)] = beta_g;

        let mut gamma = [
            na::SMatrix::<f64, 4, 4>::zeros(),
            na::SMatrix::<f64, 4, 4>::zeros(),
            na::SMatrix::<f64, 4, 4>::zeros(),
            na::SMatrix::<f64, 4, 4>::zeros(),
        ];

        for alpha in 0..4 {
            for mu in 0..4 {
                for nu in 0..4 {
                    let mut total = 0.0;
                    for delta in 0..4 {
                        let p_mu = if mu == 1 { metric_gradient[(delta, nu)] } else { 0.0 };
                        let p_nu = if nu == 1 { metric_gradient[(delta, mu)] } else { 0.0 };
                        let p_delta = if delta == 1 { metric_gradient[(mu, nu)] } else { 0.0 };
                        total += inv[(alpha, delta)] * (p_mu + p_nu - p_delta);
                    }
                    gamma[alpha][(mu, nu)] = 0.5 * total;
                }
            }
        }
        gamma
    }

    pub fn audit(&self, tolerance: f64) -> HashMap<String, f64> {
        let n = self.projector.grid_points;
        let center = n / 2;
        let beta_c = self.projector.beta_m_s[center] / LIGHT_SPEED_M_S;

        let metric_view = self.projector.lorentzian_metrics.slice(nd::s![center, .., ..]);
        let mut g = na::SMatrix::<f64, 4, 4>::zeros();
        for a in 0..4 {
            for b in 0..4 {
                g[(a, b)] = metric_view[[a, b]];
            }
        }

        let mut jacobian = na::SMatrix::<f64, 4, 4>::identity();
        jacobian[(1, 0)] = beta_c;
        let inv_jacobian = jacobian.try_inverse().expect("jacobian invertible");
        let observer_metric = inv_jacobian.transpose() * g * inv_jacobian;
        let minkowski = na::SMatrix::<f64, 4, 4>::from_diagonal(&na::SVector::<f64, 4>::new(-1.0, 1.0, 1.0, 1.0));
        let observer_metric_error = (observer_metric - minkowski).norm();

        let four_velocity = na::SVector::<f64, 4>::new(1.0, -beta_c, 0.0, 0.0);
        let normalization_error = (four_velocity.dot(&(g * four_velocity)) + 1.0).abs();

        let gamma = self.christoffel_at_index(center);
        let mut four_acceleration = na::SVector::<f64, 4>::zeros();
        for alpha in 0..4 {
            let mut sum = 0.0;
            for mu in 0..4 {
                for nu in 0..4 {
                    sum += gamma[alpha][(mu, nu)] * four_velocity[mu] * four_velocity[nu];
                }
            }
            four_acceleration[alpha] = sum;
        }
        let acceleration_norm = (four_acceleration[1].powi(2)
            + four_acceleration[2].powi(2)
            + four_acceleration[3].powi(2))
            .sqrt()
            * LIGHT_SPEED_M_S.powi(2);

        let plateau_error = (self.projector.shape[center] - 1.0).abs();
        let plateau_gradient = self.projector.shape_gradient_per_m[center].abs();
        let power_mw = Self::power_requirement_mw(self.projector.bubble_radius_m, self.projector.delta_mod);

        let passed = plateau_error <= tolerance
            && plateau_gradient <= tolerance
            && observer_metric_error <= 100.0 * tolerance
            && normalization_error <= 100.0 * tolerance
            && acceleration_norm <= tolerance;

        let mut map = HashMap::new();
        map.insert("passed".to_string(), if passed { 1.0 } else { 0.0 });
        map.insert("plateau_error".to_string(), plateau_error);
        map.insert("plateau_gradient_per_m".to_string(), plateau_gradient);
        map.insert("observer_metric_error".to_string(), observer_metric_error);
        map.insert("four_velocity_normalization_error".to_string(), normalization_error);
        map.insert("acceleration_norm_m_s2".to_string(), acceleration_norm);
        map.insert("power_requirement_mw".to_string(), power_mw);
        map
    }
}
