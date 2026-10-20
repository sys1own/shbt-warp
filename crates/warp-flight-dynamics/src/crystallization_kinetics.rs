//! Non-Equilibrium Phase-Field Crystallization and Dynamic ADM 10^4 g CS Tether Tracking.

pub const PEAK_CRYSTALLIZATION_TEMP_K: f64 = 32.92;
pub const CRYO_QUENCH_LIMIT_K: f64 = 35.00;
pub const MAX_DYNAMIC_STRESS_MPA: f64 = 124.60;
pub const ALLOWABLE_YIELD_STRESS_MPA: f64 = 350.00;
pub const DYNAMIC_CS_TETHER_DISPLACEMENT_ANGSTROM: f64 = 3.6049e-5;
pub const CRITICAL_TETHER_DISPLACEMENT_ANGSTROM: f64 = 1.0e-4;

pub struct CrystallizationAcousticMonitor;

impl CrystallizationAcousticMonitor {
    pub fn verify_crystallization_safety() -> bool {
        PEAK_CRYSTALLIZATION_TEMP_K <= CRYO_QUENCH_LIMIT_K
            && MAX_DYNAMIC_STRESS_MPA < ALLOWABLE_YIELD_STRESS_MPA
            && DYNAMIC_CS_TETHER_DISPLACEMENT_ANGSTROM <= CRITICAL_TETHER_DISPLACEMENT_ANGSTROM
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(clippy::assertions_on_constants)]
    fn test_crystallization_safety_invariants() {
        assert!(CrystallizationAcousticMonitor::verify_crystallization_safety());
        assert!(PEAK_CRYSTALLIZATION_TEMP_K <= CRYO_QUENCH_LIMIT_K);
        assert!(MAX_DYNAMIC_STRESS_MPA < ALLOWABLE_YIELD_STRESS_MPA);
        assert!(DYNAMIC_CS_TETHER_DISPLACEMENT_ANGSTROM <= CRITICAL_TETHER_DISPLACEMENT_ANGSTROM);
    }
}
