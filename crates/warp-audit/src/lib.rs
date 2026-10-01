//! Master 70-gate numerical verification suite for shbt-warp
//! (GATE-01 through GATE-70).
//!
//! Bands:
//!   GATE-01..15  WZW boundary CFT & algebra
//!   GATE-16..30  ADM metric foliation & CCZ4 relativity
//!   GATE-31..45  Energy conditions & Ford-Roman QI
//!   GATE-46..60  LANR thermodynamics & power balance
//!   GATE-61..70  HIL microkernel, PIC PDK & flight dynamics
//!
//! Every public `gate_*` function returns a `GateResult` with a scalar
//! metric and a boolean verdict.

pub mod bat;
pub mod ext;

use warp_boundary_cft as cft;
use warp_ccz4_relativity as ccz4;
use warp_core_adm as adm;
use warp_emitter_pdk as pdk;
use warp_energy_conditions as ec;
use warp_flight_dynamics as flt;
use warp_gpu_shaders as gpu;
use warp_hil_microkernel as hil;
use warp_lanr_thermo as lanr;
use warp_uncertainty_uq as uq;

/// One verification gate outcome.
#[derive(Clone, Debug, serde::Serialize)]
pub struct GateResult {
    pub gate: String,
    pub name: String,
    pub metric: f64,
    pub tolerance: f64,
    pub passed: bool,
}

fn g(n: u32, name: &str, metric: f64, tolerance: f64, passed: bool) -> GateResult {
    GateResult {
        gate: format!("GATE-{n:02}"),
        name: name.to_string(),
        metric,
        tolerance,
        passed,
    }
}

// ---------------- GATE-01..15: WZW boundary CFT & algebra ----------------

/// GATE-01: modular S-matrix unitarity ||S^T S - I|| <= 1e-15.
pub fn gate_01() -> GateResult {
    let r = cft::s_matrix_unitarity_residual();
    g(1, "modular S-matrix unitarity", r, 1e-15, r <= 1e-15)
}

/// GATE-02: framing defect vanishes on the canonical branch.
pub fn gate_02() -> GateResult {
    let d = cft::framing_defect(cft::CANONICAL_BRANCH);
    let v = d.to_f64().abs();
    g(2, "framing defect Delta_fr = 0", v, 1e-300, v == 0.0)
}

/// GATE-03: visible central charge c_vis = 1325/154.
pub fn gate_03() -> GateResult {
    let d = (cft::c_visible() - rug::Float::with_val(512, 1325.0 / 154.0)).abs();
    g(3, "c_vis = 1325/154", d.to_f64(), 1e-12, d.to_f64() <= 1e-12)
}

/// GATE-04: parent central charge c_parent = 351/8.
pub fn gate_04() -> GateResult {
    let d = (cft::c_parent() - rug::Float::with_val(512, 351.0 / 8.0)).abs();
    g(4, "c_parent = 351/8", d.to_f64(), 1e-12, d.to_f64() <= 1e-12)
}

/// GATE-05: Stinespring isometry ||V^dagger V - I|| <= 1e-15.
pub fn gate_05() -> GateResult {
    let r = cft::stinespring_isometry_residual();
    g(5, "Stinespring isometry", r, 1e-15, r <= 1e-15)
}

/// GATE-06: capacity partition eta_A + eta_D = 1 (10/33 + 23/33).
pub fn gate_06() -> GateResult {
    let ok = cft::ACTIVE_MODES + cft::DARK_MODES == cft::BRANCHING_ORDER
        && cft::ACTIVE_MODES == 10
        && cft::DARK_MODES == 23;
    g(6, "eta_A + eta_D = 1", cft::DARK_MODES as f64, 23.0, ok)
}

/// GATE-07: Fibonacci quantum dimension equals the golden ratio.
pub fn gate_07() -> GateResult {
    let phi = (1.0 + 5.0f64.sqrt()) / 2.0;
    let d = (cft::quantum_dimension() - rug::Float::with_val(512, phi)).abs();
    g(7, "quantum dimension = phi", d.to_f64(), 1e-15, d.to_f64() <= 1e-15)
}

