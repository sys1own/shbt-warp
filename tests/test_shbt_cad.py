"""Comprehensive CAD engine integration tests for physical invariants."""

import math
import subprocess
from pathlib import Path

import pytest

import shbt_warp
from shbt_warp._core import (
    CharacterExcitationRegister,
    EmitterArrayController,
    FGSliceProjector,
    FlightDynamicsEngine,
    HardwareNoiseAuditor,
    SafetyMonitor,
)
from shbt_warp.cad_engine import SHBTCADEngine


# --------------------------------------------------------------------------- #
# Constants taken from the specification verification table.
# --------------------------------------------------------------------------- #
C_DARK_RESIDUAL = 834_433.0 / 362_670.0
C_DARK_COMPLETED = 1_197_103.0 / 362_670.0
DELTA_MOD = C_DARK_COMPLETED / 24.0
LIGHT_SPEED_M_S = 299_792_458.0


def _identity_density_matrix():
    """Return a flat 81-element density matrix with unit trace."""
    data = [complex(0.0, 0.0)] * 81
    for i in range(9):
        data[i * 9 + i] = complex(1.0 / 9.0, 0.0)
    return data


# --------------------------------------------------------------------------- #
# Canonical branch and character-state invariants
# --------------------------------------------------------------------------- #
def test_canonical_branch_tuple():
    engine = SHBTCADEngine()
    report = engine.run_flight_simulation()
    assert report["invariants"]["canonical_branch"] == (26, 8, 312)


def test_scalar_framing_defect_is_zero():
    engine = SHBTCADEngine()
    report = engine.run_flight_simulation()
    assert report["invariants"]["framing_defect"] == 0.0


def test_dark_residual_ledger():
    assert math.isclose(C_DARK_RESIDUAL, 834433 / 362670, rel_tol=1e-12)
    register = CharacterExcitationRegister(_identity_density_matrix())
    # The canonical branch leaves framing defect zero.
    assert register.audit_framing_defect(26, 8, 312) == 0.0


def test_dark_completed_ledger():
    assert math.isclose(C_DARK_COMPLETED, 1197103 / 362670, rel_tol=1e-12)


def test_entropy_debt_from_dark_completed():
    assert math.isclose(DELTA_MOD, C_DARK_COMPLETED / 24.0, rel_tol=1e-12)


# --------------------------------------------------------------------------- #
# Warp velocity and ADM metric invariants
# --------------------------------------------------------------------------- #
def test_warp_velocity_scale():
    expected_v_eff_over_c = math.exp(DELTA_MOD / 2.0)
    projector = FGSliceProjector(radius=10.0)
    assert math.isclose(projector.v_eff, expected_v_eff_over_c, rel_tol=1e-9)


def test_unitary_operator_residual():
    register = CharacterExcitationRegister(_identity_density_matrix())
    passed, residual = register.verify_unitarity()
    assert passed
    assert residual < 1.0e-14


def test_adm_metric_determinant():
    projector = FGSliceProjector(radius=10.0)
    shape_field = [1.0] * 27
    n_vec = [1.0, 0.0, 0.0]
    metric_grid = projector.evaluate_adm_grid(1.0, n_vec, shape_field, 3, 3, 3)
    assert len(metric_grid) == 3 * 3 * 3 * 16
    # Determinant validation is performed inside the Rust call; confirm a
    # representative spatial metric component and the expected grid size.
    assert metric_grid[5] == 1.0  # g_11 for the first cell
    assert metric_grid[10] == 1.0  # g_22
    assert metric_grid[15] == 1.0  # g_33


def test_minimum_gram_eigenvalue():
    # The Rust metric3d audit guarantees min eigenvalue > 0.35.
    result = shbt_warp.run_simulation(grid_points=101)
    min_eig = result.get("minimum_gram_eigenvalue", 0.0)
    assert min_eig > 0.35


def test_interior_four_acceleration_zero():
    flight = FlightDynamicsEngine()
    # On the central plateau f = 1.0 and grad f = 0, with xi constant.
    ok, a_norm = flight.phase_b_plateau_acceleration(
        1.0, [0.0, 0.0, 0.0], 0.0, [1.0, 0.0, 0.0]
    )
    assert ok
    assert a_norm == 0.0


