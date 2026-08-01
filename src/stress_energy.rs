//! Numerical stress-energy auditor with 3-D finite-difference curvature.

use ndarray as nd;
use nalgebra as na;
use rand::Rng;
use rand::SeedableRng;
use rand_distr::{Distribution, StandardNormal, Uniform};
use std::collections::HashMap;

fn gradient3(field: &nd::ArrayView3<f64>, axis: usize, h: f64) -> nd::Array3<f64> {
    let mut out = nd::Array3::zeros(field.raw_dim());
    let n = field.shape()[axis];
    let n0 = field.shape()[0];
    let n1 = field.shape()[1];
    let n2 = field.shape()[2];
    if n <= 1 {
        return out;
    }
    for i in 0..n0 {
        for j in 0..n1 {
            for k in 0..n2 {
                let idx = [i, j, k];
                let get = |p: usize| {
                    let mut c = idx;
                    c[axis] = p;
                    field[[c[0], c[1], c[2]]]
                };
                let pos = idx[axis];
                let der = if pos == 0 {
                    (-3.0 * get(0) + 4.0 * get(1) - get(2)) / (2.0 * h)
                } else if pos + 1 == n {
                    (3.0 * get(n - 1) - 4.0 * get(n - 2) + get(n - 3)) / (2.0 * h)
                } else {
                    (get(pos + 1) - get(pos - 1)) / (2.0 * h)
                };
                out[[i, j, k]] = der;
            }
        }
    }
    out
}

pub struct StressEnergyAuditor {
    pub metric: nd::Array5<f64>,
    pub coordinates: (nd::Array1<f64>, nd::Array1<f64>, nd::Array1<f64>),
    pub cosmological_constant: f64,
    pub gravitational_constant: f64,
    pub inverse_metric: nd::Array5<f64>,
    pub christoffel: nd::ArrayD<f64>,
    pub ricci: nd::Array5<f64>,
    pub ricci_scalar: nd::Array3<f64>,
    pub einstein: nd::Array5<f64>,
    pub stress_energy: nd::Array5<f64>,
}

impl StressEnergyAuditor {
    pub fn new(
        metric: nd::Array5<f64>,
        coordinates: (nd::Array1<f64>, nd::Array1<f64>, nd::Array1<f64>),
        cosmological_constant: f64,
        gravitational_constant: f64,
    ) -> Self {
        let n = metric.shape()[0];
        assert_eq!(metric.shape(), &[n, n, n, 4, 4], "metric must have shape (n,n,n,4,4)");
        assert!(
            coordinates.0.len() == n && coordinates.1.len() == n && coordinates.2.len() == n,
            "coordinates must match metric grid"
        );
        for ax in [&coordinates.0, &coordinates.1, &coordinates.2] {
            assert!(ax.len() >= 3, "each axis must contain at least three points");
            for i in 1..ax.len() {
                assert!(ax[i] > ax[i - 1], "coordinate axes must be strictly increasing");
            }
        }

        let mut symmetry_error: f64 = 0.0;
        for i in 0..n {
            for j in 0..n {
                for k in 0..n {
                    for a in 0..4 {
                        for b in 0..4 {
                            let diff = (metric[[i, j, k, a, b]] - metric[[i, j, k, b, a]]).abs();
                            symmetry_error = symmetry_error.max(diff);
                        }
                    }
                }
            }
        }
        assert!(symmetry_error <= 1.0e-10, "metric must be symmetric");

        let mut inverse_metric = nd::Array5::zeros((n, n, n, 4, 4));
        for i in 0..n {
            for j in 0..n {
                for k in 0..n {
                    let mut g = na::SMatrix::<f64, 4, 4>::zeros();
                    for a in 0..4 {
                        for b in 0..4 {
                            g[(a, b)] = metric[[i, j, k, a, b]];
                        }
                    }
                    let inv = g.try_inverse().expect("metric must be invertible");
                    for a in 0..4 {
                        for b in 0..4 {
                            inverse_metric[[i, j, k, a, b]] = inv[(a, b)];
                        }
                    }
                }
            }
        }

        let mut auditor = StressEnergyAuditor {
            metric,
            coordinates,
            cosmological_constant,
            gravitational_constant,
            inverse_metric,
            christoffel: nd::ArrayD::zeros(nd::IxDyn(&[n, n, n, 4, 4, 4])),
            ricci: nd::Array5::zeros((n, n, n, 4, 4)),
            ricci_scalar: nd::Array3::zeros((n, n, n)),
            einstein: nd::Array5::zeros((n, n, n, 4, 4)),
            stress_energy: nd::Array5::zeros((n, n, n, 4, 4)),
        };
        auditor.compute_geometry();
        auditor
    }