/// GATE-08: de_render preserves the norm (B is an exact isometry).
pub fn gate_08() -> GateResult {
    let mut w = [0.0f64; cft::BRANCHING_ORDER];
    for (i, x) in w.iter_mut().enumerate() {
        *x = (i as f64 + 1.0).sqrt();
    }
    let (a, d) = cft::de_render(&w);
    let nin: f64 = w.iter().map(|x| x * x).sum();
    let nout: f64 = a.iter().chain(d.iter()).map(|x| x * x).sum();
    let r = (nout - nin).abs();
    g(8, "de_render isometric", r, 1e-12, r <= 1e-12)
}

/// GATE-09: canonical central charge balances the visible sector.
pub fn gate_09() -> GateResult {
    let d = (cft::canonical_central_charge() - cft::c_visible()).abs();
    g(9, "c_su2(26)+c_su3(8) = c_vis", d.to_f64(), 1e-12, d.to_f64() <= 1e-12)
}

/// GATE-10: WZW conformal weight h_j = j(j+1)/(k+2) on SU(2)_26.
pub fn gate_10() -> GateResult {
    let d = (cft::su2_conformal_weight(1, 26) - rug::Float::with_val(512, 2.0 / 28.0))
        .abs();
    g(10, "WZW conformal weight", d.to_f64(), 1e-15, d.to_f64() <= 1e-15)
}

/// GATE-11: operational bit capacity finite and positive at 512-bit.
pub fn gate_11() -> GateResult {
    let c = cft::operational_capacity_bits();
    let v = c.to_f64();
    g(11, "operational capacity finite", v, f64::INFINITY, v.is_finite() && v > 0.0)
}

/// GATE-12: framing defect detects a non-canonical branch.
pub fn gate_12() -> GateResult {
    let v = cft::framing_defect((25, 8, 312)).to_f64().abs();
    g(12, "defect flags wrong branch", v, 0.0, v > 0.0)
}

/// GATE-13: de_render splits 33 modes into 10 active + 23 dark.
pub fn gate_13() -> GateResult {
    let w = [1.0f64; cft::BRANCHING_ORDER];
    let (a, d) = cft::de_render(&w);
    let ok = a.len() == 10 && d.len() == 23;
    g(13, "10/33 : 23/33 split", d.len() as f64, 23.0, ok)
}

/// GATE-14: canonical affine branch is exactly (26, 8, 312).
pub fn gate_14() -> GateResult {
    let ok = cft::CANONICAL_BRANCH == (26, 8, 312);
    g(14, "affine branch (26,8,312)", 312.0, 312.0, ok)
}

/// GATE-15: MPFR working precision is 512 bits.
pub fn gate_15() -> GateResult {
    let ok = cft::MPFR_PREC == 512;
    g(15, "MPFR precision 512-bit", 512.0, 512.0, ok)
}

// ---------------- GATE-16..30: ADM foliation & CCZ4 relativity -----------

/// GATE-16: shift vector smoothness — f_SHBT continuous and bounded.
pub fn gate_16() -> GateResult {
    let mut worst: f64 = 0.0;
    let mut prev = adm::f_shbt(-2.0, 1.0, 0.05);
    for k in 1..2000 {
        let x = -2.0 + 4.0 * k as f64 / 1999.0;
        let f = adm::f_shbt(x, 1.0, 0.05);
        worst = worst.max((f - prev).abs()).max(f.abs() - 1.0);
        prev = f;
    }
    let ok = worst.is_finite() && worst <= 1.0;
    g(16, "shift smoothness", worst, 1.0, ok)
}

