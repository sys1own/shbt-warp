"""Command-line entry point for ``shbt-warp-sim``."""

import argparse
import csv
import json
import math
import sys
from pathlib import Path

import numpy as np

from shbt_warp._core import Simulation
from shbt_warp.cad_engine import SHBTCADEngine
from shbt_warp.latex import LaTeXMacroExporter
from shbt_warp.plots import PlotGenerator

_POWER_BENCHMARK_MW = 142.08
_DEFAULT_DELTA_MOD = 0.13753354748577679


def _parse_sweep(spec):
    """Parse a ``min:max:step`` sweep specification into a NumPy array."""
    if spec is None:
        return None
    parts = spec.split(":")
    if len(parts) != 3:
        raise argparse.ArgumentTypeError("sweep must be min:max:step")
    start, stop, step = map(float, parts)
    if step == 0:
        raise argparse.ArgumentTypeError("sweep step must be nonzero")
    n = max(0, int(round(abs((stop - start) / step))))
    directed_step = math.copysign(step, stop - start)
    return np.linspace(start, start + n * directed_step, n + 1)


def _phase_for_displacement(target: float, tolerance: float = 1.0e-14) -> float:
    """Return the phase-lock angle that yields a target L1 population shift."""
    if target <= 0.0:
        return 0.0

    sim = Simulation(grid_points=5)
    lo, hi = 0.0, math.pi / 2.0

    while sim.population_shift_at(hi) < target and hi < 2.0 * math.pi:
        lo = hi
        hi = min(hi + math.pi / 2.0, 2.0 * math.pi)

    for _ in range(50):
        mid = (lo + hi) / 2.0
        val = sim.population_shift_at(mid)
        if abs(val - target) <= tolerance:
            return mid
        if val < target:
            lo = mid
        else:
            hi = mid
    return (lo + hi) / 2.0


def _jsonify(obj):
    """Recursively convert NumPy arrays and scalars to plain JSON types."""
    if isinstance(obj, dict):
        return {k: _jsonify(v) for k, v in obj.items()}
    if isinstance(obj, (list, tuple)):
        return [_jsonify(v) for v in obj]
    if isinstance(obj, np.ndarray):
        return obj.tolist()
    if isinstance(obj, np.floating):
        return float(obj)
    if isinstance(obj, np.integer):
        return int(obj)
    if isinstance(obj, complex):
        return {"real": obj.real, "imag": obj.imag}
    return obj


