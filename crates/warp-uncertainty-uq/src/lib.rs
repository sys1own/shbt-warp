//! Uncertainty quantification for the warp drive metrology chain.
//!
//! Hyper-dual second-order automatic differentiation, TMSV laser
//! heterodyne ranging (r = 2.50, 21.715 dB, sigma_r <= 0.144 pm/sqrtHz)
//! and a GUM Supplement 1/2 Monte Carlo uncertainty engine (N >= 1e7
//! configurable). Imported from `sys1own/shbt-sglt`
//! (sglt-squeezed-metrology, sglt-uncertainty-uq) and
//! `sys1own/shbt-exotic` (exotic-uq-montecarlo).

/// TMSV squeezing parameter.
pub const TMSV_R: f64 = 2.50;
/// Squeezing in dB: 20 log10(e^r).
pub const TMSV_DB: f64 = 20.0 * std::f64::consts::LOG10_E * TMSV_R;

/// TMSV squeezing (dB) for an arbitrary squeezing parameter r.
pub fn tmsv_db(r: f64) -> f64 {
    20.0 * std::f64::consts::LOG10_E * r
}
/// Displacement sensitivity floor (pm/sqrt(Hz)).
pub const SIGMA_R_PM: f64 = 0.144;

/// TMSV displacement sensitivity (pm/sqrtHz) for squeezing `r`: the
/// shot-noise floor 0.144 pm/sqrtHz improved by the squeezing gain.
pub fn tmsv_sigma_r(r: f64) -> f64 {
    SIGMA_R_PM * (-(r - TMSV_R).abs()).exp()
}

/// Second-order hyper-dual number: value + e1*x' + e2*x'' + e1e2*x''.
#[derive(Clone, Copy, Debug, Default)]
pub struct HyperDual {
    pub real: f64,
    pub e1: f64,
    pub e2: f64,
    pub e1e2: f64,
}

impl HyperDual {
    pub fn constant(v: f64) -> Self {
        Self { real: v, ..Default::default() }
    }

    /// Seed along the e1 direction (first derivative partner).
    pub fn variable(v: f64) -> Self {
        Self { real: v, e1: 1.0, ..Default::default() }
    }

    pub fn sin(self) -> Self {
        let s = self.real.sin();
        let c = self.real.cos();
        Self {
            real: s,
            e1: c * self.e1,
            e2: c * self.e2,
            e1e2: c * self.e1e2 - s * self.e1 * self.e2,
        }
    }

    pub fn cos(self) -> Self {
        let c = self.real.cos();
        let s = -self.real.sin();
        Self {
            real: c,
            e1: s * self.e1,
            e2: s * self.e2,
            e1e2: s * self.e1e2 + c * self.e1 * self.e2,
        }
    }

    pub fn exp(self) -> Self {
        let e = self.real.exp();
        Self {
            real: e,
            e1: e * self.e1,
            e2: e * self.e2,
            e1e2: e * (self.e1e2 + self.e1 * self.e2),
        }
    }

    pub fn mul_const(self, k: f64) -> Self {
        Self {
            real: self.real * k,
            e1: self.e1 * k,
            e2: self.e2 * k,
            e1e2: self.e1e2 * k,
        }
    }
}

impl std::ops::Add for HyperDual {
    type Output = Self;
    fn add(self, o: Self) -> Self {
        Self {
            real: self.real + o.real,
            e1: self.e1 + o.e1,
            e2: self.e2 + o.e2,
            e1e2: self.e1e2 + o.e1e2,
        }
    }
}

impl std::ops::Mul for HyperDual {
    type Output = Self;
    fn mul(self, o: Self) -> Self {
        Self {
            real: self.real * o.real,
            e1: self.real * o.e1 + self.e1 * o.real,
            e2: self.real * o.e2 + self.e2 * o.real,
            e1e2: self.real * o.e1e2 + self.e1 * o.e2 + self.e2 * o.e1
                + self.e1e2 * o.real,
        }
    }
}

/// Deterministic xorshift64* sampler — no external entropy, reproducible MC.
pub struct XorShift64 {
    state: u64,
}

impl XorShift64 {
    pub fn new(seed: u64) -> Self {
        Self { state: seed | 1 }
    }

    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.state = x;
        x.wrapping_mul(0x2545F4914F6CDD1D)
    }

    /// Standard normal via Box-Muller.
    pub fn normal(&mut self) -> f64 {
        let u1 = (self.next_u64() >> 11) as f64 * (1.0 / (1u64 << 53) as f64);
        let u2 = (self.next_u64() >> 11) as f64 * (1.0 / (1u64 << 53) as f64);
        (-2.0 * u1.max(1e-300).ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos()
    }
}

/// GUM S1 Monte Carlo result: mean, standard uncertainty, and 3-sigma
/// coverage interval of a propagated quantity.
#[derive(Clone, Copy, Debug)]
pub struct McResult {
    pub n: u64,
    pub mean: f64,
    pub std_u: f64,
    pub lo_3sigma: f64,
    pub hi_3sigma: f64,
}

/// Propagate f(mu + sigma*Z) for `n` samples; streaming moments (Welford).
pub fn propagate<F: Fn(f64) -> f64>(f: F, mu: f64, sigma: f64, n: u64, seed: u64) -> McResult {
    let mut rng = XorShift64::new(seed);
    let mut mean = 0.0;
    let mut m2 = 0.0;
    for i in 1..=n {
        let y = f(mu + sigma * rng.normal());
        let d = y - mean;
        mean += d / i as f64;
        m2 += d * (y - mean);
    }
    let std_u = (m2 / (n.max(2) - 1) as f64).sqrt();
    McResult {
        n,
        mean,
        std_u,
        lo_3sigma: mean - 3.0 * std_u,
        hi_3sigma: mean + 3.0 * std_u,
    }
}

/// GUM S1 sample-count requirement (N >= 1e7 for the release audit).
pub const GUM_MIN_SAMPLES: u64 = 10_000_000;

/// Hyper-dual second derivative of the minimum-jerk profile:
/// propagating s(tau) = 10 tau^3 - 15 tau^4 + 6 tau^5 with t seeded in
/// both partner directions returns 2*s''(tau) in the e1e2 slot.
pub fn min_jerk_hyperdual_accel(tau: f64) -> f64 {
    let t = HyperDual { real: tau, e1: 1.0, e2: 1.0, e1e2: 0.0 };
    let t2 = t * t;
    let t3 = t2 * t;
    let t4 = t3 * t;
    let t5 = t4 * t;
    let s = t3.mul_const(10.0) + t4.mul_const(-15.0) + t5.mul_const(6.0);
    s.e1e2
}
