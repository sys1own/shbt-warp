//! Boundary register and phase-locked excitation engine.

use nalgebra as na;
use num_complex::Complex;
use std::collections::HashMap;

type C64 = Complex<f64>;

const SU2_CHARGE_LABELS: [u32; 3] = [22, 23, 26];
const SU3_LOW_WEIGHTS: [(u32, u32); 3] = [(0, 0), (1, 0), (0, 1)];

/// Distance from a real number to the nearest integer.
fn distance_to_integer(x: f64) -> f64 {
    (x - x.round()).abs()
}

/// Sign of a permutation of (0, 1, 2).
fn permutation_sign(p: (usize, usize, usize)) -> f64 {
    let a = [p.0, p.1, p.2];
    let mut inv = 0;
    for i in 0..3 {
        for j in (i + 1)..3 {
            if a[i] > a[j] {
                inv += 1;
            }
        }
    }
    if inv % 2 == 0 {
        1.0
    } else {
        -1.0
    }
}

fn su2_modular_s_entry(left: u32, right: u32, level: u32) -> f64 {
    let coef = (2.0 / (level as f64 + 2.0)).sqrt();
    let arg = std::f64::consts::PI * ((left + 1) as f64) * ((right + 1) as f64)
        / (level as f64 + 2.0);
    coef * arg.sin()
}

fn su2_conformal_weight(label: u32, level: u32) -> f64 {
    let l = label as f64;
    l * (l + 2.0) / (4.0 * (level as f64 + 2.0))
}

fn su2_central_charge(level: u32) -> f64 {
    (3.0 * level as f64) / (level as f64 + 2.0)
}

fn su3_vector(weight: (u32, u32)) -> [f64; 3] {
    let p = weight.0 as f64;
    let q = weight.1 as f64;
    [(2.0 * p + q) / 3.0, (q - p) / 3.0, (-2.0 * p - q) / 3.0]
}

fn su3_conformal_weight(weight: (u32, u32), level: u32) -> f64 {
    let p = weight.0 as f64;
    let q = weight.1 as f64;
    let numerator = p * p + q * q + p * q + 3.0 * p + 3.0 * q;
    numerator / (3.0 * (level as f64 + 3.0))
}

fn su3_central_charge(level: u32) -> f64 {
    (8.0 * level as f64) / (level as f64 + 3.0)
}

fn su3_modular_s_entry(left: (u32, u32), right: (u32, u32), level: u32) -> C64 {
    let rho = su3_vector((1, 1));
    let lambda = su3_vector(left);
    let mu = su3_vector(right);
    let lambda_rho: [f64; 3] = [lambda[0] + rho[0], lambda[1] + rho[1], lambda[2] + rho[2]];
    let mu_rho: [f64; 3] = [mu[0] + rho[0], mu[1] + rho[1], mu[2] + rho[2]];

    let permutations = [
        (0usize, 1usize, 2usize),
        (0usize, 2usize, 1usize),
        (1usize, 0usize, 2usize),
        (1usize, 2usize, 0usize),
        (2usize, 0usize, 1usize),
        (2usize, 1usize, 0usize),
    ];

    let mut weyl_sum = C64::new(0.0, 0.0);
    let denom = level as f64 + 3.0;
    for perm in permutations {
        let sign = permutation_sign(perm);
        let permuted = [
            lambda_rho[perm.0],
            lambda_rho[perm.1],
            lambda_rho[perm.2],
        ];
        let dot = permuted[0] * mu_rho[0] + permuted[1] * mu_rho[1] + permuted[2] * mu_rho[2];
        let phase = -2.0 * std::f64::consts::PI * dot / denom;
        weyl_sum += C64::new(sign * phase.cos(), sign * phase.sin());
    }
    let prefactor = C64::new(0.0, -1.0) / ((3.0_f64).sqrt() * denom);
    prefactor * weyl_sum
}

pub(crate) fn flatten_index(su2: usize, su3: usize) -> usize {
    su2 * 3 + su3
}

fn conformal_phase_angles() -> [f64; 9] {
    let c_visible = su2_central_charge(26) + su3_central_charge(8);
    let mut angles = [0.0; 9];
    for (i, &charge) in SU2_CHARGE_LABELS.iter().enumerate() {
        let h_su2 = su2_conformal_weight(charge, 26);
        for (j, &weight) in SU3_LOW_WEIGHTS.iter().enumerate() {
            let h_su3 = su3_conformal_weight(weight, 8);
            angles[flatten_index(i, j)] = h_su2 + h_su3 - c_visible / 24.0;
        }
    }
    angles
}

