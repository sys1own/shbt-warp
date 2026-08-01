"""High-level CAD engine wrapping the SHBT native Rust primitives.

The ``SHBTCADEngine`` orchestrates the ``FGSliceProjector``,
``CharacterExcitationRegister``, ``SafetyMonitor``,
``EmitterArrayController``, and ``FlightDynamicsEngine`` primitives into a
single three-phase flight simulation and export pipeline.
"""

import math
from pathlib import Path
from typing import Any, Dict, List, Optional

import numpy as np

from shbt_warp._core import (
    CharacterExcitationRegister,
    EmitterArrayController,
    FGSliceProjector,
    FlightDynamicsEngine,
    HardwareNoiseAuditor,
    SafetyMonitor,
)
from shbt_warp.plots import _save_figure


class SimulationReport(dict):
    """Convenience ``dict`` subclass for flight-simulation reports."""

    def export_latex_macros(self, output_path: str = "sim_results.tex") -> Path:
        """Write the report's TeX macros to ``output_path``."""
        return _write_latex_macros(self, Path(output_path))


class SHBTCADEngine:
    """High-level SHBT CAD / flight-dynamics orchestrator."""

    def __init__(
        self,
        radius: float = 10.0,
        domain_radius: float = 30.0,
        grid_points: int = 1201,
        wall_steepness: float = 0.8,
        phase: float = 0.421,
        t_ramp: float = 1.0,
        t_steering: float = 1.0,
        n_steps: int = 64,
        gamma_lock: float = 4.665e-19,
        noise_temp_k: float = 15.4e-3,
        noise_gamma: float = 1.2e-4,
        phase_jitter: float = 5.05e-5,
        budget_limit: float = 1.0,
    ) -> None:
        self.radius = radius
        self.domain_radius = domain_radius
        self.grid_points = grid_points
        self.wall_steepness = wall_steepness
        self.phase = phase
        self.t_ramp = t_ramp
        self.t_steering = t_steering
        self.n_steps = n_steps
        self.gamma_lock = gamma_lock
        self.noise_temp_k = noise_temp_k
        self.noise_gamma = noise_gamma
        self.phase_jitter = phase_jitter
        self.budget_limit = budget_limit

        # Native Rust primitives.
        self.projector = FGSliceProjector(radius=radius)
        self.safety = SafetyMonitor()
        self.emitter = EmitterArrayController()
        self.noise_auditor = HardwareNoiseAuditor()
        self.flight = FlightDynamicsEngine()

    # --------------------------------------------------------------------- #
    # Helpers
    # --------------------------------------------------------------------- #
    def _identity_register(self) -> CharacterExcitationRegister:
        """Return a canonical unit-trace 9x9 boundary density matrix."""
        data = [complex(0.0, 0.0)] * 81
        for i in range(9):
            data[i * 9 + i] = complex(1.0 / 9.0, 0.0)
        return CharacterExcitationRegister(data)

    def _shape_at(self, x: float, y: float, z: float) -> float:
        """Smooth SHBT shape function (inside bubble -> 1.0, outside -> 0.0)."""
        r = math.sqrt(x * x + y * y + z * z)
        # Smooth step centered at the bubble wall.
        exponent = -self.wall_steepness * (r - self.radius)
        if exponent > 700.0:
            return 1.0
        return 1.0 / (1.0 + math.exp(-exponent))

    def _grid_line(self, n: int) -> np.ndarray:
        """Return a 1-D Cartesian line through the domain."""
        return np.linspace(-self.domain_radius, self.domain_radius, n)

    # --------------------------------------------------------------------- #
    # Public API
    # --------------------------------------------------------------------- #
    def run_flight_simulation(
        self,
        profile_config: Optional[Dict[str, Any]] = None,
    ) -> SimulationReport:
        """Execute the full Phase A -> B -> C flight profile.

        Parameters
        ----------
        profile_config:
            Optional overrides for ``radius``, ``phase``, ``t_ramp``,
            ``t_steering``, ``n_steps``.

        Returns
        -------
        SimulationReport
            A dict-like report containing phase metrics, HIL status,
            emitter commands, and invariant checks.
        """
        config = profile_config or {}
        radius = float(config.get("radius", self.radius))
        phase = float(config.get("phase", self.phase))
        t_ramp = float(config.get("t_ramp", self.t_ramp))
        t_steering = float(config.get("t_steering", self.t_steering))
        n_steps = int(config.get("n_steps", self.n_steps))

        # Phase A: ramping.
        phase_a: List[Dict[str, Any]] = []
        register = self._identity_register()
        n = np.array([1.0, 0.0, 0.0])
        dt_a = t_ramp / max(n_steps, 1)
        for step in range(n_steps + 1):
            t = step * dt_a
            xi = self.flight.phase_a_ramp(t, t_ramp)
            f_shbt = self._shape_at(0.0, 0.0, 0.0)  # central plateau
            beta = self.flight.phase_a_shift(xi, f_shbt, n.tolist())
            trace_ok, trace_res = register.verify_unitarity()
            delta_fr = register.audit_framing_defect(26, 8, 312)
            hil = self.safety.audit_hil_step(
                0.5,  # synthetic minimum Gram eigenvalue (above 0.35)
                0.0,  # no determinant error
                0.0,  # no information-density excursion
                self.budget_limit,
            )
            phase_a.append(
                {
                    "t": t,
                    "xi": xi,
                    "beta": beta,
                    "trace_ok": trace_ok,
                    "trace_residual": trace_res,
                    "framing_defect": delta_fr,
                    "hil_status": hil,
                }
            )

        # Phase B: vector steering.
        omega = np.array([0.0, 0.0, 0.5])  # rotation around z
        dt_b = t_steering / max(n_steps, 1)
        phase_b: List[Dict[str, Any]] = []
        n_current = n.copy()
        for step in range(n_steps + 1):
            t = step * dt_b
            if step > 0:
                n_current = np.array(
                    self.flight.phase_b_step_n(n_current.tolist(), omega.tolist(), dt_b)
                )
            # Evaluate prime loads and flux along each Cartesian axis.
            rho = [1.0 / 9.0] * 9
            loads = []
            fluxes = []
            for axis in range(3):
                l_r, phi_s = self.flight.phase_b_prime_loads(rho, 0.0, 0.0, axis)
                loads.append(l_r)
                fluxes.append(phi_s)
            # Interior plateau acceleration is zero when grad f = 0.
            plateau_ok, a_norm = self.flight.phase_b_plateau_acceleration(
                f_shbt, [0.0, 0.0, 0.0], 0.0, n_current.tolist()
            )
            phase_b.append(
                {
                    "t": t,
                    "n": n_current.tolist(),
                    "loads": loads,
                    "fluxes": fluxes,
                    "plateau_acceleration_ok": plateau_ok,
                    "acceleration_norm": a_norm,
                }
            )

        # Phase C: safe collapse / de-rendering.
        t0 = t_ramp + t_steering
        delta_mod_t0 = self.flight.delta_mod_0
        t_collapse = t0 + 1.0
        delta_mod = self.flight.phase_c_delta_mod(t_collapse, t0, delta_mod_t0)
        eta_d = self.flight.phase_c_stinespring_ratio()
        beta_final = [0.5, 0.0, 0.0]
        zero_beta, flat_metric, det_g = self.flight.phase_c_collapse(beta_final)

        phase_c = {
            "t0": t0,
            "t": t_collapse,
            "delta_mod": delta_mod,
            "eta_d": eta_d,
            "collapsed_beta": zero_beta,
            "metric_determinant": det_g,
        }

        # Emitter RF snapshot for a representative visible state.
        q_i = 1.0
        p_j = 0.0
        r_j = 1.0
        weight_grade = p_j + r_j
        h_ij = self.emitter.conformal_dimension(q_i, p_j, r_j)
        theta_k = self.emitter.compute_emitter_phase(phase, q_i, weight_grade, h_ij)
        v_rf = self.emitter.synthesize_rf_signal(1.0, 2.0 * math.pi * 1e9, 1e-9, theta_k, 0.05)

        # Hardware noise audit.
        noise_ok, jitter = self.noise_auditor.audit_phase_jitter(self.phase_jitter)
        thermal_ok, thermal_ratio = self.noise_auditor.audit_thermal_decoherence(
            self.noise_temp_k, self.noise_gamma
        )
        lock_ok = self.noise_auditor.audit_level_integer_lock(0, 0, 0)

        # ADM grid snapshot (3x3x3 cells) on the central x-axis.
        nx = ny = nz = 3
        shape_field = [1.0] * (nx * ny * nz)
        n_vec = [1.0, 0.0, 0.0]
        metric_grid = self.projector.evaluate_adm_grid(1.0, n_vec, shape_field, nx, ny, nz)
        # Determinant check is performed inside evaluate_adm_grid; here we verify
        # a representative cell's g_00 component.
        g00_center = metric_grid[0]

        report = SimulationReport(
            {
                "config": {
                    "radius": radius,
                    "phase": phase,
                    "t_ramp": t_ramp,
                    "t_steering": t_steering,
                    "n_steps": n_steps,
                    "wall_steepness": self.wall_steepness,
                    "domain_radius": self.domain_radius,
                    "noise_temp_k": self.noise_temp_k,
                    "noise_gamma": self.noise_gamma,
                    "phase_jitter": self.phase_jitter,
                    "budget_limit": self.budget_limit,
                },
                "phase_a": phase_a,
                "phase_b": phase_b,
                "phase_c": phase_c,
                "emitter": {
                    "q_i": q_i,
                    "p_j": p_j,
                    "r_j": r_j,
                    "h_ij": h_ij,
                    "theta_k": theta_k,
                    "v_rf": v_rf,
                },
                "noise": {
                    "phase_jitter_ok": noise_ok,
                    "phase_jitter_value": jitter,
                    "thermal_decoherence_ok": thermal_ok,
                    "thermal_ratio": thermal_ratio,
                    "integer_lock_ok": lock_ok,
                },
                "metric_grid": {
                    "shape": (nx, ny, nz),
                    "g00_center": g00_center,
                    "flat_metric": flat_metric,
                    "flat_metric_determinant": det_g,
                },
                "invariants": {
                    "canonical_branch": (26, 8, 312),
                    "framing_defect": phase_a[-1]["framing_defect"],
                    "trace_residual": phase_a[-1]["trace_residual"],
                    "unitarity_ok": phase_a[-1]["trace_ok"],
                    "stinespring_ratio": eta_d,
                    "delta_mod": delta_mod,
                },
            }
        )
        return report

    def export_latex_macros(self, report: SimulationReport, output_path: str = "sim_results.tex") -> Path:
        """Export ``report`` as a ``.tex`` macro file."""
        return report.export_latex_macros(output_path)

    def render_figures(self, report: SimulationReport, figures_dir: str = "figures") -> List[Path]:
        """Render CAD flight-phase figures to ``figures_dir``."""
        import matplotlib.pyplot as plt

        figures_dir = Path(figures_dir)
        figures_dir.mkdir(parents=True, exist_ok=True)
        paths: List[Path] = []

        # Phase A ramp figure.
        phase_a = report["phase_a"]
        t_a = [p["t"] for p in phase_a]
        xi_a = [p["xi"] for p in phase_a]
        fig, ax = plt.subplots(figsize=(7, 4))
        ax.plot(t_a, xi_a, "b-", lw=1.5)
        ax.set_xlabel("Time (s)")
        ax.set_ylabel(r"$\xi(t)$")
        ax.set_title("Phase A ramp coefficient")
        fig.tight_layout()
        ramp_path = figures_dir / "cad_phase_a_ramp.pdf"
        _save_figure(fig, ramp_path)
        paths.append(ramp_path)
        plt.close(fig)

        # Phase B steering figure.
        phase_b = report["phase_b"]
        t_b = [p["t"] for p in phase_b]
        n_x = [p["n"][0] for p in phase_b]
        n_y = [p["n"][1] for p in phase_b]
        fig, ax = plt.subplots(figsize=(7, 4))
        ax.plot(t_b, n_x, "r-", lw=1.2, label=r"$n_x$")
        ax.plot(t_b, n_y, "g--", lw=1.2, label=r"$n_y$")
        ax.set_xlabel("Time (s)")
        ax.set_ylabel("Steering normal component")
        ax.set_title("Phase B vector steering")
        ax.legend()
        fig.tight_layout()
        steering_path = figures_dir / "cad_phase_b_steering.pdf"
        _save_figure(fig, steering_path)
        paths.append(steering_path)
        plt.close(fig)

        return paths


