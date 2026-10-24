//! `SHBT-IBR-312` & `SHBT-IBR-312X` Dark-Sector Anyon Braid Magazine Profiles.
//!
//! Provides single-frame and multi-bank dark-braid fuel magazine profiles for both
//! `SHBT-MAR-312C` (`38` cores, `1,152.58 EJ` net electrical energy) and the upgraded
//! `SHBT-MAR-312X` (`41` cores, `41 * 376.99 + 950.00 = 16,406.59 kg <= 16,600.00 kg`,
//! `+3` extra cores and `+193.41 kg` ADM margin, storing `1,378.06 EJ` net electrical
//! energy (`+19.56%`) in a single Stinespring frame).

use serde::{Deserialize, Serialize};
use crate::compacted_nacelle::{
    DarkBraidMagazine, MAR312_BASE_CORE_MASS_KG, MAR312C_COMPACTED_NACELLE_MASS_KG,
    STINESPRING_SINGLE_FRAME_LIMIT_KG,
};
use crate::mar312x::{
    MAR312X_41_CORE_NET_ELEC_EJ, MAR312X_CORE_NET_ELEC_EJ, MAR312X_NACELLE_MASS_KG,
};

pub const MAR312C_SINGLE_FRAME_CORES: usize = 38;
pub const MAR312C_SINGLE_FRAME_WET_KG: f64 = 16_583.32;
pub const MAR312C_SINGLE_FRAME_NET_EJ: f64 = 1_152.578; // 38 * 30.3310 EJ

pub const MAR312X_SINGLE_FRAME_CORES: usize = 41;
pub const MAR312X_SINGLE_FRAME_ADM_MARGIN_KG: f64 = 193.41; // 16,600.00 - 16,406.59 kg
pub const MAR312X_EXTRA_CORES_VS_312C: usize = 3;
pub const MAR312X_SINGLE_FRAME_NET_EJ: f64 = MAR312X_41_CORE_NET_ELEC_EJ; // 1,378.05674 EJ (~1,378.06 EJ)
pub const MAR312X_ENERGY_GAIN_VS_312C_PCT: f64 = 19.563;

/// Comparative single-frame Stinespring magazine profile (`SHBT-MAR-312C` vs `SHBT-MAR-312X`).
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SingleFrameMagazineProfile {
    pub reactor_model: &'static str,
    pub dry_nacelle_mass_kg: f64,
    pub fuel_margin_below_16600_kg: f64,
    pub full_cores_stored: usize,
    pub micro_wafer_refills: u64,
    pub total_fuel_mass_kg: f64,
    pub total_wet_mass_kg: f64,
    pub adm_headroom_kg: f64,
    pub net_elec_per_core_ej: f64,
    pub total_net_elec_ej: f64,
}

impl SingleFrameMagazineProfile {
    /// Returns the legacy `SHBT-MAR-312C` single-frame profile (38 cores, 1,152.58 EJ).
    pub fn mar312c_profile() -> Self {
        let margin = STINESPRING_SINGLE_FRAME_LIMIT_KG - MAR312C_COMPACTED_NACELLE_MASS_KG;
        let cores = (margin / MAR312_BASE_CORE_MASS_KG).floor() as usize;
        let fuel_kg = (cores as f64) * MAR312_BASE_CORE_MASS_KG;
        let wet_kg = MAR312C_COMPACTED_NACELLE_MASS_KG + fuel_kg;
        Self {
            reactor_model: "SHBT-MAR-312C",
            dry_nacelle_mass_kg: MAR312C_COMPACTED_NACELLE_MASS_KG,
            fuel_margin_below_16600_kg: margin,
            full_cores_stored: cores,
            micro_wafer_refills: (cores * 10) as u64,
            total_fuel_mass_kg: fuel_kg,
            total_wet_mass_kg: wet_kg,
            adm_headroom_kg: STINESPRING_SINGLE_FRAME_LIMIT_KG - wet_kg,
            net_elec_per_core_ej: 30.3310,
            total_net_elec_ej: (cores as f64) * 30.3310,
        }
    }

