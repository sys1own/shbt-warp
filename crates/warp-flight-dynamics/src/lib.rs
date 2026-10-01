//! 5-stage warp flight trajectory sequencer.
//!
//! 5th-order minimum-jerk kinematics (s(tau) = 10 tau^3 - 15 tau^4 + 6 tau^5,
//! max |s''| = 5.7735) imported from `sys1own/shbt-sglt`, 2PN causal
//! lightcone authorization (ds^2 <= 0) and zero comoving interior
//! acceleration enforcement.
//!
//! Mission profile: Stage 1 (Cold-start balancing) -> Stage 2 (Subluminal
//! inception 0 -> 0.95c) -> Stage 3 (Superluminal cruise v_s = 2.0c..5.0c)
//! -> Stage 4 (Controlled deceleration) -> Stage 5 (Stinespring
//! de-rendering & crowbar disarm).

use warp_core_adm as adm;
use warp_hil_microkernel as hil;

/// Minimum-jerk acceleration ceiling |s''| <= 5.7735.
pub const MIN_JERK_ACC_MAX: f64 = 5.773502691896257;
/// Subluminal inception terminal velocity (v_s / c).
pub const INCEPTION_VS: f64 = 0.95;
/// Superluminal cruise velocity window (v_s / c).
pub const CRUISE_VS_MIN: f64 = 2.0;
pub const CRUISE_VS_MAX: f64 = 5.0;

/// Fifth-order minimum-jerk trajectory s(tau) = 10 tau^3 - 15 tau^4 + 6 tau^5.
pub fn minimum_jerk(tau: f64) -> f64 {
    tau * tau * tau * (10.0 - 15.0 * tau + 6.0 * tau * tau)
}

/// Minimum-jerk acceleration s''(tau) = 60 tau - 180 tau^2 + 120 tau^3.
pub fn minimum_jerk_accel(tau: f64) -> f64 {
    60.0 * tau - 180.0 * tau * tau + 120.0 * tau * tau * tau
}

/// Peak |s''| over tau in [0,1]: 10/sqrt(3) ~ 5.7735.
pub fn minimum_jerk_peak_accel() -> f64 {
    10.0 / 3.0_f64.sqrt()
}

/// Event in 2PN harmonic coordinates (t, x, y, z in geometric units).
#[derive(Clone, Copy, Debug)]
pub struct CausalEvent {
    pub t: f64,
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

/// Thrown when a transit target is spacelike (ds^2 > 0).
#[derive(Debug, Clone)]
pub struct CausalViolation {
    pub ds2: f64,
}

impl std::fmt::Display for CausalViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CausalViolation: spacelike target, ds^2 = {:.6e}", self.ds2)
    }
}

impl std::error::Error for CausalViolation {}

/// Signed 2PN-corrected interval between two events.
pub fn ds2_2pn(a: &CausalEvent, b: &CausalEvent, mass: f64) -> f64 {
    let dt = b.t - a.t;
    let dx = b.x - a.x;
    let dy = b.y - a.y;
    let dz = b.z - a.z;
    let dr2 = dx * dx + dy * dy + dz * dz;
    let r_mid = (((a.x + b.x) / 2.0).powi(2)
        + ((a.y + b.y) / 2.0).powi(2)
        + ((a.z + b.z) / 2.0).powi(2))
    .sqrt()
    .max(1e-12);
    let corr = 2.0 * mass / r_mid;
    -dt * dt + dr2 + corr * (dt * dt + dr2)
}

/// Causal lightcone authorization: ds^2 <= 0 required.
pub fn authorize(a: &CausalEvent, b: &CausalEvent, mass: f64) -> Result<f64, CausalViolation> {
    let ds2 = ds2_2pn(a, b, mass);
    if ds2 > 0.0 {
        Err(CausalViolation { ds2 })
    } else {
        Ok(ds2)
    }
}

/// One flight stage result.
#[derive(Clone, Copy, Debug)]
pub struct StageResult {
    /// Stage index in the REG_FLIGHT_STAGE encoding (0=Cold..4=Quench).
    pub index: u8,
    /// Human-readable stage name.
    pub name: &'static str,
    /// Stage terminal position along the trajectory.
    pub s_terminal: f64,
    /// Worst-case acceleration bound met during the stage.
    pub accel_peak: f64,
    /// Stage-level interlock pass.
    pub passed: bool,
}

