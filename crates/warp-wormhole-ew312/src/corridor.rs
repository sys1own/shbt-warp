//! crates/warp-wormhole-ew312/src/corridor.rs
//! Progressive Eikonal Foliation & Adaptive TTN-MERA De-Rendering Corridor.
#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

pub const BASELINE_FRAME_NUCLEONS: f64 = 1.0e28;
pub const BASELINE_MERA_LAYERS: usize = 94;
pub const MAX_ISOMETRY_RESIDUAL: f64 = 1.0e-14;
pub const MAX_DERENDER_VELOCITY_MPS: f64 = 56.988;
pub const MAX_DIMENSIONLESS_ACCEL: f64 = 5.773502691896258;
pub const SYSTEM_TICK_HZ: f64 = 50_518.0;
pub const SYSTEM_TICK_SEC: f64 = 1.0 / SYSTEM_TICK_HZ;
pub const DMA_RING_BPS: f64 = 504.0e9;
pub const DMA_RING_Z0_OHMS: f64 = 50.12;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct EikonalKinematicState {
    pub tau: f64,
    pub s_pos: f64,
    pub s_vel_dimless: f64,
    pub s_acc_dimless: f64,
    pub velocity_mps: f64,
    pub kac_moody_char_flux_gbps: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressiveEikonalFoliation {
    pub characteristic_length_m: f64,
    pub sweep_duration_s: f64,
    pub lapse_alpha: f64,
    pub shift_beta: [f64; 3],
}

impl ProgressiveEikonalFoliation {
    pub fn new(characteristic_length_m: f64, sweep_duration_s: f64) -> Result<Self, &'static str> {
        let char_speed = characteristic_length_m / sweep_duration_s;
        if char_speed * 1.875 > MAX_DERENDER_VELOCITY_MPS + 1e-9 {
            return Err("Eikonal sweep rate exceeds 56.988 m/s kinematic stability ceiling");
        }
        Ok(Self {
            characteristic_length_m,
            sweep_duration_s,
            lapse_alpha: 1.0,
            shift_beta: [0.0; 3],
        })
    }

    #[inline]
    pub fn evaluate_kinematics(&self, tau: f64) -> EikonalKinematicState {
        let t = tau.clamp(0.0, 1.0);
        let t2 = t * t;
        let t3 = t2 * t;
        let t4 = t3 * t;
        let t5 = t4 * t;

        let s_pos = 10.0 * t3 - 15.0 * t4 + 6.0 * t5;
        let s_vel_dimless = 30.0 * t2 - 60.0 * t3 + 30.0 * t4;
        let s_acc_dimless = 60.0 * t - 180.0 * t2 + 120.0 * t3;

        let char_speed = self.characteristic_length_m / self.sweep_duration_s;
        let velocity_mps = s_vel_dimless * char_speed;
        let kac_moody_char_flux_gbps = (velocity_mps / MAX_DERENDER_VELOCITY_MPS) * 504.0;

        EikonalKinematicState {
            tau: t,
            s_pos,
            s_vel_dimless,
            s_acc_dimless,
            velocity_mps,
            kac_moody_char_flux_gbps,
        }
    }

    #[inline]
    pub fn adm_normal_congruence(&self) -> [f64; 4] {
        let inv_alpha = 1.0 / self.lapse_alpha;
        [
            inv_alpha,
            -self.shift_beta[0] * inv_alpha,
            -self.shift_beta[1] * inv_alpha,
            -self.shift_beta[2] * inv_alpha,
        ]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TtnMeraEngine {
    pub nucleon_count: f64,
    pub layers: usize,
    pub phase_step_rad: f64,
    pub per_layer_svd_defect: f64,
}

impl TtnMeraEngine {
    pub fn new(nucleon_count: f64) -> Self {
        let ratio = (nucleon_count / BASELINE_FRAME_NUCLEONS).max(1.0);
        let extra_layers = ratio.log2().ceil() as usize;
        let layers = BASELINE_MERA_LAYERS + extra_layers;
        let phase_step_rad = 1.0 / (layers as f64);

        Self {
            nucleon_count,
            layers,
            phase_step_rad,
            per_layer_svd_defect: 8.5e-17,
        }
    }

    pub fn verify_isometry_and_brem(&self) -> Result<(f64, f64), &'static str> {
        let l_f64 = self.layers as f64;
        let cumulative_residual =
            l_f64 * self.per_layer_svd_defect * (l_f64 * self.per_layer_svd_defect).exp();

        if cumulative_residual > MAX_ISOMETRY_RESIDUAL {
            return Err("TTN-MERA isometry residual exceeded 1.0e-14 bound");
        }

        let holonomy_defect = (l_f64 * self.phase_step_rad - 1.0).abs();
        let brem_power_watts = if holonomy_defect < 1.0e-15 {
            0.0000
        } else {
            holonomy_defect * 1.0e-12
        };

        Ok((cumulative_residual, brem_power_watts))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eikonal_kinematics_bounds() {
        let sweep = ProgressiveEikonalFoliation::new(30.3936, 1.0).unwrap();
        let peak = sweep.evaluate_kinematics(0.5);
        assert!(peak.velocity_mps <= MAX_DERENDER_VELOCITY_MPS + 1e-4);
        assert!(peak.s_acc_dimless.abs() < 1e-9);

        let extrema1 = sweep.evaluate_kinematics(0.5 - 0.5 / 3.0_f64.sqrt());
        assert!((extrema1.s_acc_dimless.abs() - MAX_DIMENSIONLESS_ACCEL).abs() < 1e-6);
    }

    #[test]
    fn test_ttn_mera_scaling_1e30() {
        let engine = TtnMeraEngine::new(1.0e30);
        assert_eq!(engine.layers, 101);
        let (residual, brem) = engine.verify_isometry_and_brem().unwrap();
        assert!(residual <= MAX_ISOMETRY_RESIDUAL);
        assert_eq!(brem, 0.0);
    }
}
