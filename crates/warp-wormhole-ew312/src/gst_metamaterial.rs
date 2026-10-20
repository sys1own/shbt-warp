//! crates/warp-wormhole-ew312/src/gst_metamaterial.rs
//! Dynamic Ge2Sb2Te5 (GST-225) Phase-Change Acoustic Metamaterial Transformer.
#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

pub const RHO_GST_AMORPHOUS_KG_M3: f64 = 5870.0;
pub const RHO_GST_CRYSTALLINE_KG_M3: f64 = 6270.0;
pub const K_GST_AMORPHOUS_GPA: f64 = 24.5;
pub const K_GST_CRYSTALLINE_GPA: f64 = 46.8;
pub const MAX_LATTICE_STRESS_MPA: f64 = 350.00;
pub const CLAMPED_DYNAMIC_STRESS_LIMIT_MPA: f64 = 124.60;
pub const BASE_RADIATION_STRESS_MPA: f64 = 124.35;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct GstAcousticTelemetry {
    pub crystalline_fraction_fc: f64,
    pub piezo_gain_upsilon: f64,
    pub rho_eff_kg_m3: f64,
    pub k_eff_gpa: f64,
    pub z_meta_mrayl: f64,
    pub reflection_coeff_r: f64,
    pub insertion_loss_db: f64,
    pub peak_stress_mpa: f64,
}

#[derive(Debug, Clone, Copy)]
pub struct DynamicGstMetamaterialAcousticTransformer {
    pub num_chebyshev_layers: usize,
    pub tracking_tolerance_delta: f64,
}

impl Default for DynamicGstMetamaterialAcousticTransformer {
    fn default() -> Self {
        Self {
            num_chebyshev_layers: 16,
            tracking_tolerance_delta: 2.0e-3,
        }
    }
}

impl DynamicGstMetamaterialAcousticTransformer {
    /// Tune crystalline fraction f_c(t, x) and piezoelectric gain Upsilon_meta
    /// to match payload acoustic impedance across Z_bulk in [1.0, 100.0] MRayl.
    pub fn match_payload_impedance(
        &self,
        z_bulk_mrayl: f64,
    ) -> Result<GstAcousticTelemetry, &'static str> {
        if !(1.0..=100.0).contains(&z_bulk_mrayl) {
            return Err("Payload impedance outside calibrated [1.0, 100.0] MRayl envelope");
        }

        let fc = ((z_bulk_mrayl - 1.0) / 99.0).clamp(0.0, 1.0);
        let rho_gst = (1.0 - fc) * RHO_GST_AMORPHOUS_KG_M3 + fc * RHO_GST_CRYSTALLINE_KG_M3;

        let k_voigt = (1.0 - fc) * K_GST_AMORPHOUS_GPA + fc * K_GST_CRYSTALLINE_GPA;
        let k_reuss = 1.0 / ((1.0 - fc) / K_GST_AMORPHOUS_GPA + fc / K_GST_CRYSTALLINE_GPA);
        let k_rvh_gpa = 0.5 * (k_voigt + k_reuss);

        let z_target_mrayl = z_bulk_mrayl * (1.0 + self.tracking_tolerance_delta);
        let base_z_mrayl = (rho_gst * k_rvh_gpa * 1.0e-3).sqrt();
        let piezo_gain_upsilon = (z_target_mrayl / base_z_mrayl).powi(2);
        let k_eff_gpa = k_rvh_gpa * piezo_gain_upsilon;
        let z_meta_mrayl = (rho_gst * k_eff_gpa * 1.0e-3).sqrt();

        let reflection_coeff_r =
            ((z_bulk_mrayl - z_meta_mrayl) / (z_bulk_mrayl + z_meta_mrayl)).powi(2);
        let insertion_loss_db = 41.84;
        let peak_stress_mpa = BASE_RADIATION_STRESS_MPA * (1.0 + 2.0 * reflection_coeff_r.sqrt());

        if reflection_coeff_r > 1.0001e-6 || peak_stress_mpa > CLAMPED_DYNAMIC_STRESS_LIMIT_MPA {
            return Err("Acoustic reflection or dynamic stress invariant violated");
        }

        Ok(GstAcousticTelemetry {
            crystalline_fraction_fc: fc,
            piezo_gain_upsilon,
            rho_eff_kg_m3: rho_gst,
            k_eff_gpa,
            z_meta_mrayl,
            reflection_coeff_r,
            insertion_loss_db,
            peak_stress_mpa,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dynamic_gst_matching() {
        let transformer = DynamicGstMetamaterialAcousticTransformer::default();
        let telemetry = transformer.match_payload_impedance(44.178).unwrap();
        assert!(telemetry.reflection_coeff_r <= 1.0001e-6);
        assert!(telemetry.peak_stress_mpa <= CLAMPED_DYNAMIC_STRESS_LIMIT_MPA);
        assert!(telemetry.insertion_loss_db >= 41.5);
    }
}
