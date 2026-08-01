"""End-to-end pytest suite for the shbt_warp package."""

import math
import subprocess
from pathlib import Path

import pytest

import shbt_warp


def test_c_dark_residual_matches_theory():
    result = shbt_warp.run_simulation()
    assert math.isclose(
        result["c_dark_residual"], 834_433.0 / 362_670.0, rel_tol=1e-12, abs_tol=1e-12
    )


def test_framing_defect_is_zero():
    result = shbt_warp.run_simulation()
    assert result["boundary"]["framing_defect"] == 0.0


def test_warp_velocity_matches_delta_mod():
    result = shbt_warp.run_simulation()
    expected = (result["delta_mod"] / 2.0) ** math.exp(1) ** 0  # exp(delta_mod/2)
    expected = math.exp(result["delta_mod"] / 2.0)
    assert math.isclose(result["v_eff_c"], expected, rel_tol=1e-12)


def test_operational_power_benchmark():
    result = shbt_warp.run_simulation(radius=10.0)
    assert math.isclose(result["power_mw"], 142.08, rel_tol=1e-10)


def test_all_audits_pass():
    result = shbt_warp.run_simulation(grid_points=101)
    assert result["boundary"]["audit"]["passed"] == 1.0
    assert result["excitation"]["audit"]["passed"] == 1.0
    assert result["fg_slice"]["audit"]["passed"] == 1.0
    assert result["metric3d"]["audit"]["passed"] == 1.0
    assert result["stress_energy"]["audit"]["passed"] == 1.0
    assert result["causal"]["audit"]["passed"] == 1.0
    assert result["thermodynamics"]["audit"]["passed"] == 1.0


def test_latex_macros_contain_expected_values(tmp_path):
    result = shbt_warp.run_simulation()
    exporter = shbt_warp.LaTeXMacroExporter(result)
    tex = exporter.write(tmp_path / "sim_results.tex")
    text = tex.read_text(encoding="utf-8")
    assert r"\newcommand{\SimOutputFramingDefect}{0.000000000000}" in text
    assert r"\newcommand{\SimOutputOperationalPowerMW}{142.08}" in text
    assert r"\newcommand{\SimOutputShiftFieldFormula}" in text


def test_cli_generates_outputs(tmp_path):
    tex = tmp_path / "sim_results.tex"
    figs = tmp_path / "figures"
    subprocess.run(
        [
            "shbt-warp-sim",
            "--grid-points",
            "101",
            "--tex-output",
            str(tex),
            "--figures-directory",
            str(figs),
        ],
        check=True,
    )
    assert tex.exists()
    assert (figs / "warp_bubble_profile.pdf").exists()
    assert (figs / "stress_energy_audit.pdf").exists()
    assert (figs / "derendering_transition.pdf").exists()
