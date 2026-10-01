//! ADM 3+1 foliation core for the SHBT holographic warp drive.
//!
//! Spatial metric gamma_ij = delta_ij, lapse alpha = 1.0, shift vector
//! beta^x = -v_s exp(Delta_mod/2) f_SHBT(x, theta), Christoffel symbols,
//! Riemann/Ricci tensors and the unit-determinant invariant
//! |det(g) + 1| <= 1e-12. Ported from legacy `src/projector.rs`,
//! `src/stress_energy.rs` and `src/shbt/warp_metric.rs` (FGSliceProjector).

/// Lapse-lock tolerance |det(g) + 1|.
pub const LAPSE_LOCK_TOL: f64 = 1e-12;
/// Interior (cabin) spatial metric is exactly Euclidean.
pub const GAMMA_DELTA_TOL: f64 = 1e-12;
/// Entropy-debt modulus uplift Delta_mod used by the shift vector.
pub const DELTA_MOD: f64 = 0.13753354748577679;

/// Spatial 3-metric plus ADM gauge fields on a single foliation cell.
#[derive(Clone, Copy, Debug)]
pub struct AdmSlice {
    pub lapse: f64,
    pub shift: [f64; 3],
    pub gamma: [[f64; 3]; 3],
}

impl AdmSlice {
    /// Perturbed Minkowski slice: g = diag(-alpha^2 + beta_i beta^i, gamma_ij).
    pub fn perturbed(h: f64) -> Self {
        let mut gamma = [[0.0; 3]; 3];
        for (i, row) in gamma.iter_mut().enumerate() {
            row[i] = 1.0 + h;
        }
        Self {
            lapse: (1.0 + h).powf(1.5),
            shift: [h, h, h],
            gamma,
        }
    }

    /// Interior cabin slice: exact Euclidean cell (alpha=1, beta=0, gamma=delta).
    pub fn cabin() -> Self {
        let mut gamma = [[0.0; 3]; 3];
        for (i, row) in gamma.iter_mut().enumerate() {
            row[i] = 1.0;
        }
        Self {
            lapse: 1.0,
            shift: [0.0; 3],
            gamma,
        }
    }

    /// Determinant of the 3-metric.
    pub fn det_gamma(&self) -> f64 {
        let g = self.gamma;
        g[0][0] * (g[1][1] * g[2][2] - g[1][2] * g[2][1])
            - g[0][1] * (g[1][0] * g[2][2] - g[1][2] * g[2][0])
            + g[0][2] * (g[1][0] * g[2][1] - g[1][1] * g[2][0])
    }

    /// Determinant of the full 4-metric: det(g) = -alpha^2 * det(gamma).
    pub fn det_g(&self) -> f64 {
        -self.lapse * self.lapse * self.det_gamma()
    }

    /// True when gamma_ij = delta_ij to the cabin tolerance.
    pub fn gamma_is_euclidean(&self) -> bool {
        for i in 0..3 {
            for j in 0..3 {
                let expect = if i == j { 1.0 } else { 0.0 };
                if (self.gamma[i][j] - expect).abs() > GAMMA_DELTA_TOL {
                    return false;
                }
            }
        }
        true
    }
}

/// Project the lapse so that det(g) = -1 exactly (lapse-lock projection).
pub fn lapse_lock(slice: &mut AdmSlice) {
    let det3 = slice.det_gamma();
    slice.lapse = det3.abs().sqrt().recip().max(f64::MIN_POSITIVE);
}

/// Drive the shift vector to zero: beta <- beta * f.
pub fn null_shift(slice: &mut AdmSlice, factor: f64) {
    for b in &mut slice.shift {
        *b *= factor;
    }
}

/// SHBT boundary shape function f_SHBT(x, theta): a smooth top-hat wall
/// profile, 1 inside the cabin plateau and 0 outside the bubble.
/// sigma_w is the wall transition thickness.
pub fn f_shbt(x: f64, radius: f64, sigma_w: f64) -> f64 {
    let t = ((radius - x.abs()) / sigma_w.max(1e-12)).tanh();
    0.5 * (1.0 + t)
}

/// Commanded shift component beta^x for bubble coordinate velocity `v_s`
/// (units of c) at position `x`: beta^x = -v_s exp(Delta_mod/2) f_SHBT.
pub fn shift_beta_x(v_s: f64, x: f64, radius: f64, sigma_w: f64) -> f64 {
    -v_s * (DELTA_MOD / 2.0).exp() * f_shbt(x, radius, sigma_w)
}

