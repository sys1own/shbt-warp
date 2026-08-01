# SHBT-CAD Holographic Warp Drive Engineering Simulator

`shbt-warp` (SHBT-CAD) is a production-grade Rust + PyO3 engineering design
engine for the Static Holographic Boundary Theory (SHBT) holographic warp
drive.  It extends the analytical foundations of `shbt-precision` into an
executable, high-fidelity simulator that evaluates dynamic 3+1D ADM
spacetimes, hardware-in-the-loop (HIL) safety monitors, GPU-accelerated
Fefferman-Graham metric expansions, and boundary emitter-array phase
synthesis.

The codebase is no longer a static proof-of-concept: it is intended as a
living flight-dynamics and control-prototyping instrument.  Rust provides the
numerical core, PyO3 exposes it as the `shbt_warp` Python package, and a
`Makefile` automates the full build/test/document pipeline.

## Engineering capabilities

- **3+1D ADM metric evaluation** with lapse `alpha = 1.0`, spatial metric
  `gamma_ij = delta_ij`, shift vector `beta^i(t, x^k)`, and a per-cell
  Lorentzian determinant check `det g = -1.0`.
- **Phase-locked boundary character excitation** for the visible
  `SU(2)_{26} x SU(3)_8` register, with unitary density-matrix audits and
  exact zero scalar framing defect.
- **Three-phase flight dynamics** (Phase A ramp, Phase B vector steering over
  the prime skeleton, Phase C safe collapse and Stinespring de-rendering).
- **Real-time HIL safety loops** that audit Gram positivity, metric
  determinant errors, and local bit-budget overflows on every step.
- **Boundary emitter-array hardware synthesis** that maps local phase angles
  and conformal dimensions to RF drive signals and enforces phase-jitter,
  cryogenic-noise, and decoherence limits.
- **GPU compute shader** (`src/shbt/shaders/fefferman_graham.wgsl`) for
  parallelized 3+1D metric tensor expansion over Cartesian grids.
- **Automated LaTeX + figure pipeline** that writes `sim_results.tex`,
  `cad_sim_results.tex`, vector PDF figures, and compiles `main.pdf`.

## Repository module architecture

| Path | Purpose |
|------|---------|
| `src/boundary.rs` | Visible register construction, modular `S`-matrix blocks, unitary excitation operator, framing defect and central-charge audits. |
| `src/projector.rs` | 1D FG-slice and 3D Cartesian metric calculators; evaluates the shift vector `beta_x(x) = -c exp(Delta_mod/2) f_SHBT(x, theta)` and metric spectra. |
| `src/stress_energy.rs` | Finite-difference connection, Riemann/Ricci/Einstein curvature, effective `T_{mu nu}`, and 100-vector NEC/WEC sampling. |
| `src/derender.rs` | `DerenderingEngine`: visible-to-dark weight transfer, background-metric restoration, and time-stepped re-rendering trajectories. |
| `src/thermodynamics.rs` | Entropy-debt integration, maximum framing hold time, and RK4 transient engine. |
| `src/causal_observer.rs` | Comoving Minkowski plateau observer and `P_op` power benchmark. |
| `src/shbt/warp_metric.rs` | `FGSliceProjector`: evaluates 3+1D ADM lapse (`alpha = 1.0`), spatial metric (`gamma_ij = delta_ij`), shift vector `beta^i`, and verifies `det g = -1.0` per cell. |
| `src/shbt/character_excitation.rs` | `CharacterExcitationRegister`: 9x9 boundary density matrix `rho_partial` with unitary checks and framing-defect closure. |
| `src/shbt/emitter_array.rs` | `EmitterArrayController` (phase/RF signal synthesis) and `HardwareNoiseAuditor` (phase jitter, thermal noise, decoherence, integer level lock). |
| `src/shbt/flight_phases.rs` | `FlightDynamicsEngine`: Phase A ramp, Phase B vector steering, Phase C safe collapse and Stinespring de-rendering. |
| `src/shbt/safety_monitor.rs` | `SafetyMonitor`: HIL audits for Gram eigenvalue, metric determinant, and information-density limits. |
| `src/shbt/shaders/fefferman_graham.wgsl` | WGSL compute shader (`@workgroup_size(8, 8, 8)`) that writes the 16-float 4x4 ADM metric per grid cell from a shape-field buffer. |
| `src/shbt/mod.rs` | Aggregates the `src/shbt/` submodules and exposes them through `src/lib.rs`. |
| `src/lib.rs` | PyO3 module initialization; registers `Simulation` and every `src/shbt/` class under `shbt_warp._core`. |
| `python/shbt_warp/` | Python package: `cad_engine.py`, `cli.py`, `latex.py`, `plots.py`, and `examples/warp_exploration.ipynb`. |
| `main.tex` and `sections/` | APS RevTeX 4-2 manuscript source, including the new ADM flight-phase, emitter-array, and HIL safety sections. |
| `Makefile` | One-command build pipeline: compile the Rust extension, run the pytest suite, execute the simulators, generate figures and macros, and compile `main.pdf`. |

