//! Boundary emitter-array hardware synthesis and photonic PDK.
//!
//! InP/InGaAs boundary emitter RF phase synthesis, sapphire/aerogel
//! quarter-wave acoustic impedance matching (Z_sapp = 44.178 MRayl,
//! d_m = 6.395 nm, Z_m = 1.1512 MRayl), 8x8 InP PIC GDSII mask generation,
//! ISO 10303-21 STEP CAD models and Touchstone S2P RF interposer export.
//! Ported from legacy `src/shbt/emitter_array.rs` plus the EDA exporters
//! of `sys1own/shbt-ghost` / `sys1own/shbt-exotic`.

use std::fs::File;
use std::io::{self, Write};

/// Sapphire substrate acoustic impedance (MRayl).
pub const Z_SAPPHIRE_MRAYL: f64 = 44.178;
/// Silica aerogel quarter-wave matching-layer thickness (nm).
pub const AEROGEL_QW_NM: f64 = 6.395;
/// Aerogel matching-layer acoustic impedance (MRayl).
pub const Z_AEROGEL_MRAYL: f64 = 1.1512;
/// PIC array dimension.
pub const PIC_ARRAY: usize = 8;
/// Interposer characteristic impedance target and tolerance (ohm).
pub const INTERPOSER_Z0: f64 = 50.12;
pub const INTERPOSER_Z0_TOL: f64 = 0.80;
/// Interposer layer count.
pub const INTERPOSER_LAYERS: usize = 12;

/// RF phase synthesis: map a boundary phase angle onto the 16-bit IQ phase
/// word consumed by the InP PIC emitter array (matches the kernel's
/// `shbt_warp_emitter_phase_word`).
pub fn phase_to_iq_word(phase_rad: f64) -> u16 {
    let two_pi = 2.0 * std::f64::consts::PI;
    let wrapped = phase_rad.rem_euclid(two_pi);
    (wrapped / two_pi * 65536.0) as u16
}

/// Phase-jitter audit for the emitter array: RMS jitter must stay below
/// the coherent-excitation budget (1e-3 rad).
pub const PHASE_JITTER_BUDGET_RAD: f64 = 1e-3;

pub fn phase_jitter_ok(rms_jitter_rad: f64) -> bool {
    rms_jitter_rad <= PHASE_JITTER_BUDGET_RAD
}

/// Power transmission coefficient through the aerogel quarter-wave layer:
/// T = 4 Z1 Z3 Zm^2 / (Z1 Z3 + Zm^2)^2 evaluated for the sapphire/aerogel/
/// load stack (Z1 = sapphire, Z3 = the radiating medium).
pub fn aerogel_transmission(z_load_mrayl: f64) -> f64 {
    let z1 = Z_SAPPHIRE_MRAYL;
    let z3 = z_load_mrayl;
    let zm = Z_AEROGEL_MRAYL;
    let num = 4.0 * z1 * z3 * zm * zm;
    let den = (z1 * z3 + zm * zm).powi(2);
    num / den
}

/// Quarter-wave matched radiating load: Z3 = Zm^2 / Z_sapphire.
pub fn matched_load_mrayl() -> f64 {
    Z_AEROGEL_MRAYL.powi(2) / Z_SAPPHIRE_MRAYL
}

/// Reflection coefficient magnitude at an impedance interface.
pub fn interface_reflection(z_a: f64, z_b: f64) -> f64 {
    ((z_b - z_a) / (z_b + z_a)).abs()
}

// ------------------------------------------------------------------
// EDA artifact generators
// ------------------------------------------------------------------

fn gds_u16(w: &mut impl Write, rec: u8, dtype: u8, vals: &[u16]) -> io::Result<()> {
    let len = 4 + 2 * vals.len() as u16;
    w.write_all(&len.to_be_bytes())?;
    w.write_all(&[rec, dtype])?;
    for v in vals {
        w.write_all(&v.to_be_bytes())?;
    }
    Ok(())
}

fn gds_u32(w: &mut impl Write, rec: u8, dtype: u8, vals: &[u32]) -> io::Result<()> {
    let len = 4 + 4 * vals.len() as u16;
    w.write_all(&len.to_be_bytes())?;
    w.write_all(&[rec, dtype])?;
    for v in vals {
        w.write_all(&v.to_be_bytes())?;
    }
    Ok(())
}

fn gds_str(w: &mut impl Write, rec: u8, s: &str) -> io::Result<()> {
    let mut bytes = s.as_bytes().to_vec();
    if bytes.len() % 2 == 1 {
        bytes.push(0);
    }
    let len = (4 + bytes.len()) as u16;
    w.write_all(&len.to_be_bytes())?;
    w.write_all(&[rec, 0x06])?;
    w.write_all(&bytes)?;
    Ok(())
}

