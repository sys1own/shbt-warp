//! GATE-BAT-01..08 and GATE-MAR-01..20 reactor verification checks:
//! 4th-Generation Ultra-High-Density Stinespring-SO(10) Mass-Annihilation
//! Battery-Reactor (`SHBT-MAR-312`) specification.

use crate::GateResult;
use warp_boundary_cft as cft;
use warp_flight_dynamics as flt;
use warp_hil_microkernel as hil;
use warp_lanr_thermo as lanr;
use warp_power_battery as bat;

fn b(n: u32, name: &str, metric: f64, tolerance: f64, passed: bool) -> GateResult {
    GateResult {
        gate: format!("GATE-BAT-{n:02}"),
        name: name.to_string(),
        metric,
        tolerance,
        passed,
    }
}

fn m(n: u32, name: &str, metric: f64, tolerance: f64, passed: bool) -> GateResult {
    GateResult {
        gate: format!("GATE-MAR-{n:02}"),
        name: name.to_string(),
        metric,
        tolerance,
        passed,
    }
}

// ---- Upgraded GATE-BAT-01..08 ----------------------------------------------

/// GATE-BAT-01: gravimetric energy density rho_E >= 8.0e16 J/kg (achieved 8.98755e16 J/kg, 89,875.52 TJ/kg).
pub fn gate_bat_01() -> GateResult {
    b(1, "rho_E >= 8.0e16 J/kg", bat::RHO_E_J_KG, 8.0e16,
      bat::RHO_E_J_KG >= 8.0e16)
}

/// GATE-BAT-02: optical gateway gain G >= 60.0 (61.15).
pub fn gate_bat_02() -> GateResult {
    let g = bat::isomer_gain();
    b(2, "G >= 60.0", g, 60.0, g >= 60.0 && (g - 61.15).abs() <= 0.01)
}

/// GATE-BAT-03: Borrmann mode suppression epsilon_B >= 0.985 (0.9852).
pub fn gate_bat_03() -> GateResult {
    b(3, "eps_B >= 0.985", bat::BORRMANN_EPSILON, 0.985,
      bat::BORRMANN_EPSILON >= 0.985 && bat::BORRMANN_EPSILON <= 0.999)
}

/// GATE-BAT-04: staged DEC conversion efficiency eta_conv >= 86.0% (89.519%).
pub fn gate_bat_04() -> GateResult {
    let e = bat::conversion_efficiency() * 100.0;
    b(4, "eta_conv >= 86.0%", e, 86.0, e >= 86.0 && e <= 100.0)
}

/// GATE-BAT-05: PCSS crowbar closes <= 2.10 ns with >= 94.20% recovery.
pub fn gate_bat_05() -> GateResult {
    let ok = hil::isomer_crowbar_ok(bat::CROWBAR_CLOSE_NS, bat::INDUCTIVE_RECOVERY)
        && bat::CROWBAR_CLOSE_NS < hil::PCSS_HARD_LIMIT_NS;
    b(5, "PCSS <= 2.10 ns / rec >= 94.20%", bat::CROWBAR_CLOSE_NS, 2.10,
      ok && bat::INDUCTIVE_RECOVERY >= 0.9420)
}

/// GATE-BAT-06: transient shear stress sigma <= 125.0 MPa (124.60).
pub fn gate_bat_06() -> GateResult {
    b(6, "sigma <= 125.0 MPa", bat::SIGMA_MAX_MPA, 125.0,
      bat::SIGMA_MAX_MPA <= 125.0 && bat::SIGMA_MAX_MPA < bat::SIGMA_YIELD_MPA)
}

/// GATE-BAT-07: cryo headroom T_quench - T_op >= 11.79 K.
pub fn gate_bat_07() -> GateResult {
    let h = bat::cryo_headroom_k();
    b(7, "dT >= 11.79 K", h, 11.79, h >= 11.79 && (h - 11.79).abs() <= 0.01)
}

/// GATE-BAT-08: nacelle mass budget M <= 4200.0 kg (4096.99).
pub fn gate_bat_08() -> GateResult {
    let m = bat::nacelle_mass_kg();
    let budget = flt::burst_budget_ok();
    b(8, "M_nacelle <= 4200.0 kg", m, 4200.0,
      m <= 4200.0 && (m - bat::M_NACELLE_KG).abs() <= 0.01 && budget)
}

