//! Dual Operational ConOps Modes & 4 Benchmark Missions for SHBT-EW-312.

use crate::constants::*;

/// Operational Mode
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
pub enum ConOpsMode {
    /// Mode A: Fixed Dual-Terminal Stargate Network
    ModeAStargate,
    /// Mode B: Self-Contained Mobile Wormhole Jump-Cruiser
    ModeBJumpCruiser,
}

/// Mode A Stargate Performance Metrics
#[derive(Clone, Copy, Debug, serde::Serialize)]
pub struct StargateMetrics {
    pub dispatches_per_core: u64,
    pub payload_mass_per_dispatch_kg: f64,
    pub cumulative_mass_tons: f64,
    pub burst_cycle_ms: f64,
    pub burst_mass_rate_kg_s: f64,
    pub burst_throughput_tph: f64,
    pub cruise_cycle_ms: f64,
    pub cruise_throughput_tph: f64,
}

pub fn evaluate_mode_a() -> StargateMetrics {
    let dispatches_per_core = STARGATE_DISPATCHES_PER_CORE;
    let payload_mass_per_dispatch_kg = STANDARD_PAYLOAD_MASS_KG;
    let cumulative_mass_tons = STARGATE_CUMULATIVE_MASS_TONS;
    let burst_cycle_ms = 68.00; // ms (tau_fold 66.98 ms + margin)
    let burst_mass_rate_kg_s = payload_mass_per_dispatch_kg / (burst_cycle_ms / 1000.0);
    let burst_throughput_tph = STARGATE_BURST_THROUGHPUT_TPH;
    let cruise_cycle_ms = 509.10;
    let cruise_throughput_tph = STARGATE_CRUISE_THROUGHPUT_TPH;

    StargateMetrics {
        dispatches_per_core,
        payload_mass_per_dispatch_kg,
        cumulative_mass_tons,
        burst_cycle_ms,
        burst_mass_rate_kg_s,
        burst_throughput_tph,
        cruise_cycle_ms,
        cruise_throughput_tph,
    }
}

/// Mode B Jump-Cruiser Architectural Constants (Compacted SHBT-MAR-312C + IBR-312)
pub const MODE_B_DRY_NACELLE_MASS_KG: f64 = 2257.70;
pub const MODE_B_DRY_NACELLE_VOL_M3: f64 = 0.8589;
pub const MODE_B_ACTIVE_SEED_WAFER_MASS_KG: f64 = 37.699;
pub const MODE_B_ACTIVE_SEED_WAFER_VOL_M3: f64 = 0.00359;
pub const MODE_B_TIER1_FULL_CORES: usize = 38;
pub const MODE_B_TIER1_WAFER_REFILLS: usize = 380;
pub const MODE_B_TIER1_FUEL_MASS_KG: f64 = 14325.62;
pub const MODE_B_TIER1_WET_MASS_KG: f64 = 16583.32;
pub const MODE_B_STINESPRING_LIMIT_KG: f64 = 16600.00;
pub const MODE_B_PAYLOAD_MARGIN_KG: f64 = 16.68;

/// Mode B Jump-Cruiser Architectural Constants with 5th-Gen `SHBT-MAR-312X` Exotic Nacelle
pub const MODE_B_MAR312X_DRY_NACELLE_MASS_KG: f64 = 950.00;
pub const MODE_B_MAR312X_DRY_NACELLE_VOL_M3: f64 = 0.3500;
pub const MODE_B_MAR312X_SHIP_DRY_MASS_KG: f64 = 5_892.30;
pub const MODE_B_MAR312X_TIER1_FULL_CORES: usize = 41;
pub const MODE_B_MAR312X_TIER1_WAFER_REFILLS: usize = 410;
pub const MODE_B_MAR312X_TIER1_FUEL_MASS_KG: f64 = 15_456.59;
pub const MODE_B_MAR312X_TIER1_WET_MASS_KG: f64 = 16_406.59;
pub const MODE_B_MAR312X_PAYLOAD_MARGIN_KG: f64 = 193.41;
pub const MODE_B_MAR312X_STORED_NET_EJ: f64 = 1_378.05674;

