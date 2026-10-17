//! Physical, algebraic, and engineering constants for SHBT-EW-312
//! Effective Traversable Wormhole System (Alcubierre-Stinespring Corridor).

/// Lower bound of macroscopic nucleon register regime.
pub const N_LOCAL_MIN: f64 = 1.0e23;
/// Upper bound of macroscopic nucleon register regime (16.6 metric tons).
pub const N_LOCAL_MAX: f64 = 1.0e28;
/// Standard macroscopic benchmark payload mass (kg).
pub const STANDARD_PAYLOAD_MASS_KG: f64 = 16_600.0;
/// Standard macroscopic payload information content (bits).
pub const STANDARD_PAYLOAD_BITS: f64 = 1.25e28;

/// Unrecovered folding energy demand for 16.6 t payload (Joules): 455.75 TJ.
pub const E_FOLD_UNRECOVERED_J: f64 = 4.5575e14;
/// SMES inductive energy recovery efficiency.
pub const SMES_RECOVERY_RATIO: f64 = 0.9420;
/// Net cycle energy loss per folding dispatch (Joules): 26.4335 TJ <= 45.575 TJ.
pub const CYCLE_ENERGY_LOSS_J: f64 = E_FOLD_UNRECOVERED_J * (1.0 - SMES_RECOVERY_RATIO);

/// Folding latency on legacy Hf-178 baseline (49.945 TW net) in milliseconds.
pub const TAU_FOLD_BASELINE_MS: f64 = 9126.0;
/// Folding latency on MAR-312 Continuous Cruise (895.19 TW net) in milliseconds.
pub const TAU_FOLD_CRUISE_MS: f64 = 509.1;
/// Folding latency on MAR-312 Sustained Burst (6.805 PW net) in milliseconds.
pub const TAU_FOLD_BURST_MS: f64 = 66.98;
/// Folding latency on MAR-312 Exawatt Impulse (895.19 PW net) in milliseconds.
pub const TAU_FOLD_IMPULSE_MS: f64 = 0.508;

/// Active visible Stinespring capacity partition fraction: 10/33.
pub const ETA_A: f64 = 10.0 / 33.0;
/// Topological dark-ledger capacity partition fraction: 23/33.
pub const ETA_D: f64 = 23.0 / 33.0;

/// Number of independent topological Fibonacci anyon braid channels.
pub const FIBONACCI_BRAID_CHANNELS: usize = 124;
/// Quantum dimension of Fibonacci anyon tau: (1 + sqrt(5))/2.
pub const D_TAU: f64 = 1.618033988749895;
/// Braid capacity in nats / weight units: 124 * ln(d_tau) = 38.196601.
pub const C_BRAID_NATS: f64 = 38.196601;
/// Braid capacity in bits: 124 * log2(d_tau) = 86.085 bits.
pub const C_BRAID_BITS: f64 = 86.085;

/// Solovay-Kitaev SK-9 gate compilation error upper bound (log10 exponent = -230.1937).
pub const EPSILON_SK9_LOG10: f64 = -230.1937;
/// Blossom-V minimum-weight perfect matching TQEC decoding latency (ns).
pub const TAU_TQEC_DECODE_NS: f64 = 41.8;
/// Hard real-time TQEC decode deadline (ns).
pub const TAU_TQEC_DEADLINE_NS: f64 = 45.0;
/// Logical error probability per operational cycle.
pub const P_LOGICAL_ERROR: f64 = 8.4e-13;
/// Maximum allowable logical error threshold.
pub const P_LOGICAL_THRESHOLD: f64 = 1.0e-12;

/// Stinespring isometric trace norm preservation bound.
pub const TRACE_NORM_PRESERVATION_BOUND: f64 = 1.0e-120;
/// Approximate quantum error correction code norm ||W†W - P_code||_op.
pub const CODE_NORM_OPERATOR: f64 = 3.430e-3;
/// Code norm allowable upper limit.
pub const CODE_NORM_LIMIT: f64 = 3.5e-3;

/// Symplectic DMA bus bandwidth (Gbps).
pub const SYMPLECTIC_DMA_GBPS: f64 = 504.0;
/// Characteristic line impedance of symplectic boundary bus (Ohms).
pub const SYMPLECTIC_BUS_IMPEDANCE_OHMS: f64 = 50.12;

/// Two-mode squeezed vacuum (TMSV) squeezing parameter r.
pub const TMSV_SQUEEZING_R: f64 = 2.50;
/// Squeezing noise suppression in decibels (dB).
pub const TMSV_SQUEEZING_DB: f64 = 21.715;
/// TMSV 3-sigma displacement metrology resolution (nm).
pub const TMSV_3SIGMA_RESOLUTION_NM: f64 = 0.084;
/// TMSV spectral density limit (pm / sqrt(Hz)).
pub const TMSV_SPECTRAL_DENSITY_PM_SQRT_HZ: f64 = 0.0084;

/// Shift-sheared metric modulation constant Delta_mod.
pub const DELTA_MOD: f64 = 0.1375335;
/// Maximum jerk acceleration magnitude for 5th-order minimum jerk profile.
pub const MIN_JERK_MAX_JERK: f64 = 5.7735;

/// Mode A Stargate: dispatches supported by a single 376.99 kg core.
pub const STARGATE_DISPATCHES_PER_CORE: u64 = 743_438;
/// Mode A Stargate: cumulative payload capacity per single core (metric tons).
pub const STARGATE_CUMULATIVE_MASS_TONS: f64 = 1.234e7;
/// Mode A Stargate: sustained burst mass throughput rate (metric tons / hour).
pub const STARGATE_BURST_THROUGHPUT_TPH: f64 = 878_824.0;
/// Mode A Stargate: continuous cruise mass throughput rate (metric tons / hour).
pub const STARGATE_CRUISE_THROUGHPUT_TPH: f64 = 117_383.0;

/// Speed of light in vacuum (m/s).
pub const C_LIGHT: f64 = 299_792_458.0;
/// Gravitational constant (m^3 / kg s^2).
pub const G_CONST: f64 = 6.67430e-11;
/// Astronomical Unit in meters.
pub const AU_IN_METERS: f64 = 1.495978707e11;
/// Light-year in meters.
pub const LY_IN_METERS: f64 = 9.4607304725808e15;