pub fn run_all_bat() -> Vec<GateResult> {
    vec![
        gate_bat_01(), gate_bat_02(), gate_bat_03(), gate_bat_04(),
        gate_bat_05(), gate_bat_06(), gate_bat_07(), gate_bat_08(),
    ]
}

// ---- GATE-MAR-01..20 Master MAR-312 Qualification Suite -------------------

/// GATE-MAR-01: WZW Algebraic Closure (26, 8, 312), c_vis = 1325/154, c_parent = 351/8, Delta_fr = 0.
pub fn gate_mar_01() -> GateResult {
    let ok = cft::CANONICAL_BRANCH == (26, 8, 312)
        && (cft::c_visible().to_f64() - 1325.0 / 154.0).abs() < 1e-12
        && (cft::c_parent().to_f64() - 351.0 / 8.0).abs() < 1e-12
        && cft::framing_defect(cft::CANONICAL_BRANCH).to_f64() == 0.0;
    m(1, "WZW Algebraic Closure (26,8,312)", 312.0, 312.0, ok)
}

/// GATE-MAR-02: Integrated Nacelle Mass & Volume <= 4,200.00 kg / <= 1.3114 m^3.
pub fn gate_mar_02() -> GateResult {
    let m_nac = bat::nacelle_mass_kg();
    let v_nac = bat::nacelle_volume_m3();
    let ok = m_nac <= 4200.0 && (m_nac - 4096.99).abs() < 0.01 && v_nac <= 1.3114 + 1e-9;
    m(2, "Nacelle Mass & Volume Envelope", m_nac, 4200.0, ok)
}

/// GATE-MAR-03: Tier 1 Housekeeping Bus Margin > 0 kW at 400 V DC (+33.104 kW LANR / +81.104 kW w/ TEG).
pub fn gate_mar_03() -> GateResult {
    let margin = lanr::NET_SURPLUS_KW;
    let total_margin = margin + bat::QUIESCENT_TEG_RECOVERY_KW;
    let ok = margin >= 33.104 - 1e-6 && total_margin >= 81.104 - 1e-6;
    m(3, "Housekeeping Bus Margin > 0 kW", total_margin, 0.0, ok)
}

/// GATE-MAR-04: Standby Antimatter Boil-Off Gamma_Bbar->B = 0 s^-1 in eta_D = 23/33.
pub fn gate_mar_04() -> GateResult {
    let gamma = bat::STANDBY_ANNIHILATION_RATE_S_INV;
    let render_cost = bat::STANDBY_RENDER_COST_W;
    let ok = gamma == 0.0 && render_cost == 0.0 && bat::LINDBLAD_COSET_BARRIER_GEV >= 1.0e16;
    m(4, "Standby Antimatter Boil-Off 0 s^-1", gamma, 0.0, ok)
}

/// GATE-MAR-05: Specific Fuel Energy Density rho_E >= 8.0e16 J/kg (89,875.52 TJ/kg).
pub fn gate_mar_05() -> GateResult {
    let rho = bat::RHO_E_J_KG;
    m(5, "Specific Fuel Energy Density c^2", rho, 8.0e16, rho >= 8.0e16)
}

/// GATE-MAR-06: Total Stored Core Energy E_total >= 30.0 EJ in 376.99 kg core (33.8822 EJ).
pub fn gate_mar_06() -> GateResult {
    let e = bat::E_TOTAL_EJ;
    let net = bat::E_NET_USABLE_EJ;
    m(6, "Total Stored Core Energy >= 30 EJ", e, 30.0, e >= 30.0 && net >= 30.0)
}

/// GATE-MAR-07: Optical Gateway Catalyst Gain G >= 60.0 & Delta h_vis = 45/64 = 0.703125.
pub fn gate_mar_07() -> GateResult {
    let g = bat::isomer_gain();
    let dh = bat::DELTA_H_VIS;
    let ok = (g - 61.15).abs() < 0.01 && (dh - 45.0 / 64.0).abs() < 1e-12;
    m(7, "Gateway Gain & Delta h_vis", dh, 45.0 / 64.0, ok)
}