/// Emit a valid GDSII stream with one cell containing the 8x8 waveguide
/// array as boundary elements on layer 1.
pub fn export_gdsii(path: &str) -> io::Result<()> {
    let mut f = File::create(path)?;
    gds_u16(&mut f, 0x00, 0x02, &[600])?; // HEADER v600
    gds_u16(&mut f, 0x01, 0x01, &[2026, 10, 1, 0, 0, 0, 2026, 10, 1, 0, 0, 0])?; // BGNLIB
    gds_str(&mut f, 0x02, "SHBT_WARP")?; // LIBNAME
    // UNITS record (3 8-byte reals): user=1e-3, db=1e-9.
    f.write_all(&20u16.to_be_bytes())?;
    f.write_all(&[0x03, 0x05])?;
    for v in [0x3E4189374BC6A7EFu64, 0x3944B82FA09B5A54, 0x0000000000000000] {
        f.write_all(&v.to_be_bytes())?;
    }
    gds_u16(&mut f, 0x05, 0x02, &[0])?; // BGNSTR
    gds_str(&mut f, 0x06, "PIC8X8")?;
    // 8x8 array of 5 um x 350 um waveguide stripes at 127 um pitch (db nm).
    let pitch = 127_000i32;
    for row in 0..PIC_ARRAY {
        for col in 0..PIC_ARRAY {
            gds_u16(&mut f, 0x08, 0x00, &[0])?; // BOUNDARY
            gds_u16(&mut f, 0x0D, 0x02, &[1])?; // LAYER 1
            gds_u16(&mut f, 0x0E, 0x02, &[0])?; // DATATYPE 0
            let x0 = (col as i32) * pitch;
            let y0 = (row as i32) * pitch;
            let w = 5_000i32;
            let l = 350_000i32;
            let xy: [i32; 10] = [x0, y0, x0 + w, y0, x0 + w, y0 + l, x0, y0 + l, x0, y0];
            let xy_u: [u32; 10] = xy.map(|v| v as u32);
            gds_u32(&mut f, 0x10, 0x03, &xy_u)?;
            gds_u16(&mut f, 0x11, 0x00, &[0])?; // ENDEL
        }
    }
    gds_u16(&mut f, 0x07, 0x00, &[0])?; // ENDSTR
    gds_u16(&mut f, 0x04, 0x00, &[0])?; // ENDLIB
    Ok(())
}

/// Emit an ISO 10303-21 STEP waveguide B-Rep stub (AP214-style header,
/// box geometry parameterized by length/width/height in metres).
pub fn export_step(path: &str, length: f64, width: f64, height: f64) -> io::Result<()> {
    let mut f = File::create(path)?;
    writeln!(f, "ISO-10303-21;")?;
    writeln!(f, "HEADER;")?;
    writeln!(f, "FILE_DESCRIPTION(('SHBT warp sapphire waveguide'),'2;1');")?;
    writeln!(f, "FILE_NAME('{}','2026-10-01T00:00:00',('Devin'),('SHBT'),'','','');", path)?;
    writeln!(f, "FILE_SCHEMA(('AUTOMOTIVE_DESIGN'));")?;
    writeln!(f, "ENDSEC;")?;
    writeln!(f, "DATA;")?;
    writeln!(f, "#1=CARTESIAN_POINT('ORIGIN',(0.,0.,0.));")?;
    writeln!(f, "#2=DIRECTION('Z',(0.,0.,1.));")?;
    writeln!(f, "#3=DIRECTION('X',(1.,0.,0.));")?;
    writeln!(f, "#4=AXIS2_PLACEMENT_3D('',#1,#2,#3);")?;
    writeln!(
        f,
        "#5=BLOCK('WAVEGUIDE',#4,{:.6e},{:.6e},{:.6e});",
        length * 1e3,
        width * 1e3,
        height * 1e3
    )?;
    writeln!(f, "ENDSEC;")?;
    writeln!(f, "END-ISO-10303-21;")?;
    Ok(())
}

/// Emit a Touchstone 2.0 S2P interposer model: 12-layer RO4350B stack
/// decomposed as a matched through-line, |S11| set by the Z0 deviation.
/// `z0` must lie inside the INTERPOSER_Z0 +/- 0.80 ohm tolerance band.
pub fn export_s2p(path: &str, z0: f64, freqs_ghz: &[f64]) -> io::Result<()> {
    if (z0 - INTERPOSER_Z0).abs() > INTERPOSER_Z0_TOL {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("interposer Z0 {:.2} outside tolerance band", z0),
        ));
    }
    let gamma = (z0 - 50.0) / (z0 + 50.0);
    let mut f = File::create(path)?;
    writeln!(f, "! SHBT warp 12-layer Rogers RO4350B interposer")?;
    writeln!(f, "! Z0 = {:.2} ohm, layers = {}", z0, INTERPOSER_LAYERS)?;
    writeln!(f, "# GHZ S RI R 50")?;
    let loss_db = -0.003;
    for &fg in freqs_ghz {
        let s21_mag = 10f64.powf(loss_db * fg / 20.0);
        let s21_ang = -2.0 * std::f64::consts::PI * fg * 0.12;
        writeln!(
            f,
            "{:.4} {:.6e} {:.6e} {:.6e} {:.6e} {:.6e} {:.6e} {:.6e} {:.6e}",
            fg,
            gamma,
            0.0,
            s21_mag * s21_ang.cos(),
            s21_mag * s21_ang.sin(),
            s21_mag * s21_ang.cos(),
            s21_mag * s21_ang.sin(),
            gamma,
            0.0
        )?;
    }
    Ok(())
}

/// S11 return loss in dB for a load `zl` on the 50.12 ohm interposer match.
pub fn s11_db(zl: f64) -> f64 {
    let gamma = ((zl - INTERPOSER_Z0) / (zl + INTERPOSER_Z0)).abs().max(1e-12);
    20.0 * gamma.log10()
}
