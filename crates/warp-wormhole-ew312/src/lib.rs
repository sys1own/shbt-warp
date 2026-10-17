//! SHBT-EW-312: Effective Traversable Wormhole System
//! (Alcubierre-Stinespring Superluminal Topological Corridor).

pub mod constants;
pub mod classical_nogo;
pub mod pipeline;
pub mod conops;
pub mod causal_2pn;

pub use constants::*;
pub use classical_nogo::{verify_classical_bulk_nogo, verify_effective_corridor, ClassicalWormholeAudit, EffectiveCorridorAudit};
pub use pipeline::{execute_wormhole_pipeline, PowerTier, WormholePipelineResult};
pub use conops::{evaluate_mode_a, get_benchmark_missions, ConOpsMode, MissionSpec, StargateMetrics};
pub use causal_2pn::{evaluate_2pn_metric, interval_2pn, interval_2pn_with_shift, is_causally_authorized, Metric2pn};

#[cfg(test)]
mod tests {
    use super::*;

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
}