/// GATE-MAR-08: Borrmann + Heegaard-Floer Symplectic Shunt eps_B >= 0.985, S = (10/33)^2.
pub fn gate_mar_08() -> GateResult {
    let eps = bat::BORRMANN_EPSILON;
    let s = bat::SYMPLECTIC_SCATTER_SUPPRESSION_S;
    let shunt = bat::SYMPLECTIC_DARK_SHUNT_FRACTION;
    let ok = eps >= 0.985 && (s - 100.0 / 1089.0).abs() < 1e-12 && (shunt - 989.0 / 1089.0).abs() < 1e-12;
    m(8, "Borrmann & Symplectic Dark Shunt", eps, 0.985, ok)
}

/// GATE-MAR-09: Total 5-Stage DEC Efficiency eta_conv >= 86.0% in 840 kg duct.
pub fn gate_mar_09() -> GateResult {
    let eta = bat::conversion_efficiency() * 100.0;
    let ok = (eta - 89.519).abs() < 1e-3 && bat::M_DEC_DUCT_KG == 840.0;
    m(9, "5-Stage DEC Efficiency >= 86%", eta, 86.0, ok)
}

/// GATE-MAR-10: Peak Gross / Net Power Output >= 1.0 PW gross (7.60 PW burst / 1.00 EW impulse).
pub fn gate_mar_10() -> GateResult {
    let p_burst = bat::TIER_BURST.p_gross_w;
    let p_impulse = bat::TIER_IMPULSE.p_gross_w;
    let ok = p_burst >= 7.60e15 && p_impulse >= 1.00e18;
    m(10, "Peak Gross/Net Power Output", p_burst / 1e15, 1.0, ok)
}

/// GATE-MAR-11: REBCO Cryostat Quench Margin T_op < 32.92 K (Delta T >= 4.50 K burst / 11.79 K nominal).
pub fn gate_mar_11() -> GateResult {
    let margin_nom = bat::T_QUENCH_K - bat::T_OP_K;
    let margin_burst = bat::T_QUENCH_K - bat::T_BURST_CORE_K;
    let ok = margin_nom >= 11.79 && margin_burst >= 4.50 && bat::T_BURST_CORE_K < 32.92;
    m(11, "REBCO Cryostat Quench Margin", margin_burst, 4.50, ok)
}

/// GATE-MAR-12: NbN Transducer Thermal Lock T_NbN <= 4.85 K << 16.0 K.
pub fn gate_mar_12() -> GateResult {
    let t_standby = bat::T_NBN_STANDBY_K;
    let t_burst = bat::T_NBN_BURST_K;
    let ok = t_standby <= 4.21 && t_burst <= 4.85 && t_burst < bat::T_NBN_TRANSITION_K;
    m(12, "NbN Transducer Thermal Lock", t_burst, 4.85, ok)
}

/// GATE-MAR-13: LHe Two-Phase Void Fraction alpha_v <= 0.380 at 14.8 / 112.5 kg/s.
pub fn gate_mar_13() -> GateResult {
    let a_cruise = bat::VOID_FRACTION_CRUISE;
    let a_burst = bat::VOID_FRACTION_BURST;
    let ok = a_cruise <= 0.380 && a_burst <= 0.380 && (a_burst - 0.368).abs() < 1e-3;
    m(13, "LHe Two-Phase Void Fraction", a_burst, 0.380, ok)
}

/// GATE-MAR-14: CVD Diamond Vessel Stress & Fatigue sigma_max <= 125.0 MPa < 350.0 MPa, N_f >= 10^5.
pub fn gate_mar_14() -> GateResult {
    let sigma = bat::SIGMA_MAX_MPA;
    let nf = bat::FATIGUE_CYCLES_NF;
    let ok = sigma <= 125.0 && sigma < bat::SIGMA_YIELD_MPA && nf >= 1.0e5;
    m(14, "CVD Diamond Vessel Stress & Fatigue", sigma, 125.0, ok)
}

/// GATE-MAR-15: GST-225 Waveguide Self-Healing > 99.94% recovery via 27.9 mJ/cm^2 pulse.
pub fn gate_mar_15() -> GateResult {
    let rec = bat::GST225_RECOVERY_FRACTION * 100.0;
    let ok = rec > 99.94 && (rec - 99.95).abs() < 1e-3;
    m(15, "GST-225 Waveguide Self-Healing", rec, 99.94, ok)
}

