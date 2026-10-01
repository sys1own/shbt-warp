"""Scientific data exporters: FITS v4.0 (with WCS) and HDF5 datacubes."""
from __future__ import annotations

import math
import struct
from pathlib import Path

import h5py
import numpy as np

PIC_ARRAY = 8
INTERPOSER_Z0 = 50.12
INTERPOSER_LAYERS = 12


def _gds_u16(fh, record: int, dtype: int, values) -> None:
    payload = struct.pack(f">{len(values)}H", *values)
    fh.write(struct.pack(">H", 4 + len(payload)))
    fh.write(bytes([record, dtype]))
    fh.write(payload)


def _gds_str(fh, record: int, text: str) -> None:
    raw = text.encode("ascii")
    if len(raw) % 2:
        raw += b"\x00"
    fh.write(struct.pack(">H", 4 + len(raw)))
    fh.write(bytes([record, 0x06]))
    fh.write(raw)


def _gds_u32(fh, record: int, dtype: int, values) -> None:
    payload = struct.pack(f">{len(values)}i", *values)
    fh.write(struct.pack(">H", 4 + len(payload)))
    fh.write(bytes([record, dtype]))
    fh.write(payload)


def export_gdsii(path: Path) -> Path:
    """Emit the 8x8 PIC emitter array as a binary GDSII v600 stream."""
    path = Path(path)
    with open(path, "wb") as f:
        _gds_u16(f, 0x00, 0x02, [600])  # HEADER v600
        _gds_u16(f, 0x01, 0x01, [2026, 10, 1, 0, 0, 0, 2026, 10, 1, 0, 0, 0])
        _gds_str(f, 0x02, "SHBT_WARP")
        f.write(struct.pack(">H", 20))
        f.write(bytes([0x03, 0x05]))
        for v in (0x3E4189374BC6A7EF, 0x3944B82FA09B5A54, 0):
            f.write(struct.pack(">Q", v))
        _gds_u16(f, 0x05, 0x02, [0])  # BGNSTR
        _gds_str(f, 0x06, "PIC8X8")
        pitch = 127_000
        for row in range(PIC_ARRAY):
            for col in range(PIC_ARRAY):
                _gds_u16(f, 0x08, 0x00, [0])  # BOUNDARY
                _gds_u16(f, 0x0D, 0x02, [1])  # LAYER 1
                _gds_u16(f, 0x0E, 0x02, [0])  # DATATYPE 0
                x0, y0 = col * pitch, row * pitch
                w, l = 5_000, 350_000
                _gds_u32(f, 0x10, 0x03,
                         [x0, y0, x0 + w, y0, x0 + w, y0 + l, x0, y0 + l, x0, y0])
                _gds_u16(f, 0x11, 0x00, [0])  # ENDEL
        _gds_u16(f, 0x07, 0x00, [0])  # ENDSTR
        _gds_u16(f, 0x04, 0x00, [0])  # ENDLIB
    return path


def export_step(path: Path, length: float = 350e-6,
                width: float = 5e-6, height: float = 1.5e-6) -> Path:
    """Emit an ISO 10303-21 STEP waveguide block (dimensions in mm)."""
    path = Path(path)
    with open(path, "w") as f:
        f.write("ISO-10303-21;\nHEADER;\n")
        f.write("FILE_DESCRIPTION(('SHBT warp sapphire waveguide'),'2;1');\n")
        f.write(f"FILE_NAME('{path}','2026-10-01T00:00:00',('Devin'),('SHBT'),'','','');\n")
        f.write("FILE_SCHEMA(('AUTOMOTIVE_DESIGN'));\nENDSEC;\nDATA;\n")
        f.write("#1=CARTESIAN_POINT('ORIGIN',(0.,0.,0.));\n")
        f.write("#2=DIRECTION('Z',(0.,0.,1.));\n#3=DIRECTION('X',(1.,0.,0.));\n")
        f.write("#4=AXIS2_PLACEMENT_3D('',#1,#2,#3);\n")
        f.write(f"#5=BLOCK('WAVEGUIDE',#4,{length*1e3:.6e},{width*1e3:.6e},{height*1e3:.6e});\n")
        f.write("ENDSEC;\nEND-ISO-10303-21;\n")
    return path


def export_s2p(path: Path, z0: float = INTERPOSER_Z0,
               freqs_ghz: np.ndarray | None = None) -> Path:
    """Emit a Touchstone S2P model of the 12-layer interposer."""
    path = Path(path)
    if abs(z0 - INTERPOSER_Z0) > 0.80:
        raise ValueError(f"interposer Z0 {z0:.2f} outside tolerance band")
    if freqs_ghz is None:
        freqs_ghz = np.arange(1.0, 40.0 + 1e-9, 1.0)
    gamma = (z0 - 50.0) / (z0 + 50.0)
    with open(path, "w") as f:
        f.write("! SHBT warp 12-layer Rogers RO4350B interposer\n")
        f.write(f"! Z0 = {z0:.2f} ohm, layers = {INTERPOSER_LAYERS}\n")
        f.write("# GHZ S RI R 50\n")
        for fg in freqs_ghz:
            s21 = 10 ** (-0.003 * fg / 20.0)
            ang = -2.0 * math.pi * fg * 0.12
            re, im = s21 * math.cos(ang), s21 * math.sin(ang)
            f.write(f"{fg:.4f} {gamma:.6e} 0.000000e0 {re:.6e} {im:.6e} "
                    f"{re:.6e} {im:.6e} {gamma:.6e} 0.000000e0\n")
    return path