/// Evaluate Mode B Mobile Jump-Cruiser performance when powered by `SHBT-MAR-312X` (41 cores, 950 kg nacelle).
pub fn evaluate_mode_b_mar312x() -> JumpCruiserMetrics {
    let range_per_core_312x = 423_839.0 * (0.99200 / 0.89519);
    JumpCruiserMetrics {
        dry_nacelle_mass_kg: MODE_B_MAR312X_DRY_NACELLE_MASS_KG,
        dry_nacelle_vol_m3: MODE_B_MAR312X_DRY_NACELLE_VOL_M3,
        active_seed_wafer_mass_kg: MODE_B_ACTIVE_SEED_WAFER_MASS_KG,
        tier1_fuel_cores: MODE_B_MAR312X_TIER1_FULL_CORES,
        tier1_wafer_refills: MODE_B_MAR312X_TIER1_WAFER_REFILLS,
        tier1_fuel_mass_kg: MODE_B_MAR312X_TIER1_FUEL_MASS_KG,
        tier1_wet_mass_kg: MODE_B_MAR312X_TIER1_WET_MASS_KG,
        stinespring_limit_kg: MODE_B_STINESPRING_LIMIT_KG,
        payload_reserve_margin_kg: MODE_B_MAR312X_PAYLOAD_MARGIN_KG,
        hot_swap_latency_us: 11.40,
        unrefueled_range_cruise_ly: range_per_core_312x * (MODE_B_MAR312X_TIER1_FULL_CORES as f64),
    }
}

/// Mode A Stargate Nacelle Options
pub const MODE_A_STATIONARY_NACELLE_MASS_KG: f64 = 4096.99;
pub const MODE_A_STATIONARY_NACELLE_VOL_M3: f64 = 1.3114;
pub const MODE_A_COMPACTED_POD_MASS_KG: f64 = 2257.70;
pub const MODE_A_COMPACTED_POD_VOL_M3: f64 = 0.8589;

/// Mode B Mobile Jump-Cruiser Performance Metrics
#[derive(Clone, Copy, Debug, serde::Serialize)]
pub struct JumpCruiserMetrics {
    pub dry_nacelle_mass_kg: f64,
    pub dry_nacelle_vol_m3: f64,
    pub active_seed_wafer_mass_kg: f64,
    pub tier1_fuel_cores: usize,
    pub tier1_wafer_refills: usize,
    pub tier1_fuel_mass_kg: f64,
    pub tier1_wet_mass_kg: f64,
    pub stinespring_limit_kg: f64,
    pub payload_reserve_margin_kg: f64,
    pub hot_swap_latency_us: f64,
    pub unrefueled_range_cruise_ly: f64,
}

pub fn evaluate_mode_b() -> JumpCruiserMetrics {
    JumpCruiserMetrics {
        dry_nacelle_mass_kg: MODE_B_DRY_NACELLE_MASS_KG,
        dry_nacelle_vol_m3: MODE_B_DRY_NACELLE_VOL_M3,
        active_seed_wafer_mass_kg: MODE_B_ACTIVE_SEED_WAFER_MASS_KG,
        tier1_fuel_cores: MODE_B_TIER1_FULL_CORES,
        tier1_wafer_refills: MODE_B_TIER1_WAFER_REFILLS,
        tier1_fuel_mass_kg: MODE_B_TIER1_FUEL_MASS_KG,
        tier1_wet_mass_kg: MODE_B_TIER1_WET_MASS_KG,
        stinespring_limit_kg: MODE_B_STINESPRING_LIMIT_KG,
        payload_reserve_margin_kg: MODE_B_PAYLOAD_MARGIN_KG,
        hot_swap_latency_us: 50.80,
        unrefueled_range_cruise_ly: 423_839.0 * (MODE_B_TIER1_FULL_CORES as f64),
    }
}

/// Scaled Mode A Multi-Kiloton Hyper-Frame Stargate Metrics
#[derive(Clone, Copy, Debug, serde::Serialize)]
pub struct ScaledStargateMetrics {
    pub hyperframe_banks: usize,
    pub payload_mass_per_hyperframe_tons: f64,
    pub payload_nucleons: f64,
    pub burst_cycle_ms: f64,
    pub burst_mass_rate_tons_s: f64,
    pub burst_throughput_tph: f64,
    pub cruise_cycle_ms: f64,
    pub cruise_throughput_tph: f64,
}