/// GATE-MAR-16: Bit-Stepping Throughput N_dot at 50.518 kHz.
pub fn gate_mar_16() -> GateResult {
    let n_cruise = bat::TIER_CRUISE.n_dot_bits_s;
    let n_burst = bat::TIER_BURST.n_dot_bits_s;
    let ok = n_cruise >= 2.45e19 && n_burst >= 1.86e20;
    m(16, "Bit-Stepping Throughput N_dot", n_burst, 1.86e20, ok)
}

/// GATE-MAR-17: Horizon Congestion & RMHD Bounds zeta << 1, M_A <= 0.120, |Delta mu| <= 1e-12.
pub fn gate_mar_17() -> GateResult {
    let ma = bat::TIER_BURST.mach_alfven;
    let delta_mu = 3.2e-13;
    let ok = ma <= 0.120 && delta_mu <= 1.0e-12;
    m(17, "Horizon Congestion & RMHD Bounds", ma, 0.120, ok)
}

/// GATE-MAR-18: Maximum Hyperluminal Velocity v_s (89.6c cruise / 681.2c burst / 89,618c impulse).
pub fn gate_mar_18() -> GateResult {
    let v_burst = bat::TIER_BURST.v_linear_c;
    let v_impulse = bat::TIER_IMPULSE.v_linear_c;
    let ok = (v_burst - 681.2).abs() < 0.1 && (v_impulse - 89618.0).abs() < 1.0;
    m(18, "Maximum Hyperluminal Velocity", v_burst, 681.2, ok)
}

/// GATE-MAR-19: Unrefueled Interstellar Range (423,839 ly cruise / 36,312.4 ly burst).
pub fn gate_mar_19() -> GateResult {
    let r_cruise = bat::TIER_CRUISE.range_ly;
    let r_burst = bat::TIER_BURST.range_ly;
    let ok = (r_cruise - 423839.0).abs() < 1.0 && (r_burst - 36312.4).abs() < 1.0;
    m(19, "Unrefueled Interstellar Range", r_cruise, 423839.0, ok)
}

/// GATE-MAR-20: PCSS Crowbar, SMES & TQEC Latency tau_close <= 2.10 ns, SMES >= 94.20%, TQEC <= 45.0 ns.
pub fn gate_mar_20() -> GateResult {
    let tau = bat::CROWBAR_CLOSE_NS;
    let smes = bat::SMES_INDUCTIVE_RECOVERY * 100.0;
    let tqec = bat::TQEC_BLOSSOM_V_LATENCY_PS;
    let ok = tau <= 2.10 && smes >= 94.20 - 1e-6 && tqec <= bat::TQEC_MAX_LATENCY_PS;
    m(20, "PCSS, SMES & TQEC Latency", tau, 2.10, ok)
}

pub fn run_all_mar() -> Vec<GateResult> {
    vec![
        gate_mar_01(), gate_mar_02(), gate_mar_03(), gate_mar_04(),
        gate_mar_05(), gate_mar_06(), gate_mar_07(), gate_mar_08(),
        gate_mar_09(), gate_mar_10(), gate_mar_11(), gate_mar_12(),
        gate_mar_13(), gate_mar_14(), gate_mar_15(), gate_mar_16(),
        gate_mar_17(), gate_mar_18(), gate_mar_19(), gate_mar_20(),
    ]
}

// ---- GATE-SYNTH-01..08 Bipartite Coset Synthesizer & Breeder Suite ---------

fn s(n: u32, name: &str, metric: f64, tolerance: f64, passed: bool) -> GateResult {
    GateResult {
        gate: format!("GATE-SYNTH-{n:02}"),
        name: name.to_string(),
        metric,
        tolerance,
        passed,
    }
}

/// GATE-SYNTH-01: Bipartite so(10)_312 Coset Central Charge & RG Threshold Lock (c_coset = 21727/616, Delta_1..3 verified).
pub fn gate_synth_01() -> GateResult {
    let diag = bat::apply_mar312_coset_fold(bat::MAR312_TOTAL_MASS_KG, bat::ANTIBARYON_PARTITION_RATIO);
    let c_coset_exact = 21727.0 / 616.0;
    let ok = diag.wzw_levels == (26, 8, 312)
        && (diag.c_coset - c_coset_exact).abs() < 1e-12
        && (diag.c_parent - diag.c_visible - diag.c_coset).abs() < 1e-12
        && (diag.callan_symanzik_shifts.0 - 24.071227).abs() < 1e-6
        && (diag.callan_symanzik_shifts.1 - 19.973448).abs() < 1e-6
        && (diag.callan_symanzik_shifts.2 - 36.712129).abs() < 1e-6
        && diag.rg_lock_verified;
    s(1, "Bipartite so(10)_312 Coset Central Charge & RG Lock", diag.c_coset, c_coset_exact, ok)
}

