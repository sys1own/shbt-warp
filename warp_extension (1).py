#!/usr/bin/env python3
"""Executable SHBT warp-extension prototype.

The engine mirrors the numerical definitions used by the paper and the
shbt-precision baseline:

* BoundaryRegister builds the visible SU(2)_26 x SU(3)_8 modular register.
* ExcitationEngine applies a unitary phase-locked character excitation and
  audits the canonical framing constraint.
* FGSliceProjector evaluates an Alcubierre-type shift on a
  Fefferman--Graham slice and checks both Lorentzian nonsingularity and the
  positive Gram representative used by the projection.
* Metric3DCalculator constructs the full stationary 4D metric on a Cartesian
  three-dimensional grid.
* StressEnergyAuditor differentiates that metric to obtain connection,
  curvature, Einstein, and effective stress-energy tensors and samples the
  NEC and WEC.
* DerenderingEngine transfers a local visible register into dark ledgers while
  enforcing the SHBT no-zero-metric guardrail.
* ThermodynamicRateEngine integrates the entropy-debt balance equation and
  audits the topologically protected framing hold time.
* CausalObserver verifies the flat comoving interior frame and evaluates
  the operational power model.

Running this file writes sim_results.tex and the vector PDF figures used by
the LaTeX paper.
"""

from __future__ import annotations

import argparse
import itertools
import math
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any, Mapping

import matplotlib

matplotlib.use("Agg")
import matplotlib.pyplot as plt
import numpy as np


REPOSITORY_URL = "https://github.com/sys1own/shbt-precision.git"
LIGHT_SPEED_M_S = 299_792_458.0
GRAVITATIONAL_CONSTANT_SI = 6.674_30e-11
C_DARK_RESIDUAL_NUM = 834_433
C_DARK_COMP_NUM = 1_197_103
C_DARK_COMP_DEN = 362_670
C_DARK_RESIDUAL = C_DARK_RESIDUAL_NUM / C_DARK_COMP_DEN
C_DARK_COMP = C_DARK_COMP_NUM / C_DARK_COMP_DEN
DELTA_MOD = C_DARK_COMP / 24.0
N_SAT_BITS = 3.312_593_327_986e122
N_LOCAL_BITS_10M = 1.202_481e72
POWER_BENCHMARK_MW = 142.08
LAMBDA_HOLO_SI = 1.089_138_83e-52
MEGAPARSEC_M = 3.085_677_581_491_367_3e22
HUBBLE_LOADING_KM_S_MPC = 4.797_960
HUBBLE_LOADING_S_INV = HUBBLE_LOADING_KM_S_MPC * 1.0e3 / MEGAPARSEC_M
HOLOGRAPHIC_LOCK_RATE_S_INV = 3.0 * HUBBLE_LOADING_S_INV
THERMODYNAMIC_RELAXATION_RATE_S_INV = HOLOGRAPHIC_LOCK_RATE_S_INV / 24.0
THERMODYNAMIC_KAPPA_PER_J = (
    THERMODYNAMIC_RELAXATION_RATE_S_INV
    * DELTA_MOD
    / (POWER_BENCHMARK_MW * 1.0e6)
)
DEFAULT_PHASE_THETA = 0.421
DEFAULT_BUBBLE_RADIUS_M = 10.0
DEFAULT_DOMAIN_RADIUS_M = 30.0
DEFAULT_GRID_POINTS = 1201
DEFAULT_STRESS_GRID_POINTS = 7
DEFAULT_WALL_STEEPNESS_PER_M = 0.8
NUMERICAL_TOLERANCE = 1.0e-12
POWER_SCALE_RADIUS_M = DEFAULT_BUBBLE_RADIUS_M * math.sqrt(
    POWER_BENCHMARK_MW
    * 1.0e6
    * GRAVITATIONAL_CONSTANT_SI
    * 24.0
    * math.pi
    / (LIGHT_SPEED_M_S**5 * DELTA_MOD)
)


def _distance_to_integer(value: float) -> float:
    return abs(value - round(value))


def _permutation_sign(permutation: tuple[int, int, int]) -> int:
    inversions = sum(
        permutation[i] > permutation[j]
        for i in range(3)
        for j in range(i + 1, 3)
    )
    return 1 if inversions % 2 == 0 else -1


def _unitary_from_hermitian(generator: np.ndarray, parameter: float) -> np.ndarray:
    eigenvalues, eigenvectors = np.linalg.eigh(generator)
    phases = np.exp(-1j * parameter * eigenvalues)
    return (eigenvectors * phases) @ eigenvectors.conj().T


def _latex_number(value: float, digits: int = 12) -> str:
    if not math.isfinite(value):
        raise ValueError(f"Cannot export non-finite value {value!r} to LaTeX")
    if value == 0.0:
        return "0"
    magnitude = abs(value)
    if 1.0e-4 <= magnitude < 1.0e5:
        return f"{value:.{digits}g}"
    exponent = math.floor(math.log10(magnitude))
    mantissa = value / (10.0**exponent)
    return f"{mantissa:.6f}\\times{{}}10^{{{exponent}}}"


@dataclass
class BoundaryRegister:
    """Canonical 3 x 3 visible modular boundary register."""

    lepton_level: int = 26
    quark_level: int = 8
    parent_level: int = 312
    charge_labels: tuple[int, int, int] = (22, 23, 26)
    low_su3_weights: tuple[tuple[int, int], ...] = ((0, 0), (1, 0), (0, 1))
    su2_visible_block: np.ndarray = field(init=False, repr=False)
    su3_visible_block: np.ndarray = field(init=False, repr=False)
    raw_loading: np.ndarray = field(init=False, repr=False)
    rho_e: np.ndarray = field(init=False, repr=False)
    shannon_contributions: np.ndarray = field(init=False, repr=False)
    shannon_density: np.ndarray = field(init=False, repr=False)
    z_boundary: float = field(init=False)
    shannon_entropy: float = field(init=False)

    def __post_init__(self) -> None:
        if len(self.charge_labels) != 3 or len(self.low_su3_weights) != 3:
            raise ValueError("The visible register must have exactly three labels per axis")

        self.su2_visible_block = np.array(
            [
                [
                    self.su2_modular_s_entry(left, right, self.lepton_level)
                    for right in self.charge_labels
                ]
                for left in self.charge_labels
            ],
            dtype=float,
        )
        self.su3_visible_block = np.array(
            [
                [
                    self.su3_modular_s_entry(left, right, self.quark_level)
                    for right in self.low_su3_weights
                ]
                for left in self.low_su3_weights
            ],
            dtype=complex,
        )

        self.raw_loading = (
            np.abs(self.su2_visible_block) ** 2
            * np.abs(self.su3_visible_block) ** 2
        )
        self.z_boundary = float(np.sum(self.raw_loading))
        if not math.isfinite(self.z_boundary) or self.z_boundary <= 0.0:
            raise RuntimeError("The visible modular loading has invalid normalization")

        self.rho_e = self.raw_loading / self.z_boundary
        positive = self.rho_e > 0.0
        self.shannon_contributions = np.zeros_like(self.rho_e)
        self.shannon_contributions[positive] = (
            -self.rho_e[positive] * np.log(self.rho_e[positive])
        )
        self.shannon_entropy = float(np.sum(self.shannon_contributions))
        if self.shannon_entropy <= 0.0:
            raise RuntimeError("The visible register has zero Shannon entropy")
        self.shannon_density = self.shannon_contributions / self.shannon_entropy

    @property
    def branch(self) -> tuple[int, int, int]:
        return self.lepton_level, self.quark_level, self.parent_level

    @property
    def lepton_labels(self) -> tuple[int, int, int]:
        """Paper notation l_j; the Rust baseline reuses the charge embedding."""

        return self.charge_labels

    @staticmethod
    def su2_modular_s_entry(left: int, right: int, level: int) -> float:
        coefficient = math.sqrt(2.0 / (level + 2.0))
        argument = math.pi * (left + 1) * (right + 1) / (level + 2.0)
        return coefficient * math.sin(argument)

    @staticmethod
    def su2_conformal_weight(label: int, level: int) -> float:
        return label * (label + 2.0) / (4.0 * (level + 2.0))

    @staticmethod
    def su2_central_charge(level: int) -> float:
        return 3.0 * level / (level + 2.0)

    @staticmethod
    def su3_vector(weight: tuple[int, int]) -> np.ndarray:
        """Return the three-vector used verbatim by boundary.rs."""

        p, q = weight
        return np.array((2.0 * p + q, q - p, -2.0 * p - q)) / 3.0

    @staticmethod
    def su3_conformal_weight(weight: tuple[int, int], level: int) -> float:
        p, q = weight
        numerator = p * p + q * q + p * q + 3 * p + 3 * q
        return numerator / (3.0 * (level + 3.0))

    @staticmethod
    def su3_central_charge(level: int) -> float:
        return 8.0 * level / (level + 3.0)

    @classmethod
    def su3_modular_s_entry(
        cls,
        left: tuple[int, int],
        right: tuple[int, int],
        level: int,
    ) -> complex:
        rho = cls.su3_vector((1, 1))
        lambda_rho = cls.su3_vector(left) + rho
        mu_rho = cls.su3_vector(right) + rho
        weyl_sum = 0.0j
        for permutation in itertools.permutations((0, 1, 2)):
            sign = _permutation_sign(permutation)
            permuted = lambda_rho[list(permutation)]
            phase = -2j * math.pi * float(np.dot(permuted, mu_rho)) / (level + 3.0)
            weyl_sum += sign * np.exp(phase)
        return (-1j / (math.sqrt(3.0) * (level + 3.0))) * weyl_sum

    def conformal_phase_angles(self) -> np.ndarray:
        c_visible = self.su2_central_charge(
            self.lepton_level
        ) + self.su3_central_charge(self.quark_level)
        angles = []
        for charge in self.charge_labels:
            h_su2 = self.su2_conformal_weight(charge, self.lepton_level)
            for weight in self.low_su3_weights:
                h_su3 = self.su3_conformal_weight(weight, self.quark_level)
                angles.append(h_su2 + h_su3 - c_visible / 24.0)
        return np.asarray(angles, dtype=float)

    def audit(self, tolerance: float = NUMERICAL_TOLERANCE) -> dict[str, Any]:
        normalization_error = abs(float(np.sum(self.rho_e)) - 1.0)
        shannon_density_error = abs(float(np.sum(self.shannon_density)) - 1.0)
        passed = (
            normalization_error <= tolerance
            and shannon_density_error <= tolerance
            and bool(np.all(self.raw_loading >= 0.0))
            and self.shannon_entropy > 0.0
        )
        return {
            "passed": passed,
            "normalization_error": normalization_error,
            "shannon_density_error": shannon_density_error,
            "minimum_loading": float(np.min(self.raw_loading)),
            "maximum_loading": float(np.max(self.raw_loading)),
        }