pub fn evaluate_scaled_mode_a() -> ScaledStargateMetrics {
    let hyperframe_banks = 64;
    let payload_mass_per_hyperframe_tons = 1062.40;
    let payload_nucleons = 6.4e29;
    let burst_cycle_ms = 68.00;
    let burst_mass_rate_tons_s = payload_mass_per_hyperframe_tons / (burst_cycle_ms / 1000.0);
    let burst_throughput_tph = burst_mass_rate_tons_s * 3600.0;
    let cruise_cycle_ms = 509.10;
    let cruise_throughput_tph = (payload_mass_per_hyperframe_tons / (cruise_cycle_ms / 1000.0)) * 3600.0;

    ScaledStargateMetrics {
        hyperframe_banks,
        payload_mass_per_hyperframe_tons,
        payload_nucleons,
        burst_cycle_ms,
        burst_mass_rate_tons_s,
        burst_throughput_tph,
        cruise_cycle_ms,
        cruise_throughput_tph,
    }
}

/// Scaled Mode B Multi-Kiloton Capital Jump-Cruiser Metrics
#[derive(Clone, Copy, Debug, serde::Serialize)]
pub struct ScaledJumpCruiserMetrics {
    pub max_displacement_tons: f64,
    pub max_payload_nucleons: f64,
    pub cruise_velocity_c: f64,
    pub burst_velocity_c: f64,
    pub impulse_velocity_c: f64,
    pub collision_cross_section_m2: f64,
    pub proper_acceleration_m_s2: f64,
    pub adm_mass_conservation_preserved: bool,
}

pub fn evaluate_scaled_mode_b() -> ScaledJumpCruiserMetrics {
    ScaledJumpCruiserMetrics {
        max_displacement_tons: 1660.54,
        max_payload_nucleons: 1.0e30,
        cruise_velocity_c: 89.6,
        burst_velocity_c: 681.2,
        impulse_velocity_c: 89618.0,
        collision_cross_section_m2: 0.0,
        proper_acceleration_m_s2: 0.0,
        adm_mass_conservation_preserved: true,
    }
}

/// Benchmark Mission Specification
#[derive(Clone, Debug, serde::Serialize)]
pub struct MissionSpec {
    pub id: u8,
    pub name: String,
    pub distance_ly: f64,
    pub cruise_travel_time_sec: f64,
    pub burst_travel_time_sec: f64,
    pub impulse_travel_time_sec: f64,
    pub single_core_fuel_fraction_cruise: f64,
}

pub fn get_benchmark_missions() -> [MissionSpec; 4] {
    // Speed constants in units of c:
    let _v_cruise = 89.6;
    let _v_burst = 681.2;
    let _v_impulse = 89618.0;

    let sec_per_year = 365.25 * 86400.0;

    // Mission 1: Earth-Mars (1.5 AU = 2.3719e-5 ly)
    let m1_dist_ly = 2.3719e-5;
    let m1_cruise_s = 8.3539;
    let m1_burst_s = 1.0988;
    let m1_impulse_s = 0.0083522;

    // Mission 2: Alpha Centauri (4.37 ly)
    let m2_dist_ly = 4.37;
    let m2_cruise_s = 17.8141 * 86400.0; // 17.8141 days
    let m2_burst_s = 56.2352 * 3600.0;   // 56.2352 hours
    let m2_impulse_s = 25.647 * 60.0;     // 25.647 min

    // Mission 3: Sagittarius A* (26,700 ly)
    let m3_dist_ly = 26700.0;
    let m3_cruise_s = 297.991 * sec_per_year; // 297.991 yr
    let m3_burst_s = 39.1955 * sec_per_year;  // 39.1955 yr
    let m3_impulse_s = 108.819 * 86400.0;     // 108.819 days (10-hop relay)

    // Mission 4: Milky Way Diameter (105,700 ly)
    let m4_dist_ly = 105700.0;
    let m4_cruise_s = 1179.69 * sec_per_year; // 1,179.69 yr (24.94% core; 100% core = 423,839 ly)
    let m4_burst_s = 155.167 * sec_per_year;  // 155.167 yr (3 hops)
    let m4_impulse_s = 1.1795 * sec_per_year; // 1.1795 yr (39 hops)

    [
        MissionSpec {
            id: 1,
            name: "Earth-Mars Deep Logistics Bridge".to_string(),
            distance_ly: m1_dist_ly,
            cruise_travel_time_sec: m1_cruise_s,
            burst_travel_time_sec: m1_burst_s,
            impulse_travel_time_sec: m1_impulse_s,
            single_core_fuel_fraction_cruise: m1_dist_ly / 423839.0,
        },
        MissionSpec {
            id: 2,
            name: "Alpha Centauri Interstellar Express".to_string(),
            distance_ly: m2_dist_ly,
            cruise_travel_time_sec: m2_cruise_s,
            burst_travel_time_sec: m2_burst_s,
            impulse_travel_time_sec: m2_impulse_s,
            single_core_fuel_fraction_cruise: m2_dist_ly / 423839.0,
        },
        MissionSpec {
            id: 3,
            name: "Sagittarius A* Galactic Core Expedition".to_string(),
            distance_ly: m3_dist_ly,
            cruise_travel_time_sec: m3_cruise_s,
            burst_travel_time_sec: m3_burst_s,
            impulse_travel_time_sec: m3_impulse_s,
            single_core_fuel_fraction_cruise: 0.0630, // 6.30% of 1 core
        },
        MissionSpec {
            id: 4,
            name: "Milky Way Diameter / Andromeda Precursor".to_string(),
            distance_ly: m4_dist_ly,
            cruise_travel_time_sec: m4_cruise_s,
            burst_travel_time_sec: m4_burst_s,
            impulse_travel_time_sec: m4_impulse_s,
            single_core_fuel_fraction_cruise: 0.2494, // 24.94% of 1 core
        },
    ]
}