def _write_json(results, path):
    """Write simulation results to ``path`` as JSON."""
    path = Path(path)
    path.parent.mkdir(parents=True, exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        json.dump(_jsonify(results), f, indent=2, sort_keys=True)


def _sweep(args):
    radii = _parse_sweep(args.sweep_radius)
    phases = _parse_sweep(args.sweep_phase)
    if radii is None and phases is None:
        return None
    if radii is None:
        radii = np.array([args.radius], dtype=float)
    if phases is None:
        phases = np.array([args.phase], dtype=float)

    rows = []
    print("Running SHBT parameter sweep...", flush=True)
    for radius in radii:
        for phase in phases:
            sim = Simulation(
                radius=float(radius),
                domain_radius=args.domain_radius,
                grid_points=args.grid_points,
                wall_steepness=args.wall_steepness,
                phase=float(phase),
                stress_grid_points=7,
            )
            results = sim.run(args.tolerance)
            power_mw = results.get("power_mw", 0.0)
            delta_mod = results.get("delta_mod", 0.0)
            entropy_debt = delta_mod * power_mw / _POWER_BENCHMARK_MW
            rows.append(
                {
                    "radius_m": radius,
                    "phase": phase,
                    "v_eff_c": results.get("v_eff_c", 0.0),
                    "power_mw": power_mw,
                    "entropy_debt": entropy_debt,
                    "population_shift_l1": results.get("population_shift_l1", 0.0),
                }
            )

    # Print a Markdown table.
    print(
        "\n| radius (m) | phase | v_eff/c | P_op (MW) | entropy debt | population shift |"
    )
    print(
        "|------------|-------|----------:|----------:|-------------:|-----------------:|"
    )
    for r in rows:
        print(
            f"| {r['radius_m']:10.3f} | {r['phase']:.3f} | {r['v_eff_c']:9.6f} | "
            f"{r['power_mw']:11.6f} | {r['entropy_debt']:12.6e} | {r['population_shift_l1']:16.12f} |"
        )

    if args.sweep_output:
        with open(args.sweep_output, "w", newline="") as f:
            writer = csv.DictWriter(f, fieldnames=rows[0].keys())
            writer.writeheader()
            writer.writerows(rows)
        print(f"\nWrote sweep results: {args.sweep_output}")

    return rows


def _is_custom_run(args) -> bool:
    """Return ``True`` when the user requested non-canonical velocity/displacement."""
    return args.velocity is not None or args.displacement is not None


def main(argv=None):
    parser = argparse.ArgumentParser(
        description="SHBT Holographic Warp Drive Simulator"
    )
    parser.add_argument(
        "--radius",
        type=float,
        default=10.0,
        help="Boundary radius R in meters (default: 10.0)",
    )
    parser.add_argument(
        "--velocity",
        type=float,
        default=None,
        help="Target shift velocity in units of c (default: 1.071186)",
    )
    parser.add_argument(
        "--displacement",
        type=float,
        default=None,
        help="Unitary character population displacement ||delta rho||_1 (default: 0.2636895)",
    )
    parser.add_argument(
        "--output-json",
        type=str,
        default=None,
        help="File path to export numerical results as JSON",
    )
    parser.add_argument(
        "--plot",
        action="store_true",
        default=False,
        help="Generate metric shift and energy condition diagnostic plots",
    )
    parser.add_argument(
        "--audit",
        action="store_true",
        default=False,
        help="Run the paper-audit pipeline: generate sim_results.tex and check benchmarks",
    )
    parser.add_argument(
        "--domain-radius",
        type=float,
        default=30.0,
        help="Domain radius in meters (default: 30.0)",
    )
    parser.add_argument(
        "--grid-points",
        type=int,
        default=1201,
        help="Number of grid points for the 1-D FG slice (default: 1201)",
    )
    parser.add_argument(
        "--wall-steepness",
        type=float,
        default=0.8,
        help="Wall steepness in 1/m (default: 0.8)",
    )
    parser.add_argument(
        "--phase",
        type=float,
        default=0.421,
        help="Phase-lock angle theta (default: 0.421)",
    )
    parser.add_argument(
        "--tex-output",
        type=str,
        default="sim_results.tex",
        help="Output LaTeX macro file (default: sim_results.tex)",
    )
    parser.add_argument(
        "--figures-directory",
        type=str,
        default="figures",
        help="Directory for vector PDF figures (default: figures)",
    )
    parser.add_argument(
        "--tolerance",
        type=float,
        default=1.0e-12,
        help="Audit numerical tolerance (default: 1e-12)",
    )
    parser.add_argument(
        "--sweep-radius",
        type=str,
        default=None,
        help="Radius sweep min:max:step (e.g. 5:15:2.5)",
    )
    parser.add_argument(
        "--sweep-phase",
        type=str,
        default=None,
        help="Phase sweep min:max:step (e.g. 0.1:0.8:0.1)",
    )
    parser.add_argument(
        "--sweep-output",
        type=str,
        default="sweep_results.csv",
        help="CSV output for sweep results (default: sweep_results.csv)",
    )

    args = parser.parse_args(argv)

    if args.sweep_radius or args.sweep_phase:
        _sweep(args)
        return 0

    if args.velocity is not None and args.velocity <= 0.0:
        parser.error("--velocity must be positive")

    phase = args.phase
    if args.displacement is not None:
        phase = _phase_for_displacement(args.displacement)

    sim_kwargs = {
        "radius": args.radius,
        "domain_radius": args.domain_radius,
        "grid_points": args.grid_points,
        "wall_steepness": args.wall_steepness,
        "phase": phase,
        "stress_grid_points": 7,
    }
    if args.velocity is not None:
        sim_kwargs["delta_mod"] = 2.0 * math.log(args.velocity)

    sim = Simulation(**sim_kwargs)
    print("Running SHBT warp simulation...", flush=True)
    results = sim.run(args.tolerance)

    if args.output_json:
        _write_json(results, args.output_json)
        print(f"Wrote JSON results: {args.output_json}")

    if args.plot:
        plots = PlotGenerator(results)
        plots.render_all(args.figures_directory)
        print(f"Wrote figures to: {args.figures_directory}/")

    do_audit = args.audit or (args.output_json is None and not args.plot)
    if do_audit:
        latex = LaTeXMacroExporter(results)
        latex_path = latex.write(args.tex_output)
        print(f"\nWrote LaTeX macros: {latex_path}")

        plots = PlotGenerator(results)
        plots.render_all(args.figures_directory)
        print(f"Wrote figures to: {args.figures_directory}/")

        def check(section):
            return section.get("audit", {}).get("passed", 0.0) == 1.0

        print("\nAudit summary:")
        for label, section in [
            ("Boundary", results.get("boundary", {})),
            ("Excitation", results.get("excitation", {})),
            ("FG slice", results.get("fg_slice", {})),
            ("3-D metric", results.get("metric3d", {})),
            ("Stress-energy", results.get("stress_energy", {})),
            ("Causal observer", results.get("causal", {})),
            ("Thermodynamics", results.get("thermodynamics", {})),
        ]:
            status = "PASS" if check(section) else "FAIL"
            print(f"  {label:20s} {status}")

        print(f"\nEffective warp velocity (c): {results.get('v_eff_c', 0.0):.12e}")
        print(f"Operational power (MW):        {results.get('power_mw', 0.0):.12e}")

        if not all(
            check(s)
            for s in [
                results.get("boundary", {}),
                results.get("excitation", {}),
                results.get("fg_slice", {}),
                results.get("metric3d", {}),
                results.get("stress_energy", {}),
                results.get("causal", {}),
                results.get("thermodynamics", {}),
            ]
        ):
            if _is_custom_run(args):
                print(
                    "\nWarning: one or more audits did not pass for the requested custom parameters.",
                    file=sys.stderr,
                )
            else:
                print("\nOne or more audits failed.", file=sys.stderr)
                return 1

    return 0


def cad_main(argv=None):
    """Entry point for ``shbt-cad-sim``."""
    parser = argparse.ArgumentParser(
        description="SHBT CAD flight-phase simulator"
    )
    parser.add_argument(
        "--radius",
        type=float,
        default=10.0,
        help="Bubble radius in meters (default: 10.0)",
    )
    parser.add_argument(
        "--t-ramp",
        type=float,
        default=1.0,
        help="Phase A ramp duration in seconds (default: 1.0)",
    )
    parser.add_argument(
        "--t-steering",
        type=float,
        default=1.0,
        help="Phase B steering duration in seconds (default: 1.0)",
    )
    parser.add_argument(
        "--grid-points",
        type=int,
        default=1201,
        help="Grid resolution reference (default: 1201)",
    )
    parser.add_argument(
        "--phase",
        type=float,
        default=0.421,
        help="Phase-lock angle theta (default: 0.421)",
    )
    parser.add_argument(
        "--noise-temp",
        type=float,
        default=15.4e-3,
        help="Array noise temperature in Kelvin (default: 15.4 mK)",
    )
    parser.add_argument(
        "--noise-gamma",
        type=float,
        default=1.2e-4,
        help="Decoherence rate in s^-1 (default: 1.2e-4)",
    )
    parser.add_argument(
        "--phase-jitter",
        type=float,
        default=5.05e-5,
        help="Phase jitter in radians (default: 5.05e-5)",
    )
    parser.add_argument(
        "--tex-output",
        type=str,
        default="sim_results.tex",
        help="Output LaTeX macro file (default: sim_results.tex)",
    )
    parser.add_argument(
        "--figures-directory",
        type=str,
        default="figures",
        help="Directory for vector PDF figures (default: figures)",
    )
    parser.add_argument(
        "--budget-limit",
        type=float,
        default=1.0,
        help="HIL information-density budget limit (default: 1.0)",
    )

    args = parser.parse_args(argv)

    engine = SHBTCADEngine(
        radius=args.radius,
        grid_points=args.grid_points,
        phase=args.phase,
        t_ramp=args.t_ramp,
        t_steering=args.t_steering,
        noise_temp_k=args.noise_temp,
        noise_gamma=args.noise_gamma,
        phase_jitter=args.phase_jitter,
        budget_limit=args.budget_limit,
    )

    print("Running SHBT CAD flight simulation...", flush=True)
    report = engine.run_flight_simulation()

    print("\nCAD audit summary:")
    print(f"  Canonical branch:         {report['invariants']['canonical_branch']}")
    print(
        f"  Framing defect:           {report['invariants']['framing_defect']:.12f}"
    )
    print(f"  Unitarity residual:       {report['invariants']['trace_residual']:.6e}")
    print(f"  Phase jitter OK:          {report['noise']['phase_jitter_ok']}")
    print(f"  Thermal decoherence OK:   {report['noise']['thermal_decoherence_ok']}")
    print(f"  Integer lock OK:          {report['noise']['integer_lock_ok']}")
    print(
        f"  Stinespring ratio:        {report['invariants']['stinespring_ratio']:.12f}"
    )
    print(
        f"  Collapse metric det(g):   {report['metric_grid']['flat_metric_determinant']:.12f}"
    )

    engine.export_latex_macros(report, args.tex_output)
    print(f"\nWrote LaTeX macros: {args.tex_output}")

    figure_paths = engine.render_figures(report, args.figures_directory)
    print(f"Wrote CAD figures to: {args.figures_directory}/")
    for p in figure_paths:
        print(f"  - {p}")

    return 0


if __name__ == "__main__":
    if "cad" in sys.argv:
        sys.argv.remove("cad")
        sys.exit(cad_main())
    if "--cad" in sys.argv:
        sys.argv.remove("--cad")
        sys.exit(cad_main())
    sys.exit(main())