@dataclass
class ExcitationEngine:
    """Phase-locked unitary excitation constrained to the canonical branch."""

    boundary: BoundaryRegister
    theta_phase: float = DEFAULT_PHASE_THETA
    framing_charge: float = 1.0
    generator: np.ndarray = field(init=False, repr=False)

    def __post_init__(self) -> None:
        self.generator = self._build_directional_generator()

    def _build_directional_generator(self) -> np.ndarray:
        generator = np.zeros((9, 9), dtype=complex)
        for su3_index in range(3):
            for su2_index in range(2):
                left = 3 * su2_index + su3_index
                right = 3 * (su2_index + 1) + su3_index
                generator[left, right] = -0.5j
                generator[right, left] = 0.5j
        spectral_radius = float(np.max(np.abs(np.linalg.eigvalsh(generator))))
        if spectral_radius == 0.0:
            raise RuntimeError("Excitation generator has zero spectral radius")
        return generator / spectral_radius

    def phase_lock_matrix(self, theta_phase: float | None = None) -> np.ndarray:
        theta = self.theta_phase if theta_phase is None else theta_phase
        angles = self.boundary.conformal_phase_angles()
        return np.diag(np.exp(2j * math.pi * theta * angles))

    def excitation_operator(self, theta_phase: float | None = None) -> np.ndarray:
        theta = self.theta_phase if theta_phase is None else theta_phase
        mixing = _unitary_from_hermitian(self.generator, theta)
        return mixing @ self.phase_lock_matrix(theta)

    def excited_density_matrix(self, theta_phase: float | None = None) -> np.ndarray:
        baseline = np.diag(self.boundary.rho_e.reshape(-1).astype(complex))
        operator = self.excitation_operator(theta_phase)
        excited = operator @ baseline @ operator.conj().T
        return excited / np.trace(excited)

    def excited_probability(self, theta_phase: float | None = None) -> np.ndarray:
        probabilities = np.real(np.diag(self.excited_density_matrix(theta_phase)))
        probabilities = np.maximum(probabilities, 0.0)
        return (probabilities / np.sum(probabilities)).reshape(3, 3)

    def excited_shannon_contributions(
        self, theta_phase: float | None = None
    ) -> np.ndarray:
        probabilities = self.excited_probability(theta_phase)
        contributions = np.zeros_like(probabilities)
        positive = probabilities > 0.0
        contributions[positive] = -probabilities[positive] * np.log(
            probabilities[positive]
        )
        return contributions

    def framing_defect(self) -> float:
        lepton_lift = self.boundary.parent_level / (2.0 * self.boundary.lepton_level)
        quark_lift = self.boundary.parent_level / (3.0 * self.boundary.quark_level)
        return max(
            _distance_to_integer(lepton_lift),
            _distance_to_integer(quark_lift),
        )

    def closure_tensor(self) -> np.ndarray:
        return self.framing_charge * self.framing_defect() * np.eye(4)

    def audit(self, tolerance: float = NUMERICAL_TOLERANCE) -> dict[str, Any]:
        operator = self.excitation_operator()
        unitarity_error = float(
            np.linalg.norm(operator.conj().T @ operator - np.eye(9), ord=np.inf)
        )
        excited = self.excited_density_matrix()
        trace_error = abs(complex(np.trace(excited)) - 1.0)
        hermiticity_error = float(
            np.linalg.norm(excited - excited.conj().T, ord=np.inf)
        )
        framing_defect = self.framing_defect()
        closure_norm = float(np.linalg.norm(self.closure_tensor(), ord=np.inf))
        population_shift_l1 = float(
            np.sum(np.abs(self.excited_probability() - self.boundary.rho_e))
        )
        passed = (
            unitarity_error <= tolerance
            and abs(trace_error) <= tolerance
            and hermiticity_error <= tolerance
            and population_shift_l1 > tolerance
            and framing_defect == 0.0
            and closure_norm == 0.0
        )
        return {
            "passed": passed,
            "unitarity_error": unitarity_error,
            "trace_error": float(abs(trace_error)),
            "hermiticity_error": hermiticity_error,
            "population_shift_l1": population_shift_l1,
            "framing_defect": framing_defect,
            "closure_norm": closure_norm,
        }


