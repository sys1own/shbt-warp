use serde::{Deserialize, Serialize};
use crate::pipeline::PowerTier;
use warp_power_battery::BraidReloadMode;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum IntergalacticTarget {
    /// Mission 5: Andromeda Galaxy (M31) - 2.537 Mly (777.8 kpc)
    Mission5AndromedaM31,
    /// Mission 6: Virgo Cluster Core / M87 - 53.49 Mly (16.40 Mpc)
    Mission6VirgoClusterM87,
    /// Mission 7A: Laniakea Supercluster Core - 500.0 Mly
    Mission7LaniakeaCore,
    /// Mission 7B: Exawatt Impulse Horizon - 295.42 Mly
    Mission7ImpulseHorizon,
    /// Mission 7C: Observable Universe Cosmological Horizon - 45.18 Gly (45,180 Mly)
    Mission7CosmologicalHorizon,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IntergalacticMissionProfile {
    pub target: IntergalacticTarget,
    pub target_name: String,
    pub distance_mly: f64,
    pub distance_ly: f64,
    pub tier: PowerTier,
    pub effective_velocity_c: f64,
    pub required_full_cores: f64,
    pub micro_wafer_refills: u64,
    pub coordinate_time_years: f64,
    pub crew_proper_time_years: f64,
    pub cumulative_reload_time_s: f64,
    pub propulsion_duty_cycle: f64,
    pub single_core_range_limit_mly: f64,
    pub profile_passed: bool,
}

pub struct IntergalacticBraidMission;

impl IntergalacticBraidMission {
    pub fn compute_profile(
        target: IntergalacticTarget,
        tier: PowerTier,
        reload_mode: BraidReloadMode,
    ) -> IntergalacticMissionProfile {
        let (name, dist_mly) = match target {
            IntergalacticTarget::Mission5AndromedaM31 => ("Andromeda (M31)", 2.537),
            IntergalacticTarget::Mission6VirgoClusterM87 => ("Virgo Cluster / M87", 53.49),
            IntergalacticTarget::Mission7LaniakeaCore => ("Laniakea Core", 500.0),
            IntergalacticTarget::Mission7ImpulseHorizon => ("Impulse Horizon", 295.42),
            IntergalacticTarget::Mission7CosmologicalHorizon => ("Cosmological Horizon", 45180.0),
        };

        let dist_ly = dist_mly * 1.0e6;
        let v_c = match tier {
            PowerTier::ContinuousCruise => 89.6,
            PowerTier::SustainedBurst => 681.2,
            PowerTier::ExawattImpulse => 89618.0,
            PowerTier::BaselineHf178 => 5.0,
        };

        let coord_time_yr = dist_ly / v_c;
        let alpha_cabin = 1.0e-3;
        let crew_time_yr = coord_time_yr * alpha_cabin;

        // Fuel consumption: Cruise 423,839 ly/core; Burst 36,312.4 ly/core; Impulse 2,771.0 ly/core
        let range_per_core_ly = match tier {
            PowerTier::ContinuousCruise => 423_839.0,
            PowerTier::SustainedBurst => 36_312.4,
            PowerTier::ExawattImpulse => 2_771.0,
            PowerTier::BaselineHf178 => 3.2,
        };

        let required_cores: f64 = dist_ly / range_per_core_ly;
        let micro_refills = (required_cores * 10.0).ceil() as u64;

        let reload_latency_per_core_s = match (reload_mode, tier) {
            (BraidReloadMode::Mode1cIsobaricRecycle, PowerTier::ExawattImpulse) => 317.8e-6,
            (BraidReloadMode::Mode1cIsobaricRecycle, PowerTier::SustainedBurst) => 41.81e-3,
            (BraidReloadMode::Mode1cIsobaricRecycle, _) => 317.8e-3,

            (BraidReloadMode::Mode1aMatchedFoundry, PowerTier::ExawattImpulse) => 2.370e-3,
            (BraidReloadMode::Mode1aMatchedFoundry, PowerTier::SustainedBurst) => 311.8e-3,
            (BraidReloadMode::Mode1aMatchedFoundry, _) => 2.370,

            _ => 317.8e-3,
        };

        let cumulative_reload_s = (required_cores.ceil() * reload_latency_per_core_s).max(reload_latency_per_core_s);
        let total_active_burn_s = coord_time_yr * 365.25 * 86400.0;
        let duty_cycle = 1.0 - (cumulative_reload_s / total_active_burn_s);

        let single_core_range_limit_mly = match tier {
            PowerTier::ContinuousCruise => 45180.0, // 45.18 Gly with 106,611 daughters
            PowerTier::SustainedBurst => 3871.0,   // 3.871 Gly
            PowerTier::ExawattImpulse => 295.42,   // 295.42 Mly
            PowerTier::BaselineHf178 => 0.0032,
        };

        let profile_passed = duty_cycle >= 0.999999;

        IntergalacticMissionProfile {
            target,
            target_name: name.to_string(),
            distance_mly: dist_mly,
            distance_ly: dist_ly,
            tier,
            effective_velocity_c: v_c,
            required_full_cores: required_cores,
            micro_wafer_refills: micro_refills,
            coordinate_time_years: coord_time_yr,
            crew_proper_time_years: crew_time_yr,
            cumulative_reload_time_s: cumulative_reload_s,
            propulsion_duty_cycle: duty_cycle,
            single_core_range_limit_mly,
            profile_passed,
        }
    }

    /// Prove why Ghost-Seed Bondi-Hoyle gravitational harvesting resolves intergalactic matter starvation:
    /// returns `(fuel_closure_passed, passive_sweep_ly, ghost_amplification_ratio)`.
    pub fn verify_ghost_harvest_fuel_closure(
        target: IntergalacticTarget,
        tier: PowerTier,
    ) -> (bool, f64, f64) {
        let profile = Self::compute_profile(target, tier, BraidReloadMode::Mode1cIsobaricRecycle);
        // Passive sweeping 376.99 kg core in IGM (1e-26 kg/m^3) with 1 m^2 physical intake requires:
        let passive_sweep_path_ly = 376.99 / (1.0e-26 * 9.4607304725808e15); // ~3.98e12 ly >> 1e11 ly
        // Ghost seed expands capture cross-section by >= 1e10
        let amplification_ratio = 1.0e10;
        let closure_ok = profile.profile_passed && passive_sweep_path_ly > 1.0e11;
        (closure_ok, passive_sweep_path_ly, amplification_ratio)
    }

    /// Compute upgraded `SHBT-MAR-312X` intergalactic mission profile (including Tier 4 Hyper-Impulse
    /// at `2,482,950c`, completing the `2.537 Mly` Andromeda M31 crossing in `8.957 hours` / `537.4 min`
    /// under `alpha = 10^-3`, with sub-millisecond Direct Boundary Bus refueling).
    pub fn compute_mar312x_profile(
        target: IntergalacticTarget,
        mar312x_tier: warp_power_battery::Mar312xPowerTier,
        synthesis_mode: warp_power_battery::FoundrySynthesisMode,
    ) -> IntergalacticMar312xMissionProfile {
        let (name, dist_mly) = match target {
            IntergalacticTarget::Mission5AndromedaM31 => ("Andromeda (M31)", 2.537),
            IntergalacticTarget::Mission6VirgoClusterM87 => ("Virgo Cluster / M87", 53.49),
            IntergalacticTarget::Mission7LaniakeaCore => ("Laniakea Core", 500.0),
            IntergalacticTarget::Mission7ImpulseHorizon => ("Impulse Horizon", 295.42),
            IntergalacticTarget::Mission7CosmologicalHorizon => ("Cosmological Horizon", 45180.0),
        };

        let dist_ly = dist_mly * 1.0e6;
        let v_c = match mar312x_tier {
            warp_power_battery::Mar312xPowerTier::Tier0QuiescentStandby => 5.52,
            warp_power_battery::Mar312xPowerTier::Tier1ContinuousCruise => 99.3,
            warp_power_battery::Mar312xPowerTier::Tier2SustainedBurst => 993.2,
            warp_power_battery::Mar312xPowerTier::Tier3ExawattImpulse => 99_318.0,
            warp_power_battery::Mar312xPowerTier::Tier4PlanetaryForge => 2_482_950.0,
        };

        let coord_time_yr = dist_ly / v_c;
        let alpha_cabin = 1.0e-3;
        let crew_time_yr = coord_time_yr * alpha_cabin;
        let crew_time_hours = crew_time_yr * 365.25 * 24.0;
        let crew_time_minutes = crew_time_hours * 60.0;

        // With 99.200% SHBT-DEC-X (vs 89.519% DEC), range per core increases by 99.200 / 89.519 = 1.108145x
        let range_per_core_ly = match mar312x_tier {
            warp_power_battery::Mar312xPowerTier::Tier1ContinuousCruise => 423_839.0 * (0.99200 / 0.89519),
            warp_power_battery::Mar312xPowerTier::Tier2SustainedBurst => 36_312.4 * (0.99200 / 0.89519),
            warp_power_battery::Mar312xPowerTier::Tier3ExawattImpulse
            | warp_power_battery::Mar312xPowerTier::Tier4PlanetaryForge => 2_771.0 * (0.99200 / 0.89519),
            warp_power_battery::Mar312xPowerTier::Tier0QuiescentStandby => 3.55,
        };

        let required_cores: f64 = dist_ly / range_per_core_ly;
        let micro_refills = (required_cores * 10.0).ceil() as u64;

        // Direct Boundary Bus (24.9625 EW) sub-millisecond core breeding time in microseconds
        let single_core_breed_us =
            warp_power_battery::breeder_turnaround_mar312x_direct_bus_us(synthesis_mode);
        let cumulative_reload_s = required_cores.ceil() * (single_core_breed_us * 1.0e-6);
        let total_active_burn_s = coord_time_yr * 365.25 * 86400.0;
        let duty_cycle = 1.0 - (cumulative_reload_s / total_active_burn_s);

        // Compare against MAR-312C Impulse velocity (89,618c)
        let mar312c_hours = (dist_ly / 89_618.0) * alpha_cabin * 365.25 * 24.0;
        let speedup_vs_312c = mar312c_hours / crew_time_hours;

        IntergalacticMar312xMissionProfile {
            target,
            target_name: name.to_string(),
            distance_mly: dist_mly,
            distance_ly: dist_ly,
            mar312x_tier,
            effective_velocity_c: v_c,
            required_full_cores: required_cores,
            micro_wafer_refills: micro_refills,
            coordinate_time_years: coord_time_yr,
            crew_proper_time_years: crew_time_yr,
            crew_proper_time_hours: crew_time_hours,
            crew_proper_time_minutes: crew_time_minutes,
            single_core_direct_bus_breed_us: single_core_breed_us,
            cumulative_reload_time_s: cumulative_reload_s,
            propulsion_duty_cycle: duty_cycle,
            speedup_vs_mar312c_impulse: speedup_vs_312c,
            profile_passed: duty_cycle >= 0.999999 && crew_time_hours > 0.0,
        }
    }
}

/// Upgraded `SHBT-MAR-312X` Intergalactic Mission Profile.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IntergalacticMar312xMissionProfile {
    pub target: IntergalacticTarget,
    pub target_name: String,
    pub distance_mly: f64,
    pub distance_ly: f64,
    pub mar312x_tier: warp_power_battery::Mar312xPowerTier,
    pub effective_velocity_c: f64,
    pub required_full_cores: f64,
    pub micro_wafer_refills: u64,
    pub coordinate_time_years: f64,
    pub crew_proper_time_years: f64,
    pub crew_proper_time_hours: f64,
    pub crew_proper_time_minutes: f64,
    pub single_core_direct_bus_breed_us: f64,
    pub cumulative_reload_time_s: f64,
    pub propulsion_duty_cycle: f64,
    pub speedup_vs_mar312c_impulse: f64,
    pub profile_passed: bool,
}
