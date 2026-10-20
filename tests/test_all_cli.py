"""End-to-end CLI regression: exercise every shbt_warp subcommand.

Each test shells out to ``python -m shbt_warp.cli <cmd>`` (or make for the
kernel C ABI) and asserts the command exits cleanly plus the expected
artifact or summary line is produced.
"""
import json
import subprocess
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[1]
VENV_PYTHON = sys.executable


def cli(*args, timeout=600):
    return subprocess.run(
        [VENV_PYTHON, "-m", "shbt_warp.cli", *args],
        cwd=REPO, capture_output=True, text=True, timeout=timeout,
    )


def test_build_kernel_and_reference_c():
    proc = cli("build-kernel")
    assert proc.returncode == 0, proc.stderr
    assert (REPO / "kernel/build/libshbt_warp_reference.so").exists()
    proc = subprocess.run(
        ["make", "-C", str(REPO / "kernel"), "test-c"],
        capture_output=True, text=True, timeout=120,
    )
    assert proc.returncode == 0, proc.stderr
    assert "all checks passed" in proc.stdout


def test_verify_203_gates():
    proc = cli("verify", timeout=900)
    assert proc.returncode == 0, proc.stdout + proc.stderr
    matrix = json.loads((REPO / "verification_matrix.json").read_text())
    assert matrix["total"] == 203
    assert matrix["passed"] == 203
    assert all(g["passed"] for g in matrix["gates"])
    assert (REPO / "warp_results.tex").stat().st_size > 0


def test_wormhole_cli():
    proc = cli("wormhole", "--mode", "mode-a", "--tier", "burst")
    assert proc.returncode == 0, proc.stdout + proc.stderr
    assert "SHBT-EW-312" in proc.stdout
    assert "Mode A" in proc.stdout

    proc_b = cli("wormhole", "--mode", "mode-b", "--tier", "impulse")
    assert proc_b.returncode == 0, proc_b.stdout + proc_b.stderr
    assert "Mode B" in proc_b.stdout


def test_ibr_cli():
    proc = cli("ibr", "--tier", "impulse-ew", "--mode", "mode-1c", "--cores", "38")
    assert proc.returncode == 0, proc.stdout + proc.stderr
    assert "SHBT-MAR-312-IBR" in proc.stdout
    assert "Compacted Nacelle Mass" in proc.stdout


def test_intergalactic_cli():
    proc = cli("intergalactic", "--target", "m31", "--tier", "impulse", "--mode", "mode-1c")
    assert proc.returncode == 0, proc.stdout + proc.stderr
    assert "Andromeda (M31)" in proc.stdout
    assert "Mission Feasibility      : PASS" in proc.stdout


def test_sim_canonical():
    proc = cli("sim", timeout=300)
    assert proc.returncode == 0, proc.stdout + proc.stderr


def test_hud_headless():
    proc = cli("hud", "--headless", timeout=60)
    assert proc.returncode == 0, proc.stdout + proc.stderr
    assert "margin=" in proc.stdout


def test_export_eda(tmp_path):
    proc = cli("export-eda", "--out-dir", str(tmp_path))
    assert proc.returncode == 0, proc.stdout + proc.stderr
    for name in ("pic8x8.gds", "waveguide.step", "interposer.s2p"):
        assert (tmp_path / name).stat().st_size > 0


def test_export_fits(tmp_path):
    out = tmp_path / "warp_adm.fits"
    proc = cli("export-fits", "--out", str(out))
    assert proc.returncode == 0, proc.stdout + proc.stderr
    assert out.stat().st_size > 0


def test_export_hdf5(tmp_path):
    out = tmp_path / "warp_mission.h5"
    proc = cli("export-hdf5", "--out", str(out))
    assert proc.returncode == 0, proc.stdout + proc.stderr
    assert out.stat().st_size > 0
