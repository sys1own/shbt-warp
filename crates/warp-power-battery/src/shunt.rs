//! crates/warp-power-battery/src/shunt.rs
//! Heegaard–Floer Symplectic Landauer Shunting & Cryogenic Thermal Envelopes.
#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

/// Canonical suppression factor S = (10/33)^2 = 100/1089 ~ 0.09182736.
pub const SYMPLECTIC_SUPPRESSION_S: f64 = 100.0 / 1089.0;
/// Dark-ledger entropy rejection fraction: 1 - S = 989/1089 ~ 0.90817264 (90.82%).
pub const DARK_LEDGER_REJECTION_FRAC: f64 = 989.0 / 1089.0;
/// Anomalous Borrmann transmission efficiency.
pub const BORRMANN_EFFICIENCY: f64 = 0.9852;
/// SMES inductive energy recovery efficiency.
pub const SMES_RECOVERY_EFFICIENCY: f64 = 0.9420;
/// Net sensible heat fraction:
/// f_sensible = S * (1 - eps_B) * (1 - eta_SMES) = (100/1089) * 0.0148 * 0.0580 = 7.88246e-5.
pub const SENSIBLE_HEAT_FRACTION: f64 =
    SYMPLECTIC_SUPPRESSION_S * (1.0 - BORRMANN_EFFICIENCY) * (1.0 - SMES_RECOVERY_EFFICIENCY);

/// Nominal cryogenic operating temperature (K).
pub const BASELINE_CRYO_TEMP_K: f64 = 21.13;
/// Critical quench temperature ceiling (K).
pub const CRITICAL_TEMP_LIMIT_K: f64 = 32.92;
/// Peak surge allowable temperature (K).
pub const PEAK_SURGE_TEMP_LIMIT_K: f64 = 28.40;
/// Maximum supercritical LHe mass flow rate (kg/s).
pub const MAX_LHE_MASS_FLOW_KG_S: f64 = 112.5;
/// Supercritical liquid helium specific heat capacity c_p (J/(kg*K)).
pub const LHE_SPECIFIC_HEAT_J_KG_K: f64 = 5200.0;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct LandauerShuntTelemetry {
    pub gross_landauer_power_w: f64,
    pub dark_ledger_shunted_w: f64,
    pub sensible_heat_w: f64,
    pub operating_temp_k: f64,
    pub lhe_mass_flow_kg_s: f64,
}

/// Evaluate the Heegaard–Floer symplectic Landauer shunt under specified gross Landauer debt power
/// and supercritical LHe mass flow rate.
pub fn evaluate_symplectic_landauer_shunt(
    gross_landauer_power_w: f64,
    lhe_mass_flow_kg_s: f64,
) -> Result<LandauerShuntTelemetry, &'static str> {
    if lhe_mass_flow_kg_s <= 0.0 || lhe_mass_flow_kg_s > MAX_LHE_MASS_FLOW_KG_S {
        return Err("LHe mass flow rate outside (0.0, 112.5] kg/s operating envelope");
    }

    let dark_ledger_shunted_w = gross_landauer_power_w * DARK_LEDGER_REJECTION_FRAC;
    let sensible_heat_w = gross_landauer_power_w * SENSIBLE_HEAT_FRACTION;
    let delta_t_k = sensible_heat_w / (lhe_mass_flow_kg_s * LHE_SPECIFIC_HEAT_J_KG_K);
    let operating_temp_k = BASELINE_CRYO_TEMP_K + delta_t_k;

    if operating_temp_k > CRITICAL_TEMP_LIMIT_K {
        return Err("Cryogenic operating temperature exceeded 32.92 K ceiling");
    }

    Ok(LandauerShuntTelemetry {
        gross_landauer_power_w,
        dark_ledger_shunted_w,
        sensible_heat_w,
        operating_temp_k,
        lhe_mass_flow_kg_s,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shunt_rejection_and_fractions() {
        assert!((SYMPLECTIC_SUPPRESSION_S - 100.0 / 1089.0).abs() < 1e-12);
        assert!((DARK_LEDGER_REJECTION_FRAC - 989.0 / 1089.0).abs() < 1e-12);
        assert!((SENSIBLE_HEAT_FRACTION - 7.882460973370064e-5).abs() < 1e-10);
    }

    #[test]
    fn test_cryo_operating_envelope() {
        let telem = evaluate_symplectic_landauer_shunt(1.5e6, 112.5).unwrap();
        assert!(telem.operating_temp_k <= PEAK_SURGE_TEMP_LIMIT_K);
        assert!(telem.operating_temp_k < CRITICAL_TEMP_LIMIT_K);
    }
}