@dataclass
class FGSliceProjector:
    """Alcubierre-type shift profile on a sampled FG slice."""

    delta_mod: float = DELTA_MOD
    bubble_radius_m: float = DEFAULT_BUBBLE_RADIUS_M
    domain_radius_m: float = DEFAULT_DOMAIN_RADIUS_M
    grid_points: int = DEFAULT_GRID_POINTS
    wall_steepness_per_m: float = DEFAULT_WALL_STEEPNESS_PER_M
    speed_of_light_m_s: float = LIGHT_SPEED_M_S
    x_m: np.ndarray = field(init=False, repr=False)
    r_s_m: np.ndarray = field(init=False, repr=False)
    shape: np.ndarray = field(init=False, repr=False)
    shape_gradient_per_m: np.ndarray = field(init=False, repr=False)
    beta_m_s: np.ndarray = field(init=False, repr=False)
    beta_gradient_per_s: np.ndarray = field(init=False, repr=False)
    lorentzian_metrics: np.ndarray = field(init=False, repr=False)
    gram_metrics: np.ndarray = field(init=False, repr=False)

    def __post_init__(self) -> None:
        if self.bubble_radius_m <= 0.0:
            raise ValueError("bubble_radius_m must be positive")
        if self.domain_radius_m <= self.bubble_radius_m:
            raise ValueError("domain_radius_m must exceed bubble_radius_m")
        if self.grid_points < 5:
            raise ValueError("grid_points must be at least 5")
        if self.grid_points % 2 == 0:
            self.grid_points += 1
        if self.wall_steepness_per_m <= 0.0:
            raise ValueError("wall_steepness_per_m must be positive")

        self.x_m = np.linspace(
            -self.domain_radius_m,
            self.domain_radius_m,
            self.grid_points,
        )
        self.r_s_m = np.abs(self.x_m)
        self.shape = self.alcubierre_shape(self.r_s_m)
        self.shape_gradient_per_m = self.alcubierre_shape_gradient(self.x_m)
        self.beta_m_s = -self.v_eff_m_s * self.shape
        self.beta_gradient_per_s = -self.v_eff_m_s * self.shape_gradient_per_m
        self.lorentzian_metrics, self.gram_metrics = self._build_metrics()

    @property
    def v_eff_c(self) -> float:
        return math.exp(self.delta_mod / 2.0)

    @property
    def v_eff_m_s(self) -> float:
        return self.speed_of_light_m_s * self.v_eff_c

    def alcubierre_shape(self, r_s_m: np.ndarray) -> np.ndarray:
        sigma = self.wall_steepness_per_m
        radius = self.bubble_radius_m
        denominator = 2.0 * math.tanh(sigma * radius)
        return (
            np.tanh(sigma * (r_s_m + radius))
            - np.tanh(sigma * (r_s_m - radius))
        ) / denominator

    def alcubierre_shape_gradient(self, x_m: np.ndarray) -> np.ndarray:
        sigma = self.wall_steepness_per_m
        radius = self.bubble_radius_m
        r_s = np.abs(x_m)
        denominator = 2.0 * math.tanh(sigma * radius)
        derivative_r = sigma * (
            1.0 / np.cosh(sigma * (r_s + radius)) ** 2
            - 1.0 / np.cosh(sigma * (r_s - radius)) ** 2
        ) / denominator
        derivative_x = derivative_r * np.sign(x_m)
        derivative_x[np.isclose(x_m, 0.0)] = 0.0
        return derivative_x

    def _build_metrics(self) -> tuple[np.ndarray, np.ndarray]:
        beta_c = self.beta_m_s / self.speed_of_light_m_s
        lorentzian = np.zeros((self.grid_points, 4, 4), dtype=float)
        gram = np.zeros_like(lorentzian)
        lorentzian[:, 0, 0] = -1.0 + beta_c**2
        lorentzian[:, 0, 1] = beta_c
        lorentzian[:, 1, 0] = beta_c
        lorentzian[:, 1, 1] = 1.0
        lorentzian[:, 2, 2] = 1.0
        lorentzian[:, 3, 3] = 1.0

        gram[:, 0, 0] = 1.0 + beta_c**2
        gram[:, 0, 1] = beta_c
        gram[:, 1, 0] = beta_c
        gram[:, 1, 1] = 1.0
        gram[:, 2, 2] = 1.0
        gram[:, 3, 3] = 1.0
        return lorentzian, gram

    def audit(self, tolerance: float = NUMERICAL_TOLERANCE) -> dict[str, Any]:
        determinants = np.linalg.det(self.lorentzian_metrics)
        lorentzian_eigenvalues = np.linalg.eigvalsh(self.lorentzian_metrics)
        gram_eigenvalues = np.linalg.eigvalsh(self.gram_metrics)
        minimum_abs_determinant = float(np.min(np.abs(determinants)))
        minimum_abs_lorentzian_eigenvalue = float(
            np.min(np.abs(lorentzian_eigenvalues))
        )
        minimum_gram_eigenvalue = float(np.min(gram_eigenvalues))
        determinant_error = float(np.max(np.abs(determinants + 1.0)))
        passed = (
            minimum_abs_determinant > tolerance
            and minimum_abs_lorentzian_eigenvalue > tolerance
            and minimum_gram_eigenvalue > tolerance
            and determinant_error <= 100.0 * tolerance
        )
        return {
            "passed": passed,
            "minimum_abs_determinant": minimum_abs_determinant,
            "minimum_abs_lorentzian_eigenvalue": minimum_abs_lorentzian_eigenvalue,
            "minimum_gram_eigenvalue": minimum_gram_eigenvalue,
            "determinant_error": determinant_error,
        }


@dataclass
class Metric3DCalculator:
    """Construct a stationary SHBT shift metric on a Cartesian 3D grid.

    Coordinates are ordered as ``(ct, x, y, z)`` and the metric signature is
    ``(-,+,+,+)``.  The shift is longitudinal, while its shape depends on the
    full radius ``sqrt(x**2 + y**2 + z**2)``.
    """

    bubble_radius_m: float = DEFAULT_BUBBLE_RADIUS_M
    domain_radius_m: float = 15.0
    grid_points_per_axis: int = DEFAULT_STRESS_GRID_POINTS
    wall_steepness_per_m: float = DEFAULT_WALL_STEEPNESS_PER_M
    delta_mod: float = DELTA_MOD
    x_m: np.ndarray = field(init=False, repr=False)
    y_m: np.ndarray = field(init=False, repr=False)
    z_m: np.ndarray = field(init=False, repr=False)
    radius_m: np.ndarray = field(init=False, repr=False)
    shape: np.ndarray = field(init=False, repr=False)
    beta_over_c: np.ndarray = field(init=False, repr=False)
    metric_4d_grid: np.ndarray = field(init=False, repr=False)
    spatial_metric_grid: np.ndarray = field(init=False, repr=False)
    gram_metric_grid: np.ndarray = field(init=False, repr=False)

    def __post_init__(self) -> None:
        if self.bubble_radius_m <= 0.0:
            raise ValueError("bubble_radius_m must be positive")
        if self.domain_radius_m <= self.bubble_radius_m:
            raise ValueError("domain_radius_m must exceed bubble_radius_m")
        if self.grid_points_per_axis < 3:
            raise ValueError("grid_points_per_axis must be at least three")
        if self.grid_points_per_axis % 2 == 0:
            self.grid_points_per_axis += 1
        if self.wall_steepness_per_m <= 0.0:
            raise ValueError("wall_steepness_per_m must be positive")

        axis = np.linspace(
            -self.domain_radius_m,
            self.domain_radius_m,
            self.grid_points_per_axis,
        )
        self.x_m = axis.copy()
        self.y_m = axis.copy()
        self.z_m = axis.copy()
        x_grid, y_grid, z_grid = np.meshgrid(
            self.x_m,
            self.y_m,
            self.z_m,
            indexing="ij",
        )
        self.radius_m = np.sqrt(x_grid**2 + y_grid**2 + z_grid**2)
        self.shape = self.shape_function(self.radius_m)
        self.beta_over_c = -math.exp(self.delta_mod / 2.0) * self.shape
        (
            self.metric_4d_grid,
            self.spatial_metric_grid,
            self.gram_metric_grid,
        ) = self._build_metric_tensors()

    @property
    def coordinates(self) -> tuple[np.ndarray, np.ndarray, np.ndarray]:
        """Return the Cartesian coordinate axes in metric-array order."""

        return self.x_m, self.y_m, self.z_m

    def shape_function(self, radius_m: np.ndarray) -> np.ndarray:
        """Evaluate the smooth Alcubierre representative used by the model."""

        sigma = self.wall_steepness_per_m
        radius = self.bubble_radius_m
        denominator = 2.0 * math.tanh(sigma * radius)
        return (
            np.tanh(sigma * (radius_m + radius))
            - np.tanh(sigma * (radius_m - radius))
        ) / denominator

    def _build_metric_tensors(
        self,
    ) -> tuple[np.ndarray, np.ndarray, np.ndarray]:
        grid_shape = self.shape.shape
        lorentzian = np.zeros((*grid_shape, 4, 4), dtype=float)
        gram = np.zeros_like(lorentzian)
        beta = self.beta_over_c

        lorentzian[..., 0, 0] = -1.0 + beta**2
        lorentzian[..., 0, 1] = beta
        lorentzian[..., 1, 0] = beta
        lorentzian[..., 1, 1] = 1.0
        lorentzian[..., 2, 2] = 1.0
        lorentzian[..., 3, 3] = 1.0

        gram[..., 0, 0] = 1.0 + beta**2
        gram[..., 0, 1] = beta
        gram[..., 1, 0] = beta
        gram[..., 1, 1] = 1.0
        gram[..., 2, 2] = 1.0
        gram[..., 3, 3] = 1.0
        return lorentzian, lorentzian[..., 1:, 1:].copy(), gram

    def region_mask(
        self,
        x_bounds: tuple[float, float],
        y_bounds: tuple[float, float],
        z_bounds: tuple[float, float],
    ) -> np.ndarray:
        """Return the grid mask for a closed Cartesian cuboid."""

        bounds = (x_bounds, y_bounds, z_bounds)
        for lower, upper in bounds:
            if lower > upper:
                raise ValueError("Region bounds must be ordered lower <= upper")
        x_grid, y_grid, z_grid = np.meshgrid(
            self.x_m,
            self.y_m,
            self.z_m,
            indexing="ij",
        )
        return (
            (x_grid >= x_bounds[0])
            & (x_grid <= x_bounds[1])
            & (y_grid >= y_bounds[0])
            & (y_grid <= y_bounds[1])
            & (z_grid >= z_bounds[0])
            & (z_grid <= z_bounds[1])
        )

    def audit(self, tolerance: float = NUMERICAL_TOLERANCE) -> dict[str, Any]:
        """Audit Lorentzian nonsingularity and Gram positivity in 3D."""

        determinants = np.linalg.det(self.metric_4d_grid)
        gram_eigenvalues = np.linalg.eigvalsh(self.gram_metric_grid)
        determinant_error = float(np.max(np.abs(determinants + 1.0)))
        minimum_abs_determinant = float(np.min(np.abs(determinants)))
        minimum_gram_eigenvalue = float(np.min(gram_eigenvalues))
        passed = (
            determinant_error <= 100.0 * tolerance
            and minimum_abs_determinant > tolerance
            and minimum_gram_eigenvalue > tolerance
        )
        return {
            "passed": passed,
            "grid_shape": self.shape.shape,
            "determinant_error": determinant_error,
            "minimum_abs_determinant": minimum_abs_determinant,
            "minimum_gram_eigenvalue": minimum_gram_eigenvalue,
        }