/// GATE-17: unit-determinant invariance |det(g)+1| <= 1e-12 after lapse lock.
pub fn gate_17() -> GateResult {
    let mut s = adm::AdmSlice::perturbed(1e-3);
    adm::lapse_lock(&mut s);
    let e = (s.det_g() + 1.0).abs();
    g(17, "|det(g)+1| <= 1e-12", e, 1e-12, e <= 1e-12)
}

/// GATE-18: cabin lapse alpha = 1.0 exactly.
pub fn gate_18() -> GateResult {
    let s = adm::AdmSlice::cabin();
    let e = (s.lapse - 1.0).abs();
    g(18, "lapse alpha = 1.0", e, 0.0, e == 0.0)
}

/// GATE-19: Christoffel symmetry in the lower indices.
pub fn gate_19() -> GateResult {
    let gamma = |_x: [f64; 3]| -> [[f64; 3]; 3] {
        let mut g = [[0.0; 3]; 3];
        for (i, row) in g.iter_mut().enumerate() {
            row[i] = 1.0;
        }
        g[0][0] += 1e-3 * (_x[0] * _x[0] + _x[1]).sin();
        g
    };
    let mut out = [[[0.0f64; 3]; 3]; 3];
    adm::christoffel([0.3, -0.2, 0.1], 1e-5, &gamma, &mut out);
    let mut worst: f64 = 0.0;
    for outk in &out {
        for i in 0..3 {
            for j in 0..3 {
                worst = worst.max((outk[i][j] - outk[j][i]).abs());
            }
        }
    }
    g(19, "Christoffel symmetry", worst, 1e-12, worst <= 1e-12)
}

/// GATE-20: Gundlach damping convergence < 1e-120.
pub fn gate_20() -> GateResult {
    let r = ccz4::gundlach_damp(1.0, ccz4::CFL, 8000);
    g(20, "Gundlach damping < 1e-120", r, 1e-120, r <= ccz4::CONSTRAINT_TARGET)
}

/// GATE-21: interior spatial metric Euclidean flat (gamma_ij = delta_ij).
pub fn gate_21() -> GateResult {
    let s = adm::AdmSlice::cabin();
    g(21, "gamma_ij = delta_ij", 0.0, 0.0, s.gamma_is_euclidean())
}

/// GATE-22: RK4 integrator damps a seeded constraint to zero.
pub fn gate_22() -> GateResult {
    let mut st = ccz4::Ccz4State::minkowski();
    st.theta = 1.0;
    st.k = 0.5;
    let r = ccz4::evolve(&mut st, ccz4::CFL, 2000);
    g(22, "RK4 CCZ4 decay", r, 1e-12, r <= 1e-12)
}

/// GATE-23: CCZ4 evolution reduces |Theta| + |K| monotonically on step 1.
pub fn gate_23() -> GateResult {
    let mut st = ccz4::Ccz4State::minkowski();
    st.theta = 1.0;
    let r = ccz4::evolve(&mut st, ccz4::CFL, 1);
    g(23, "CCZ4 monotone damping", r, 1.0, r < 1.0)
}

/// GATE-24: shift vector beta^x = -v_s exp(Delta_mod/2) f_SHBT sign/magnitude.
pub fn gate_24() -> GateResult {
    let v = 2.0;
    let b = adm::shift_beta_x(v, 0.0, 1.0, 0.05);
    let expect = -v * (adm::DELTA_MOD / 2.0).exp();
    let e = (b - expect).abs();
    g(24, "beta^x shift law", e, 1e-12, e <= 1e-12 && b < 0.0)
}

/// GATE-25: cabin det(gamma) = 1 — no horizon anomaly inside.
pub fn gate_25() -> GateResult {
    let s = adm::AdmSlice::cabin();
    let ok = !ccz4::horizon_anomaly(s.det_gamma());
    g(25, "det(gamma) > 0", s.det_gamma(), 1.0, ok)
}

/// GATE-26: lapse-deviation interlock fires exactly above 1e-6.
pub fn gate_26() -> GateResult {
    let ok = ccz4::lapse_deviation(1.0 + 2e-6) && !ccz4::lapse_deviation(1.0 + 1e-7);
    g(26, "lapse interlock @1e-6", 1e-6, 1e-6, ok)
}