/// Visible SU(2)_26 x SU(3)_8 modular boundary register.
#[derive(Clone, Debug)]
pub struct BoundaryRegister {
    pub lepton_level: u32,
    pub quark_level: u32,
    pub parent_level: u32,
    pub charge_labels: [u32; 3],
    pub low_su3_weights: [(u32, u32); 3],
    pub su2_visible_block: [[f64; 3]; 3],
    pub su3_visible_block: [[C64; 3]; 3],
    pub raw_loading: [[f64; 3]; 3],
    pub z_boundary: f64,
    pub rho_e: [[f64; 3]; 3],
    pub shannon_contributions: [[f64; 3]; 3],
    pub shannon_density: [[f64; 3]; 3],
    pub shannon_entropy: f64,
}

impl BoundaryRegister {
    pub fn new() -> Self {
        let mut su2 = [[0.0; 3]; 3];
        let mut su3 = [[C64::new(0.0, 0.0); 3]; 3];
        for i in 0..3 {
            for j in 0..3 {
                su2[i][j] = su2_modular_s_entry(SU2_CHARGE_LABELS[i], SU2_CHARGE_LABELS[j], 26);
                su3[i][j] = su3_modular_s_entry(SU3_LOW_WEIGHTS[i], SU3_LOW_WEIGHTS[j], 8);
            }
        }

        let mut raw_loading = [[0.0; 3]; 3];
        let mut z = 0.0;
        for i in 0..3 {
            for j in 0..3 {
                let val = su2[i][j].powi(2) * su3[i][j].norm_sqr();
                raw_loading[i][j] = val;
                z += val;
            }
        }
        if !z.is_finite() || z <= 0.0 {
            panic!("visible modular loading has invalid normalization");
        }

        let mut rho_e = [[0.0; 3]; 3];
        let mut shannon_contributions = [[0.0; 3]; 3];
        let mut entropy = 0.0;
        for i in 0..3 {
            for j in 0..3 {
                rho_e[i][j] = raw_loading[i][j] / z;
                if rho_e[i][j] > 0.0 {
                    let c = -rho_e[i][j] * rho_e[i][j].ln();
                    shannon_contributions[i][j] = c;
                    entropy += c;
                }
            }
        }
        if entropy <= 0.0 {
            panic!("visible register has zero Shannon entropy");
        }
        let mut shannon_density = [[0.0; 3]; 3];
        for i in 0..3 {
            for j in 0..3 {
                shannon_density[i][j] = shannon_contributions[i][j] / entropy;
            }
        }

        BoundaryRegister {
            lepton_level: 26,
            quark_level: 8,
            parent_level: 312,
            charge_labels: SU2_CHARGE_LABELS,
            low_su3_weights: SU3_LOW_WEIGHTS,
            su2_visible_block: su2,
            su3_visible_block: su3,
            raw_loading,
            z_boundary: z,
            rho_e,
            shannon_contributions,
            shannon_density,
            shannon_entropy: entropy,
        }
    }

    pub fn branch(&self) -> (u32, u32, u32) {
        (self.lepton_level, self.quark_level, self.parent_level)
    }

    pub fn audit(&self, tolerance: f64) -> HashMap<String, f64> {
        let mut map = HashMap::new();
        let sum_rho: f64 = self.rho_e.iter().flat_map(|row| row.iter()).sum();
        let sum_density: f64 = self.shannon_density.iter().flat_map(|row| row.iter()).sum();
        map.insert("normalization_error".to_string(), (sum_rho - 1.0).abs());
        map.insert("shannon_density_error".to_string(), (sum_density - 1.0).abs());
        map.insert(
            "minimum_loading".to_string(),
            self.raw_loading
                .iter()
                .flat_map(|row| row.iter())
                .copied()
                .fold(f64::INFINITY, |a, b| a.min(b)),
        );
        map.insert(
            "maximum_loading".to_string(),
            self.raw_loading
                .iter()
                .flat_map(|row| row.iter())
                .copied()
                .fold(f64::NEG_INFINITY, |a, b| a.max(b)),
        );
        let passed = (sum_rho - 1.0).abs() <= tolerance
            && (sum_density - 1.0).abs() <= tolerance
            && self.raw_loading.iter().all(|row| row.iter().all(|&v| v >= -1e-15))
            && self.shannon_entropy > 0.0;
        map.insert("passed".to_string(), if passed { 1.0 } else { 0.0 });
        map
    }
}

