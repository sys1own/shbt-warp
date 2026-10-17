"""SHBT Holographic Warp Drive Simulator — Python package."""

from shbt_warp._core import Simulation
from shbt_warp.cad_engine import SHBTCADEngine, SimulationReport
from shbt_warp.cli import main, simulate_wormhole_refuel_hop, synthesize_mar312_core
from shbt_warp.custom_sim import BoundaryRegister, BulkSliceMetric, FGSliceProjector
from shbt_warp.latex import LaTeXMacroExporter
from shbt_warp.plots import PlotGenerator

__version__ = "0.1.0"
__all__ = [
    "BoundaryRegister",
    "BulkSliceMetric",
    "FGSliceProjector",
    "Simulation",
    "SHBTCADEngine",
    "SimulationReport",
    "LaTeXMacroExporter",
    "PlotGenerator",
    "main",
    "run_simulation",
    "synthesize_mar312_core",
    "simulate_wormhole_refuel_hop",
]


def run_simulation(**kwargs):
    """Run the full simulation and return the results dictionary."""
    sim = Simulation(**kwargs)
    return sim.run()