    /// Returns the upgraded `SHBT-MAR-312X` single-frame profile (41 cores, 16,406.59 kg, 1,378.06 EJ).
    pub fn mar312x_profile() -> Self {
        let margin = STINESPRING_SINGLE_FRAME_LIMIT_KG - MAR312X_NACELLE_MASS_KG;
        let cores = (margin / MAR312_BASE_CORE_MASS_KG).floor() as usize;
        let fuel_kg = (cores as f64) * MAR312_BASE_CORE_MASS_KG;
        let wet_kg = MAR312X_NACELLE_MASS_KG + fuel_kg;
        Self {
            reactor_model: "SHBT-MAR-312X",
            dry_nacelle_mass_kg: MAR312X_NACELLE_MASS_KG,
            fuel_margin_below_16600_kg: margin,
            full_cores_stored: cores,
            micro_wafer_refills: (cores * 10) as u64,
            total_fuel_mass_kg: fuel_kg,
            total_wet_mass_kg: wet_kg,
            adm_headroom_kg: STINESPRING_SINGLE_FRAME_LIMIT_KG - wet_kg,
            net_elec_per_core_ej: MAR312X_CORE_NET_ELEC_EJ,
            total_net_elec_ej: (cores as f64) * MAR312X_CORE_NET_ELEC_EJ,
        }
    }
}

impl DarkBraidMagazine {
    /// Initialize Tier 1 direct magazine for `SHBT-MAR-312X` (41 cores, 410 micro-wafer refills, 1,378.06 EJ).
    pub fn new_tier1_mar312x() -> Self {
        let max_cores = Self::max_tier1_mar312x_full_cores();
        Self {
            tier: 1,
            banks_used: 1,
            total_cores_stored: max_cores as f64,
            micro_refills_remaining: (max_cores * 10) as u64,
            current_wafer_soc: 1.0,
            total_reloads: 0,
            last_reload_latency_s: 0.0,
        }
    }

    /// Single-frame payload fuel margin below 16,600.00 kg for `SHBT-MAR-312X` (`15,650.00 kg`).
    pub fn single_frame_mar312x_fuel_margin_kg() -> f64 {
        STINESPRING_SINGLE_FRAME_LIMIT_KG - MAR312X_NACELLE_MASS_KG
    }

    /// Maximum full `376.99 kg` cores storable in a single frame with `SHBT-MAR-312X` (`41` cores).
    pub fn max_tier1_mar312x_full_cores() -> usize {
        (Self::single_frame_mar312x_fuel_margin_kg() / MAR312_BASE_CORE_MASS_KG).floor() as usize
    }

    /// Tier 1 `SHBT-MAR-312X` fuel mass (`15,456.59 kg`).
    pub fn tier1_mar312x_fuel_mass_kg() -> f64 {
        (Self::max_tier1_mar312x_full_cores() as f64) * MAR312_BASE_CORE_MASS_KG
    }

    /// Tier 1 `SHBT-MAR-312X` wet mass (`16,406.59 kg <= 16,600.00 kg`).
    pub fn tier1_mar312x_wet_mass_kg() -> f64 {
        MAR312X_NACELLE_MASS_KG + Self::tier1_mar312x_fuel_mass_kg()
    }

    /// Tier 1 `SHBT-MAR-312X` net electrical energy (`1,378.05674 EJ`).
    pub fn tier1_mar312x_net_energy_ej() -> f64 {
        (Self::max_tier1_mar312x_full_cores() as f64) * MAR312X_CORE_NET_ELEC_EJ
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mar312x_41_core_single_frame_magazine() {
        let p_312c = SingleFrameMagazineProfile::mar312c_profile();
        let p_312x = SingleFrameMagazineProfile::mar312x_profile();

        assert_eq!(p_312c.full_cores_stored, 38);
        assert_eq!(p_312x.full_cores_stored, 41);
        assert_eq!(p_312x.full_cores_stored - p_312c.full_cores_stored, 3);
        assert!((p_312x.total_wet_mass_kg - 16_406.59).abs() < 1e-6);
        assert!((p_312x.adm_headroom_kg - 193.41).abs() < 1e-6);
        assert!((p_312x.total_net_elec_ej - 1_378.05674).abs() < 1e-4);

        let gain_pct = ((p_312x.total_net_elec_ej - p_312c.total_net_elec_ej) / p_312c.total_net_elec_ej) * 100.0;
        assert!((gain_pct - 19.563).abs() < 0.01);

        let mag = DarkBraidMagazine::new_tier1_mar312x();
        assert_eq!(mag.total_cores_stored, 41.0);
        assert_eq!(mag.micro_refills_remaining, 410);
    }
}