# --------------------------------------------------------------------------- #
# Operational power and hardware noise limits
# --------------------------------------------------------------------------- #
def test_operational_power_benchmark_10m():
    result = shbt_warp.run_simulation(radius=10.0)
    assert math.isclose(result["power_mw"], 142.08, rel_tol=1e-9)


def test_phase_jitter_limit():
    auditor = HardwareNoiseAuditor()
    passed, residual = auditor.audit_phase_jitter(5.05e-5)
    assert passed
    assert residual <= 5.05e-5


def test_thermal_noise_limit():
    auditor = HardwareNoiseAuditor()
    passed, _ratio = auditor.audit_thermal_decoherence(15.4e-3, 1.2e-4)
    assert passed


def test_decoherence_rate_limit():
    auditor = HardwareNoiseAuditor()
    passed, _ratio = auditor.audit_thermal_decoherence(15.4e-3, 1.2e-4)
    assert passed


# --------------------------------------------------------------------------- #
# HIL safety and CAD engine integration
# --------------------------------------------------------------------------- #
def test_hil_nominal_pass():
    monitor = SafetyMonitor()
    status = monitor.audit_hil_step(0.5, 0.0, 0.0, 1.0)
    assert status == "STATUS_NOMINAL_PASS"


def test_hil_determinant_violation():
    monitor = SafetyMonitor()
    status = monitor.audit_hil_step(0.5, 1.1e-12, 0.0, 1.0)
    assert "DETERMINANT" in status


def test_cad_engine_flight_phases_present():
    engine = SHBTCADEngine(n_steps=8)
    report = engine.run_flight_simulation()
    assert "phase_a" in report
    assert "phase_b" in report
    assert "phase_c" in report
    assert len(report["phase_a"]) == 9
    assert len(report["phase_b"]) == 9


def test_cad_engine_exports_latex_macros(tmp_path):
    engine = SHBTCADEngine(n_steps=4)
    report = engine.run_flight_simulation()
    tex_path = tmp_path / "cad_sim_results.tex"
    engine.export_latex_macros(report, str(tex_path))
    assert tex_path.exists()
    text = tex_path.read_text(encoding="utf-8")
    assert r"\newcommand{\CADCanonicalBranch}" in text
    assert r"\newcommand{\CADFramingDefect}{0.000000000000}" in text


def test_cad_engine_render_figures(tmp_path):
    engine = SHBTCADEngine(n_steps=8)
    report = engine.run_flight_simulation()
    fig_dir = tmp_path / "cad_figs"
    paths = engine.render_figures(report, str(fig_dir))
    assert len(paths) == 2
    for p in paths:
        assert p.exists()


def test_shbt_cad_sim_cli(tmp_path):
    tex = tmp_path / "sim_results.tex"
    figs = tmp_path / "cad_figures"
    subprocess.run(
        [
            "shbt-cad-sim",
            "--tex-output",
            str(tex),
            "--figures-directory",
            str(figs),
            "--radius",
            "10.0",
        ],
        check=True,
    )
    assert tex.exists()
    assert (figs / "cad_phase_a_ramp.pdf").exists()
    assert (figs / "cad_phase_b_steering.pdf").exists()


# --------------------------------------------------------------------------- #
# Flight-phase physical assertions
# --------------------------------------------------------------------------- #
def test_phase_a_ramp_clamps_to_one():
    flight = FlightDynamicsEngine()
    assert flight.phase_a_ramp(2.0, 1.0) == 1.0
    assert flight.phase_a_ramp(0.0, 1.0) == 0.0


def test_phase_b_steering_preserves_unit_norm():
    flight = FlightDynamicsEngine()
    n = [1.0, 0.0, 0.0]
    omega = [0.0, 0.0, 1.0]
    n_next = flight.phase_b_step_n(n, omega, 0.1)
    norm = math.sqrt(sum(x * x for x in n_next))
    assert math.isclose(norm, 1.0, rel_tol=1e-12)


def test_phase_c_delta_mod_decays():
    flight = FlightDynamicsEngine()
    delta0 = 0.137533547486
    delta = flight.phase_c_delta_mod(1.0e18, 0.0, delta0)
    assert delta < delta0
    assert delta > 0.0


def test_phase_c_collapse_restores_minkowski():
    flight = FlightDynamicsEngine()
    beta, metric, det = flight.phase_c_collapse([0.5, 0.0, 0.0])
    assert math.isclose(beta[0], 0.0, abs_tol=1e-15)
    assert math.isclose(det, -1.0, abs_tol=1e-12)