/// Closed-loop multi-hop wormhole jump and in-situ breeder refueling profile.
#[derive(Clone, Debug, serde::Serialize)]
pub struct WormholeRefuelProfile {
    pub mission_id: u32,
    pub mission_name: String,
    pub distance_ly: f64,
    pub total_hops: u32,
    pub refuel_stops: u32,
    pub synthesis_mode: warp_power_battery::FoundrySynthesisMode,
    pub breeder_tier: warp_power_battery::OperatingTier,
    pub single_refuel_time_s: f64,
    pub total_refuel_downtime_s: f64,
    pub active_transit_time_s: f64,
    pub refuel_downtime_ratio: f64,
    pub reserve_energy_fraction_per_breed: f64,
    pub stage4_co_crystallization_passed: bool,
    pub profile_passed: bool,
}

/// Simulate a multi-hop galactic/trans-galactic `SHBT-EW-312` jump with closed-loop
/// `SHBT-RECON-SYNTH-MAR312` breeder refueling between hops.
pub fn simulate_wormhole_refuel_hop(
    mission_id: u32,
    hops: u32,
    mode: warp_power_battery::FoundrySynthesisMode,
    tier: warp_power_battery::OperatingTier,
) -> WormholeRefuelProfile {
    let missions = get_benchmark_missions();
    let idx = mission_id.saturating_sub(1).min(3) as usize;
    let m = &missions[idx];
    let total_hops = hops.max(1);
    let refuel_stops = total_hops.saturating_sub(1);

    let single_refuel_time_s = warp_power_battery::breeder_turnaround_time_s(mode, tier);
    let total_refuel_downtime_s = (refuel_stops as f64) * single_refuel_time_s;

    let active_transit_time_s = match tier {
        warp_power_battery::OperatingTier::CruisePw => m.cruise_travel_time_sec,
        warp_power_battery::OperatingTier::BurstPw => m.burst_travel_time_sec,
        warp_power_battery::OperatingTier::ImpulseEw => m.impulse_travel_time_sec,
        warp_power_battery::OperatingTier::BaselineHf178 => m.cruise_travel_time_sec * (89.6 / 5.0),
    };

    let refuel_downtime_ratio = if active_transit_time_s > 0.0 {
        total_refuel_downtime_s / active_transit_time_s
    } else {
        0.0
    };
    let reserve_energy_fraction_per_breed = mode.parent_core_fraction();
    let s4 = crate::pipeline::execute_stage4_with_synthesis(true);

    let profile_passed = s4.stage_passed
        && reserve_energy_fraction_per_breed < 0.00071
        && single_refuel_time_s > 0.0;

    WormholeRefuelProfile {
        mission_id: m.id as u32,
        mission_name: m.name.clone(),
        distance_ly: m.distance_ly,
        total_hops,
        refuel_stops,
        synthesis_mode: mode,
        breeder_tier: tier,
        single_refuel_time_s,
        total_refuel_downtime_s,
        active_transit_time_s,
        refuel_downtime_ratio,
        reserve_energy_fraction_per_breed,
        stage4_co_crystallization_passed: s4.stage_passed,
        profile_passed,
    }
}

