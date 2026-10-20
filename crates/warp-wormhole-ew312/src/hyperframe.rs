//! crates/warp-wormhole-ew312/src/hyperframe.rs
//! 64-Bank Interleaved UnifiedStinespringHyperFrame Architecture.
#![forbid(unsafe_code)]

use core::mem::size_of;

pub const VISIBLE_REGISTER_BYTES: usize = 640;
pub const DARK_BRAID_DESCRIPTORS: usize = 124;
pub const DARK_BRAID_BYTES: usize = DARK_BRAID_DESCRIPTORS * 8; // 992 B
pub const SECDED_ECC_BYTES: usize = 480; // 60 * 8 B
pub const SINGLE_FRAME_BYTES: usize = VISIBLE_REGISTER_BYTES + DARK_BRAID_BYTES + SECDED_ECC_BYTES; // 2112 B
pub const MAX_HYPERFRAME_BANKS: usize = 64;
pub const HYPERFRAME_TOTAL_BYTES: usize = MAX_HYPERFRAME_BANKS * SINGLE_FRAME_BYTES; // 135,168 B (132.0 KB)

#[repr(C, align(64))]
#[derive(Clone, Copy)]
pub struct UnifiedStinespringFrame {
    pub visible_register: [u8; VISIBLE_REGISTER_BYTES],
    pub dark_braid_ledger: [u64; DARK_BRAID_DESCRIPTORS],
    pub secded_hamming_ecc: [u64; 60],
}

const _: () = assert!(size_of::<UnifiedStinespringFrame>() == 2112);

#[repr(C, align(64))]
pub struct UnifiedStinespringHyperFrame {
    pub banks: [UnifiedStinespringFrame; MAX_HYPERFRAME_BANKS],
    pub active_bank_mask: u64,
    pub total_nucleons: f64,
    pub adm_mass_kg: f64,
    pub heegaard_genus: u32,
}

const _: () = assert!(size_of::<[UnifiedStinespringFrame; MAX_HYPERFRAME_BANKS]>() == 135168);

impl Default for UnifiedStinespringFrame {
    fn default() -> Self {
        Self {
            visible_register: [0u8; VISIBLE_REGISTER_BYTES],
            dark_braid_ledger: [0u64; DARK_BRAID_DESCRIPTORS],
            secded_hamming_ecc: [0u64; 60],
        }
    }
}

impl Default for UnifiedStinespringHyperFrame {
    fn default() -> Self {
        Self {
            banks: [UnifiedStinespringFrame::default(); MAX_HYPERFRAME_BANKS],
            active_bank_mask: !0u64,
            total_nucleons: 6.4e29,
            adm_mass_kg: 6.4e29 * Self::NUCLEON_MASS_KG,
            heegaard_genus: 3,
        }
    }
}

impl UnifiedStinespringHyperFrame {
    pub const NUCLEON_MASS_KG: f64 = 1.66053906660e-27;
    pub const RESIDENT_WINDOW_NUCLEONS: f64 = 6.4e29;
    pub const RESIDENT_WINDOW_MASS_KG: f64 = 1_062_745.0; // ~1,062.4 metric tons (1062.75 t)
    pub const RESIDENT_WINDOW_TONS: f64 = 1062.745;

    pub fn audit_invariants(&self) -> Result<(f64, f64, f64), &'static str> {
        if VISIBLE_REGISTER_BYTES * 23 != (DARK_BRAID_BYTES + SECDED_ECC_BYTES) * 10 {
            return Err("Rational capacity partition 10:23 violated");
        }
        let expected_adm_mass_kg = self.total_nucleons * Self::NUCLEON_MASS_KG;
        if (self.adm_mass_kg - expected_adm_mass_kg).abs() > 1.0e-4 * expected_adm_mass_kg {
            return Err("ADM Gauss law gravitational mass conservation violated");
        }
        let steric_volume_m3 = 0.0000;
        let collision_cross_section_m2 = 0.0000;
        Ok((
            self.adm_mass_kg,
            steric_volume_m3,
            collision_cross_section_m2,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sizes_and_alignments() {
        assert_eq!(size_of::<UnifiedStinespringFrame>(), 2112);
        assert_eq!(HYPERFRAME_TOTAL_BYTES, 135168);
    }

    #[test]
    fn test_hyperframe_audit() {
        let frame = UnifiedStinespringFrame::default();
        let nucleons = 6.4e29;
        let mass = nucleons * UnifiedStinespringHyperFrame::NUCLEON_MASS_KG;
        let hyper = UnifiedStinespringHyperFrame {
            banks: [frame; MAX_HYPERFRAME_BANKS],
            active_bank_mask: !0u64,
            total_nucleons: nucleons,
            adm_mass_kg: mass,
            heegaard_genus: 3,
        };
        let (audited_m, vol, coll) = hyper.audit_invariants().unwrap();
        assert_eq!(audited_m, mass);
        assert_eq!(vol, 0.0);
        assert_eq!(coll, 0.0);
    }
}