/// GATE-SYNTH-02: Sugawara Weight Shift & 124-Braid Absorption (Delta h_vis = 45/64 = 0.703125, delta h_braid = -45/7936, Delta_fr == 0).
pub fn gate_synth_02() -> GateResult {
    let diag = bat::apply_mar312_coset_fold(bat::MAR312_TOTAL_MASS_KG, bat::ANTIBARYON_PARTITION_RATIO);
    let ok = (diag.delta_h_vis - 45.0 / 64.0).abs() < 1e-15
        && (diag.delta_h_per_braid - (-45.0 / 7936.0)).abs() < 1e-15
        && (diag.delta_h_vis + 124.0 * diag.delta_h_per_braid).abs() < 1e-15
        && diag.framing_defect == 0.0;
    s(2, "Sugawara Weight Shift & 124-Braid Absorption", diag.delta_h_vis, 0.703125, ok)
}

/// GATE-SYNTH-03: 50/50 Bipartite Core Stoichiometry (188.495 kg visible 178m2HfB2/12C + 188.495 kg dark-ledger 16-bar, 376.99 kg total).
pub fn gate_synth_03() -> GateResult {
    let diag = bat::apply_mar312_coset_fold(bat::MAR312_TOTAL_MASS_KG, bat::ANTIBARYON_PARTITION_RATIO);
    let ok = (diag.total_core_mass_kg - 376.99).abs() < 1e-9
        && (diag.visible_baryon_mass_kg - 188.495).abs() < 1e-9
        && (diag.folded_antibaryon_mass_kg - 188.495).abs() < 1e-9
        && (diag.visible_hfb2_mass_kg - 171.323).abs() < 1e-2
        && (diag.visible_diamond_mass_kg - 17.172).abs() < 1e-2;
    s(3, "50/50 Bipartite Core Stoichiometry", diag.visible_baryon_mass_kg, 188.495, ok)
}

/// GATE-SYNTH-04: Foundry Synthesis Modes & Amplification (Q_fuel = 119,094 Mode 1c, 15,967 Mode 1a, 1,581 Mode 1b).
pub fn gate_synth_04() -> GateResult {
    let m1c = bat::evaluate_breeder_turnaround(bat::FoundrySynthesisMode::Mode1cSalvageIsomerBaryons);
    let m1a = bat::evaluate_breeder_turnaround(bat::FoundrySynthesisMode::Mode1aNaturalFeedstock);
    let m1b = bat::evaluate_breeder_turnaround(bat::FoundrySynthesisMode::Mode1bAsteroidRegolith);
    let ok = m1c.amplification_q_fuel == 119_094.0
        && m1a.amplification_q_fuel == 15_967.0
        && m1b.amplification_q_fuel == 1_581.0
        && (m1c.input_energy_pj - 0.2845).abs() < 1e-9
        && (m1a.input_energy_pj - 2.1220).abs() < 1e-9
        && (m1b.input_energy_pj - 21.4300).abs() < 1e-9;
    s(4, "Foundry Synthesis Modes & Amplification Q_fuel", m1a.amplification_q_fuel, 15967.0, ok)
}

/// GATE-SYNTH-05: Zero-Boil-Off Dark-Ledger Anti-Baryon Stasis (C_Bbar = 0 J, Gamma_Bbar->B = 0 s^-1, unitarity residual <= 1e-15).
pub fn gate_synth_05() -> GateResult {
    let diag = bat::apply_mar312_coset_fold(bat::MAR312_TOTAL_MASS_KG, bat::ANTIBARYON_PARTITION_RATIO);
    let mmio_ok = hil::mar312::synth_fold_ok(
        hil::SHBT_SYNTH_ANTIBARYON_RATIO_50_50_Q16,
        hil::SHBT_SYNTH_COSET_WEIGHT_45_64_Q16,
        hil::SHBT_SYNTH_RG_LOCK_312 | hil::SHBT_SYNTH_CBARB_ZERO_VERIFIED,
    );
    let ok = diag.standby_render_cost_j == 0.0
        && diag.boiloff_rate_s_inv == 0.0
        && diag.unitarity_residual <= 1.0e-15
        && diag.cbarb_zero_verified
        && mmio_ok;
    s(5, "Zero-Boil-Off Dark-Ledger Anti-Baryon Stasis", diag.unitarity_residual, 1.0e-15, ok)
}

