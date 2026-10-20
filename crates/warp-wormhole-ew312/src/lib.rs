//! SHBT-EW-312: Effective Traversable Wormhole System
//! (Alcubierre-Stinespring Superluminal Topological Corridor).

pub mod constants;
pub mod classical_nogo;
pub mod pipeline;
pub mod conops;
pub mod causal_2pn;
pub mod intergalactic;
pub mod gst_metamaterial;
pub mod corridor;
pub mod hyperframe;
pub mod derender;
pub mod eikonal;

pub use constants::*;
pub use classical_nogo::{verify_classical_bulk_nogo, verify_effective_corridor, ClassicalWormholeAudit, EffectiveCorridorAudit};
pub use pipeline::{execute_stage4_with_synthesis, execute_wormhole_pipeline, PowerTier, WormholePipelineResult};
pub use conops::{
    evaluate_mode_a, evaluate_mode_b, evaluate_scaled_mode_a, evaluate_scaled_mode_b,
    get_benchmark_missions, simulate_wormhole_refuel_hop, ConOpsMode, JumpCruiserMetrics,
    MissionSpec, ScaledJumpCruiserMetrics, ScaledStargateMetrics, StargateMetrics,
    WormholeRefuelProfile,
};
pub use causal_2pn::{evaluate_2pn_metric, interval_2pn, interval_2pn_with_shift, is_causally_authorized, Metric2pn};
pub use intergalactic::*;
pub use gst_metamaterial::*;
pub use corridor::*;
pub use derender::{
    evaluate_microscopic_derender, AdaptiveTtnMeraEngine, MicroscopicDeRenderReport,
    BENCHMARK_SCREENED_PAIR_EV, BREMSSTRAHLUNG_POWER_W, CRYO_QUENCH_LIMIT_K,
    CRYOGENIC_OP_TEMP_K, DCE_PUMP_FREQ_GHZ, EFFECTIVE_SCREENED_PAIR_EV,
    ISOMETRY_MAX_RESIDUAL, MOSSBAUER_MIN_FRACTION, MOSSBAUER_RECOILLESS_FRACTION_FM,
    SCREENING_DISTANCE_ANGSTROM, SCREENING_SCALE_FACTOR, TMSV_CARRIER_FREQ_GHZ,
    TMSV_POSITION_SQUEEZING_NM,
};
pub use hyperframe::*;

#[cfg(test)]
mod tests {
    use super::*;
    use warp_power_battery::BraidReloadMode;

    #[test]
    fn test_classical_nogo_and_corridor() {
        let nogo = verify_classical_bulk_nogo();
        assert!(nogo.classical_handle_rejected);

        let corridor = verify_effective_corridor();
        assert!(corridor.corridor_authorized);
        assert_eq!(corridor.metric_rank, 3);
        assert_eq!(corridor.lapse_alpha, 1.0);
    }

    #[test]
    fn test_4stage_pipeline_across_tiers() {
        let res_cruise = execute_wormhole_pipeline(1.0e28, PowerTier::ContinuousCruise, 89.6);
        assert!(res_cruise.all_passed);
        assert_eq!(res_cruise.stage1.folding_latency_ms, TAU_FOLD_CRUISE_MS);
        assert_eq!(res_cruise.stage2.anyon_braid_channels, 124);
        assert_eq!(res_cruise.stage3.proper_accel_m_s2, 0.0);
        assert_eq!(res_cruise.stage4.egress_entropy_delta, 0.0);

        let res_burst = execute_wormhole_pipeline(1.0e28, PowerTier::SustainedBurst, 681.2);
        assert!(res_burst.all_passed);
        assert_eq!(res_burst.stage1.folding_latency_ms, TAU_FOLD_BURST_MS);

        let res_impulse = execute_wormhole_pipeline(1.0e28, PowerTier::ExawattImpulse, 89618.0);
        assert!(res_impulse.all_passed);
        assert_eq!(res_impulse.stage1.folding_latency_ms, TAU_FOLD_IMPULSE_MS);
    }

    #[test]
    fn test_mode_a_stargate_metrics() {
        let sg = evaluate_mode_a();
        assert_eq!(sg.dispatches_per_core, STARGATE_DISPATCHES_PER_CORE);
        assert_eq!(sg.burst_throughput_tph, STARGATE_BURST_THROUGHPUT_TPH);
        assert_eq!(sg.cruise_throughput_tph, STARGATE_CRUISE_THROUGHPUT_TPH);
    }

    #[test]
    fn test_mode_b_jump_cruiser_metrics() {
        let jc = evaluate_mode_b();
        assert_eq!(jc.dry_nacelle_mass_kg, 2257.70);
        assert_eq!(jc.dry_nacelle_vol_m3, 0.8589);
        assert_eq!(jc.active_seed_wafer_mass_kg, 37.699);
        assert_eq!(jc.tier1_fuel_cores, 38);
        assert_eq!(jc.tier1_wafer_refills, 380);
        assert_eq!(jc.tier1_wet_mass_kg, 16583.32);
        assert!(jc.tier1_wet_mass_kg <= jc.stinespring_limit_kg);
        assert!((jc.payload_reserve_margin_kg - 16.68).abs() < 1e-4);
        assert_eq!(jc.hot_swap_latency_us, 50.80);
    }

    #[test]
    fn test_benchmark_missions() {
        let missions = get_benchmark_missions();
        assert_eq!(missions.len(), 4);
        assert_eq!(missions[0].id, 1);
        assert_eq!(missions[1].id, 2);
        assert_eq!(missions[2].id, 3);
        assert_eq!(missions[3].id, 4);
        assert!(missions[0].burst_travel_time_sec < 1.10);
    }

    #[test]
    fn test_2pn_causality() {
        let r = [1.496e11, 0.0, 0.0];
        assert!(is_causally_authorized(1.0, [1.0e5, 0.0, 0.0], r));
        assert!(!is_causally_authorized(1.0e-5, [1.0e9, 0.0, 0.0], r));
    }

    #[test]
    fn test_intergalactic_missions() {
        let m5_cruise = IntergalacticBraidMission::compute_profile(
            IntergalacticTarget::Mission5AndromedaM31,
            PowerTier::ContinuousCruise,
            BraidReloadMode::Mode1cIsobaricRecycle,
        );
        assert!(m5_cruise.profile_passed);
        assert!(m5_cruise.required_full_cores < 6.5);
        assert!(m5_cruise.propulsion_duty_cycle > 0.999999);

        let m5_imp = IntergalacticBraidMission::compute_profile(
            IntergalacticTarget::Mission5AndromedaM31,
            PowerTier::ExawattImpulse,
            BraidReloadMode::Mode1cIsobaricRecycle,
        );
        assert!(m5_imp.profile_passed);
        assert!((m5_imp.crew_proper_time_years * 365.25 - 10.33).abs() < 0.5);

        let m7_core = IntergalacticBraidMission::compute_profile(
            IntergalacticTarget::Mission7LaniakeaCore,
            PowerTier::SustainedBurst,
            BraidReloadMode::Mode1cIsobaricRecycle,
        );
        assert!(m7_core.profile_passed);
        assert!(m7_core.propulsion_duty_cycle > 0.999999);
    }
}