/// Phase-locked unitary excitation engine.
pub struct ExcitationEngine {
    pub boundary: BoundaryRegister,
    pub theta_phase: f64,
    pub framing_charge: f64,
    _spectral_radius: f64,
    k_norm: na::SMatrix<f64, 9, 9>,
}

impl ExcitationEngine {
    pub fn new(boundary: BoundaryRegister, theta_phase: f64) -> Self {
        let mut k = na::SMatrix::<f64, 9, 9>::zeros();
        for su3_index in 0..3 {
            for su2_index in 0..2 {
                let left = flatten_index(su2_index, su3_index);
                let right = flatten_index(su2_index + 1, su3_index);
                k[(left, right)] = -0.5;
                k[(right, left)] = 0.5;
            }
        }

        // G = i * K is Hermitian; its spectral radius is the largest singular value of K.
        let g2 = -k * k;
        let eigen = na::linalg::SymmetricEigen::new(g2);
        let spectral_radius_squared = eigen
            .eigenvalues
            .iter()
            .map(|v| v.abs())
            .fold(0.0_f64, |a, b| a.max(b));
        if spectral_radius_squared == 0.0 {
            panic!("excitation generator has zero spectral radius");
        }
        let spectral_radius = spectral_radius_squared.sqrt();
        let k_norm = k / spectral_radius;

        ExcitationEngine {
            boundary,
            theta_phase,
            framing_charge: 1.0,
            _spectral_radius: spectral_radius,
            k_norm,
        }
    }

    fn phase_lock_matrix(&self, theta: f64) -> na::SMatrix<C64, 9, 9> {
        let angles = conformal_phase_angles();
        let diag = na::SVector::<C64, 9>::from_fn(|i, _| {
            let im = 2.0 * std::f64::consts::PI * theta * angles[i];
            C64::new(im.cos(), im.sin())
        });
        na::SMatrix::<C64, 9, 9>::from_diagonal(&diag)
    }

    fn excitation_operator_matrix(&self, theta: f64) -> na::SMatrix<C64, 9, 9> {
        // exp(-i * theta * G) = exp(theta * K) for G = i * K.
        // K is block-wise tridiagonal real antisymmetric; exp is computed per block
        // using the Rodrigues formula for 3x3 real antisymmetric matrices.
        let s = theta.sin();
        let c = 1.0 - theta.cos();

        let mut mixing = na::SMatrix::<C64, 9, 9>::identity();
        for block in 0..3 {
            let idx = [block, block + 3, block + 6];
            let mut a = na::SMatrix::<f64, 3, 3>::zeros();
            for i in 0..3 {
                for j in 0..3 {
                    a[(i, j)] = self.k_norm[(idx[i], idx[j])];
                }
            }
            let a2 = &a * &a;
            let exp_block = na::SMatrix::<f64, 3, 3>::identity() + a * s + a2 * c;
            for i in 0..3 {
                for j in 0..3 {
                    mixing[(idx[i], idx[j])] = C64::new(exp_block[(i, j)], 0.0);
                }
            }
        }

        mixing * self.phase_lock_matrix(theta)
    }

    pub fn excitation_operator(&self) -> na::SMatrix<C64, 9, 9> {
        self.excitation_operator_matrix(self.theta_phase)
    }

    pub fn excited_density_matrix(&self) -> na::SMatrix<C64, 9, 9> {
        let op = self.excitation_operator();
        let rho_vec: Vec<C64> = self
            .boundary
            .rho_e
            .iter()
            .flat_map(|row| row.iter())
            .map(|&v| C64::new(v, 0.0))
            .collect();
        let diag = na::SVector::<C64, 9>::from_iterator(rho_vec.into_iter());
        let baseline = na::SMatrix::<C64, 9, 9>::from_diagonal(&diag);
        let excited = op * baseline * op.adjoint();
        let tr = excited.trace().re;
        excited / C64::new(tr, 0.0)
    }