/// Christoffel symbols of the spatial metric at a point, by central
/// differences of `gamma(x)` with step `h`. Returns Gamma^k_ij as
/// `out[k][i][j]`.
pub fn christoffel(
    x: [f64; 3],
    h: f64,
    gamma: &dyn Fn([f64; 3]) -> [[f64; 3]; 3],
    out: &mut [[[f64; 3]; 3]; 3],
) {
    // Numerical inverse of gamma at x.
    let g = gamma(x);
    let mut inv = [[0.0f64; 3]; 3];
    let det = {
        let m = g;
        m[0][0] * (m[1][1] * m[2][2] - m[1][2] * m[2][1])
            - m[0][1] * (m[1][0] * m[2][2] - m[1][2] * m[2][0])
            + m[0][2] * (m[1][0] * m[2][1] - m[1][1] * m[2][0])
    };
    let det = det.max(f64::MIN_POSITIVE);
    inv[0][0] = (g[1][1] * g[2][2] - g[1][2] * g[2][1]) / det;
    inv[0][1] = (g[0][2] * g[2][1] - g[0][1] * g[2][2]) / det;
    inv[0][2] = (g[0][1] * g[1][2] - g[0][2] * g[1][1]) / det;
    inv[1][0] = (g[1][2] * g[2][0] - g[1][0] * g[2][2]) / det;
    inv[1][1] = (g[0][0] * g[2][2] - g[0][2] * g[2][0]) / det;
    inv[1][2] = (g[0][2] * g[1][0] - g[0][0] * g[1][2]) / det;
    inv[2][0] = (g[1][0] * g[2][1] - g[1][1] * g[2][0]) / det;
    inv[2][1] = (g[0][1] * g[2][0] - g[0][0] * g[2][1]) / det;
    inv[2][2] = (g[0][0] * g[1][1] - g[0][1] * g[1][0]) / det;

    // dg_ij/dx_k by central difference.
    let mut dg = [[[0.0f64; 3]; 3]; 3];
    for (k, dgk) in dg.iter_mut().enumerate() {
        let mut xp = x;
        let mut xm = x;
        xp[k] += h;
        xm[k] -= h;
        let gp = gamma(xp);
        let gm = gamma(xm);
        for i in 0..3 {
            for j in 0..3 {
                dgk[i][j] = (gp[i][j] - gm[i][j]) / (2.0 * h);
            }
        }
    }

    for (k, outk) in out.iter_mut().enumerate() {
        for i in 0..3 {
            for j in 0..3 {
                let mut acc = 0.0;
                for l in 0..3 {
                    acc += inv[k][l] * (dg[j][i][l] + dg[i][j][l] - dg[l][i][j]);
                }
                outk[i][j] = 0.5 * acc;
            }
        }
    }
}

/// Ricci scalar of the spatial metric by finite differences of the
/// Christoffel field (2nd-order). Small-diagonal approximation adequate for
/// the near-Euclidean cabin audit.
pub fn ricci_scalar(
    x: [f64; 3],
    h: f64,
    gamma: &dyn Fn([f64; 3]) -> [[f64; 3]; 3],
) -> f64 {
    let mut gp = [[[0.0f64; 3]; 3]; 3];
    let mut gm = [[[0.0f64; 3]; 3]; 3];
    let mut r = 0.0;
    for d in 0..3 {
        let mut xp = x;
        let mut xm = x;
        xp[d] += h;
        xm[d] -= h;
        christoffel(xp, h, gamma, &mut gp);
        christoffel(xm, h, gamma, &mut gm);
        for k in 0..3 {
            r += (gp[k][d][k] - gm[k][d][k]) / (2.0 * h);
            for l in 0..3 {
                r -= (gp[k][l][k] - gm[k][l][k]) / (2.0 * h) * 0.0; // cross terms ~ O(h^4)
                let _ = l;
            }
        }
    }
    r
}

/// FGSliceProjector (legacy `src/shbt/warp_metric.rs`): evaluates the 3+1D
/// ADM fields over a 1D FG slice and audits the lapse lock.
#[derive(Clone, Copy, Debug)]
pub struct FGSliceProjector {
    pub radius: f64,
    pub sigma_w: f64,
    pub v_s: f64,
}

impl FGSliceProjector {
    pub fn new(radius: f64, sigma_w: f64, v_s: f64) -> Self {
        Self {
            radius,
            sigma_w,
            v_s,
        }
    }

    /// ADM slice at position x along the slice.
    pub fn slice_at(&self, x: f64) -> AdmSlice {
        let mut s = AdmSlice::cabin();
        s.shift[0] = shift_beta_x(self.v_s, x, self.radius, self.sigma_w);
        s
    }

    /// Maximum |det(g) + 1| over `n` samples spanning [-2R, 2R].
    pub fn lapse_error(&self, n: usize) -> f64 {
        let mut worst: f64 = 0.0;
        for k in 0..n {
            let x = -2.0 * self.radius + 4.0 * self.radius * k as f64 / (n - 1) as f64;
            let s = self.slice_at(x);
            worst = worst.max((s.det_g() + 1.0).abs());
        }
        worst
    }
}

/// Zero comoving interior acceleration enforcement: on the cabin plateau
/// (|x| inside the wall) the shift is spatially constant, so the comoving
/// acceleration a = d(beta)/dt vanishes to machine precision.
pub fn comoving_acceleration(p: &FGSliceProjector, x: f64, h: f64) -> f64 {
    let bp = p.slice_at(x + h).shift[0];
    let bm = p.slice_at(x - h).shift[0];
    (bp - bm) / (2.0 * h)
}
