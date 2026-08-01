"""Command-line entry point for ``shbt-warp-sim``."""

import argparse
import sys

from shbt_warp._core import Simulation
from shbt_warp.latex import LaTeXMacroExporter
from shbt_warp.plots import PlotGenerator


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

    args = parser.parse_args(argv)

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
