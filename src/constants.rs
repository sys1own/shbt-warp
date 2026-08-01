/// Physical and algorithmic constants shared by the SHBT warp simulator.

pub const LIGHT_SPEED_M_S: f64 = 299_792_458.0;
pub const GRAVITATIONAL_CONSTANT_SI: f64 = 6.674_30e-11;

pub const C_DARK_RESIDUAL_NUM: f64 = 834_433.0;
pub const C_DARK_COMP_NUM: f64 = 1_197_103.0;
pub const C_DARK_DEN: f64 = 362_670.0;
pub const C_DARK_RESIDUAL: f64 = C_DARK_RESIDUAL_NUM / C_DARK_DEN;
pub const C_DARK_COMP: f64 = C_DARK_COMP_NUM / C_DARK_DEN;
pub const DELTA_MOD: f64 = C_DARK_COMP / 24.0;

pub const N_SAT_BITS: f64 = 3.312_593_327_986e122;
pub const N_LOCAL_BITS_10M: f64 = 1.202_481e72;
pub const POWER_BENCHMARK_MW: f64 = 142.08;
pub const LAMBDA_HOLO_SI: f64 = 1.089_138_83e-52;

pub const MEGAPARSEC_M: f64 = 3.085_677_581_491_367_3e22;
pub const HUBBLE_LOADING_KM_S_MPC: f64 = 4.797_960;
pub const HUBBLE_LOADING_S_INV: f64 = HUBBLE_LOADING_KM_S_MPC * 1.0e3 / MEGAPARSEC_M;
pub const HOLOGRAPHIC_LOCK_RATE_S_INV: f64 = 3.0 * HUBBLE_LOADING_S_INV;
pub const THERMODYNAMIC_RELAXATION_RATE_S_INV: f64 = HOLOGRAPHIC_LOCK_RATE_S_INV / 24.0;

pub const DEFAULT_PHASE_THETA: f64 = 0.421;
pub const DEFAULT_BUBBLE_RADIUS_M: f64 = 10.0;
pub const DEFAULT_DOMAIN_RADIUS_M: f64 = 30.0;
pub const DEFAULT_GRID_POINTS: usize = 1201;
pub const DEFAULT_STRESS_GRID_POINTS: usize = 7;
pub const DEFAULT_WALL_STEEPNESS_PER_M: f64 = 0.8;
pub const NUMERICAL_TOLERANCE: f64 = 1.0e-12;

/// Power-scale radius used by the operational-power benchmark.
pub fn power_scale_radius_m() -> f64 {
    DEFAULT_BUBBLE_RADIUS_M
        * (POWER_BENCHMARK_MW * 1.0e6 * GRAVITATIONAL_CONSTANT_SI * 24.0 * std::f64::consts::PI
            / (LIGHT_SPEED_M_S.powi(5) * DELTA_MOD))
            .sqrt()
}

/// Thermodynamic kappa used by the entropy-debt engine.
pub fn thermodynamic_kappa_per_j() -> f64 {
    THERMODYNAMIC_RELAXATION_RATE_S_INV * DELTA_MOD / (POWER_BENCHMARK_MW * 1.0e6)
}
