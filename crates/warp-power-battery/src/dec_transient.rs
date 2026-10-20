//! Relativistic Child-Langmuir Space-Charge Headroom and REBCO Quench Margin Monitor.

pub const NOMINAL_BEAM_CURRENT_DENSITY: f64 = 2.0120e14; // A/m^2
pub const EFFECTIVE_CL_SATURATION_DENSITY: f64 = 2.4500e14; // A/m^2
pub const NOMINAL_REBCO_OPERATING_CURRENT: f64 = 1.4050e10; // A/m^2
pub const REBCO_CRITICAL_CURRENT_20T: f64 = 1.6850e10; // A/m^2

pub struct DecTransientMonitor;

impl DecTransientMonitor {
    pub fn calculate_cl_headroom() -> f64 {
        (EFFECTIVE_CL_SATURATION_DENSITY - NOMINAL_BEAM_CURRENT_DENSITY) / EFFECTIVE_CL_SATURATION_DENSITY
    }

    pub fn calculate_rebco_quench_headroom() -> f64 {
        (REBCO_CRITICAL_CURRENT_20T - NOMINAL_REBCO_OPERATING_CURRENT) / REBCO_CRITICAL_CURRENT_20T
    }

    pub fn verify_pulse_stability() -> bool {
        let cl_headroom = Self::calculate_cl_headroom();
        let quench_headroom = Self::calculate_rebco_quench_headroom();
        cl_headroom >= 0.1500 && quench_headroom >= 0.1500
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dec_transient_stability() {
        assert!(DecTransientMonitor::verify_pulse_stability());
        let cl = DecTransientMonitor::calculate_cl_headroom();
        assert!((cl - 0.1787755).abs() < 1e-4);
        let quench = DecTransientMonitor::calculate_rebco_quench_headroom();
        assert!((quench - 0.166172).abs() < 1e-4);
    }
}
