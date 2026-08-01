"""High-level custom simulation API for the SHBT warp drive.

This module exposes developer-friendly Python classes that wrap the Rust/PyO3
numerical core so researchers can run parameter sweeps and bulk metric
projections without managing the low-level ``Simulation`` dict directly.
"""

import math
from typing import Optional

import numpy as np

from shbt_warp._core import CharacterExcitationRegister, Simulation

_DEFAULT_K_L = 26
_DEFAULT_K_Q = 8
_DEFAULT_K = 312
_DEFAULT_DELTA_MOD = 0.13753354748577679


def _identity_density_matrix():
    """Return a flat, unit-trace 9x9 diagonal density matrix."""
    data = [complex(0.0, 0.0)] * 81
    for i in range(9):
        data[i * 9 + i] = complex(1.0 / 9.0, 0.0)
    return data


class BoundaryRegister:
    """Canonical 2D CFT visible boundary register.

    Parameters
    ----------
    k_l, k_q, K:
        Integer branch levels for the SU(2), SU(3), and parent sectors.
    population_displacement:
        Optional target ``||delta rho||_1``.  When provided, the phase-lock
        angle is chosen to reproduce this displacement.
    """

    def __init__(
        self,
        k_l: int = _DEFAULT_K_L,
        k_q: int = _DEFAULT_K_Q,
        K: int = _DEFAULT_K,
        population_displacement: Optional[float] = None,
    ) -> None:
        self.k_l = int(k_l)
        self.k_q = int(k_q)
        self.K = int(K)
        self.population_displacement = population_displacement
        if population_displacement is None:
            self.phase = 0.421
        else:
            self.phase = self._phase_for_displacement(float(population_displacement))

    def _phase_for_displacement(self, target: float) -> float:
        """Invert the phase -> population shift map by binary search."""
        if target <= 0.0:
            return 0.0

        sim = Simulation(grid_points=5)
        lo, hi = 0.0, math.pi / 2.0

        # Expand the upper bound if the requested displacement is larger than
        # the maximum reached on [0, pi/2].
        while sim.population_shift_at(hi) < target and hi < 2.0 * math.pi:
            lo = hi
            hi = min(hi + math.pi / 2.0, 2.0 * math.pi)

        for _ in range(50):
            mid = (lo + hi) / 2.0
            val = sim.population_shift_at(mid)
            if val < target:
                lo = mid
            else:
                hi = mid
        return (lo + hi) / 2.0

    def audit(self) -> dict:
        """Return unitarity and framing-defect diagnostics for the branch."""
        char = CharacterExcitationRegister(_identity_density_matrix())
        ok, residual = char.verify_unitarity()
        delta_fr = char.audit_framing_defect(self.k_l, self.k_q, self.K)
        return {
            "unitarity_ok": ok,
            "unitarity_residual": residual,
            "framing_defect": delta_fr,
        }

    @property
    def canonical_branch(self):
        return (self.k_l, self.k_q, self.K)

    def __repr__(self) -> str:
        return f"BoundaryRegister(k_l={self.k_l}, k_q={self.k_q}, K={self.K})"


class BulkSliceMetric:
    """Result wrapper for a projected bulk metric slice.

    Attributes
    ----------
    operational_power_mw : float
        Estimated operational power at the requested radius and velocity.
    proper_acceleration_norm : float
        Comoving proper-acceleration norm (m/s^2) on the plateau.
    wec_satisfied : bool
        Whether the weak energy condition audit passed.
    nec_satisfied : bool
        Whether the null energy condition audit passed.
    shift_profile_r_s : np.ndarray
        Radial coordinate ``r_s`` (m) for the 1-D FG shift profile.
    shift_profile_f : np.ndarray
        Warp shape function ``f(r_s)`` values.
    """

    def __init__(self, result: dict, radius_m: float, target_velocity_c: float) -> None:
        self._result = result
        self.radius_m = float(radius_m)
        self.target_velocity_c = float(target_velocity_c)
        self.operational_power_mw = float(result.get("power_mw", 0.0))
        self.delta_mod = float(result.get("delta_mod", 0.0))
        self.proper_acceleration_norm = float(
            result.get(
                "acceleration_norm_m_s2",
                result.get("causal", {})
                .get("audit", {})
                .get("acceleration_norm_m_s2", 0.0),
            )
        )
        se_audit = result.get("stress_energy", {}).get("audit", {})
        self.wec_satisfied = bool(
            se_audit.get("wec_passed", se_audit.get("passed", 0.0)) == 1.0
        )
        self.nec_satisfied = bool(
            se_audit.get("nec_passed", se_audit.get("passed", 0.0)) == 1.0
        )

        fg = result.get("fg_slice", {})
        self.shift_profile_r_s = np.array(np.abs(fg.get("x_m", [])))
        self.shift_profile_f = np.array(fg.get("shape", []))

    def __repr__(self) -> str:
        return (
            f"<BulkSliceMetric R={self.radius_m}m v={self.target_velocity_c}c "
            f"P={self.operational_power_mw:.2f}MW WEC={self.wec_satisfied}>"
        )


class FGSliceProjector:
    """High-level Fefferman-Graham slice projector.

    The constructor accepts a ``BoundaryRegister`` and grid settings; each call
    to ``project_bulk_slice`` runs a full Rust-backed simulation at the
    requested radius and target velocity and returns a ``BulkSliceMetric``.
    """

    def __init__(
        self,
        register: Optional[BoundaryRegister] = None,
        radius: float = 10.0,
        domain_radius: Optional[float] = None,
        grid_points: int = 301,
        wall_steepness: float = 0.8,
    ) -> None:
        self.register = register if register is not None else BoundaryRegister()
        self.radius = float(radius)
        self.domain_radius = float(
            domain_radius if domain_radius is not None else 3.0 * self.radius
        )
        self.grid_points = int(grid_points)
        self.wall_steepness = float(wall_steepness)

    def project_bulk_slice(
        self,
        radius_m: Optional[float] = None,
        target_velocity_c: Optional[float] = None,
        population_displacement: Optional[float] = None,
        grid_points: Optional[int] = None,
    ) -> BulkSliceMetric:
        """Project a bulk metric slice for the given radius and velocity.

        Parameters
        ----------
        radius_m:
            Bubble boundary radius in meters (default: projector radius).
        target_velocity_c:
            Target shift velocity in units of ``c``.
        population_displacement:
            Optional override for the unitary population displacement.
        grid_points:
            Optional override for the 1-D FG grid resolution.
        """
        radius_m = self.radius if radius_m is None else float(radius_m)
        grid_points = self.grid_points if grid_points is None else int(grid_points)

        if target_velocity_c is None:
            target_velocity_c = math.exp(_DEFAULT_DELTA_MOD / 2.0)
        else:
            target_velocity_c = float(target_velocity_c)
        if target_velocity_c <= 0.0:
            raise ValueError("target_velocity_c must be positive")

        if population_displacement is None:
            phase = self.register.phase
        else:
            phase = BoundaryRegister(
                population_displacement=float(population_displacement)
            ).phase

        delta_mod = 2.0 * math.log(target_velocity_c)
        result = Simulation(
            radius=radius_m,
            domain_radius=self.domain_radius,
            grid_points=grid_points,
            wall_steepness=self.wall_steepness,
            phase=phase,
            delta_mod=delta_mod,
            stress_grid_points=7,
        ).run()

        return BulkSliceMetric(result, radius_m, target_velocity_c)
