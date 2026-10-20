//! Spatiotemporal Maxwell-Bloch Eikonal Wavefront Controller and Bremsstrahlung Suppression.

use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

pub const SCREENED_KOHN_SHAM_SHIFT_EV: f64 = 352.4800;
pub const CRITICAL_PHASE_JITTER_RAD: f64 = 1.8288e-6;
pub const CRITICAL_TIMING_JITTER_AS: u32 = 4; // Attoseconds
pub const NOMINAL_PUMP_FREQ_HZ: f64 = 72.0e9;
pub const MAX_BREM_POWER_WATTS: f64 = 1.0e-6;

#[repr(C, align(64))]
pub struct WavefrontController {
    pub current_tau_q32: AtomicU32,
    pub measured_jitter_as: AtomicU32,
    pub calculated_brem_power_nw: AtomicU64,
    pub opll_status_flags: AtomicU32,
}

impl WavefrontController {
    pub const fn new() -> Self {
        Self {
            current_tau_q32: AtomicU32::new(0),
            measured_jitter_as: AtomicU32::new(0),
            calculated_brem_power_nw: AtomicU64::new(0),
            opll_status_flags: AtomicU32::new(1),
        }
    }

    #[inline(always)]
    pub fn evaluate_eikonal_progress(&self, tau: f64) -> f64 {
        let t = tau.clamp(0.0, 1.0);
        t * t * t * (10.0 + t * (-15.0 + 6.0 * t))
    }

    #[inline(always)]
    pub fn verify_bremsstrahlung_safety(&self, jitter_as: u32) -> bool {
        self.measured_jitter_as.store(jitter_as, Ordering::Release);
        let jitter_sec = (jitter_as as f64) * 1e-18;
        let omega = 2.0 * std::f64::consts::PI * NOMINAL_PUMP_FREQ_HZ;
        let sigma_phi = omega * jitter_sec;
        let p_incoherent = 299000.0; // 299 kW
        let p_brem = p_incoherent * (sigma_phi * sigma_phi);
        let p_nw = (p_brem * 1e9) as u64;
        self.calculated_brem_power_nw.store(p_nw, Ordering::Release);
        jitter_as <= CRITICAL_TIMING_JITTER_AS && p_brem <= MAX_BREM_POWER_WATTS
    }
}

impl Default for WavefrontController {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wavefront_controller_invariants() {
        let c = WavefrontController::new();
        assert_eq!(c.evaluate_eikonal_progress(0.0), 0.0);
        assert_eq!(c.evaluate_eikonal_progress(1.0), 1.0);
        assert_eq!(c.evaluate_eikonal_progress(0.5), 0.5);
        assert!(c.verify_bremsstrahlung_safety(4));
        assert!(!c.verify_bremsstrahlung_safety(5));
    }
}
