# SHBT Holographic Warp Drive Simulator

`shbt-warp` is a production-grade Rust + PyO3 implementation of the SHBT
(Scalar Holographic Boundary Theory) holographic warp-drive simulator.  It
transports and audits the physics developed in `shbt-precision` into a
standalone, high-performance codebase with a Python-facing CLI and an
interactive exploration notebook.

The simulator tracks:

- phase-locked `SU(2)_{26} x SU(3)_8` boundary entanglement and character
  excitation;
- Fefferman-Graham slice geometry, 4D metric tensors, and Lorentzian / Gram
  positivity audits;
- numerical stress-energy tensors, Christoffel symbols, Ricci / Einstein
  curvature, and automated NEC/WEC sampling;
- artificial de-rendering events that transfer visible register weight into
  dark channels while preserving the holographic bit budget;
- entropy-debt thermodynamics, including a time-stepping transient engine for
  `Delta_mod(t)`;
- operational power scaling and causal observer plateau verification.

## Repository structure

| Path | Purpose |
|------|---------|
| `src/boundary.rs` | Visible register construction, modular S-matrix blocks, unitary excitation operator, framing defect and central-charge audits. |
| `src/projector.rs` | 1D FG-slice and 3D Cartesian metric calculators; evaluates the shift vector `beta_x(x) = -c exp(Delta_mod/2) f_SHBT(x, theta)` and metric spectra. |
| `src/stress_energy.rs` | Finite-difference connection, Riemann/Ricci/Einstein curvature, effective `T_{mu nu}`, and 100-vector NEC/WEC sampling. |
| `src/derender.rs` | `DerenderingEngine`: visible-to-dark weight transfer, background-metric restoration, and time-stepped re-rendering trajectories. |
| `src/thermodynamics.rs` | Entropy-debt integration, maximum framing hold time, and RK4 transient engine. |
| `src/causal_observer.rs` | Comoving Minkowski plateau observer and `P_op` power benchmark. |
| `src/lib.rs` | PyO3 `Simulation` class that runs every audit and exposes a Python dictionary. |
| `python/shbt_warp/` | Pure-Python package: CLI, `LaTeXMacroExporter`, `PlotGenerator`, and the `examples/warp_exploration.ipynb` notebook. |
| `main.tex` and `sections/` | APS RevTeX 4-2 manuscript source. |
| `Makefile` | One-command build pipeline: compile the Rust extension, run the simulator, generate figures / `sim_results.tex`, and compile `main.pdf`. |

## Prerequisites

- Rust toolchain (recent stable `rustc` / `cargo`, 1.70+).
- Python 3.10 or newer.
- A working TeX Live installation with `revtex4-2` (only required to compile `main.pdf`).

The `Makefile` will create a local `.venv` automatically if it does not exist.

## Installation and build

```bash
# Clone and enter the repository
git clone https://github.com/sys1own/shbt-warp.git
cd shbt-warp

# Build the Rust extension and install the Python package in editable mode
make sim
```

`make sim` performs the following:

1. Creates `.venv` and installs `requirements.txt`.
2. Runs `maturin develop --release` to build and install the `_core` PyO3 extension.
3. Executes `python -m shbt_warp.cli` to generate `sim_results.tex` and the vector PDF figures in `figures/`.

To build the full paper as well:

```bash
make
```

This additionally runs `pdflatex` on `main.tex` twice, producing `main.pdf`.

If you prefer a manual Python-only build:

```bash
python3 -m venv .venv
source .venv/bin/activate
pip install -r requirements.txt
maturin develop --release
```

## Running tests

```bash
# Rust unit tests
cargo test

# Python tests (requires the PyO3 extension to be installed)
source .venv/bin/activate
pytest
```

The Rust suite checks the exact `c_dark^residual = 834433/362670` fraction, zero framing defect, warp-velocity formula, the 142.08 MW power benchmark, and 3D metric/stress-energy audits.  The Python suite checks CLI output, figure generation, macro export, parameter sweeps, and the time-stepping transient data.

## Command-line usage

### Single simulation

```bash
shbt-warp-sim \
    --radius 10.0 \
    --domain-radius 30.0 \
    --grid-points 1201 \
    --wall-steepness 0.8 \
    --phase 0.421 \
    --tex-output sim_results.tex \
    --figures-directory figures
```

Generated artifacts:

- `sim_results.tex` — LaTeX macros consumed by `main.tex`.
- `figures/warp_bubble_profile.pdf`
- `figures/shift_profile.pdf`
- `figures/stress_energy_audit.pdf`
- `figures/derendering_transition.pdf`
- `figures/entropy_gradient.pdf`

### Parameter sweeps

Sweep the bubble radius and/or phase-lock angle and write a CSV of power and entropy-debt scaling:

