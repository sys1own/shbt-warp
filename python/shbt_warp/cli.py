"""Command-line entry point for ``shbt-warp-sim``."""

import argparse
import csv
import math
import sys

import numpy as np

_POWER_BENCHMARK_MW = 142.08

from shbt_warp._core import Simulation
from shbt_warp.latex import LaTeXMacroExporter
from shbt_warp.plots import PlotGenerator


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
            rows.append({
                "radius_m": radius,
                "phase": phase,
                "v_eff_c": results.get("v_eff_c", 0.0),
                "power_mw": power_mw,
                "entropy_debt": entropy_debt,
                "population_shift_l1": results.get("population_shift_l1", 0.0),
            })

    # Print a Markdown table.
    print("\n| radius (m) | phase | v_eff/c | P_op (MW) | entropy debt | population shift |")
    print("|------------|-------|----------:|----------:|-------------:|-----------------:|")
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


def main(argv=None):
    parser = argparse.ArgumentParser(
        description="SHBT Holographic Warp Drive Simulator"
    )
    parser.add_argument(
        "--radius",
        type=float,
        default=10.0,
        help="Bubble radius in meters (default: 10.0)",
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

    sim = Simulation(
        radius=args.radius,
        domain_radius=args.domain_radius,
        grid_points=args.grid_points,
        wall_steepness=args.wall_steepness,
        phase=args.phase,
        stress_grid_points=7,
    )
    print("Running SHBT warp simulation...", flush=True)
    results = sim.run(args.tolerance)

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

    latex = LaTeXMacroExporter(results)
    latex_path = latex.write(args.tex_output)
    print(f"\nWrote LaTeX macros: {latex_path}")

    plots = PlotGenerator(results)
    plots.render_all(args.figures_directory)
    print(f"Wrote figures to: {args.figures_directory}/")

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
        print("\nOne or more audits failed.", file=sys.stderr)
        return 1

    return 0


if __name__ == "__main__":
    sys.exit(main())