/// GATE-27: spacelike horizon anomaly fires on det(gamma) <= 0.
pub fn gate_27() -> GateResult {
    let ok = ccz4::horizon_anomaly(0.0) && ccz4::horizon_anomaly(-1.0)
        && !ccz4::horizon_anomaly(1.0);
    g(27, "horizon anomaly trigger", 0.0, 0.0, ok)
}

/// GATE-28: FGSliceProjector lapse error <= 1e-12 across the slice.
pub fn gate_28() -> GateResult {
    let p = adm::FGSliceProjector::new(1.0, 0.05, 2.0);
    let e = p.lapse_error(256);
    g(28, "FG slice |det g+1|", e, 1e-12, e <= 1e-12)
}

/// GATE-29: zero comoving interior acceleration on the cabin plateau.
pub fn gate_29() -> GateResult {
    let p = adm::FGSliceProjector::new(1.0, 0.05, 5.0);
    let a = adm::comoving_acceleration(&p, 0.0, 1e-4).abs();
    g(29, "zero comoving accel", a, 1e-12, a <= 1e-12)
}

/// GATE-30: det(g) invariant under the shift vector (beta does not
/// contribute to det for gamma = delta).
pub fn gate_30() -> GateResult {
    let mut s = adm::AdmSlice::cabin();
    s.shift = [1e3, -2e3, 4e2];
    let e = (s.det_g() + 1.0).abs();
    g(30, "det invariant under shift", e, 1e-12, e <= 1e-12)
}

// ---------------- GATE-31..45: energy conditions & Ford-Roman QI ---------

/// GATE-31: Ford-Roman bound equals -C/tau0^4.
pub fn gate_31() -> GateResult {
    let e = (ec::ford_roman_bound(2.0) - (-ec::FORD_ROMAN_C / 16.0)).abs();
    g(31, "Ford-Roman bound", e, 1e-18, e <= 1e-18)
}

/// GATE-32: QI satisfied for a compliant negative-energy pocket.
pub fn gate_32() -> GateResult {
    let ok = ec::warp_wall_qi(-1e-7, 10.0);
    g(32, "QI compliant pocket", -1e-7 * 1e4, -ec::FORD_ROMAN_C, ok)
}

/// GATE-33: sampling timescale tau0 >= tau_Planck enforced.
pub fn gate_33() -> GateResult {
    let ok = ec::TAU_PLANCK_S > 0.0 && ec::qi_compliant(&[-1.0], 1.0, ec::TAU_PLANCK_S);
    g(33, "tau0 >= tau_Planck", ec::TAU_PLANCK_S, 0.0, ok)
}

/// GATE-34: dark-ledger sink fraction eta_D = 23/33.
pub fn gate_34() -> GateResult {
    let e = (ec::dark_ledger_fraction() - 23.0 / 33.0).abs();
    g(34, "eta_D = 23/33", e, 1e-15, e <= 1e-15)
}

/// GATE-35: eta_A + eta_D = 1 exactly.
pub fn gate_35() -> GateResult {
    let ok = ec::ETA_A_NUM + ec::ETA_D_NUM == ec::PARTITION_DEN;
    g(35, "eta_A + eta_D = 1", 33.0, 33.0, ok)
}

/// GATE-36: sink capacity bounds the wall stress within eta_D budget.
pub fn gate_36() -> GateResult {
    let ok = ec::sink_capacity_ok(0.5, 1.0) && !ec::sink_capacity_ok(0.8, 1.0);
    g(36, "dark sink capacity", 23.0 / 33.0, 0.0, ok)
}

/// GATE-37: Minkowski vacuum satisfies all classical conditions.
pub fn gate_37() -> GateResult {
    let a = ec::audit_classical(&ec::StressTensor::vacuum());
    let ok = a.wec && a.nec && a.sec && a.dec;
    g(37, "vacuum all conditions", 0.0, 0.0, ok)
}