    fn spacing_x(&self) -> f64 {
        let n = self.coordinates.0.len();
        (self.coordinates.0[n - 1] - self.coordinates.0[0]) / (n as f64 - 1.0)
    }
    fn spacing_y(&self) -> f64 {
        let n = self.coordinates.1.len();
        (self.coordinates.1[n - 1] - self.coordinates.1[0]) / (n as f64 - 1.0)
    }
    fn spacing_z(&self) -> f64 {
        let n = self.coordinates.2.len();
        (self.coordinates.2[n - 1] - self.coordinates.2[0]) / (n as f64 - 1.0)
    }

    fn compute_christoffel_symbols(&mut self) {
        let n = self.metric.shape()[0];
        let mut metric_deriv = nd::Array6::zeros((4, n, n, n, 4, 4));

        for nu in 0..4 {
            for sigma in 0..4 {
                let comp = self.metric.slice(nd::s![.., .., .., nu, sigma]);
                let gx = gradient3(&comp, 0, self.spacing_x());
                let gy = gradient3(&comp, 1, self.spacing_y());
                let gz = gradient3(&comp, 2, self.spacing_z());
                for i in 0..n {
                    for j in 0..n {
                        for k in 0..n {
                            metric_deriv[[0, i, j, k, nu, sigma]] = 0.0;
                            metric_deriv[[1, i, j, k, nu, sigma]] = gx[[i, j, k]];
                            metric_deriv[[2, i, j, k, nu, sigma]] = gy[[i, j, k]];
                            metric_deriv[[3, i, j, k, nu, sigma]] = gz[[i, j, k]];
                        }
                    }
                }
            }
        }

        for i in 0..n {
            for j in 0..n {
                for k in 0..n {
                    for alpha in 0..4 {
                        for mu in 0..4 {
                            for nu in 0..4 {
                                let mut total = 0.0;
                                for sigma in 0..4 {
                                    let d_mu_nu_sig = metric_deriv[[mu, i, j, k, nu, sigma]];
                                    let d_nu_mu_sig = metric_deriv[[nu, i, j, k, mu, sigma]];
                                    let d_sig_mu_nu = metric_deriv[[sigma, i, j, k, mu, nu]];
                                    total += self.inverse_metric[[i, j, k, alpha, sigma]]
                                        * (d_mu_nu_sig + d_nu_mu_sig - d_sig_mu_nu);
                                }
                                self.christoffel[nd::IxDyn(&[i, j, k, alpha, mu, nu])] = 0.5 * total;
                            }
                        }
                    }
                }
            }
        }
    }

    fn gamma_component(&self, i: usize, j: usize, k: usize, alpha: usize, mu: usize, nu: usize) -> f64 {
        self.christoffel[nd::IxDyn(&[i, j, k, alpha, mu, nu])]
    }

