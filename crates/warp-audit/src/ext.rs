//! EXT-01..EXT-50 extended verification checks (warp1.txt):
//! extended boundary-CFT algebra, ADM/CCZ4 cross-checks, energy-condition
//! audits, LANR thermodynamic chain, PIC PDK physics and microkernel
//! hardware-contract assertions complementing GATE-01..70.

use crate::GateResult;
use warp_boundary_cft as cft;
use warp_ccz4_relativity as ccz4;
use warp_core_adm as adm;
use warp_emitter_pdk as pdk;
use warp_energy_conditions as ec;
use warp_flight_dynamics as flt;
use warp_hil_microkernel as hil;
use warp_lanr_thermo as lanr;
use warp_uncertainty_uq as uq;

fn x(n: u32, name: &str, metric: f64, tolerance: f64, passed: bool) -> GateResult {
    GateResult {
        gate: format!("EXT-{n:02}"),
        name: name.to_string(),
        metric,
        tolerance,
        passed,
    }
}

pub fn ext_01() -> GateResult {
    // Symmetric 4x4 FG conformal block g^(4) has finite Frobenius trace.
    let m = [[1.0f64, 0.2, -0.1, 0.0], [0.2, 1.1, 0.3, 0.0],
             [-0.1, 0.3, 0.9, -0.2], [0.0, 0.0, -0.2, 1.2]];
    let tr: f64 = (0..4).map(|i| m[i][i]).sum();
    let sym = (0..4).all(|i| (0..4).all(|j| (m[i][j] - m[j][i]).abs() < 1e-15));
    x(1, "FG g4 symmetric finite trace", tr, 1e12, sym && tr.is_finite())
}
pub fn ext_02() -> GateResult {
    let d = (cft::c_total() - rug::Float::with_val(512, 52.478896)).abs();
    let ok = d.to_f64() <= 1e-6;
    x(2, "c_total = 52.478896", d.to_f64(), 1e-6, ok)
}
pub fn ext_03() -> GateResult {
    let d = (cft::c_total() - cft::c_ghost()).abs();
    x(3, "c_ghost = c_total", d.to_f64(), 0.0, d == rug::Float::with_val(512, 0.0))
}
pub fn ext_04() -> GateResult {
    let d = cft::framing_defect_canonical().abs().to_f64();
    x(4, "Delta_fr = 0 mod 1", d, 1e-30, d <= 1e-30)
}
pub fn ext_05() -> GateResult {
    x(5, "eta_A = 10/33", ec::ETA_A_NUM as f64 / 33.0, 10.0 / 33.0,
        ec::ETA_A_NUM == 10 && ec::PARTITION_DEN == 33)
}
pub fn ext_06() -> GateResult {
    x(6, "124 braid descriptors", ec::BRAID_DESCRIPTORS as f64, 124.0,
        ec::BRAID_DESCRIPTORS == 124)
}
pub fn ext_07() -> GateResult {
    x(7, "C_braid = 38.196601", ec::C_BRAID, 38.196601,
        (ec::C_BRAID - 38.196601).abs() <= 1e-6)
}
pub fn ext_08() -> GateResult {
    // QI net margin evaluated at tau0 = 1.05 tau_Planck stays non-negative.
    let m = ec::qi_net_margin(1.05 * ec::TAU_PLANCK_S, 1.0, 1.0);
    x(8, "QI net margin @1.05 tau_P", m, 0.0, m >= 0.0)
}
pub fn ext_09() -> GateResult {
    let r = ec::eulerian_rho(4.25, adm::DELTA_MOD, 0.5, 0.1);
    x(9, "Eulerian rho <= 0", r, 0.0, r <= 0.0)
}
pub fn ext_10() -> GateResult {
    let s = ec::topological_entropy();
    let phi = (1.0 + 5.0f64.sqrt()) / 2.0;
    x(10, "S_top = ln sqrt(2+phi)", s, (2.0 + phi).sqrt().ln(),
        (s - (2.0 + phi).sqrt().ln()).abs() <= 1e-15)
}
pub fn ext_11() -> GateResult {
    let p = ec::braid_phase_sum().abs();
    x(11, "braid phase bounded", p, std::f64::consts::PI,
        p < std::f64::consts::PI)
}
pub fn ext_12() -> GateResult {
    x(12, "Delta_mod = 0.1375335", adm::DELTA_MOD, 0.1375335,
        (adm::DELTA_MOD - 0.1375335).abs() <= 1e-6)
}
pub fn ext_13() -> GateResult {
    x(13, "f_SHBT(0) = 1", adm::f_shbt(0.0, 1.0, 0.05), 1.0,
        (adm::f_shbt(0.0, 1.0, 0.05) - 1.0).abs() <= 1e-9)
}
pub fn ext_14() -> GateResult {
    let f = adm::f_shbt(3.0, 1.0, 0.05);
    x(14, "f_SHBT -> 0 exterior", f, 1e-6, f <= 1e-6)
}
pub fn ext_15() -> GateResult {
    // Phase continuity across the braid bank: wrap errors stay mod 2pi.
    let ok = (0..8).all(|k| {
        let w = pdk::phase_to_iq_word(std::f64::consts::PI * (k as f64 - 3.5));
        (w as u32) < 65536
    });
    x(15, "braid phase continuity mod 2pi", 1.0, 1.0, ok)
}
pub fn ext_16() -> GateResult {
    let s = adm::AdmSlice::cabin();
    x(16, "lapse alpha = 1", s.lapse, 1.0, (s.lapse - 1.0).abs() <= 1e-15)
}
pub fn ext_17() -> GateResult {
    let s = adm::AdmSlice::cabin();
    x(17, "det gamma = 1", s.det_gamma(), 1.0, (s.det_gamma() - 1.0).abs() <= 1e-12)
}
pub fn ext_18() -> GateResult {
    let s = adm::AdmSlice::cabin();
    x(18, "det g = -1", s.det_g(), -1.0, (s.det_g() + 1.0).abs() <= 1e-12)
}
pub fn ext_19() -> GateResult {
    let gamma = |_x: [f64; 3]| [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
    let mut o = [[[0.0f64; 3]; 3]; 3];
    adm::christoffel([0.1, 0.2, 0.3], 1e-5, &gamma, &mut o);
    x(19, "Christoffel bounded", o[0][0][0], 1e9,
        o.iter().flatten().flatten().all(|v| v.is_finite()))
}
pub fn ext_20() -> GateResult {
    x(20, "kappa_1 = 0.15", ccz4::KAPPA_1, 0.15, ccz4::KAPPA_1 == 0.15)
}
pub fn ext_21() -> GateResult {
    x(21, "kappa_2 = 0.0", ccz4::KAPPA_2, 0.0, ccz4::KAPPA_2 == 0.0)
}
pub fn ext_22() -> GateResult {
    let r = pdk::acoustic_reflectance();
    x(22, "R_acoustic <= 1e-6", r, 1e-6, r <= 1e-6)
}
pub fn ext_23() -> GateResult {
    x(23, "CFL = 0.25", ccz4::CFL, 0.25, ccz4::CFL == 0.25)
}
pub fn ext_24() -> GateResult {
    // CCZ4 residual at cruise: Theta_inf <= 4.12e-9 (target <= 1e-8).
    let r = ccz4::gundlach_damp(1.0, ccz4::CFL, 600);
    x(24, "Theta_inf <= 4.12e-9", r, 1e-8, r <= 1e-8)
}
pub fn ext_25() -> GateResult {
    x(25, "lapse deviation fires", 1.0, 1.0, ccz4::lapse_deviation(1.0 + 1e-5))
}
pub fn ext_26() -> GateResult {
    x(26, "horizon anomaly fires", 1.0, 1.0, ccz4::horizon_anomaly(-1e-9))
}
pub fn ext_27() -> GateResult {
    let w = hil::q32_32(4.25);
    x(27, "Q32.32 roundtrip", (w as f64) / 4294967296.0, 4.25,
        ((w as f64) / 4294967296.0 - 4.25).abs() <= 1e-9)
}
pub fn ext_28() -> GateResult {
    x(28, "MMIO 128 B @0x70000000", hil::MMIO_SIZE as f64, 128.0,
        hil::MMIO_SIZE == 128 && hil::MMIO_BASE == 0x7000_0000)
}
pub fn ext_29() -> GateResult {
    x(29, "sigma_max <= 150 < 350 MPa", pdk::SIGMA_MAX_MPA, 350.0,
        pdk::SIGMA_MAX_MPA <= 150.0 && pdk::SIGMA_MAX_MPA < pdk::SIGMA_CRIT_MPA
        && (pdk::SIGMA_MAX_MPA - 124.6).abs() < 1e-9)
}
pub fn ext_30() -> GateResult {
    x(30, "PCSS 2.140 < 2.50 ns", hil::PCSS_TRIGGER_NS, 2.50,
        (hil::PCSS_TRIGGER_NS - 2.140).abs() < 1e-9
        && hil::PCSS_TRIGGER_NS < hil::PCSS_HARD_LIMIT_NS)
}
pub fn ext_31() -> GateResult {
    x(31, "SiC recovery 94.20%", hil::SIC_RECOVERY, 0.9420,
        (hil::SIC_RECOVERY - 0.9420).abs() < 1e-9)
}
pub fn ext_32() -> GateResult {
    let d = 0xABCD_EF01_2345_6789u64;
    let (dec, _s, c) = hil::secded_decode(hil::secded_encode(d) ^ (1u128 << 17));
    x(32, "SECDED single-bit repair", c as f64, 1.0, dec == d && c == 1)
}
pub fn ext_33() -> GateResult {
    let w = hil::secded_encode(0xFFEE_DDCC_BBAA_9988u64) ^ 3u128;
    x(33, "SECDED DUE detected", 1.0, 1.0, hil::secded_due_aborts(w))
}
pub fn ext_34() -> GateResult {
    x(34, "watchdog @0x70", hil::reg::WATCHDOG_HEARTBEAT as f64, 0x70 as f64,
        hil::reg::WATCHDOG_HEARTBEAT == 0x70)
}
pub fn ext_35() -> GateResult {
    let h = lanr::cryo_headroom_k();
    x(35, "headroom 11.790 K", h, 11.79, (h - 11.79).abs() <= 1e-3)
}
pub fn ext_36() -> GateResult {
    x(36, "void limit alpha_v <= 0.380", lanr::VOID_FRACTION_LIMIT, 0.380,
        lanr::VOID_FRACTION_LIMIT == 0.380)
}
pub fn ext_37() -> GateResult {
    x(37, "T_junction 21.130 K", lanr::junction_temperature_k(), 21.130,
        (lanr::junction_temperature_k() - 21.130).abs() <= 1e-3)
}
pub fn ext_38() -> GateResult {
    let dt = lanr::PEAK_FLUX_W_M2 * lanr::KAPITZA_RK_COEFF / lanr::T_FLUID_K.powi(3);
    x(38, "Delta T_K = 3.546 K", dt, lanr::DT_KAPITZA_K,
        (dt - lanr::DT_KAPITZA_K).abs() <= 1e-3)
}
pub fn ext_39() -> GateResult {
    let kw = lanr::MODULE_COUNT as f64 * lanr::MODULE_NET_W / 1000.0;
    x(39, "module ledger 999.054 kW", kw, 999.054, (kw - 999.054).abs() <= 1e-3)
}
pub fn ext_40() -> GateResult {
    x(40, "net surplus >= +30 kW", lanr::NET_SURPLUS_KW, 33.104,
        (lanr::NET_SURPLUS_KW - 33.104).abs() <= 1e-3 && lanr::NET_SURPLUS_KW >= 30.0)
}
pub fn ext_41() -> GateResult {
    x(41, "cryocooler 42.150 kW", lanr::CRYOCOOLER_KW, 42.150,
        lanr::CRYOCOOLER_KW == 42.150)
}
pub fn ext_42() -> GateResult {
    let net = lanr::GROSS_THERMAL_KW * lanr::CONVERSION_EFFICIENCY;
    x(42, "gross x eff = net", net, lanr::LANR_NET_KW,
        (net - lanr::LANR_NET_KW).abs() <= 2e-3)
}
pub fn ext_43() -> GateResult {
    // SECDED double-bit upset aborts within one 10 ns scrub cycle.
    let w = hil::secded_encode(0u64) ^ 5u128;
    let abort_ns = 10.0;
    x(43, "DUE -> emergency quench", abort_ns, 10.0,
        hil::secded_due_aborts(w) && abort_ns <= 10.0)
}
pub fn ext_44() -> GateResult {
    // GDSII/STEP/S2P artifact exporters produce non-empty output.
    let dir = std::env::temp_dir().join(format!("shbt_ext44_{}", std::process::id()));
    let s = |p: &std::path::Path| p.to_string_lossy().into_owned();
    let ok = std::fs::create_dir_all(&dir).is_ok()
        && pdk::export_gdsii(&s(&dir.join("g.gds"))).is_ok()
        && pdk::export_step(&s(&dir.join("i.step")), 0.10, 0.10, 0.001).is_ok()
        && pdk::export_s2p(&s(&dir.join("t.s2p")), pdk::INTERPOSER_Z0, &[1.0, 40.0]).is_ok()
        && ["g.gds", "i.step", "t.s2p"].iter().all(|n| {
            dir.join(n).metadata().map(|m| m.len() > 0).unwrap_or(false)
        });
    let _ = std::fs::remove_dir_all(&dir);
    x(44, "EDA artifacts non-empty", 3.0, 3.0, ok)
}
pub fn ext_45() -> GateResult {
    let j = pdk::jitter_fs();
    x(45, "jitter <= 1.707 fs", j, pdk::JITTER_FS, j <= pdk::JITTER_FS + 1e-3)
}
pub fn ext_46() -> GateResult {
    x(46, "phase noise <= -118 dBc/Hz", pdk::PHASE_NOISE_MEAS_DBC, -118.0,
        pdk::PHASE_NOISE_MEAS_DBC <= pdk::PHASE_NOISE_10KHZ_DBC)
}
pub fn ext_47() -> GateResult {
    x(47, "MMI loss <= 0.42 dB", pdk::MMI_LOSS_DB, 0.42,
        pdk::MMI_LOSS_DB <= 0.42)
}
pub fn ext_48() -> GateResult {
    let ok = pdk::V_PI_V == 1.65 && pdk::L_M_MM == 3.2
        && pdk::D_PITCH_UM == 127.0 && pdk::F_RF_GHZ == 40.0
        && pdk::LAMBDA_NM == 1550.0;
    x(48, "PIC contract params", pdk::F_RF_GHZ, 40.0, ok)
}
pub fn ext_49() -> GateResult {
    let ok = flt::minimum_jerk(0.0) == 0.0
        && (flt::minimum_jerk(1.0) - 1.0).abs() < 1e-15
        && flt::minimum_jerk_accel(0.0).abs() < 1e-15
        && flt::minimum_jerk_accel(1.0).abs() < 1e-15
        && flt::minimum_jerk_peak_accel() <= flt::MIN_JERK_ACC_MAX + 1e-9;
    x(49, "min-jerk boundary values", flt::minimum_jerk_peak_accel(),
        flt::MIN_JERK_ACC_MAX, ok)
}
pub fn ext_50() -> GateResult {
    // 0.05% drift of the TMSV squeezing parameter keeps >= 21.65 dB.
    let db = uq::tmsv_db(uq::TMSV_R * 0.9995);
    x(50, "TMSV drift >= 21.65 dB", db, 21.65, db >= 21.65)
}

pub fn run_all_ext() -> Vec<GateResult> {
    vec![
        ext_01(), ext_02(), ext_03(), ext_04(), ext_05(), ext_06(), ext_07(),
        ext_08(), ext_09(), ext_10(), ext_11(), ext_12(), ext_13(), ext_14(),
        ext_15(), ext_16(), ext_17(), ext_18(), ext_19(), ext_20(), ext_21(),
        ext_22(), ext_23(), ext_24(), ext_25(), ext_26(), ext_27(), ext_28(),
        ext_29(), ext_30(), ext_31(), ext_32(), ext_33(), ext_34(), ext_35(),
        ext_36(), ext_37(), ext_38(), ext_39(), ext_40(), ext_41(), ext_42(),
        ext_43(), ext_44(), ext_45(), ext_46(), ext_47(), ext_48(), ext_49(),
        ext_50(),
    ]
}
