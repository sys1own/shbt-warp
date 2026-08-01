"""Export SHBT warp simulation results as LaTeX macros."""

import math
from pathlib import Path


def _scientific(value, decimal_places=12):
    if not math.isfinite(value) or value == 0.0:
        return "0"
    exponent = math.floor(math.log10(abs(value)))
    mantissa = value / (10.0 ** exponent)
    return f"{mantissa:.{decimal_places}f}\\times{{}}10^{{{exponent}}}"


def _latex_number(value, digits=12):
    if not math.isfinite(value):
        return "\\infty"
    if value == 0.0:
        return "0"
    magnitude = abs(value)
    if 1.0e-4 <= magnitude < 1.0e5:
        return f"{value:.{digits}g}"
    exponent = math.floor(math.log10(magnitude))
    mantissa = value / (10.0 ** exponent)
    return f"{mantissa:.6f}\\times{{}}10^{{{exponent}}}"


class LaTeXMacroExporter:
    """Write a .tex file containing simulation output macros."""

    def __init__(self, results):
        self.results = results

    def macros(self):
        """Return an ordered dictionary of macro-name -> macro-value strings."""
        r = self.results
        boundary = r.get("boundary", {})
        excitation = r.get("excitation", {})
        fg = r.get("fg_slice", {})
        metric3d = r.get("metric3d", {})
        stress = r.get("stress_energy", {})
        causal = r.get("causal", {})
        thermo = r.get("thermodynamics", {})

        branch = (boundary.get("lepton_level", 26), boundary.get("quark_level", 8), boundary.get("parent_level", 312))
        power_mw = causal.get("power_requirement_mw", r.get("power_mw", 0.0))

        return {
            "SimOutputBranch": f"({branch[0]}, {branch[1]}, {branch[2]})",
            "SimOutputCanonicalBranch": f"({branch[0]}, {branch[1]}, {branch[2]})",
            "SimOutputLeptonLift": f"{r.get('lepton_lift', 6.0):.0f}",
            "SimOutputQuarkLift": f"{r.get('quark_lift', 13.0):.0f}",
            "SimOutputFramingDefect": f"{boundary.get('framing_defect', 0.0):.12f}",
            "SimOutputDarkResidual": f"{r.get('c_dark_residual', 0.0):.12f}",
            "SimOutputDarkLedger": f"{r.get('c_dark', 0.0):.12f}",
            "SimOutputDarkCompleted": f"{r.get('c_dark', 0.0):.12f}",
            "SimOutputEntropyDebt": f"{r.get('delta_mod', 0.0):.12f}",
            "SimOutputWarpVelocity": f"{fg.get('v_eff_c', r.get('v_eff_c', 0.0)):.9f}",
            "SimOutputPowerReq": f"{power_mw:.2f}",
            "SimOutputOperationalPowerMW": f"{power_mw:.2f}",
            "SimOutputPowerScaleRadius": _scientific(r.get("power_scale_radius_m", 0.0), 12),
            "SimOutputSaturatedBitBudget": _scientific(r.get("n_sat_bits", 0.0), 12),
            "SimOutputLocalMemoryBits": _scientific(r.get("n_local_bits", 0.0), 6),
            "SimOutputShiftFieldFormula": (
                "\\beta_x(\\mathbf{x}) = -c \\,"
                "e^{\\Delta_{\\mathrm{mod}}/2} "
                "f_{\\text{SHBT}}(\\mathbf{x}, \\theta)"
            ),
            "SimOutputBoundaryPartition": _latex_number(r.get("z_boundary", 0.0)),
            "SimOutputShannonEntropy": _latex_number(r.get("shannon_entropy", 0.0)),
            "SimOutputPhaseTheta": _latex_number(r.get("phase", 0.421)),
            "SimOutputClosureNorm": _latex_number(r.get("closure_norm", 0.0)),
            "SimOutputBoundaryNormError": _latex_number(boundary.get("audit", {}).get("normalization_error", 0.0)),
            "SimOutputUnitarityError": _latex_number(excitation.get("audit", {}).get("unitarity_error", 0.0)),
            "SimOutputPopulationShift": _latex_number(r.get("population_shift_l1", 0.0)),
            "SimOutputMetricDetMin": _latex_number(r.get("minimum_abs_metric_determinant", 0.0)),
            "SimOutputMetricEigenMin": _latex_number(r.get("minimum_gram_eigenvalue", 0.0)),
            "SimOutputObserverMetricError": _latex_number(r.get("observer_metric_error", 0.0)),
            "SimOutputAccelerationNorm": _latex_number(r.get("acceleration_norm_m_s2", 0.0)),
            "SimOutputNEC": _latex_number(stress.get("audit", {}).get("minimum_nec_energy_density", 0.0)),
            "SimOutputWEC": _latex_number(stress.get("audit", {}).get("minimum_wec_energy_density", 0.0)),
            "SimOutputSteadyStateEntropyDebt": _latex_number(thermo.get("steady_state_entropy_debt", 0.0)),
            "SimOutputMaximumHoldTimeS": _latex_number(thermo.get("maximum_hold_time_s", math.inf)),
        }

    def write(self, path):
        """Write the macros to ``path``."""
        path = Path(path)
        path.parent.mkdir(parents=True, exist_ok=True)
        lines = [
            "% SHBT Holographic Warp Drive Simulator output macros",
            "% Generated automatically by shbt-warp-sim",
        ]
        for name, value in self.macros().items():
            lines.append(f"\\newcommand{{\\{name}}}{{{value}}}")
        lines.append("")
        path.write_text("\n".join(lines), encoding="utf-8")
        return path