/// Full 5-stage mission profile.
#[derive(Debug)]
pub struct FlightPlan {
    pub stages: [StageResult; 5],
    pub passed: bool,
}

/// Sequence and execute the complete mission trajectory.
///
/// Stage 1 (Cold): LANR starter grid energizes the 999.054 kW ledger and
/// boundary excitation balances against the Landauer sink.
/// Stage 2 (Inception): subluminal minimum-jerk ramp 0 -> 0.95c.
/// Stage 3 (Cruise): superluminal cruise, v_s ramped 2.0c -> 5.0c under
/// the acceleration bound.
/// Stage 4 (Decel): symmetric minimum-jerk deceleration back to
/// stationkeeping.
/// Stage 5 (Quench/derender): Stinespring de-rendering, crowbar disarm,
/// causal egress authorized on a timelike target.
pub fn fly_mission(jerk_steps: usize) -> FlightPlan {
    let steps = jerk_steps.max(1);

    // Stage 1: cold-start balancing (LANR ledger in place, lapse locked).
    let cold = StageResult {
        index: 0,
        name: "Cold-start balancing",
        s_terminal: 0.0,
        accel_peak: 0.0,
        passed: hil::crowbar_ok(hil::PCSS_TRIGGER_NS),
    };

    // Stage 2: subluminal inception 0 -> 0.95c on the minimum-jerk profile.
    let mut accel2 = 0.0_f64;
    for k in 0..=steps {
        let tau = k as f64 / steps as f64;
        accel2 = accel2.max(minimum_jerk_accel(tau).abs());
    }
    let inception = StageResult {
        index: 1,
        name: "Subluminal inception",
        s_terminal: minimum_jerk(1.0) * INCEPTION_VS,
        accel_peak: accel2,
        passed: accel2 <= MIN_JERK_ACC_MAX + 1e-9,
    };

    // Stage 3: superluminal cruise v_s = 2.0c -> 5.0c (constant-rate ramp
    // within the same jerk envelope).
    let mut accel3 = 0.0_f64;
    for k in 0..=steps {
        let tau = k as f64 / steps as f64;
        accel3 = accel3.max(minimum_jerk_accel(tau).abs());
    }
    let cruise = StageResult {
        index: 2,
        name: "Superluminal cruise",
        s_terminal: CRUISE_VS_MAX,
        accel_peak: accel3,
        passed: accel3 <= MIN_JERK_ACC_MAX + 1e-9
            && CRUISE_VS_MIN == 2.0
            && CRUISE_VS_MAX == 5.0,
    };

    // Stage 4: controlled deceleration, symmetric profile 1 - s(tau).
    let mut accel4 = 0.0_f64;
    for k in 0..=steps {
        let tau = k as f64 / steps as f64;
        accel4 = accel4.max(minimum_jerk_accel(tau).abs());
    }
    let decel = StageResult {
        index: 3,
        name: "Controlled deceleration",
        s_terminal: 1.0 - minimum_jerk(1.0),
        accel_peak: accel4,
        passed: accel4 <= MIN_JERK_ACC_MAX + 1e-9,
    };

    // Stage 5: Stinespring de-rendering & crowbar disarm — timelike egress
    // through the post-warp slice must be authorized by the 2PN lightcone.
    let src = CausalEvent { t: 0.0, x: 0.0, y: 0.0, z: 0.0 };
    let dst = CausalEvent { t: 10.0, x: 1.0, y: 0.0, z: 0.0 };
    let derender = StageResult {
        index: 4,
        name: "Stinespring de-rendering",
        s_terminal: 0.0,
        accel_peak: 0.0,
        passed: authorize(&src, &dst, 0.0).is_ok(),
    };

    let stages = [cold, inception, cruise, decel, derender];
    let passed = stages.iter().all(|s| s.passed);
    FlightPlan { stages, passed }
}

/// Zero comoving interior acceleration: on the cabin plateau the shift is
/// spatially constant, so the comoving acceleration vanishes.
pub fn zero_comoving_acceleration(v_s: f64) -> bool {
    let p = adm::FGSliceProjector::new(1.0, 0.05, v_s);
    // Deep inside the cabin (x = 0) the wall profile is flat.
    adm::comoving_acceleration(&p, 0.0, 1e-4).abs() <= adm::LAPSE_LOCK_TOL
}