## Prerequisites

- Rust toolchain (recent stable `rustc` / `cargo`, 1.70+).
- Python 3.10 or newer.
- A working TeX Live installation with `revtex4-2` (only required to compile `main.pdf`).
- A GPU with WGPU/Vulkan support is optional; the WGSL shader is loaded as an embedded asset and can be dispatched by any WGPU-based compute harness.

The `Makefile` creates a local `.venv` automatically if it does not exist.

## Installation and build quick start

```bash
# Clone and enter the repository
git clone https://github.com/sys1own/shbt-warp.git
cd shbt-warp

# Editable Python install (builds the Rust extension automatically)
pip install -e .

# Or use Maturin directly
python3 -m venv .venv
source .venv/bin/activate
pip install -r requirements.txt
maturin develop --release
```

For the full automated pipeline (recommended):

```bash
make
```

`make` performs the following steps in order:

1. Creates `.venv` and installs `requirements.txt` if needed.
2. Runs `maturin develop --release` to build and install the `_core` PyO3 extension.
3. Runs `pytest` to verify the Rust/Python integration.
4. Executes `python -m shbt_warp.cli` to generate `sim_results.tex` and the standard vector PDF figures.
5. Executes `shbt-cad-sim` to generate `cad_sim_results.tex` and CAD flight-phase figures.
6. Runs `pdflatex` on `main.tex` twice, producing `main.pdf`.

To run only the simulator and build the paper:

```bash
make clean && make
```

To compile only the manuscript after the simulation outputs already exist:

```bash
make paper
```

## Usage

### Option A: Verification & Paper Build

To run the automated audit, generate `sim_results.tex`, run tests, and build
`main.pdf`:

```bash
make
```

The pipeline also runs the SHBT-CAD three-phase flight simulator
(`shbt-cad-sim`) to produce `cad_sim_results.tex` and the CAD figures before
compiling the manuscript.

### Option B: Custom Simulations

#### 1. Via Command Line (CLI)

Run a custom warp simulation directly from your terminal:

```bash
# Run simulation at 15 m radius and 2.5c target velocity
shbt-warp-sim --radius 15.0 --velocity 2.5 --plot

# Export numerical diagnostics to JSON
shbt-warp-sim --radius 10.0 --velocity 1.5 --output-json sim_out.json
```

CLI flags:

- `--radius` (float, default `10.0`) — boundary radius `R` in meters.
- `--velocity` (float, default `1.071186`) — target shift velocity in units of `c`.
- `--displacement` (float, default `0.2636895`) — unitary character population
  displacement `||delta rho||_1`.
- `--output-json` (str, optional) — path to export numerical results as JSON.
- `--plot` — generate metric shift and energy-condition diagnostic plots.
- `--audit` — default behavior; generate `sim_results.tex` and run benchmark
  checks when no `--output-json` or `--plot` flags are supplied.

Parameter sweeps are still supported with `--sweep-radius`, `--sweep-phase`, and
`--sweep-output`.

