"""SHBT Holographic Warp Drive Simulator — Python package."""

from shbt_warp._core import Simulation
from shbt_warp.cli import main
from shbt_warp.latex import LaTeXMacroExporter
from shbt_warp.plots import PlotGenerator

__version__ = "0.1.0"
__all__ = ["Simulation", "LaTeXMacroExporter", "PlotGenerator", "main", "run_simulation"]


def run_simulation(**kwargs):
    """Run the full simulation and return the results dictionary."""
    sim = Simulation(**kwargs)
    return sim.run()
