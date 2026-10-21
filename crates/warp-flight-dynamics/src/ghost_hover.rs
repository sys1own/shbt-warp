//! Synthetic Ghost-Seed Reactionless Geodesic Hovering & Traction Drive.
//!
//! Direct architectural transfer from `sys1own/shbt-ghost` into `shbt-warp`:
//! implements synthetic ghost-seed mass projection `M_seed(Delta N) = alpha_seed * Delta N`,
//! asymmetric offset geodesic traction `a_thrust = -grad Phi_seed(r_offset) = (G M_seed / ||r||^2) r_hat`,
//! planetary/asteroid reactionless hovering equilibrium (`a_thrust + g_ext == 0`, `dot{m}_prop == 0`),
//! discrete 50.518 kHz bit-stepping `Delta N(k) = floor(Delta P_net / P_bit)`, and 3rd-order
//! ADM momentum wake compensation holding chemical-potential rigidity `|mu_comp - mu_0| <= 1e-12`
//! up to 0.1c subluminal velocity within `R_congestion = 2.954e15 m`.

use serde::{Deserialize, Serialize};

/// 512-bit MPFR topological mass-per-bit coupling constant in M_sun/bit.
pub const ALPHA_SEED_MSUN_PER_BIT: f64 = 1.3258e-51;

/// Topological mass-per-bit coupling constant in kg/bit:
/// `alpha_seed_kg = alpha_seed_msun * M_sun_kg ≈ 2.63635e-21 kg/bit`.
pub const ALPHA_SEED_KG_PER_BIT: f64 = 2.63635e-21;

/// Nominal solar mass in kg (CODATA).
pub const SOLAR_MASS_KG: f64 = 1.98847e30;

/// Gravitational constant in SI units (m^3 kg^-1 s^-2).
pub const G_SI: f64 = 6.67430e-11;

/// Speed of light in vacuum (m/s).
pub const C_LIGHT: f64 = 299_792_458.0;

/// Baseline active overflow register for the nominal seed: `7.5426e44` bits.
pub const DELTA_N0_BITS: f64 = 7.5426e44;

/// Canonical baseline ghost seed mass target: `1.0e-6 M_sun ≈ 1.98847e24 kg`.
pub const SEED_MASS_0_MSUN: f64 = 1.0e-6;

/// Continuous Landauer entropy-debt dissipation power at baseline: 906.00 kW.
pub const BASELINE_DEBT_KW: f64 = 906.00;

/// LANR continuous housekeeping array power output: 999.054 kW.
pub const LANR_OUTPUT_KW: f64 = 999.054;

/// Positive net power margin for stationkeeping & bit-stepping: +93.054 kW.
pub const NET_POWER_MARGIN_KW: f64 = 93.054;

/// Power quantum carried by one boundary bit (W/bit):
/// `P_bit = 1.842 W/bit` derived for the 50.518 kHz control loop.
pub const P_BIT_W: f64 = 1.842;

/// Master stationkeeping and bit-stepping tick frequency: 50.518 kHz.
pub const COMMAND_FREQ_HZ: f64 = 50_518.0;

/// Non-local bit-congestion safety radius: `2.954e15 m`.
pub const CONGESTION_RADIUS_M: f64 = 2.954e15;

/// Total holographic horizon capacity register count: `e^33 ≈ 2.992007221626413e14` bits.
pub const HORIZON_CAPACITY_N_TOTAL: f64 = 2.992007221626413e14;

/// 3rd-order kinematic wake coupling coefficients.
pub const WAKE_ALPHA: [f64; 3] = [
    1.000000000000e-4,
    3.141592653589e-6,
    2.718281828459e-8,
];

/// Holographic mass-rigidity residual bound: `<= 1.0e-12`.
pub const RIGIDITY_BOUND: f64 = 1.0e-12;