    fn compute_ricci(&mut self) {
        let n = self.metric.shape()[0];
        let h = (self.spacing_x(), self.spacing_y(), self.spacing_z());

        // Precompute spatial derivatives of every Christoffel component.
        let mut dgamma_x = nd::ArrayD::zeros(nd::IxDyn(&[n, n, n, 4, 4, 4]));
        let mut dgamma_y = nd::ArrayD::zeros(nd::IxDyn(&[n, n, n, 4, 4, 4]));
        let mut dgamma_z = nd::ArrayD::zeros(nd::IxDyn(&[n, n, n, 4, 4, 4]));

        for alpha in 0..4 {
            for mu in 0..4 {
                for nu in 0..4 {
                    let comp = self.christoffel.slice(nd::s![.., .., .., alpha, mu, nu]);
                    let comp3 = comp.into_dimensionality::<nd::Ix3>().unwrap();
                    let gx = gradient3(&comp3.view(), 0, h.0);
                    let gy = gradient3(&comp3.view(), 1, h.1);
                    let gz = gradient3(&comp3.view(), 2, h.2);
                    for i in 0..n {
                        for j in 0..n {
                            for k in 0..n {
                                dgamma_x[nd::IxDyn(&[i, j, k, alpha, mu, nu])] = gx[[i, j, k]];
                                dgamma_y[nd::IxDyn(&[i, j, k, alpha, mu, nu])] = gy[[i, j, k]];
                                dgamma_z[nd::IxDyn(&[i, j, k, alpha, mu, nu])] = gz[[i, j, k]];
                            }
                        }
                    }
                }
            }
        }

        // Gamma^alpha_{alpha beta}
        let mut gamma_trace = nd::Array4::zeros((n, n, n, 4));
        for i in 0..n {
            for j in 0..n {
                for k in 0..n {
                    for beta in 0..4 {
                        let mut sum = 0.0;
                        for alpha in 0..4 {
                            sum += self.gamma_component(i, j, k, alpha, alpha, beta);
                        }
                        gamma_trace[[i, j, k, beta]] = sum;
                    }
                }
            }
        }

        // Spatial derivatives of the trace.
        let mut dtrace_x = nd::Array4::zeros((n, n, n, 4));
        let mut dtrace_y = nd::Array4::zeros((n, n, n, 4));
        let mut dtrace_z = nd::Array4::zeros((n, n, n, 4));
        for beta in 0..4 {
            let comp = gamma_trace.slice(nd::s![.., .., .., beta]);
            let comp3 = comp.into_dimensionality::<nd::Ix3>().unwrap();
            let gx = gradient3(&comp3.view(), 0, h.0);
            let gy = gradient3(&comp3.view(), 1, h.1);
            let gz = gradient3(&comp3.view(), 2, h.2);
            for i in 0..n {
                for j in 0..n {
                    for k in 0..n {
                        dtrace_x[[i, j, k, beta]] = gx[[i, j, k]];
                        dtrace_y[[i, j, k, beta]] = gy[[i, j, k]];
                        dtrace_z[[i, j, k, beta]] = gz[[i, j, k]];
                    }
                }
            }
        }

        for i in 0..n {
            for j in 0..n {
                for k in 0..n {
                    for beta in 0..4 {
                        for nu in 0..4 {
                            // div_alpha = sum_alpha d/dx^alpha Gamma^alpha_{nu beta}
                            let mut div_alpha = 0.0;
                            for alpha in 0..4 {
                                let d = match alpha {
                                    1 => dgamma_x[nd::IxDyn(&[i, j, k, alpha, nu, beta])],
                                    2 => dgamma_y[nd::IxDyn(&[i, j, k, alpha, nu, beta])],
                                    3 => dgamma_z[nd::IxDyn(&[i, j, k, alpha, nu, beta])],
                                    _ => 0.0,
                                };
                                div_alpha += d;
                            }

                            // d_nu of Gamma^alpha_{alpha beta}
                            let d_trace = match nu {
                                1 => dtrace_x[[i, j, k, beta]],
                                2 => dtrace_y[[i, j, k, beta]],
                                3 => dtrace_z[[i, j, k, beta]],
                                _ => 0.0,
                            };

                            // prod1 = Gamma^alpha_{alpha lambda} Gamma^lambda_{nu beta}
                            let mut prod1 = 0.0;
                            for lambda in 0..4 {
                                prod1 += gamma_trace[[i, j, k, lambda]]
                                    * self.gamma_component(i, j, k, lambda, nu, beta);
                            }

                            // prod2 = Gamma^alpha_{nu lambda} Gamma^lambda_{alpha beta}
                            let mut prod2 = 0.0;
                            for alpha in 0..4 {
                                for lambda in 0..4 {
                                    prod2 += self.gamma_component(i, j, k, alpha, nu, lambda)
                                        * self.gamma_component(i, j, k, lambda, alpha, beta);
                                }
                            }

                            let r = div_alpha - d_trace + prod1 - prod2;
                            self.ricci[[i, j, k, beta, nu]] = r;
                        }
                    }

                    // Symmetrize Ricci.
                    for a in 0..4 {
                        for b in a + 1..4 {
                            let avg = 0.5 * (self.ricci[[i, j, k, a, b]] + self.ricci[[i, j, k, b, a]]);
                            self.ricci[[i, j, k, a, b]] = avg;
                            self.ricci[[i, j, k, b, a]] = avg;
                        }
                    }

                    // Ricci scalar
                    let mut rs = 0.0;
                    for a in 0..4 {
                        for b in 0..4 {
                            rs += self.inverse_metric[[i, j, k, a, b]] * self.ricci[[i, j, k, a, b]];
                        }
                    }
                    self.ricci_scalar[[i, j, k]] = rs;

                    // Einstein and stress-energy.
                    for a in 0..4 {
                        for b in 0..4 {
                            let ein = self.ricci[[i, j, k, a, b]]
                                - 0.5 * rs * self.metric[[i, j, k, a, b]];
                            self.einstein[[i, j, k, a, b]] = ein;
                            self.stress_energy[[i, j, k, a, b]] =
                                (ein + self.cosmological_constant * self.metric[[i, j, k, a, b]])
                                    / (8.0 * std::f64::consts::PI * self.gravitational_constant);
                        }
                    }
                }
            }
        }
    }