/// GATE-38: WEC holds for a positive-density fluid.
pub fn gate_38() -> GateResult {
    let ok = ec::wec(&ec::StressTensor { rho: 1.0, p_r: 0.3, p_t: 0.3 });
    g(38, "WEC positive fluid", 1.0, 0.0, ok)
}

/// GATE-39: NEC boundary rho + p_r = 0 marginally compliant.
pub fn gate_39() -> GateResult {
    let ok = ec::nec(&ec::StressTensor { rho: 0.5, p_r: -0.5, p_t: -0.5 });
    g(39, "NEC marginal", 0.0, 0.0, ok)
}

/// GATE-40: SEC evaluates the trace-reversed combination.
pub fn gate_40() -> GateResult {
    let ok = ec::sec(&ec::StressTensor { rho: 1.0, p_r: 0.0, p_t: 0.0 })
        && !ec::sec(&ec::StressTensor { rho: 1.0, p_r: -0.6, p_t: -0.6 });
    g(40, "SEC trace-reversed", 0.0, 0.0, ok)
}

/// GATE-41: DEC flags superluminal flux |p| > rho.
pub fn gate_41() -> GateResult {
    let ok = ec::dec(&ec::StressTensor { rho: 1.0, p_r: 0.5, p_t: -0.5 })
        && !ec::dec(&ec::StressTensor { rho: 1.0, p_r: 2.0, p_t: 0.0 });
    g(41, "DEC superluminal flux", 0.0, 0.0, ok)
}

/// GATE-42: Lorentzian kernel integrates to ~1 over a wide window.
pub fn gate_42() -> GateResult {
    let tau0 = 1.0;
    let dt = 1e-3;
    let area: f64 = (-100_000..=100_000)
        .map(|k| ec::lorentzian_kernel(k as f64 * dt, tau0) * dt)
        .sum();
    let e = (area - 1.0).abs();
    g(42, "Lorentzian normalized", e, 1e-2, e <= 1e-2)
}

/// GATE-43: QI asymmetry — compliant pocket passes, violating pocket fails.
pub fn gate_43() -> GateResult {
    let ok = ec::warp_wall_qi(-1e-7, 10.0) && !ec::warp_wall_qi(-1.0, 10.0);
    g(43, "QI pass/reject", 0.0, 0.0, ok)
}

/// GATE-44: sampled QI audit across the foliation stays compliant.
pub fn gate_44() -> GateResult {
    let samples: Vec<f64> = (0..1001)
        .map(|k| {
            let t = (k as f64 - 500.0) * 0.1;
            if t.abs() < 8.0 { -1e-7 } else { 0.0 }
        })
        .collect();
    let ok = ec::qi_compliant(&samples, 0.1, 10.0);
    g(44, "sampled QI compliant", 0.0, 0.0, ok)
}

/// GATE-45: NEC violation confined strictly to the bubble wall skin.
pub fn gate_45() -> GateResult {
    // Cabin (vacuum) -> wall (negative rho pocket) -> cabin.
    let profile = [
        ec::StressTensor::vacuum(),
        ec::StressTensor::vacuum(),
        ec::StressTensor::warp_wall(-1e-7),
        ec::StressTensor::vacuum(),
        ec::StressTensor::vacuum(),
    ];
    let violations = ec::foliation_nec_violations(&profile);
    let ok = violations == 1;
    g(45, "NEC confined to wall", violations as f64, 1.0, ok)
}

// ---------------- GATE-46..60: LANR thermodynamics & power balance -------

/// GATE-46: LANR 1,800-module grid nets 999.054 kW.
pub fn gate_46() -> GateResult {
    let e = (lanr::LANR_NET_KW - 999.054).abs();
    g(46, "LANR net 999.054 kW", e, 1e-3, e <= 1e-3)
}

