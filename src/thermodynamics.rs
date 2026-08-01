//! Thermodynamic rate engine for entropy-debt integration.

use crate::boundary::{BoundaryRegister, ExcitationEngine};
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

    pub fn audit(&self, power_mw: f64, delta_mod: f64, radius_m: f64) -> HashMap<String, f64> {
        let steady = self.steady_state_entropy_debt(power_mw);
        let expected = delta_mod.abs() * (DEFAULT_BUBBLE_RADIUS_M / radius_m).powi(2);
        let hold_time = self.maximum_hold_time(0.0);
        let initial_rate = self.entropy_debt_rate(0.0, power_mw);
        let passed = (steady - expected).abs() <= 1.0e-12 * delta_mod.abs() && hold_time.is_infinite();
        let mut map = HashMap::new();
        map.insert("passed".to_string(), if passed { 1.0 } else { 0.0 });
        map.insert("steady_state_entropy_debt".to_string(), steady);
        map.insert("initial_accumulation_rate_s_inv".to_string(), initial_rate);
        map.insert("maximum_hold_time_s".to_string(), hold_time);
        map.insert("framing_defect_breached".to_string(), 0.0);
        map
    }
}

/// Time-stepping transient engine for start-up boundary excitation and entropy-debt accumulation.
pub struct TransientRateEngine {
    pub dt: f64,
    pub total_time: f64,
    pub velocity_m_per_locktime: f64,
    pub lock_rate_s_inv: f64,
    pub relaxation_rate_s_inv: f64,
    pub kappa_per_joule: f64,
}

impl TransientRateEngine {
    pub fn new(dt: f64, total_time: f64, velocity_m_per_locktime: f64) -> Self {
        assert!(dt > 0.0, "dt must be positive");
        assert!(total_time >= 0.0, "total_time must be nonnegative");
        TransientRateEngine {
            dt,
            total_time,
            velocity_m_per_locktime,
            lock_rate_s_inv: HOLOGRAPHIC_LOCK_RATE_S_INV,
            relaxation_rate_s_inv: THERMODYNAMIC_RELAXATION_RATE_S_INV,
            kappa_per_joule: thermodynamic_kappa_per_j(),
        }
    }

    /// Phase-lock angle as a function of dimensionless lock time.
    pub fn theta_at(&self, theta_phase: f64, tau: f64) -> f64 {
        theta_phase * (1.0 - (-self.lock_rate_s_inv * tau / self.lock_rate_s_inv).exp())
    }

    /// Dimensionless time in units of the holographic lock time.
    pub fn lock_time_s(&self) -> f64 {
        1.0 / self.lock_rate_s_inv
    }

    /// RK4 integration of `dΔ/dτ = τ_lock (κ P - λ Δ)` in dimensionless lock-time units.
    pub fn entropy_debt_trajectory(&self, power_mw: f64, initial: f64) -> Vec<f64> {
        let tau_lock = self.lock_time_s();
        let n_steps = (self.total_time / self.dt).ceil() as usize;
        let mut debt = vec![initial; n_steps + 1];
        for i in 0..n_steps {
            let _tau = i as f64 * self.dt;
            let k1 = self.debt_rate(debt[i], power_mw, tau_lock);
            let k2 = self.debt_rate(debt[i] + 0.5 * self.dt * k1, power_mw, tau_lock);
            let k3 = self.debt_rate(debt[i] + 0.5 * self.dt * k2, power_mw, tau_lock);
            let k4 = self.debt_rate(debt[i] + self.dt * k3, power_mw, tau_lock);
            debt[i + 1] = debt[i] + (self.dt / 6.0) * (k1 + 2.0 * k2 + 2.0 * k3 + k4);
        }
        debt
    }

    fn debt_rate(&self, debt: f64, power_mw: f64, tau_lock: f64) -> f64 {
        tau_lock * (self.kappa_per_joule * power_mw * 1.0e6 - self.relaxation_rate_s_inv * debt)
    }

    /// Run the full transient for a given boundary/excitation and vessel radius.
    pub fn run(
        &self,
        _boundary: &BoundaryRegister,
        engine: &ExcitationEngine,
        _radius_m: f64,
        power_mw: f64,
    ) -> HashMap<String, Vec<f64>> {
        let debt = self.entropy_debt_trajectory(power_mw, 0.0);
        let n_steps = debt.len();
        let mut time_s = Vec::with_capacity(n_steps);
        let mut lock_times = Vec::with_capacity(n_steps);
        let mut theta_t = Vec::with_capacity(n_steps);
        let mut population_shift = Vec::with_capacity(n_steps);
        let mut power = Vec::with_capacity(n_steps);
        let tau_lock = self.lock_time_s();

        for i in 0..n_steps {
            let tau = i as f64 * self.dt;
            time_s.push(tau * tau_lock);
            lock_times.push(tau);
            let theta = self.theta_at(engine.theta_phase, tau);
            theta_t.push(theta);
            population_shift.push(engine.population_shift_at(theta));
            power.push(power_mw);
        }

        let mut map = HashMap::new();
        map.insert("time_s".to_string(), time_s);
        map.insert("lock_times".to_string(), lock_times);
        map.insert("theta_t".to_string(), theta_t);
        map.insert("population_shift".to_string(), population_shift);
        map.insert("power_mw".to_string(), power);
        map.insert("entropy_debt".to_string(), debt);
        map
    }
}
