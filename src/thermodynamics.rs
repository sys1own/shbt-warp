//! Thermodynamic rate engine for entropy-debt integration.

use crate::constants::*;
use std::collections::HashMap;

pub struct ThermodynamicRateEngine {
    pub kappa_per_joule: f64,
    pub relaxation_rate_s_inv: f64,
    pub canonical_framing_defect: f64,
    pub branch_preserving: bool,
}

impl ThermodynamicRateEngine {
    pub fn new() -> Self {
        ThermodynamicRateEngine {
            kappa_per_joule: thermodynamic_kappa_per_j(),
            relaxation_rate_s_inv: THERMODYNAMIC_RELAXATION_RATE_S_INV,
            canonical_framing_defect: 0.0,
            branch_preserving: true,
        }
    }

    pub fn dissipation_rate(&self, entropy_debt: f64) -> f64 {
        self.relaxation_rate_s_inv * entropy_debt
    }

    pub fn entropy_debt_rate(&self, entropy_debt: f64, power_mw: f64) -> f64 {
        assert!(power_mw >= 0.0, "power_mw must be nonnegative");
        self.kappa_per_joule * power_mw * 1.0e6 - self.dissipation_rate(entropy_debt)
    }

    pub fn steady_state_entropy_debt(&self, power_mw: f64) -> f64 {
        assert!(self.relaxation_rate_s_inv > 0.0, "A positive relaxation rate is required");
        self.kappa_per_joule * power_mw * 1.0e6 / self.relaxation_rate_s_inv
    }

    pub fn integrate(&self, time_s: &[f64], initial: f64, power_mw: f64) -> Vec<f64> {
        let steady = self.steady_state_entropy_debt(power_mw);
        time_s
            .iter()
            .map(|&t| {
                assert!(t >= 0.0, "time_s must be nonnegative");
                steady + (initial - steady) * (-self.relaxation_rate_s_inv * t).exp()
            })
            .collect()
    }

    pub fn maximum_hold_time(&self, framing_threshold: f64) -> f64 {
        assert!(framing_threshold >= 0.0, "framing_threshold must be nonnegative");
        if self.branch_preserving {
            f64::INFINITY
        } else if self.canonical_framing_defect > framing_threshold {
            0.0
        } else {
            panic!("A branch-mutation law is required for finite hold time");
        }
    }

    pub fn audit(&self) -> HashMap<String, f64> {
        let steady = self.steady_state_entropy_debt(POWER_BENCHMARK_MW);
        let hold_time = self.maximum_hold_time(0.0);
        let initial_rate = self.entropy_debt_rate(0.0, POWER_BENCHMARK_MW);
        let passed = (steady - DELTA_MOD).abs() <= 1.0e-12 * DELTA_MOD.abs() && hold_time.is_infinite();
        let mut map = HashMap::new();
        map.insert("passed".to_string(), if passed { 1.0 } else { 0.0 });
        map.insert("steady_state_entropy_debt".to_string(), steady);
        map.insert("initial_accumulation_rate_s_inv".to_string(), initial_rate);
        map.insert("maximum_hold_time_s".to_string(), hold_time);
        map.insert("framing_defect_breached".to_string(), 0.0);
        map
    }
}