    fn compute_geometry(&mut self) {
        self.compute_christoffel_symbols();
        self.compute_ricci();
    }

    fn orthonormal_tetrad(metric: &na::SMatrix<f64, 4, 4>) -> na::SMatrix<f64, 4, 4> {
        let eigen = na::linalg::SymmetricEigen::new(*metric);
        let mut pairs: Vec<(f64, usize)> = eigen
            .eigenvalues
            .iter()
            .enumerate()
            .map(|(i, &v)| (v, i))
            .collect();
        pairs.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

        let negatives: Vec<usize> = pairs.iter().filter(|(v, _)| *v < 0.0).map(|(_, i)| *i).collect();
        let positives: Vec<usize> = pairs.iter().filter(|(v, _)| *v > 0.0).map(|(_, i)| *i).collect();
        assert_eq!(negatives.len(), 1, "metric does not have Lorentzian signature");
        assert_eq!(positives.len(), 3, "metric does not have Lorentzian signature");

        let order = [negatives[0], positives[0], positives[1], positives[2]];
        let mut tetrad = na::SMatrix::<f64, 4, 4>::zeros();
        for (col, &idx) in order.iter().enumerate() {
            let scale = 1.0 / eigen.eigenvalues[idx].abs().sqrt();
            for row in 0..4 {
                tetrad[(row, col)] = eigen.eigenvectors[(row, idx)] * scale;
            }
        }
        tetrad
    }