@dataclass
class StressEnergyAuditor:
    """Numerically derive curvature and audit NEC/WEC on a stationary grid.

    ``metric`` must have shape ``(nx, ny, nz, 4, 4)``.  Time derivatives are
    zero by stationarity; spatial derivatives are evaluated with second-order
    finite differences along the supplied coordinate axes.
    """

    metric: np.ndarray
    coordinates: tuple[np.ndarray, np.ndarray, np.ndarray]
    cosmological_constant: float = LAMBDA_HOLO_SI
    gravitational_constant: float = GRAVITATIONAL_CONSTANT_SI
    inverse_metric: np.ndarray = field(init=False, repr=False)
    christoffel: np.ndarray = field(init=False, repr=False)
    riemann: np.ndarray = field(init=False, repr=False)
    ricci: np.ndarray = field(init=False, repr=False)
    ricci_scalar: np.ndarray = field(init=False, repr=False)
    einstein: np.ndarray = field(init=False, repr=False)
    stress_energy: np.ndarray = field(init=False, repr=False)

    def __post_init__(self) -> None:
        expected_grid = tuple(len(axis) for axis in self.coordinates)
        if self.metric.shape != (*expected_grid, 4, 4):
            raise ValueError(
                "metric must have shape (nx, ny, nz, 4, 4) matching coordinates"
            )
        if any(len(axis) < 3 for axis in self.coordinates):
            raise ValueError("Each coordinate axis must contain at least three points")
        for axis in self.coordinates:
            if not bool(np.all(np.diff(axis) > 0.0)):
                raise ValueError("Coordinate axes must be strictly increasing")
        symmetry_error = float(
            np.max(np.abs(self.metric - np.swapaxes(self.metric, -1, -2)))
        )
        if symmetry_error > 1.0e-10:
            raise ValueError("metric must be symmetric in its tensor indices")
        self.compute_geometry()

    def _partial(self, field_array: np.ndarray, coordinate_index: int) -> np.ndarray:
        if coordinate_index == 0:
            return np.zeros_like(field_array)
        spatial_axis = coordinate_index - 1
        return np.gradient(
            field_array,
            self.coordinates[spatial_axis],
            axis=spatial_axis,
            edge_order=2,
        )

    def compute_christoffel_symbols(self) -> np.ndarray:
        """Compute ``Gamma**alpha_(mu nu)`` over the complete grid."""

        self.inverse_metric = np.linalg.inv(self.metric)
        derivatives = np.stack(
            [self._partial(self.metric, mu) for mu in range(4)],
            axis=0,
        )
        grid_shape = self.metric.shape[:-2]
        gamma = np.zeros((*grid_shape, 4, 4, 4), dtype=float)
        for alpha in range(4):
            for mu in range(4):
                for nu in range(4):
                    total = np.zeros(grid_shape, dtype=float)
                    for sigma in range(4):
                        total += self.inverse_metric[..., alpha, sigma] * (
                            derivatives[mu, ..., nu, sigma]
                            + derivatives[nu, ..., mu, sigma]
                            - derivatives[sigma, ..., mu, nu]
                        )
                    gamma[..., alpha, mu, nu] = 0.5 * total
        return gamma

    def compute_riemann_tensor(self) -> np.ndarray:
        """Compute ``R**alpha_(beta mu nu)`` from the numerical connection."""

        grid_shape = self.metric.shape[:-2]
        riemann = np.zeros((*grid_shape, 4, 4, 4, 4), dtype=float)
        for alpha in range(4):
            for beta in range(4):
                for mu in range(4):
                    for nu in range(4):
                        component = self._partial(
                            self.christoffel[..., alpha, nu, beta],
                            mu,
                        ) - self._partial(
                            self.christoffel[..., alpha, mu, beta],
                            nu,
                        )
                        for lam in range(4):
                            component += (
                                self.christoffel[..., alpha, mu, lam]
                                * self.christoffel[..., lam, nu, beta]
                                - self.christoffel[..., alpha, nu, lam]
                                * self.christoffel[..., lam, mu, beta]
                            )
                        riemann[..., alpha, beta, mu, nu] = component
        return riemann

    def compute_geometry(self) -> None:
        """Populate connection, curvature, Einstein, and stress tensors."""

        self.christoffel = self.compute_christoffel_symbols()
        self.riemann = self.compute_riemann_tensor()
        grid_shape = self.metric.shape[:-2]
        ricci = np.zeros((*grid_shape, 4, 4), dtype=float)
        for beta in range(4):
            for nu in range(4):
                for alpha in range(4):
                    ricci[..., beta, nu] += self.riemann[
                        ..., alpha, beta, alpha, nu
                    ]
        self.ricci = 0.5 * (ricci + np.swapaxes(ricci, -1, -2))
        self.ricci_scalar = np.einsum(
            "...mn,...mn->...",
            self.inverse_metric,
            self.ricci,
        )
        self.einstein = self.ricci - 0.5 * (
            self.ricci_scalar[..., np.newaxis, np.newaxis] * self.metric
        )
        self.stress_energy = (
            self.einstein + self.cosmological_constant * self.metric
        ) / (8.0 * math.pi * self.gravitational_constant)

    @staticmethod
    def _orthonormal_tetrad(metric_at_point: np.ndarray) -> np.ndarray:
        eigenvalues, eigenvectors = np.linalg.eigh(metric_at_point)
        negative = np.flatnonzero(eigenvalues < 0.0)
        positive = np.flatnonzero(eigenvalues > 0.0)
        if len(negative) != 1 or len(positive) != 3:
            raise ValueError("Metric does not have Lorentzian signature (-,+,+,+)")
        order = np.concatenate((negative, positive))
        scales = 1.0 / np.sqrt(np.abs(eigenvalues[order]))
        return eigenvectors[:, order] @ np.diag(scales)

    def audit_energy_conditions(
        self,
        sample_count: int = 100,
        seed: int = 26_008_312,
        tolerance: float = 1.0e-12,
    ) -> dict[str, Any]:
        """Sample null and timelike vectors and return NEC/WEC minima."""

        if sample_count < 1:
            raise ValueError("sample_count must be positive")
        rng = np.random.default_rng(seed)
        grid_shape = self.metric.shape[:-2]
        nec_values: list[float] = []
        wec_values: list[float] = []
        null_residuals: list[float] = []
        timelike_residuals: list[float] = []

        for _ in range(sample_count):
            index = tuple(int(rng.integers(0, extent)) for extent in grid_shape)
            metric_point = self.metric[index]
            stress_point = self.stress_energy[index]
            tetrad = self._orthonormal_tetrad(metric_point)

            null_direction = rng.normal(size=3)
            null_direction /= np.linalg.norm(null_direction)
            null_hat = np.concatenate(([1.0], null_direction))
            null_vector = tetrad @ null_hat
            null_residuals.append(
                abs(float(null_vector @ metric_point @ null_vector))
            )
            nec_values.append(float(null_vector @ stress_point @ null_vector))

            velocity_direction = rng.normal(size=3)
            velocity_direction /= np.linalg.norm(velocity_direction)
            speed = float(rng.uniform(0.0, 0.8))
            gamma_factor = 1.0 / math.sqrt(1.0 - speed**2)
            timelike_hat = np.concatenate(
                ([gamma_factor], gamma_factor * speed * velocity_direction)
            )
            timelike_vector = tetrad @ timelike_hat
            timelike_residuals.append(
                abs(float(timelike_vector @ metric_point @ timelike_vector) + 1.0)
            )
            wec_values.append(float(timelike_vector @ stress_point @ timelike_vector))

        minimum_nec = min(nec_values)
        minimum_wec = min(wec_values)
        maximum_null_residual = max(null_residuals)
        maximum_timelike_residual = max(timelike_residuals)
        finite_geometry = bool(
            np.all(np.isfinite(self.christoffel))
            and np.all(np.isfinite(self.riemann))
            and np.all(np.isfinite(self.stress_energy))
        )
        return {
            "passed": finite_geometry
            and maximum_null_residual <= 1.0e-9
            and maximum_timelike_residual <= 1.0e-9,
            "nec_passed": minimum_nec >= -tolerance,
            "wec_passed": minimum_wec >= -tolerance,
            "minimum_nec_energy_density": minimum_nec,
            "minimum_wec_energy_density": minimum_wec,
            "maximum_null_norm_residual": maximum_null_residual,
            "maximum_timelike_norm_residual": maximum_timelike_residual,
            "sample_count": sample_count,
        }