    pub fn excited_probability(&self) -> [f64; 9] {
        let rho = self.excited_density_matrix();
        let mut probs = [0.0; 9];
        let mut sum = 0.0;
        for i in 0..9 {
            probs[i] = rho[(i, i)].re.max(0.0);
            sum += probs[i];
        }
        if sum > 0.0 {
            for p in probs.iter_mut() {
                *p /= sum;
            }
        }
        probs
    }

    pub fn excited_shannon_contributions(&self) -> [[f64; 3]; 3] {
        let probs = self.excited_probability();
        let mut out = [[0.0; 3]; 3];
        for i in 0..3 {
            for j in 0..3 {
                let p = probs[flatten_index(i, j)];
                out[i][j] = if p > 0.0 { -p * p.ln() } else { 0.0 };
            }
        }
        out
    }

    pub fn excited_density_matrix_at(&self, theta: f64) -> na::SMatrix<C64, 9, 9> {
        let op = self.excitation_operator_matrix(theta);
        let rho_vec: Vec<C64> = self
            .boundary
            .rho_e
            .iter()
            .flat_map(|row| row.iter())
            .map(|&v| C64::new(v, 0.0))
            .collect();
        let diag = na::SVector::<C64, 9>::from_iterator(rho_vec.into_iter());
        let baseline = na::SMatrix::<C64, 9, 9>::from_diagonal(&diag);
        let excited = op * baseline * op.adjoint();
        let tr = excited.trace().re;
        excited / C64::new(tr, 0.0)
    }

    pub fn excited_probability_at(&self, theta: f64) -> [f64; 9] {
        let rho = self.excited_density_matrix_at(theta);
        let mut probs = [0.0; 9];
        let mut sum = 0.0;
        for i in 0..9 {
            probs[i] = rho[(i, i)].re.max(0.0);
            sum += probs[i];
        }
        if sum > 0.0 {
            for p in probs.iter_mut() {
                *p /= sum;
            }
        }
        probs
    }

    pub fn population_shift_at(&self, theta: f64) -> f64 {
        let probs = self.excited_probability_at(theta);
        let mut sum = 0.0;
        for i in 0..3 {
            for j in 0..3 {
                let base = self.boundary.rho_e[i][j];
                sum += (probs[flatten_index(i, j)] - base).abs();
            }
        }
        sum
    }

    pub fn framing_defect(&self) -> f64 {
        let lepton_lift = self.boundary.parent_level as f64 / (2.0 * self.boundary.lepton_level as f64);
        let quark_lift = self.boundary.parent_level as f64 / (3.0 * self.boundary.quark_level as f64);
        distance_to_integer(lepton_lift).max(distance_to_integer(quark_lift))
    }

    pub fn closure_norm(&self) -> f64 {
        let defect = self.framing_defect();
        let closure = na::SMatrix::<f64, 4, 4>::identity() * self.framing_charge * defect;
        closure.norm()
    }

    pub fn audit(&self, tolerance: f64) -> HashMap<String, f64> {
        let op = self.excitation_operator();
        let id = na::SMatrix::<C64, 9, 9>::identity();
        let unitarity_error = (op.adjoint() * op - id).norm();
        let excited = self.excited_density_matrix();
        let trace_error = (excited.trace() - C64::new(1.0, 0.0)).norm();
        let hermiticity_error = (excited - excited.adjoint()).norm();
        let pop_shift = {
            let excited_probs = self.excited_probability();
            let mut sum = 0.0;
            for i in 0..3 {
                for j in 0..3 {
                    let base = self.boundary.rho_e[i][j];
                    sum += (excited_probs[flatten_index(i, j)] - base).abs();
                }
            }
            sum
        };
        let framing = self.framing_defect();
        let closure_norm = self.closure_norm();
        let passed = unitarity_error <= tolerance
            && trace_error <= tolerance
            && hermiticity_error <= tolerance
            && pop_shift > tolerance
            && framing.abs() <= 0.0
            && closure_norm.abs() <= 0.0;

        let mut map = HashMap::new();
        map.insert("passed".to_string(), if passed { 1.0 } else { 0.0 });
        map.insert("unitarity_error".to_string(), unitarity_error);
        map.insert("trace_error".to_string(), trace_error);
        map.insert("hermiticity_error".to_string(), hermiticity_error);
        map.insert("population_shift_l1".to_string(), pop_shift);
        map.insert("framing_defect".to_string(), framing);
        map.insert("closure_norm".to_string(), closure_norm);
        map
    }
}