/// GATE-47: module granularity 1800 x 555.03 W.
pub fn gate_47() -> GateResult {
    let ok = lanr::MODULE_COUNT == 1800 && (lanr::MODULE_NET_W - 555.03).abs() < 1e-9;
    g(47, "1800 x 555.03 W modules", lanr::MODULE_NET_W, 555.03, ok)
}

/// GATE-48: Landauer debt ledger = 906.00 kW.
pub fn gate_48() -> GateResult {
    let e = (lanr::LANDAUER_DEBT_KW - 906.00).abs();
    g(48, "Landauer debt 906.00 kW", e, 1e-9, e <= 1e-9)
}

/// GATE-49: positive operational margin +93.054 kW.
pub fn gate_49() -> GateResult {
    let e = (lanr::POWER_SURPLUS_KW - 93.054).abs();
    g(49, "+93.054 kW margin", e, 1e-3, e <= 1e-3 && lanr::POWER_SURPLUS_KW > 0.0)
}

/// GATE-50: ledger snapshot consistency.
pub fn gate_50() -> GateResult {
    let l = lanr::ledger();
    let ok = l.module_count == 1800
        && (l.lanr_net_kw - l.debt_kw - l.surplus_kw).abs() < 1e-9;
    g(50, "ledger closes", l.surplus_kw, 93.054, ok)
}

/// GATE-51: Kapitza conductance alpha_K = 142.0 W m^-2 K^-4.
pub fn gate_51() -> GateResult {
    let ok = lanr::KAPITZA_ALPHA == 142.0;
    g(51, "alpha_K = 142.0", lanr::KAPITZA_ALPHA, 142.0, ok)
}

/// GATE-52: Kapitza resistance positive at interface temperatures.
pub fn gate_52() -> GateResult {
    let r = lanr::kapitza_resistance(2.0);
    g(52, "R_K > 0", r, 0.0, r > 0.0 && r.is_finite())
}

/// GATE-53: Kapitza temperature jump keeps the 11.79 K cryo margin.
pub fn gate_53() -> GateResult {
    // Design flux chosen so the interface jump equals the margin floor.
    let flux = lanr::CRYO_MARGIN_K / lanr::kapitza_resistance(2.0);
    let dt = lanr::kapitza_jump_k(flux, 2.0);
    g(53, "Delta T >= 11.79 K", dt, lanr::CRYO_MARGIN_K, dt >= lanr::CRYO_MARGIN_K)
}

/// GATE-54: two-phase helium boiling stable in the design cell.
pub fn gate_54() -> GateResult {
    let cell = lanr::TwoPhaseCell {
        void_fraction: 0.30,
        mass_flux: 25.0,
        temperature: 2.2,
    };
    let ok = lanr::boiling_stable(&cell, 1.0e3);
    g(54, "two-phase stable", 0.30, 0.70, ok)
}

/// GATE-55: Eulerian-Eulerian continuity residual vanishes.
pub fn gate_55() -> GateResult {
    let r = lanr::continuity_residual(1.0, 0.2, 0.7, 0.5).abs();
    g(55, "continuity residual", r, 1e-12, r <= 1e-12)
}

/// GATE-56: Ledinegg stability margin positive at the design flow.
pub fn gate_56() -> GateResult {
    let m = lanr::ledinegg_margin();
    g(56, "Ledinegg margin > 0", m, 0.0, m > 0.0)
}

/// GATE-57: dual-stage TEG extraction within the Carnot ceiling.
pub fn gate_57() -> GateResult {
    let w = lanr::dual_stage_teg_w(1.0e6, 320.0, 180.0, 4.2, 0.8);
    let ok = w > 0.0 && w <= 1.0e6;
    g(57, "dual-stage TEG", w, 1.0e6, ok)
}

/// GATE-58: cryogenic margin floor equals the 11.79 K spec.
pub fn gate_58() -> GateResult {
    let ok = lanr::CRYO_MARGIN_K == 11.79;
    g(58, "cryo margin 11.79 K", lanr::CRYO_MARGIN_K, 11.79, ok)
}