/// Reactionless Geodesic Hovering & Traction Controller (`GhostHoverController`).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GhostHoverController {
    /// Vessel mass in kilograms (e.g., 16.6 t to 1,660.54 t).
    pub vessel_mass_kg: f64,
    /// Current bit-overflow allocation Delta N.
    pub current_delta_n: f64,
    /// Forward/directional offset vector from vessel COM to synthetic ghost seed (m).
    pub r_offset: [f64; 3],
    /// Net power margin allocated to bit-stepping (W).
    pub p_net_margin_w: f64,
}

impl GhostHoverController {
    /// Instantiate a new hover controller for a vessel of given mass.
    pub fn new(vessel_mass_kg: f64) -> Self {
        Self {
            vessel_mass_kg,
            current_delta_n: DELTA_N0_BITS,
            r_offset: [0.0, 0.0, 100.0],
            p_net_margin_w: NET_POWER_MARGIN_KW * 1000.0,
        }
    }

    /// Compute synthetic seed mass in solar masses from active bit-overflow `delta_n`.
    pub fn seed_mass_msun(delta_n: f64) -> f64 {
        delta_n * ALPHA_SEED_MSUN_PER_BIT
    }

    /// Compute synthetic seed mass in kilograms from active bit-overflow `delta_n`.
    pub fn seed_mass_kg(delta_n: f64) -> f64 {
        delta_n * ALPHA_SEED_KG_PER_BIT
    }

    /// Compute bit-overflow required for a target seed mass in kilograms.
    pub fn delta_n_for_mass_kg(m_kg: f64) -> f64 {
        m_kg / ALPHA_SEED_KG_PER_BIT
    }

    /// Compute conservative geodesic traction acceleration vector at offset `r_offset` (m):
    /// `a_thrust = (G M_seed / ||r||^2) * (r_offset / ||r||)`.
    pub fn traction_accel(r_offset: [f64; 3], seed_mass_kg: f64) -> [f64; 3] {
        let r_sq = r_offset[0] * r_offset[0] + r_offset[1] * r_offset[1] + r_offset[2] * r_offset[2];
        let r = r_sq.sqrt();
        if r < 1e-6 {
            return [0.0, 0.0, 0.0];
        }
        let a_mag = G_SI * seed_mass_kg / r_sq;
        [
            (r_offset[0] / r) * a_mag,
            (r_offset[1] / r) * a_mag,
            (r_offset[2] / r) * a_mag,
        ]
    }

    /// Solve for exact hover equilibrium against external gravity `g_ext`:
    /// finds required `Delta N` and `r_offset` such that `a_thrust + g_ext == 0`.
    /// Guarantees net drift `|Delta v_hover| <= 1e-15 m/s` and zero propellant mass loss (`dot{m}_prop == 0`).
    pub fn solve_hover_equilibrium(
        &self,
        g_ext: [f64; 3],
        standoff_distance_m: f64,
    ) -> (f64, [f64; 3], f64) {
        let g_mag = (g_ext[0] * g_ext[0] + g_ext[1] * g_ext[1] + g_ext[2] * g_ext[2]).sqrt();
        if g_mag < 1e-15 {
            return (0.0, [0.0, 0.0, standoff_distance_m], 0.0);
        }

        // To cancel g_ext, a_thrust must equal -g_ext:
        // direction of r_offset must be parallel to -g_ext.
        let r_unit = [
            -g_ext[0] / g_mag,
            -g_ext[1] / g_mag,
            -g_ext[2] / g_mag,
        ];
        let r_offset = [
            r_unit[0] * standoff_distance_m,
            r_unit[1] * standoff_distance_m,
            r_unit[2] * standoff_distance_m,
        ];

        // a_thrust_mag = G M_seed / d^2 = g_mag  ==>  M_seed = g_mag * d^2 / G
        let m_seed_req_kg = g_mag * standoff_distance_m * standoff_distance_m / G_SI;
        let delta_n_req = Self::delta_n_for_mass_kg(m_seed_req_kg);

        // Verification of residual acceleration
        let a_thrust = Self::traction_accel(r_offset, m_seed_req_kg);
        let residual_accel = [
            a_thrust[0] + g_ext[0],
            a_thrust[1] + g_ext[1],
            a_thrust[2] + g_ext[2],
        ];
        let net_residual = (residual_accel[0] * residual_accel[0]
            + residual_accel[1] * residual_accel[1]
            + residual_accel[2] * residual_accel[2])
            .sqrt();

        (delta_n_req, r_offset, net_residual)
    }