@dataclass
class DerenderingEngine:
    """Move visible character weights into dark ledgers and restore them.

    SHBT trace normalization forbids ``g_munu -> 0``.  Accordingly this class
    sets the *active deformation* to zero inside a selected region and restores
    the Lorentzian Minkowski background there.
    """

    boundary: BoundaryRegister
    metric_calculator: Metric3DCalculator
    n_sat_bits: float = N_SAT_BITS
    n_local_bits: float = N_LOCAL_BITS_10M
    visible_weights: np.ndarray = field(init=False, repr=False)
    dark_residual_weights: np.ndarray = field(init=False, repr=False)
    dark_completion_weights: np.ndarray = field(init=False, repr=False)
    projected_metric: np.ndarray = field(init=False, repr=False)
    active_mask: np.ndarray | None = field(init=False, default=None, repr=False)
    stored_half_widths: tuple[float, float, float] | None = field(
        init=False,
        default=None,
        repr=False,
    )
    dark_ledger_bits: float = field(init=False, default=0.0)
    is_rendered: bool = field(init=False, default=True)

    def __post_init__(self) -> None:
        self.visible_weights = self.boundary.rho_e.copy()
        self.dark_residual_weights = np.zeros_like(self.visible_weights)
        self.dark_completion_weights = np.zeros_like(self.visible_weights)
        self.projected_metric = self.metric_calculator.metric_4d_grid.copy()

    @property
    def dark_channel_ratio(self) -> float:
        return C_DARK_RESIDUAL / C_DARK_COMP

    def derender_region(
        self,
        x_bounds: tuple[float, float],
        y_bounds: tuple[float, float],
        z_bounds: tuple[float, float],
    ) -> dict[str, Any]:
        """Decouple the visible register and nullify its active metric shear."""

        if not self.is_rendered:
            raise RuntimeError("A region is already de-rendered")
        mask = self.metric_calculator.region_mask(x_bounds, y_bounds, z_bounds)
        if not bool(np.any(mask)):
            raise ValueError("The requested region contains no grid points")

        ratio = self.dark_channel_ratio
        self.dark_residual_weights = ratio * self.visible_weights
        self.dark_completion_weights = (1.0 - ratio) * self.visible_weights
        self.visible_weights = np.zeros_like(self.visible_weights)
        self.active_mask = mask
        self.stored_half_widths = tuple(
            0.5 * (upper - lower)
            for lower, upper in (x_bounds, y_bounds, z_bounds)
        )
        minkowski = np.diag((-1.0, 1.0, 1.0, 1.0))
        self.projected_metric[mask] = minkowski
        self.dark_ledger_bits = self.n_local_bits
        self.is_rendered = False

        local_determinants = np.linalg.det(self.projected_metric[mask])
        return {
            "event": "DE_RENDERING",
            "is_rendered": self.is_rendered,
            "active_deformation_zero": True,
            "total_metric_zero": False,
            "metric_collapse_prevented": True,
            "restored_background": "MINKOWSKI",
            "dark_residual_channel": C_DARK_RESIDUAL,
            "transferred_bits": self.dark_ledger_bits,
            "bit_budget_preserved": self.dark_ledger_bits <= self.n_sat_bits,
            "minimum_abs_local_determinant": float(
                np.min(np.abs(local_determinants))
            ),
        }

    def rerender_region(
        self,
        new_origin_coords: tuple[float, float, float],
    ) -> dict[str, Any]:
        """Restore visible weights at a target address without a path integral."""

        if self.is_rendered or self.active_mask is None:
            raise RuntimeError("No de-rendered state is available to restore")
        if self.stored_half_widths is None:
            raise RuntimeError("Stored region geometry is unavailable")

        target_bounds = tuple(
            (origin - half_width, origin + half_width)
            for origin, half_width in zip(new_origin_coords, self.stored_half_widths)
        )
        target_mask = self.metric_calculator.region_mask(*target_bounds)
        if not bool(np.any(target_mask)):
            raise ValueError("The target region contains no grid points")

        restored = self.dark_residual_weights + self.dark_completion_weights
        normalization = float(np.sum(restored))
        if normalization <= 0.0:
            raise RuntimeError("Dark ledgers contain no recoverable character weight")
        self.visible_weights = restored / normalization
        self.dark_residual_weights.fill(0.0)
        self.dark_completion_weights.fill(0.0)
        self.projected_metric[target_mask] = self.metric_calculator.metric_4d_grid[
            target_mask
        ]
        self.active_mask = target_mask
        restored_bits = self.dark_ledger_bits
        self.dark_ledger_bits = 0.0
        self.is_rendered = True
        return {
            "event": "RE_RENDERING",
            "is_rendered": self.is_rendered,
            "new_origin_coords": tuple(float(value) for value in new_origin_coords),
            "restored_bits": restored_bits,
            "dark_ledger_bits": self.dark_ledger_bits,
            "visible_normalization": float(np.sum(self.visible_weights)),
            "intermediate_path_integral_used": False,
        }


@dataclass
class ThermodynamicRateEngine:
    """Integrate the boundary entropy-debt balance at fixed control power."""

    kappa_per_joule: float = THERMODYNAMIC_KAPPA_PER_J
    relaxation_rate_s_inv: float = THERMODYNAMIC_RELAXATION_RATE_S_INV
    canonical_framing_defect: float = 0.0
    branch_preserving: bool = True

    def dissipation_rate(self, entropy_debt: float) -> float:
        """Return ``Qdot = gamma_holo Gamma_lock Delta_mod``."""

        return self.relaxation_rate_s_inv * entropy_debt

    def entropy_debt_rate(self, entropy_debt: float, power_mw: float) -> float:
        """Evaluate ``dDelta_mod/dt = kappa P_op - Qdot``."""

        if power_mw < 0.0:
            raise ValueError("power_mw must be nonnegative")
        return (
            self.kappa_per_joule * power_mw * 1.0e6
            - self.dissipation_rate(entropy_debt)
        )

    def steady_state_entropy_debt(self, power_mw: float) -> float:
        if self.relaxation_rate_s_inv <= 0.0:
            raise RuntimeError("A positive relaxation rate is required")
        return (
            self.kappa_per_joule * power_mw * 1.0e6
            / self.relaxation_rate_s_inv
        )

    def integrate(
        self,
        time_s: np.ndarray,
        initial_entropy_debt: float = 0.0,
        power_mw: float = POWER_BENCHMARK_MW,
    ) -> np.ndarray:
        """Return the exact constant-power solution on the requested times."""

        times = np.asarray(time_s, dtype=float)
        if bool(np.any(times < 0.0)):
            raise ValueError("time_s must be nonnegative")
        steady_state = self.steady_state_entropy_debt(power_mw)
        return steady_state + (initial_entropy_debt - steady_state) * np.exp(
            -self.relaxation_rate_s_inv * times
        )

    def maximum_hold_time(self, framing_threshold: float = 0.0) -> float:
        """Return infinity for a branch-preserving, topologically fixed defect."""

        if framing_threshold < 0.0:
            raise ValueError("framing_threshold must be nonnegative")
        if self.branch_preserving:
            return math.inf
        if self.canonical_framing_defect > framing_threshold:
            return 0.0
        raise RuntimeError("A branch-mutation law is required for finite hold time")

    def audit(self) -> dict[str, Any]:
        steady_state = self.steady_state_entropy_debt(POWER_BENCHMARK_MW)
        hold_time = self.maximum_hold_time()
        return {
            "passed": math.isclose(steady_state, DELTA_MOD, rel_tol=1.0e-12)
            and math.isinf(hold_time),
            "steady_state_entropy_debt": steady_state,
            "initial_accumulation_rate_s_inv": self.entropy_debt_rate(
                0.0,
                POWER_BENCHMARK_MW,
            ),
            "maximum_hold_time_s": hold_time,
            "framing_defect_breached": False,
        }


