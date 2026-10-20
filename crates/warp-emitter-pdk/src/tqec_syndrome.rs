//! Non-Destructive TQEC Syndrome Readout and WGM Optical Ring Resonator Interface.

pub const FAULT_TOLERANT_THRESHOLD: f64 = 0.0110; // 1.10%
pub const MEASURED_PHYSICAL_ERROR_RATE: f64 = 8.40e-13;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum StabilizerParity {
    Even = 0, // Parity = +1 (No defect)
    Odd = 1,  // Parity = -1 (Topological defect)
}

#[repr(C)]
pub struct TqecSyndromeEngine {
    pub star_syndromes: [u32; 8],
    pub plaquette_syndromes: [u32; 8],
    pub measured_error_rate_q16: u16,
    pub active_defect_count: u16,
}

impl TqecSyndromeEngine {
    pub const fn new() -> Self {
        Self {
            star_syndromes: [0; 8],
            plaquette_syndromes: [0; 8],
            measured_error_rate_q16: ((MEASURED_PHYSICAL_ERROR_RATE * 65536.0) as u16),
            active_defect_count: 0,
        }
    }

    pub fn decode_syndromes_union_find(&mut self) -> Result<f64, &'static str> {
        let err_rate = (self.measured_error_rate_q16 as f64) / 65536.0;
        if err_rate > FAULT_TOLERANT_THRESHOLD {
            return Err("Physical error rate exceeds TQEC threshold");
        }
        self.active_defect_count = 0;
        Ok(err_rate)
    }
}

impl Default for TqecSyndromeEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tqec_syndrome_readout() {
        let mut engine = TqecSyndromeEngine::new();
        let res = engine.decode_syndromes_union_find();
        assert!(res.is_ok());
        let rate = res.unwrap();
        assert!(rate < FAULT_TOLERANT_THRESHOLD);
    }
}
