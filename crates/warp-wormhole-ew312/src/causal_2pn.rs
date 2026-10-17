//! Second Post-Newtonian (2PN) Causal Lightcone Authorization Engine.
//!
//! Evaluates the 2PN spacetime metric tensor in harmonic coordinates:
//!   ds² = g00 c²dt² + 2 g0i c dt dx^i + gij dx^i dx^j
//!
//! Causal authorization requires Δs²_2PN ≤ 0 along the dragged superluminal
//! shift-sheared lightcone slice.

use crate::constants::*;

pub const M_SUN: f64 = 1.98847e30;
pub const J2_SUN: f64 = 2.20e-7;
pub const R_SUN: f64 = 6.96342e8;
pub const S_SUN: f64 = 1.92e33;

#[derive(Clone, Copy, Debug)]
pub struct Metric2pn {
    pub g00: f64,
    pub g0i: [f64; 3],
    pub g_space: f64,
}

fn legendre_p2(r_vec: [f64; 3]) -> f64 {
    let r = (r_vec[0].powi(2) + r_vec[1].powi(2) + r_vec[2].powi(2)).sqrt();
    if r == 0.0 {
        return 0.0;
    }
    let ct = r_vec[2] / r;
    0.5 * (3.0 * ct * ct - 1.0)
}

pub fn evaluate_2pn_metric(r_vec: [f64; 3]) -> Metric2pn {
    let r = (r_vec[0].powi(2) + r_vec[1].powi(2) + r_vec[2].powi(2)).sqrt().max(1.0);
    let u = G_CONST * M_SUN / (C_LIGHT * C_LIGHT * r);
    let u2 = u * u;
    let j2_term = 3.0 * G_CONST * M_SUN * J2_SUN * R_SUN * R_SUN / (C_LIGHT * C_LIGHT * r * r * r)
        * legendre_p2(r_vec);
    let g00 = -(1.0 - 2.0 * u + 2.0 * u2 + j2_term);

    let s_cross_r = [S_SUN * -r_vec[1], S_SUN * r_vec[0], 0.0];
    let g0i_scale = -2.0 * G_CONST / (C_LIGHT * C_LIGHT * C_LIGHT * r * r * r) * (1.0 + 3.0 * u);
    let g0i = [
        g0i_scale * s_cross_r[0],
        g0i_scale * s_cross_r[1],
        g0i_scale * s_cross_r[2],
    ];
    let g_space = 1.0 + 2.0 * u + 1.5 * u2;
    Metric2pn { g00, g0i, g_space }
}

/// Computes the 2PN invariant interval ds^2_2PN.
/// Negative values (ds^2 <= 0) indicate authorized causal connections.
pub fn interval_2pn(dt: f64, dr: [f64; 3], r_vec: [f64; 3]) -> f64 {
    let m = evaluate_2pn_metric(r_vec);
    let dr2 = dr[0] * dr[0] + dr[1] * dr[1] + dr[2] * dr[2];
    let cross = 2.0 * C_LIGHT * dt * (m.g0i[0] * dr[0] + m.g0i[1] * dr[1] + m.g0i[2] * dr[2]);
    m.g00 * C_LIGHT * C_LIGHT * dt * dt + cross + m.g_space * dr2
}

/// Shift-sheared ADM 2PN interval including the dragged soliton shift vector beta^i.
pub fn interval_2pn_with_shift(dt: f64, dr: [f64; 3], beta: [f64; 3], r_vec: [f64; 3]) -> f64 {
    let m = evaluate_2pn_metric(r_vec);
    let dx_shifted = [
        dr[0] + beta[0] * dt,
        dr[1] + beta[1] * dt,
        dr[2] + beta[2] * dt,
    ];
    let dr2_shifted = dx_shifted[0] * dx_shifted[0] + dx_shifted[1] * dx_shifted[1] + dx_shifted[2] * dx_shifted[2];
    let cross = 2.0 * C_LIGHT * dt * (m.g0i[0] * dx_shifted[0] + m.g0i[1] * dx_shifted[1] + m.g0i[2] * dx_shifted[2]);
    m.g00 * C_LIGHT * C_LIGHT * dt * dt + cross + m.g_space * dr2_shifted
}

/// Evaluates whether (dt, dr) is causally authorized (returns true if ds^2 <= 0).
pub fn is_causally_authorized(dt: f64, dr: [f64; 3], r_vec: [f64; 3]) -> bool {
    interval_2pn(dt, dr, r_vec) <= 0.0
}