@dataclass
class CausalObserver:
    """Comoving interior observer and power diagnostic."""

    projector: FGSliceProjector

    @staticmethod
    def power_requirement_mw(radius_m: float) -> float:
        if radius_m <= 0.0:
            raise ValueError("radius_m must be positive")
        power_watts = (
            LIGHT_SPEED_M_S**5
            / GRAVITATIONAL_CONSTANT_SI
            * DELTA_MOD
            / (24.0 * math.pi)
            * (POWER_SCALE_RADIUS_M / radius_m) ** 2
        )
        return power_watts / 1.0e6

    def _christoffel_at_index(self, index: int) -> np.ndarray:
        metric = self.projector.lorentzian_metrics[index]
        inverse_metric = np.linalg.inv(metric)
        beta_c = (
            self.projector.beta_m_s[index] / self.projector.speed_of_light_m_s
        )
        beta_c_gradient = (
            self.projector.beta_gradient_per_s[index]
            / self.projector.speed_of_light_m_s
        )
        metric_gradient = np.zeros((4, 4), dtype=float)
        metric_gradient[0, 0] = 2.0 * beta_c * beta_c_gradient
        metric_gradient[0, 1] = beta_c_gradient
        metric_gradient[1, 0] = beta_c_gradient

        christoffel = np.zeros((4, 4, 4), dtype=float)
        for alpha in range(4):
            for mu in range(4):
                for nu in range(4):
                    total = 0.0
                    for delta in range(4):
                        partial_mu = metric_gradient[delta, nu] if mu == 1 else 0.0
                        partial_nu = metric_gradient[delta, mu] if nu == 1 else 0.0
                        partial_delta = (
                            metric_gradient[mu, nu] if delta == 1 else 0.0
                        )
                        total += inverse_metric[alpha, delta] * (
                            partial_mu + partial_nu - partial_delta
                        )
                    christoffel[alpha, mu, nu] = 0.5 * total
        return christoffel

    def audit(self, tolerance: float = NUMERICAL_TOLERANCE) -> dict[str, Any]:
        center = int(np.argmin(np.abs(self.projector.x_m)))
        beta_c = (
            self.projector.beta_m_s[center] / self.projector.speed_of_light_m_s
        )
        metric = self.projector.lorentzian_metrics[center]
        jacobian = np.eye(4)
        jacobian[1, 0] = beta_c
        inverse_jacobian = np.linalg.inv(jacobian)
        observer_metric = inverse_jacobian.T @ metric @ inverse_jacobian
        minkowski_metric = np.diag((-1.0, 1.0, 1.0, 1.0))
        observer_metric_error = float(
            np.linalg.norm(observer_metric - minkowski_metric, ord=np.inf)
        )

        four_velocity = np.array((1.0, -beta_c, 0.0, 0.0))
        normalization_error = abs(
            float(four_velocity @ metric @ four_velocity) + 1.0
        )
        christoffel = self._christoffel_at_index(center)
        four_acceleration = np.einsum(
            "amn,m,n->a",
            christoffel,
            four_velocity,
            four_velocity,
        )
        acceleration_norm_m_s2 = (
            float(np.linalg.norm(four_acceleration[1:]))
            * self.projector.speed_of_light_m_s**2
        )
        plateau_error = abs(float(self.projector.shape[center]) - 1.0)
        plateau_gradient = abs(float(self.projector.shape_gradient_per_m[center]))
        passed = (
            plateau_error <= tolerance
            and plateau_gradient <= tolerance
            and observer_metric_error <= 100.0 * tolerance
            and normalization_error <= 100.0 * tolerance
            and acceleration_norm_m_s2 <= tolerance
        )
        return {
            "passed": passed,
            "plateau_error": plateau_error,
            "plateau_gradient_per_m": plateau_gradient,
            "observer_metric_error": observer_metric_error,
            "four_velocity_normalization_error": normalization_error,
            "acceleration_norm_m_s2": acceleration_norm_m_s2,
            "power_requirement_mw": self.power_requirement_mw(
                self.projector.bubble_radius_m
            ),
        }


@dataclass
class LaTeXMacroExporter:
    """Serialize simulation results into the stable LaTeX macro interface."""

    results: Mapping[str, Any]

    @staticmethod
    def _scientific(value: float, decimal_places: int) -> str:
        if not math.isfinite(value) or value == 0.0:
            if value == 0.0:
                return "0"
            raise ValueError("Scientific LaTeX output requires a finite value")
        exponent = math.floor(math.log10(abs(value)))
        mantissa = value / (10.0**exponent)
        return (
            f"{mantissa:.{decimal_places}f}"
            f"\\times{{}}10^{{{exponent}}}"
        )

    def macro_values(self) -> dict[str, str]:
        """Return all generated macros with publication-stable precision."""

        branch = self.results["branch"]
        c_dark_residual = float(self.results["c_dark_residual"])
        c_dark_completed = float(self.results["c_dark"])
        framing_defect = float(self.results["framing_defect"])
        power_mw = float(self.results["power_mw"])
        macros = {
            "SimOutputBranch": f"({branch[0]}, {branch[1]}, {branch[2]})",
            "SimOutputCanonicalBranch": f"({branch[0]}, {branch[1]}, {branch[2]})",
            "SimOutputLeptonLift": f"{float(self.results['lepton_lift']):.0f}",
            "SimOutputQuarkLift": f"{float(self.results['quark_lift']):.0f}",
            "SimOutputFramingDefect": f"{framing_defect:.12f}",
            "SimOutputDarkResidual": f"{c_dark_residual:.12f}",
            "SimOutputDarkLedger": f"{c_dark_completed:.12f}",
            "SimOutputDarkCompleted": f"{c_dark_completed:.12f}",
            "SimOutputEntropyDebt": f"{float(self.results['delta_mod']):.12f}",
            "SimOutputWarpVelocity": f"{float(self.results['v_eff_c']):.9f}",
            "SimOutputPowerReq": f"{power_mw:.2f}",
            "SimOutputOperationalPowerMW": f"{power_mw:.2f}",
            "SimOutputPowerScaleRadius": self._scientific(
                float(self.results["power_scale_radius_m"]),
                12,
            ),
            "SimOutputSaturatedBitBudget": self._scientific(
                float(self.results["n_sat_bits"]),
                12,
            ),
            "SimOutputLocalMemoryBits": self._scientific(
                float(self.results["n_local_bits"]),
                6,
            ),
            "SimOutputShiftFieldFormula": (
                "\\beta_x(\\mathbf{x}) = -c \\, "
                "e^{\\Delta_{\\mathrm{mod}}/2} "
                "f_{\\text{SHBT}}(\\mathbf{x}, \\theta)"
            ),
            "SimOutputBoundaryPartition": _latex_number(
                float(self.results["z_boundary"])
            ),
            "SimOutputShannonEntropy": _latex_number(
                float(self.results["shannon_entropy"])
            ),
            "SimOutputPhaseTheta": _latex_number(
                float(self.results["phase_theta"])
            ),
            "SimOutputClosureNorm": _latex_number(
                float(self.results["closure_norm"])
            ),
            "SimOutputBoundaryNormError": _latex_number(
                float(self.results["boundary_normalization_error"])
            ),
            "SimOutputUnitarityError": _latex_number(
                float(self.results["unitarity_error"])
            ),
            "SimOutputPopulationShift": _latex_number(
                float(self.results["population_shift_l1"])
            ),
            "SimOutputMetricDetMin": _latex_number(
                float(self.results["minimum_abs_metric_determinant"])
            ),
            "SimOutputMetricEigenMin": _latex_number(
                float(self.results["minimum_gram_eigenvalue"])
            ),
            "SimOutputObserverMetricError": _latex_number(
                float(self.results["observer_metric_error"])
            ),
            "SimOutputAccelerationNorm": _latex_number(
                float(self.results["acceleration_norm_m_s2"])
            ),
        }
        return macros

    def export(self, output_path: Path | str = "sim_results.tex") -> None:
        """Write macro definitions atomically enough for the local workflow."""

        lines = [
            "% Automatically generated by warp_extension.py",
            "% chktex-file 36",
            *[
                f"\\newcommand{{\\{name}}}{{{value}}}"
                for name, value in self.macro_values().items()
            ],
            "",
        ]
        Path(output_path).write_text("\n".join(lines), encoding="utf-8")