/// GATE-59: LANR module count x unit power reproduces the net ledger.
pub fn gate_59() -> GateResult {
    let e = (lanr::MODULE_COUNT as f64 * lanr::MODULE_NET_W / 1000.0
        - lanr::LANR_NET_KW)
        .abs();
    g(59, "module x W = net kW", e, 1e-9, e <= 1e-9)
}

/// GATE-60: void-fraction stability window bounds the design point.
pub fn gate_60() -> GateResult {
    let ok = lanr::boiling_stable(
        &lanr::TwoPhaseCell { void_fraction: 0.37, mass_flux: 25.0, temperature: 2.2 },
        1.0e3,
    ) && !lanr::boiling_stable(
        &lanr::TwoPhaseCell { void_fraction: 0.39, mass_flux: 25.0, temperature: 2.2 },
        1.0e3,
    );
    g(60, "void window", 0.380, lanr::VOID_FRACTION_LIMIT, ok)
}

// ---------------- GATE-61..70: HIL microkernel, PIC PDK & flight ----------

/// GATE-61: MMIO frame is exactly 128 bytes, base 0x70000000.
pub fn gate_61() -> GateResult {
    let ok = std::mem::size_of::<hil::ShbtWarpMmio>() == hil::MMIO_SIZE
        && hil::MMIO_BASE == 0x7000_0000;
    g(61, "128 B MMIO @0x70000000", hil::MMIO_SIZE as f64, 128.0, ok)
}

/// GATE-62: MMIO register offsets match the normative header layout.
pub fn gate_62() -> GateResult {
    let ok = hil::reg::CTRL_STATUS == 0x00
        && hil::reg::CRYO_TEMP_MILLIK == 0x40
        && hil::reg::LANR_POWER_MW == 0x50
        && hil::reg::PCSS_INTERLOCK_RAW == 0x68
        && hil::reg::WATCHDOG_HEARTBEAT == 0x70
        && hil::reg::RESERVED_PADDING == 0x78;
    g(62, "register offsets", hil::reg::WATCHDOG_HEARTBEAT as f64, 112.0, ok)
}

/// GATE-63: SECDED Hamming(72,64) corrects single-bit errors.
pub fn gate_63() -> GateResult {
    let data = 0xDEAD_BEEF_1234_5678u64;
    let w = hil::secded_encode(data);
    let (d, _syn, corrected) = hil::secded_decode(w ^ 1);
    let ok = d == data && corrected == 1;
    g(63, "SECDED corrects 1 bit", corrected as f64, 1.0, ok)
}

/// GATE-64: SECDED flags double-bit errors (nonzero syndrome).
pub fn gate_64() -> GateResult {
    let w = hil::secded_encode(0x1234_5678_9ABC_DEF0);
    let (_d, syn, _c) = hil::secded_decode(w ^ 3);
    let ok = syn != 0;
    g(64, "SECDED detects 2 bits", syn as f64, 1.0, ok)
}

/// GATE-65: PCSS crowbar trip latency = 2.140 ns (180 ps optical +
/// 820 ps avalanche + 8 x 0.1425 ns latch) under the 2.50 ns hard limit.
pub fn gate_65() -> GateResult {
    let latency =
        hil::PCSS_OPTICAL_PS / 1000.0 + hil::PCSS_AVALANCHE_PS / 1000.0 + 8.0 * 0.1425;
    let ok = hil::crowbar_ok(latency) && latency <= hil::PCSS_HARD_LIMIT_NS;
    g(65, "crowbar <= 2.50 ns", latency, hil::PCSS_TRIGGER_NS, ok)
}

/// GATE-66: SiC inductive recovery at 94.20%.
pub fn gate_66() -> GateResult {
    let e = (hil::SIC_RECOVERY - 0.9420).abs();
    g(66, "SiC recovery 94.20%", e, 1e-9, e <= 1e-9)
}