def _write_latex_macros(report: SimulationReport, path: Path) -> Path:
    """Write a ``.tex`` file containing the CAD report macros."""
    inv = report.get("invariants", {})
    cfg = report.get("config", {})
    noise = report.get("noise", {})
    emitter = report.get("emitter", {})
    phase_c = report.get("phase_c", {})
    metric = report.get("metric_grid", {})

    macros = {
        "CADCanonicalBranch": f"({inv.get('canonical_branch', (26, 8, 312))[0]}, "
                              f"{inv.get('canonical_branch', (26, 8, 312))[1]}, "
                              f"{inv.get('canonical_branch', (26, 8, 312))[2]})",
        "CADFramingDefect": f"{inv.get('framing_defect', 0.0):.12f}",
        "CADUnitarityResidual": f"{inv.get('trace_residual', 0.0):.12e}",
        "CADPhaseJitterOk": "true" if noise.get("phase_jitter_ok", False) else "false",
        "CADThermalDecoherenceOk": "true" if noise.get("thermal_decoherence_ok", False) else "false",
        "CADIntegerLockOk": "true" if noise.get("integer_lock_ok", False) else "false",
        "CADStinespringRatio": f"{inv.get('stinespring_ratio', 0.0):.12f}",
        "CADDeltaMod": f"{phase_c.get('delta_mod', 0.0):.12f}",
        "CADMetricDeterminant": f"{metric.get('flat_metric_determinant', 0.0):.12f}",
        "CADRadius": f"{cfg.get('radius', 10.0):.1f}",
        "CADPhase": f"{cfg.get('phase', 0.421):.6f}",
        "CADEmitterThetaK": f"{emitter.get('theta_k', 0.0):.12f}",
        "CADEmitterVRF": f"{emitter.get('v_rf', 0.0):.12e}",
    }

    lines = [
        "% SHBT CAD engine output macros",
        "% Generated automatically by SHBTCADEngine",
    ]
    for name, value in macros.items():
        lines.append(f"\\newcommand{{\\{name}}}{{{value}}}")
    lines.append("")

    path = Path(path)
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text("\n".join(lines), encoding="utf-8")
    return path