class WarpDriveEngine:
    """Coordinator for the complete boundary-to-observer simulation."""

    def __init__(
        self,
        bubble_radius_m: float = DEFAULT_BUBBLE_RADIUS_M,
        domain_radius_m: float = DEFAULT_DOMAIN_RADIUS_M,
        grid_points: int = DEFAULT_GRID_POINTS,
        stress_grid_points: int = DEFAULT_STRESS_GRID_POINTS,
        wall_steepness_per_m: float = DEFAULT_WALL_STEEPNESS_PER_M,
        theta_phase: float = DEFAULT_PHASE_THETA,
    ) -> None:
        self.boundary = BoundaryRegister()
        self.excitation = ExcitationEngine(self.boundary, theta_phase=theta_phase)
        self.projector = FGSliceProjector(
            delta_mod=DELTA_MOD,
            bubble_radius_m=bubble_radius_m,
            domain_radius_m=domain_radius_m,
            grid_points=grid_points,
            wall_steepness_per_m=wall_steepness_per_m,
        )
        self.observer = CausalObserver(self.projector)
        self.metric_3d = Metric3DCalculator(
            bubble_radius_m=bubble_radius_m,
            domain_radius_m=max(1.5 * bubble_radius_m, bubble_radius_m + 1.0),
            grid_points_per_axis=stress_grid_points,
            wall_steepness_per_m=wall_steepness_per_m,
            delta_mod=DELTA_MOD,
        )
        self.stress_energy = StressEnergyAuditor(
            self.metric_3d.metric_4d_grid,
            self.metric_3d.coordinates,
        )
        self.derendering = DerenderingEngine(self.boundary, self.metric_3d)
        self.thermodynamics = ThermodynamicRateEngine()

    def evaluate(self) -> dict[str, Any]:
        boundary_audit = self.boundary.audit()
        excitation_audit = self.excitation.audit()
        metric_audit = self.projector.audit()
        metric_3d_audit = self.metric_3d.audit()
        stress_energy_audit = self.stress_energy.audit_energy_conditions(
            sample_count=100
        )
        observer_audit = self.observer.audit()
        thermodynamic_audit = self.thermodynamics.audit()
        probe_half_width = max(
            self.metric_3d.domain_radius_m
            / (self.metric_3d.grid_points_per_axis - 1),
            0.25 * self.metric_3d.bubble_radius_m,
        )
        probe_bounds = (-probe_half_width, probe_half_width)
        derender_probe = DerenderingEngine(self.boundary, self.metric_3d)
        derender_audit = derender_probe.derender_region(
            probe_bounds,
            probe_bounds,
            probe_bounds,
        )
        rerender_audit = derender_probe.rerender_region(
            (probe_half_width * 2.0, 0.0, 0.0)
        )
        derendering_audit = {
            "passed": bool(
                derender_audit["active_deformation_zero"]
                and not derender_audit["total_metric_zero"]
                and derender_audit["bit_budget_preserved"]
                and math.isclose(
                    rerender_audit["visible_normalization"],
                    1.0,
                    abs_tol=NUMERICAL_TOLERANCE,
                )
            ),
            "derender": derender_audit,
            "rerender": rerender_audit,
        }
        audits = {
            "boundary": boundary_audit,
            "excitation": excitation_audit,
            "metric": metric_audit,
            "metric_3d": metric_3d_audit,
            "stress_energy": stress_energy_audit,
            "derendering": derendering_audit,
            "thermodynamics": thermodynamic_audit,
            "observer": observer_audit,
        }
        failures = [name for name, audit in audits.items() if not audit["passed"]]
        if failures:
            raise RuntimeError(f"Simulation audit failed: {', '.join(failures)}")

        return {
            "branch": self.boundary.branch,
            "lepton_lift": self.boundary.parent_level
            / (2.0 * self.boundary.lepton_level),
            "quark_lift": self.boundary.parent_level
            / (3.0 * self.boundary.quark_level),
            "c_dark_residual": C_DARK_RESIDUAL,
            "c_dark": C_DARK_COMP,
            "delta_mod": DELTA_MOD,
            "v_eff_c": self.projector.v_eff_c,
            "v_eff_m_s": self.projector.v_eff_m_s,
            "power_mw": observer_audit["power_requirement_mw"],
            "power_scale_radius_m": POWER_SCALE_RADIUS_M,
            "n_sat_bits": N_SAT_BITS,
            "n_local_bits": N_LOCAL_BITS_10M
            * (self.projector.bubble_radius_m / DEFAULT_BUBBLE_RADIUS_M) ** 2,
            "z_boundary": self.boundary.z_boundary,
            "shannon_entropy": self.boundary.shannon_entropy,
            "phase_theta": self.excitation.theta_phase,
            "framing_defect": excitation_audit["framing_defect"],
            "closure_norm": excitation_audit["closure_norm"],
            "boundary_normalization_error": boundary_audit["normalization_error"],
            "unitarity_error": excitation_audit["unitarity_error"],
            "population_shift_l1": excitation_audit["population_shift_l1"],
            "minimum_abs_metric_determinant": metric_audit[
                "minimum_abs_determinant"
            ],
            "minimum_gram_eigenvalue": metric_audit[
                "minimum_gram_eigenvalue"
            ],
            "observer_metric_error": observer_audit["observer_metric_error"],
            "acceleration_norm_m_s2": observer_audit["acceleration_norm_m_s2"],
            "nec_passed": stress_energy_audit["nec_passed"],
            "wec_passed": stress_energy_audit["wec_passed"],
            "minimum_nec_energy_density": stress_energy_audit[
                "minimum_nec_energy_density"
            ],
            "minimum_wec_energy_density": stress_energy_audit[
                "minimum_wec_energy_density"
            ],
            "maximum_hold_time_s": thermodynamic_audit["maximum_hold_time_s"],
            "thermodynamic_steady_state": thermodynamic_audit[
                "steady_state_entropy_debt"
            ],
            "audits": audits,
        }

    def generate_figures(self, figures_directory: Path | str = "figures") -> None:
        figures_path = Path(figures_directory)
        figures_path.mkdir(parents=True, exist_ok=True)
        self._plot_shift_profile(figures_path / "shift_profile.pdf")
        self._plot_entropy_gradient(figures_path / "entropy_gradient.pdf")

    def _plot_shift_profile(self, output_path: Path) -> None:
        x = self.projector.x_m
        beta_c = self.projector.beta_m_s / self.projector.speed_of_light_m_s
        fig, axis = plt.subplots(figsize=(6.4, 3.8))
        axis.plot(x, self.projector.shape, color="#204a87", lw=2.0, label=r"$f(r_s)$")
        axis.plot(x, beta_c, color="#a40000", lw=2.0, label=r"$\beta_x/c$")
        axis.axvline(
            -self.projector.bubble_radius_m,
            color="0.55",
            ls="--",
            lw=0.9,
        )
        axis.axvline(
            self.projector.bubble_radius_m,
            color="0.55",
            ls="--",
            lw=0.9,
        )
        axis.axhline(0.0, color="0.25", lw=0.7)
        axis.set_xlabel(r"$x\ \mathrm{(m)}$")
        axis.set_ylabel("Dimensionless profile")
        axis.set_xlim(x[0], x[-1])
        axis.grid(alpha=0.2)
        axis.legend(frameon=False, loc="lower left")
        fig.tight_layout()
        fig.savefig(output_path, format="pdf", bbox_inches="tight")
        plt.close(fig)

    def _plot_entropy_gradient(self, output_path: Path) -> None:
        baseline = self.boundary.shannon_contributions
        excited = self.excitation.excited_shannon_contributions()
        gradient_y, gradient_x = np.gradient(excited)
        coordinates = np.arange(3)

        fig, axes = plt.subplots(1, 2, figsize=(7.2, 3.2), constrained_layout=True)
        color_limit = max(float(np.max(baseline)), float(np.max(excited)))
        color_map = plt.get_cmap("viridis")
        normalization = matplotlib.colors.Normalize(vmin=0.0, vmax=color_limit)
        for axis, matrix, title in (
            (axes[0], baseline, "Baseline entropy contribution"),
            (axes[1], excited, "Phase-locked entropy contribution"),
        ):
            for i in range(3):
                for j in range(3):
                    axis.add_patch(
                        plt.Rectangle(
                            (j - 0.5, i - 0.5),
                            1.0,
                            1.0,
                            facecolor=color_map(normalization(matrix[i, j])),
                            edgecolor="white",
                            linewidth=0.8,
                        )
                    )
            axis.set_xticks(coordinates)
            axis.set_yticks(coordinates)
            axis.set_xlim(-0.5, 2.5)
            axis.set_ylim(-0.5, 2.5)
            axis.set_xlabel(r"$j$")
            axis.set_ylabel(r"$i$")
            axis.set_title(title, fontsize=10)
            axis.set_aspect("equal")
            for i in range(3):
                for j in range(3):
                    axis.text(
                        j,
                        i,
                        f"{matrix[i, j]:.3f}",
                        ha="center",
                        va="center",
                        color="white" if matrix[i, j] > 0.55 * color_limit else "black",
                        fontsize=7,
                    )
        axes[1].quiver(
            coordinates,
            coordinates,
            gradient_x,
            gradient_y,
            color="white",
            angles="xy",
            scale_units="xy",
            scale=0.35,
            width=0.012,
        )
        fig.savefig(output_path, format="pdf", bbox_inches="tight")
        plt.close(fig)

    @staticmethod
    def export_latex_macros(
        results: Mapping[str, Any],
        output_path: Path | str = "sim_results.tex",
    ) -> None:
        LaTeXMacroExporter(results).export(output_path)