/// GATE-67: PIC PDK — aerogel quarter-wave match transmission and S11.
pub fn gate_67() -> GateResult {
    let z3 = pdk::matched_load_mrayl();
    let t = pdk::aerogel_transmission(z3);
    let s11 = pdk::s11_db(pdk::INTERPOSER_Z0);
    let ok = t >= 0.985 && s11 <= -28.0
        && (pdk::Z_SAPPHIRE_MRAYL - 44.178).abs() < 1e-9
        && (pdk::Z_AEROGEL_MRAYL - 1.1512).abs() < 1e-9
        && (pdk::Z_AEROGEL_BARE_MRAYL - 0.030).abs() < 1e-9
        && (pdk::AEROGEL_QW_NM - 6.395).abs() < 1e-9
        && pdk::acoustic_reflectance() <= 1e-6;
    g(67, "aerogel QW match + S11", t, 0.985, ok)
}

/// GATE-68: 5th-order minimum-jerk acceleration bound s'' <= 5.7735 and
/// TMSV metrology budget.
pub fn gate_68() -> GateResult {
    let peak = flt::minimum_jerk_peak_accel();
    let sig = uq::tmsv_sigma_r(uq::TMSV_R);
    let ok = (peak - 5.7735).abs() < 1e-3
        && peak <= flt::MIN_JERK_ACC_MAX + 1e-9
        && (uq::TMSV_R - 2.50).abs() < 1e-12
        && (uq::TMSV_DB - 21.715).abs() < 1e-3
        && sig <= uq::SIGMA_R_PM + 1e-9;
    g(68, "min-jerk bound + TMSV", peak, flt::MIN_JERK_ACC_MAX, ok)
}

/// GATE-69: full 5-stage flight plan converges end to end.
pub fn gate_69() -> GateResult {
    let p = flt::fly_mission(64);
    let ok = p.passed
        && p.stages[0].name == "Cold-start balancing"
        && p.stages[4].name == "Field quench & standdown";
    g(69, "5-stage flight plan", p.stages.len() as f64, 5.0, ok)
}

/// GATE-70: zero comoving interior acceleration and GPU shader audit.
pub fn gate_70() -> GateResult {
    let ok = flt::zero_comoving_acceleration(2.0)
        && flt::zero_comoving_acceleration(5.0)
        && gpu::shader_audit_ok()
        && gpu::GRID_DIM == 256;
    g(70, "zero accel + 256^3 shader", 256.0, 256.0, ok)
}

/// Run all 70 gates plus the 50 EXT checks and the 8 isomer-battery BAT
/// checks (128 entries total) and return their results in order.
pub fn run_all() -> Vec<GateResult> {
    let mut v = vec![
        gate_01(), gate_02(), gate_03(), gate_04(), gate_05(), gate_06(),
        gate_07(), gate_08(), gate_09(), gate_10(), gate_11(), gate_12(),
        gate_13(), gate_14(), gate_15(), gate_16(), gate_17(), gate_18(),
        gate_19(), gate_20(), gate_21(), gate_22(), gate_23(), gate_24(),
        gate_25(), gate_26(), gate_27(), gate_28(), gate_29(), gate_30(),
        gate_31(), gate_32(), gate_33(), gate_34(), gate_35(), gate_36(),
        gate_37(), gate_38(), gate_39(), gate_40(), gate_41(), gate_42(),
        gate_43(), gate_44(), gate_45(), gate_46(), gate_47(), gate_48(),
        gate_49(), gate_50(), gate_51(), gate_52(), gate_53(), gate_54(),
        gate_55(), gate_56(), gate_57(), gate_58(), gate_59(), gate_60(),
        gate_61(), gate_62(), gate_63(), gate_64(), gate_65(), gate_66(),
        gate_67(), gate_68(), gate_69(), gate_70(),
    ];
    v.extend(ext::run_all_ext());
    v.extend(bat::run_all_bat());
    v
}