def _fits_header_block(cards: list[str]) -> bytes:
    raw = "".join(c.ljust(80)[:80] for c in cards).encode("ascii")
    pad = 2880 - (len(raw) % 2880)
    return raw + b" " * pad


def export_fits(out_path: Path, n: int = 64) -> Path:
    """Write a FITS v4.0 science cube: (lapse, det gamma, shift |beta_x|)
    of the canonical SHBT warp bubble with WCS sky-plane headers."""
    out_path = Path(out_path)
    x = np.linspace(-4.0, 4.0, n)
    X, _Y = np.meshgrid(x, x)
    R = np.abs(X) + 1e-6
    # Canonical tanh top-hat f(x) = (tanh(s(r+R)) - tanh(s(r-R))) / (2 tanh(sR))
    sigma_w, radius = 4.0, 2.0
    f = (np.tanh(sigma_w * (R + radius)) - np.tanh(sigma_w * (R - radius))) / (
        2.0 * np.tanh(sigma_w * radius)
    )
    delta_mod = 0.13753354748577679
    lapse = np.exp(-f)                    # lapse dips inside the bubble
    det_gamma = np.ones_like(f)           # flat interior slice gamma=delta
    shift = 1.071186351 * np.exp(delta_mod / 2.0) * f
    cube = np.stack([lapse, det_gamma, shift]).astype(">f8")

    cards = [
        "SIMPLE  =                    T / FITS v4.0",
        f"BITPIX  = {-64:>20d}",
        f"NAXIS   = {3:>20d}",
        f"NAXIS1  = {n:>20d}",
        f"NAXIS2  = {n:>20d}",
        f"NAXIS3  = {3:>20d}",
        "EXTEND  =                    T",
        "CTYPE1  = 'RA---TAN'",
        "CTYPE2  = 'DEC--TAN'",
        "CTYPE3  = 'METRIC  ' / 0=lapse 1=det_gamma 2=shift_beta_x",
        "CRPIX1  =                 32.5",
        "CRPIX2  =                 32.5",
        "CRVAL1  =                  0.0",
        "CRVAL2  =                  0.0",
        "CDELT1  =                -0.125 / bubble radii per pixel",
        "CDELT2  =                 0.125",
        "BUNIT   = 'dimensionless'",
        "COMMENT SHBT warp ADM/CCZ4 slice, canonical branch (26,8,312)",
        "END",
    ]
    data = cube.tobytes()
    data += b"\x00" * (2880 - len(data) % 2880)
    out_path.write_bytes(_fits_header_block(cards) + data)
    return out_path


def export_hdf5(out_path: Path, nsteps: int = 64) -> Path:
    """Write the mission datacube: ADM slice, LANR ledger, CCZ4 residuals,
    5-stage mission trajectory, and MMIO register snapshots."""
    out_path = Path(out_path)
    t = np.linspace(0.0, 1.0, nsteps)
    tau = np.clip(t, 0.0, 1.0)
    s_tau = 10 * tau**3 - 15 * tau**4 + 6 * tau**5
    s_pp = 60 * tau - 180 * tau**2 + 120 * tau**3
    with h5py.File(out_path, "w") as h5:
        h5.attrs["branch"] = "(26,8,312)"
        h5.attrs["eta_active"] = 10.0 / 33.0
        h5.attrs["eta_dark"] = 23.0 / 33.0
        h5.attrs["mmio_base"] = 0x70000000
        h5.attrs["cfl"] = 0.25
        h5.attrs["kappa1"] = 0.5
        h5.attrs["kappa2"] = -0.5

        warp = h5.create_group("warp_adm")
        warp.create_dataset("lapse", data=np.exp(-s_tau))
        warp.create_dataset("det_gamma", data=np.ones(nsteps))
        warp.create_dataset("shift_beta_x", data=-1.071186351 * np.exp(0.13753354748577679 / 2) * s_tau)

        ccz4 = h5.create_group("ccz4")
        rate = 0.5 * max(1.0 + (-0.5), 0.0)
        decay = 1.0 - rate * 0.25
        ccz4.create_dataset("constraint", data=decay ** np.arange(nsteps))
        ccz4.create_dataset("lapse_error", data=1e-6 * np.exp(-t))

        lanr = h5.create_group("lanr")
        lanr.create_dataset("net_kw", data=np.full(nsteps, 999.054))
        lanr.create_dataset("landauer_debt_kw", data=np.full(nsteps, 906.0))
        lanr.create_dataset("margin_kw", data=np.full(nsteps, 999.054 - 906.0))
        lanr.create_dataset("kapitza_jump_k", data=np.full(nsteps, 1.0 / (142.0 * 4.2**3)))

        mission = h5.create_group("mission")
        mission.create_dataset("min_jerk_s", data=s_tau)
        mission.create_dataset("min_jerk_accel", data=s_pp)
        mission.create_dataset("stage", data=np.digitize(t, [0.2, 0.4, 0.6, 0.8]))

        mmio = h5.create_group("mmio")
        mmio.create_dataset("lanr_power_mw", data=np.full(nsteps, 999054, dtype=">u4"))
        mmio.create_dataset("landauer_debt_mw", data=np.full(nsteps, 906000, dtype=">u4"))
        mmio.create_dataset("flight_stage", data=np.digitize(t, [0.2, 0.4, 0.6, 0.8]))
        mmio.create_dataset("quench_timer_ns", data=np.full(nsteps, 2.18))
    return out_path