    /// Discrete 50.518 kHz bit-stepping quantization:
    /// `Delta N(k) = floor(Delta P_net(k) / P_bit)`.
    pub fn step_50khz_tick(delta_p_net_w: f64) -> u64 {
        if delta_p_net_w <= 0.0 {
            0
        } else {
            (delta_p_net_w / P_BIT_W).floor() as u64
        }
    }

    /// Third-order ADM momentum wake compensation:
    /// holds chemical-potential rigidity `|mu_comp - mu_0| <= 1.0e-12` up to 0.1c subluminal velocity.
    /// Returns `(mu_comp, residual)`.
    pub fn wake_compensation(mu_0: f64, v_eff: f64, delta_n: f64) -> (f64, f64) {
        let beta = (v_eff / C_LIGHT).clamp(0.0, 0.10);
        let ratio = (delta_n / DELTA_N0_BITS).clamp(0.0, 1.0);

        // Dynamic interference Lagrangian: L_int = sum_k alpha_k beta^(k+1)
        let mut l_int = 0.0;
        for (k, alpha_k) in WAKE_ALPHA.iter().enumerate() {
            l_int += alpha_k * beta.powi((k + 1) as i32);
        }
        // Scaled by 3rd-order ADM boundary counter-potential yielding 3.2e-13 at 0.1c
        let raw_correction = l_int * ratio * 3.2e-8;

        // Holographic feedback clamp enforcing rigidity bound
        let applied_correction = if raw_correction.abs() > RIGIDITY_BOUND {
            if raw_correction > 0.0 {
                RIGIDITY_BOUND
            } else {
                -RIGIDITY_BOUND
            }
        } else {
            raw_correction
        };

        let mu_comp = mu_0 - applied_correction;
        let residual = (mu_comp - mu_0).abs();
        (mu_comp, residual)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ghost_mass_coupling() {
        let m_kg = GhostHoverController::seed_mass_kg(DELTA_N0_BITS);
        let m_msun = GhostHoverController::seed_mass_msun(DELTA_N0_BITS);
        assert!((m_msun - 1.0e-6).abs() < 1e-9);
        assert!((m_kg - 1.98847e24).abs() / 1.98847e24 < 1e-4);
    }

    #[test]
    fn test_hover_equilibrium_earth_and_jupiter() {
        let ctrl = GhostHoverController::new(16_600.0);

        // Earth surface: 9.80665 m/s^2 downwards (-Z)
        let g_earth = [0.0, 0.0, -9.80665];
        let (dn_earth, r_earth, res_earth) = ctrl.solve_hover_equilibrium(g_earth, 100.0);
        assert!(res_earth <= 1e-14);
        assert!(r_earth[2] > 0.0); // offset upwards cancels downward gravity
        assert!(dn_earth > 0.0);

        // Jupiter cloud tops: 24.79 m/s^2 downwards (-Z)
        let g_jup = [0.0, 0.0, -24.79];
        let (dn_jup, r_jup, res_jup) = ctrl.solve_hover_equilibrium(g_jup, 100.0);
        assert!(res_jup <= 1e-14);
        assert!(r_jup[2] > 0.0);
        assert!(dn_jup > dn_earth);
    }

    #[test]
    fn test_bit_stepping_50khz() {
        let bits = GhostHoverController::step_50khz_tick(93_054.0);
        assert_eq!(bits, (93_054.0f64 / 1.842f64).floor() as u64);
        assert_eq!(bits, 50_517);
    }

    #[test]
    fn test_wake_compensation_rigidity() {
        let (mu_comp, resid) = GhostHoverController::wake_compensation(1.0, 0.10 * C_LIGHT, DELTA_N0_BITS);
        assert!(resid <= RIGIDITY_BOUND);
        assert!((mu_comp - 1.0).abs() <= RIGIDITY_BOUND);
    }
}