```bash
shbt-warp-sim \
    --sweep-radius 5:15:2.5 \
    --sweep-phase 0.1:0.8:0.1 \
    --sweep-output sweep_results.csv \
    --grid-points 301
```

The CSV contains one row per `(radius, phase)` pair with `v_eff/c`, `P_op (MW)`, entropy debt, and population shift.

### Python API

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
print(result["v_eff_c"])                # exp(Delta_mod / 2)
print(result["c_dark_residual"])        # 834433 / 362670
print(result["transient"]["entropy_debt"])
print(result["derender"]["rerender_trajectory"]["origin_x"])
```

### Interactive Jupyter notebook

```bash
source .venv/bin/activate
jupyter notebook examples/warp_exploration.ipynb
```

The notebook uses `ipywidgets` sliders for bubble radius, phase angle, wall steepness, and grid resolution.  Each slider change re-runs the Rust `Simulation` through PyO3 and updates Matplotlib figures for the stress-energy audit, shift profile, and modular-register entanglement density.

## LaTeX manuscript

The paper source is in `main.tex` and `sections/`.  A `make` run produces the full document:

```bash
make
```

If you want to compile only the manuscript after the simulation outputs already exist:

```bash
make paper
```

The `TEXINPUTS` environment variable in the `Makefile` ensures `main.tex` finds the modular section files in `sections/`.

## Key physics macro outputs

The simulator writes the following macros to `sim_results.tex` for direct inclusion in `main.tex`:

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
| `\SimOutputPowerScaleRadius` | Power-scale radius `r_s` from the Alcubierre-style scaling | `1.465189552753 x 10^{-20}` m |
| `\SimOutputSaturatedBitBudget` | Horizon bit budget `N_sat` | `3.312593327986 x 10^{122}` bits |
| `\SimOutputLocalMemoryBits` | Local 10 m register memory | `1.202481 x 10^{72}` bits |
| `\SimOutputBoundaryPartition` | Boundary partition function `Z_partial` | `0.00342010966891` |
| `\SimOutputShannonEntropy` | Visible Shannon entropy `S_E` | `0.999490161461` |
| `\SimOutputBoundaryNormError` | `|sum rho_E - 1|` | `0` |
| `\SimOutputUnitarityError` | `||O_excitation^dagger O - I_9||_inf` | `O(10^{-16})` |
| `\SimOutputPopulationShift` | `||rho_E^(theta) - rho_E||_1` | `0.263689500253` |
| `\SimOutputMetricDetMin` | `min_x |det g^(L)(x)|` | `1` |
| `\SimOutputMetricEigenMin` | `min_x lambda_min(g^(G)(x))` | `0.358567865584` |
| `\SimOutputObserverMetricError` | `||g^obs - eta||_inf` | `0` |
| `\SimOutputAccelerationNorm` | Comoving proper-acceleration norm | `0` m s^{-2} |
| `\SimOutputShiftFieldFormula` | LaTeX form of the FG shift vector | `beta_x(x) = -c e^{Delta_mod/2} f_SHBT(x, theta)` |

## Audit benchmarks

| Audit | Criterion |
|-------|-------------|
| Boundary normalization | `|sum rho_E - 1| <= tolerance` |
| Excitation unitarity | `||O^dagger O - I_9||_inf <= tolerance` |
| Framing defect | `Delta_fr = 0` exactly (integer lifts) |
| Lorentzian signature | `|det g^(L)(x) + 1|` small, metric eigenvalues negative/positive in `(-,+,+,+)` order |
| Gram positivity | `lambda_min(g^(G)(x)) > 0` everywhere |
| Causal observer | `g^obs = eta` in the plateau, zero proper acceleration |
| Stress-energy | NEC/WEC sampled over 100 random null/timelike vectors; residuals below tolerance |
| Thermodynamics | Steady-state entropy debt equals `Delta_mod`; maximum hold time infinite for the closed-defect branch |

## Generated artifacts and version control

Most files produced by the build pipeline are **not** committed to the repository:

- `sim_results.tex`
- `figures/*.pdf`
- `*.aux`, `*.log`, `*.out`, `*.toc`, `mainNotes.bib`, `*.synctex.gz`
- `sweep_results.csv`

They are listed in `.gitignore` and should always be regenerated locally with `make`.  This keeps the repository focused on source code and guarantees that `main.pdf` reflects the exact current state of the Rust core, Python scripts, and LaTeX source.

`main.pdf` is the one exception: it is tracked as a convenience so the paper can be read without a TeX Live installation.  If `sim_results.tex` or the figure PDFs are missing, run `make` to regenerate them before compiling `main.tex` directly with `pdflatex`; `main.tex` also defines fallback macros for `sim_results.tex` so the manuscript remains compilable, but `make` should still be used for the live, simulation-driven values.

## License

MIT.  See `LICENSE` for details.