    pub fn audit_energy_conditions(
        &self,
        sample_count: usize,
        seed: u64,
        tolerance: f64,
    ) -> HashMap<String, f64> {
        assert!(sample_count > 0, "sample_count must be positive");
        let n = self.metric.shape()[0];
        let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
        let normal = StandardNormal;
        let speed_dist = Uniform::new(0.0, 0.8);

        let mut nec_values = Vec::with_capacity(sample_count);
        let mut wec_values = Vec::with_capacity(sample_count);
        let mut null_residuals = Vec::with_capacity(sample_count);
        let mut timelike_residuals = Vec::with_capacity(sample_count);

        for _ in 0..sample_count {
            let idx = (rng.gen_range(0..n), rng.gen_range(0..n), rng.gen_range(0..n));
            let mut g = na::SMatrix::<f64, 4, 4>::zeros();
            let mut t = na::SMatrix::<f64, 4, 4>::zeros();
            for a in 0..4 {
                for b in 0..4 {
                    g[(a, b)] = self.metric[[idx.0, idx.1, idx.2, a, b]];
                    t[(a, b)] = self.stress_energy[[idx.0, idx.1, idx.2, a, b]];
                }
            }
            let tetrad = Self::orthonormal_tetrad(&g);

            let mut dir = [0.0; 3];
            let mut norm: f64 = 0.0;
            for d in &mut dir {
                let v: f64 = normal.sample(&mut rng);
                *d = v;
                norm += v * v;
            }
            norm = norm.sqrt();
            for d in &mut dir {
                *d /= norm;
            }

            let null_hat = na::SVector::<f64, 4>::new(1.0, dir[0], dir[1], dir[2]);
            let null_vector = tetrad * null_hat;
            let null_norm = null_vector.dot(&(g * null_vector));
            null_residuals.push(null_norm.abs());
            nec_values.push(null_vector.dot(&(t * null_vector)));

            let speed: f64 = speed_dist.sample(&mut rng);
            let gamma = 1.0_f64 / (1.0 - speed * speed).sqrt();
            let timelike_hat = na::SVector::<f64, 4>::new(
                gamma,
                gamma * speed * dir[0],
                gamma * speed * dir[1],
                gamma * speed * dir[2],
            );
            let timelike_vector = tetrad * timelike_hat;
            let timelike_norm = timelike_vector.dot(&(g * timelike_vector));
            timelike_residuals.push((timelike_norm + 1.0).abs());
            wec_values.push(timelike_vector.dot(&(t * timelike_vector)));
        }

        let min_nec = nec_values.into_iter().fold(f64::INFINITY, |a, b| a.min(b));
        let min_wec = wec_values.into_iter().fold(f64::INFINITY, |a, b| a.min(b));
        let max_null_res = null_residuals.into_iter().fold(f64::NEG_INFINITY, |a, b| a.max(b));
        let max_time_res = timelike_residuals.into_iter().fold(f64::NEG_INFINITY, |a, b| a.max(b));

        let finite_geometry = self.christoffel.iter().all(|v| v.is_finite())
            && self.ricci.iter().all(|v| v.is_finite())
            && self.stress_energy.iter().all(|v| v.is_finite());

        let passed = finite_geometry && max_null_res <= 1.0e-7 && max_time_res <= 1.0e-7;

        let mut map = HashMap::new();
        map.insert("passed".to_string(), if passed { 1.0 } else { 0.0 });
        map.insert("nec_passed".to_string(), if min_nec >= -tolerance { 1.0 } else { 0.0 });
        map.insert("wec_passed".to_string(), if min_wec >= -tolerance { 1.0 } else { 0.0 });
        map.insert("minimum_nec_energy_density".to_string(), min_nec);
        map.insert("minimum_wec_energy_density".to_string(), min_wec);
        map.insert("maximum_null_norm_residual".to_string(), max_null_res);
        map.insert("maximum_timelike_norm_residual".to_string(), max_time_res);
        map.insert("sample_count".to_string(), sample_count as f64);
        map
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nalgebra as na;

    #[test]
    fn orthonormal_tetrad_minkowski() {
        let g = na::SMatrix::<f64, 4, 4>::new(
            -1.0, 0.0, 0.0, 0.0,
            0.0, 1.0, 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
            0.0, 0.0, 0.0, 1.0,
        );
        let tetrad = StressEnergyAuditor::orthonormal_tetrad(&g);
        let eta = tetrad.transpose() * g * tetrad;
        let expected = na::SMatrix::<f64, 4, 4>::new(
            -1.0, 0.0, 0.0, 0.0,
            0.0, 1.0, 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
            0.0, 0.0, 0.0, 1.0,
        );
        let diff = (eta - expected).norm();
        assert!(diff < 1.0e-12, "eta residual = {}", diff);
    }

    #[test]
    fn orthonormal_tetrad_bubble_center() {
        let beta = 1.071186351229e0;
        let g = na::SMatrix::<f64, 4, 4>::new(
            -1.0 + beta * beta, beta, 0.0, 0.0,
            beta, 1.0, 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
            0.0, 0.0, 0.0, 1.0,
        );
        let eigen = na::linalg::SymmetricEigen::new(g);
        eprintln!("eigenvalues = {:?}", eigen.eigenvalues);
        eprintln!("V^T V = {:?}", eigen.eigenvectors.transpose() * eigen.eigenvectors);
        eprintln!("V g V^T = {:?}", eigen.eigenvectors * g * eigen.eigenvectors.transpose());
        eprintln!("V^T g V = {:?}", eigen.eigenvectors.transpose() * g * eigen.eigenvectors);
        let tetrad = StressEnergyAuditor::orthonormal_tetrad(&g);
        let eta = tetrad.transpose() * g * tetrad;
        eprintln!("eta = {:?}", eta);
        let expected = na::SMatrix::<f64, 4, 4>::new(
            -1.0, 0.0, 0.0, 0.0,
            0.0, 1.0, 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
            0.0, 0.0, 0.0, 1.0,
        );
        let diff = (eta - expected).norm();
        assert!(diff < 1.0e-9, "eta residual = {}", diff);
    }
}
