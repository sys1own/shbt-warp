"""End-to-end pytest suite for the shbt_warp package."""

import csv
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
    assert (figs / "shift_profile.pdf").exists()
    assert (figs / "stress_energy_audit.pdf").exists()
    assert (figs / "derendering_transition.pdf").exists()
    assert (figs / "entropy_gradient.pdf").exists()


def test_transient_results_present():
    result = shbt_warp.run_simulation(grid_points=101)
    assert "transient" in result
    transient = result["transient"]
    for key in ["time_s", "lock_times", "theta_t", "population_shift", "power_mw", "entropy_debt"]:
        assert key in transient
        assert len(transient[key]) > 1
    assert transient["theta_t"][0] == 0.0
    assert abs(transient["theta_t"][-1] - result["phase"]) < 1.0e-6

    assert "rerender_trajectory" in result["derender"]
    trajectory = result["derender"]["rerender_trajectory"]
    for key in ["lock_time", "origin_x", "origin_y", "origin_z", "restored_bits", "transferred_bits", "bit_budget_preserved"]:
        assert key in trajectory
        assert len(trajectory[key]) > 1
    assert trajectory["origin_x"][0] == 0.0


def test_cli_sweep_outputs_csv(tmp_path):
    csv_path = tmp_path / "sweep.csv"
    subprocess.run(
        [
            "shbt-warp-sim",
            "--sweep-radius",
            "5:15:5",
            "--sweep-output",
            str(csv_path),
            "--grid-points",
            "101",
        ],
        check=True,
    )
    assert csv_path.exists()
    with csv_path.open(newline="") as f:
        reader = csv.DictReader(f)
        rows = list(reader)
    assert len(rows) == 3
    for row in rows:
        assert float(row["power_mw"]) > 0.0
        assert float(row["entropy_debt"]) >= 0.0