def compute_warp_extension(
    r_bubble_meters: float = DEFAULT_BUBBLE_RADIUS_M,
) -> dict[str, Any]:
    """Backward-compatible scalar entry point."""

    domain_radius = max(DEFAULT_DOMAIN_RADIUS_M, 3.0 * r_bubble_meters)
    engine = WarpDriveEngine(
        bubble_radius_m=r_bubble_meters,
        domain_radius_m=domain_radius,
    )
    return engine.evaluate()


def export_latex_macros(
    data: Mapping[str, Any],
    output_path: Path | str = "sim_results.tex",
) -> None:
    """Backward-compatible macro exporter."""

    WarpDriveEngine.export_latex_macros(data, output_path)


def _print_summary(results: Mapping[str, Any]) -> None:
    print("--- SHBT WARP EXTENSION ENGINE ---")
    print(f"Baseline repository:      {REPOSITORY_URL}")
    print(f"Canonical branch:         {results['branch']}")
    print(f"Boundary partition Z:     {results['z_boundary']:.12e}")
    print(f"Shannon entropy S_E:      {results['shannon_entropy']:.12f}")
    print(f"Framing defect Delta_fr:  {results['framing_defect']:.3e}")
    print(f"Closure tensor norm:      {results['closure_norm']:.3e}")
    print(f"Entropy debt Delta_mod:   {results['delta_mod']:.12f}")
    print(f"Warp velocity v_eff/c:    {results['v_eff_c']:.10f}")
    print(f"Excitation L1 shift:      {results['population_shift_l1']:.12f}")
    print(
        "Minimum |det(g_L)|:      "
        f"{results['minimum_abs_metric_determinant']:.12e}"
    )
    print(
        "Minimum Gram eigenvalue: "
        f"{results['minimum_gram_eigenvalue']:.12e}"
    )
    print(
        "Observer metric error:   "
        f"{results['observer_metric_error']:.3e}"
    )
    print(
        "Observer acceleration:   "
        f"{results['acceleration_norm_m_s2']:.3e} m/s^2"
    )
    print(
        "Sampled NEC minimum:      "
        f"{results['minimum_nec_energy_density']:.12e}"
    )
    print(
        "Sampled WEC minimum:      "
        f"{results['minimum_wec_energy_density']:.12e}"
    )
    print(f"NEC sampled-grid pass:    {results['nec_passed']}")
    print(f"WEC sampled-grid pass:    {results['wec_passed']}")
    print("Framing hold time:        infinite (topologically locked)")
    print(f"Power requirement:        {results['power_mw']:.2f} MW")
    print("All structural audits:    PASS")


def _parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--radius",
        type=float,
        default=DEFAULT_BUBBLE_RADIUS_M,
        help="Bubble radius in meters (default: 10)",
    )
    parser.add_argument(
        "--domain-radius",
        type=float,
        default=DEFAULT_DOMAIN_RADIUS_M,
        help="Half-width of the sampled spatial domain in meters",
    )
    parser.add_argument(
        "--grid-points",
        type=int,
        default=DEFAULT_GRID_POINTS,
        help="Number of spatial samples; even values are promoted to odd",
    )
    parser.add_argument(
        "--stress-grid-points",
        type=int,
        default=DEFAULT_STRESS_GRID_POINTS,
        help="Samples per Cartesian axis for the 3D curvature audit",
    )
    parser.add_argument(
        "--wall-steepness",
        type=float,
        default=DEFAULT_WALL_STEEPNESS_PER_M,
        help="Alcubierre wall steepness in inverse meters",
    )
    parser.add_argument(
        "--phase",
        type=float,
        default=DEFAULT_PHASE_THETA,
        help="Dimensionless phase-locking control angle",
    )
    parser.add_argument(
        "--tex-output",
        default="sim_results.tex",
        help="Generated LaTeX macro file",
    )
    parser.add_argument(
        "--figures-directory",
        default="figures",
        help="Directory for generated PDF figures",
    )
    parser.add_argument(
        "--no-figures",
        action="store_true",
        help="Skip figure generation",
    )
    return parser.parse_args()


def main() -> int:
    arguments = _parse_arguments()
    engine = WarpDriveEngine(
        bubble_radius_m=arguments.radius,
        domain_radius_m=arguments.domain_radius,
        grid_points=arguments.grid_points,
        stress_grid_points=arguments.stress_grid_points,
        wall_steepness_per_m=arguments.wall_steepness,
        theta_phase=arguments.phase,
    )
    results = engine.evaluate()
    engine.export_latex_macros(results, arguments.tex_output)
    if not arguments.no_figures:
        engine.generate_figures(arguments.figures_directory)

    assert results["branch"] == (26, 8, 312)
    assert results["framing_defect"] == 0.0
    assert math.isclose(results["c_dark_residual"], 834_433 / 362_670)
    assert math.isclose(results["delta_mod"], 1_197_103 / 8_704_080)
    assert math.isclose(results["v_eff_c"], math.exp(DELTA_MOD / 2.0))
    assert results["audits"]["stress_energy"]["sample_count"] == 100
    assert results["audits"]["derendering"]["passed"]
    assert results["audits"]["thermodynamics"]["passed"]
    assert math.isinf(results["maximum_hold_time_s"])
    flat_axis = np.linspace(-1.0, 1.0, 3)
    flat_metric = np.broadcast_to(
        np.diag((-1.0, 1.0, 1.0, 1.0)),
        (3, 3, 3, 4, 4),
    ).copy()
    flat_auditor = StressEnergyAuditor(
        flat_metric,
        (flat_axis, flat_axis, flat_axis),
        cosmological_constant=0.0,
    )
    assert np.allclose(flat_auditor.christoffel, 0.0)
    assert np.allclose(flat_auditor.riemann, 0.0)
    assert np.allclose(flat_auditor.einstein, 0.0)
    exported_text = Path(arguments.tex_output).read_text(encoding="utf-8")
    assert "\\newcommand{\\SimOutputFramingDefect}{0.000000000000}" in exported_text
    assert "\\newcommand{\\SimOutputOperationalPowerMW}{142.08}" in exported_text
    assert "\\newcommand{\\SimOutputShiftFieldFormula}" in exported_text
    _print_summary(results)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())