#### 2. Via Python API

Import `shbt_warp` into your Python scripts or Jupyter notebooks:

```python
import shbt_warp

# Initialize canonical 2D CFT register
register = shbt_warp.BoundaryRegister(k_l=26, k_q=8, K=312)
projector = shbt_warp.FGSliceProjector(register)

# Project custom bulk metric at 12 m radius and 2.0c velocity
metric = projector.project_bulk_slice(radius_m=12.0, target_velocity_c=2.0)

print(f"Operational Power: {metric.operational_power_mw:.2f} MW")
print(f"Proper Acceleration Norm: {metric.proper_acceleration_norm} m/s^2")
print(f"Weak Energy Condition Met: {metric.wec_satisfied}")
```

Check `examples/custom_simulation.py` for a velocity sweep and
`examples/warp_exploration.ipynb` for an interactive plot of the shift profile
`f(r_s)` and operational power vs boundary radius `R`.

## Running tests

```bash
# Rust unit tests
cargo test

# Full Python test suite (requires the PyO3 extension to be installed)
source .venv/bin/activate
pytest

# CAD-focused integration tests only
pytest tests/test_shbt_cad.py
```

The Rust suite checks the exact `c_dark^residual = 834433/362670` fraction,
zero framing defect, warp-velocity formula, the 142.08 MW power benchmark,
and 3D metric/stress-energy audits.  The Python suite checks CLI output,
figure generation, macro export, parameter sweeps, the time-stepping
transient data, and the SHBT-CAD flight-phase / HIL / emitter invariants.

## Python API

### Standard simulation

```python
import shbt_warp

result = shbt_warp.run_simulation(
    radius=10.0,
    domain_radius=30.0,
    grid_points=1201,
    wall_steepness=0.8,
    phase=0.421,
    stress_grid_points=7,
)

print(result["power_mw"])              # 142.08 MW at R = 10 m
print(result["v_eff_c"])             # exp(Delta_mod / 2)
print(result["c_dark_residual"])     # 834433 / 362670
print(result["transient"]["entropy_debt"])
print(result["derender"]["rerender_trajectory"]["origin_x"])
```

### SHBT-CAD flight engine

```python
from shbt_warp import SHBTCADEngine

engine = SHBTCADEngine(
    radius=10.0,
    phase=0.421,
    t_ramp=1.0,
    t_steering=1.0,
    n_steps=64,
)

report = engine.run_flight_simulation()
engine.export_latex_macros(report, "cad_sim_results.tex")
engine.render_figures(report, "figures")

print(report["invariants"]["canonical_branch"])   # (26, 8, 312)
print(report["invariants"]["framing_defect"])     # 0.0
print(report["metric_grid"]["flat_metric_determinant"])  # -1.0
```

### Interactive Jupyter notebook

```bash
source .venv/bin/activate
jupyter notebook examples/warp_exploration.ipynb
```

The notebook imports `shbt_warp.BoundaryRegister` and `shbt_warp.FGSliceProjector`
and plots the warp shift profile `f(r_s)` and operational power as a function of
boundary radius `R`.

## LaTeX manuscript

The paper source is in `main.tex` and `sections/`.  The new sections are:

- `sections/09_flight_phases.tex` — 3+1D ADM line element, shift vector, and
  three-phase flight dynamics.
- `sections/10_emitter_array.tex` — boundary emitter-array hardware synthesis
  and noise sensitivity.
- `sections/11_hil_safety.tex` — real-time HIL control loops and safety
  algorithms.

`main.tex` loads both `sim_results.tex` and `cad_sim_results.tex`; if either
is missing it falls back to the canonical benchmark values so the document
remains compilable.  `make` guarantees the live, simulation-driven values are
used in `main.pdf`.

## Key physics macro outputs

The standard simulator writes the following macros to `sim_results.tex`:

| Macro | Definition | Benchmark value |
|-------|------------|-----------------|
| `\SimOutputBranch` | Canonical visible branch `(lepton, quark, parent)` | `(26, 8, 312)` |
| `\SimOutputCanonicalBranch` | Alias for `\SimOutputBranch` | `(26, 8, 312)` |
| `\SimOutputFramingDefect` | Scalar framing defect `Delta_fr` | `0` |
| `\SimOutputDarkResidual` | `c_dark^residual = 834433 / 362670` | `2.300805139659` |
| `\SimOutputDarkCompleted` | Completed dark central charge `c_dark` | `3.300805139659` |
| `\SimOutputEntropyDebt` | Entropy-debt uplift `Delta_mod = c_dark / 24` | `0.137533547486` |
| `\SimOutputWarpVelocity` | Effective warp velocity `v_eff / c = exp(Delta_mod / 2)` | `1.071186351` |
| `\SimOutputOperationalPowerMW` | Operational power at `R = 10 m` | `142.08` MW |
| `\SimOutputMetricEigenMin` | `min_x lambda_min(g^(G)(x))` | `0.358567865584` |
| `\SimOutputAccelerationNorm` | Comoving proper-acceleration norm | `0` m s^{-2} |

The CAD engine writes the following additional macros to `cad_sim_results.tex`:

| Macro | Definition | Benchmark value |
|-------|------------|-----------------|
| `\CADCanonicalBranch` | Canonical branch `(k_l, k_q, K)` | `(26, 8, 312)` |
| `\CADFramingDefect` | Scalar framing defect `Delta_fr` | `0.000000000000` |
| `\CADUnitarityResidual` | `abs(Re(Tr rho) - 1) + abs(Im(Tr rho))` | `< 1e-14` |
| `\CADStinespringRatio` | `eta_D = c_dark^res / c_dark^comp` | `0.697043612789` |
| `\CADMetricDeterminant` | Collapse metric determinant `det g` | `-1.000000000000` |
| `\CADPhaseJitterOk` | Phase jitter pass flag | `true` |
| `\CADThermalDecoherenceOk` | Thermal/decoherence pass flag | `true` |
| `\CADIntegerLockOk` | Integer level lock pass flag | `true` |

## Audit benchmarks

| Audit | Criterion |
|-------|-----------|
| Boundary normalization | `abs(sum rho_E - 1) <= tolerance` |
| Excitation unitarity | `norm(O^dagger O - I_9, inf) <= tolerance` |
| Framing defect | `Delta_fr = 0` exactly (integer lifts) |
| Lorentzian signature | `abs(det g^(L)(x) + 1)` small, metric eigenvalues negative/positive in `(-,+,+,+)` order |
| Gram positivity | `lambda_min(g^(G)(x)) > 0` everywhere |
| Causal observer | `g^obs = eta` in the plateau, zero proper acceleration |
| Stress-energy | NEC/WEC sampled over 100 random null/timelike vectors; residuals below tolerance |
| Thermodynamics | Steady-state entropy debt equals `Delta_mod`; maximum hold time infinite for the closed-defect branch |
| Phase jitter | `sigma_theta <= 5.05e-5` rad |
| Thermal noise | `T_N <= 15.4` mK |
| Decoherence | `gamma_dec <= 1.2e-4` s^{-1} |
| Integer level lock | `delta k_l = delta k_q = delta K = 0` |

## Generated artifacts and version control

Most files produced by the build pipeline are **not** committed to the
repository:

- `sim_results.tex`
- `cad_sim_results.tex`
- `figures/*.pdf`
- `*.aux`, `*.log`, `*.out`, `*.toc`, `mainNotes.bib`, `*.synctex.gz`
- `sweep_results.csv`

They are listed in `.gitignore` and should always be regenerated locally with
`make`.  This keeps the repository focused on source code and guarantees that
`main.pdf` reflects the exact current state of the Rust core, Python scripts,
and LaTeX source.

`main.pdf` is the one exception: it is tracked as a convenience so the paper
can be read without a TeX Live installation.  If the generated macros or figure
PDFs are missing, run `make` to regenerate them before compiling `main.tex`
directly with `pdflatex`.

## License

MIT.  See `LICENSE` for details.