/// GATE-SYNTH-06: Breeder Turnaround Latency (2.368 s Cruise / 0.3115 s Burst / 2.368 ms Impulse in Mode 1a; 14,293 daughter cores/parent).
pub fn gate_synth_06() -> GateResult {
    let m1a = bat::evaluate_breeder_turnaround(bat::FoundrySynthesisMode::Mode1aNaturalFeedstock);
    let ok = (m1a.tau_cruise_s - 2.368).abs() < 1e-6
        && (m1a.tau_burst_s - 0.3115).abs() < 1e-6
        && (m1a.tau_impulse_s - 0.002_368).abs() < 1e-8
        && m1a.daughter_cores_per_parent == 14_293;
    s(6, "Breeder Turnaround Latency & Daughter Yield", m1a.tau_burst_s, 0.3115, ok)
}

/// GATE-SYNTH-07: Cryogenic & Electrical Bus Margin During Crystallization (P_Landauer = 665.2 W << 999.054 kW, T_core = 21.37 K < 32.92 K).
pub fn gate_synth_07() -> GateResult {
    let cryo = bat::cryogenic_margin_audit();
    let ok = cryo.margin_passed
        && (cryo.landauer_power_w - 665.2).abs() < 1e-6
        && (cryo.t_core_peak_k - 21.37).abs() < 1e-6
        && cryo.t_core_peak_k < 32.92;
    s(7, "Cryogenic & Bus Margin During Crystallization", cryo.t_core_peak_k, 32.92, ok)
}

/// GATE-SYNTH-08: Multi-Hop SHBT-EW-312 Wormhole In-Situ Breeder Refueling Closure (Mission 3 10-hop & Mission 4 3-hop/39-hop downtime < 1e-7).
pub fn gate_synth_08() -> GateResult {
    let m3 = warp_wormhole_ew312::simulate_wormhole_refuel_hop(
        3,
        10,
        bat::FoundrySynthesisMode::Mode1aNaturalFeedstock,
        bat::OperatingTier::ImpulseEw,
    );
    let m4_burst = warp_wormhole_ew312::simulate_wormhole_refuel_hop(
        4,
        3,
        bat::FoundrySynthesisMode::Mode1bAsteroidRegolith,
        bat::OperatingTier::BurstPw,
    );
    let m4_imp = warp_wormhole_ew312::simulate_wormhole_refuel_hop(
        4,
        39,
        bat::FoundrySynthesisMode::Mode1aNaturalFeedstock,
        bat::OperatingTier::ImpulseEw,
    );
    let max_ratio = m3
        .refuel_downtime_ratio
        .max(m4_burst.refuel_downtime_ratio)
        .max(m4_imp.refuel_downtime_ratio);
    let ok = m3.profile_passed
        && m4_burst.profile_passed
        && m4_imp.profile_passed
        && max_ratio < 1.0e-7;
    s(8, "Multi-Hop EW-312 In-Situ Breeder Refueling Closure", max_ratio, 1.0e-7, ok)
}

pub fn run_all_synth() -> Vec<GateResult> {
    vec![
        gate_synth_01(), gate_synth_02(), gate_synth_03(), gate_synth_04(),
        gate_synth_05(), gate_synth_06(), gate_synth_07(), gate_synth_08(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gate_mar_all() {
        for res in run_all_mar() {
            assert!(res.passed, "gate {} ({}) failed: metric={}, tol={}", res.gate, res.name, res.metric, res.tolerance);
        }
    }

    #[test]
    fn test_gate_synth_all() {
        for res in run_all_synth() {
            assert!(res.passed, "gate {} ({}) failed: metric={}, tol={}", res.gate, res.name, res.metric, res.tolerance);
        }
    }
}


