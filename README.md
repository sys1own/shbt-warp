# SHBT Holographic Warp Drive Simulator

`shbt-warp` is a production-grade Rust + PyO3 implementation of the SHBT
holographic warp-drive simulator. It extends the ideas in `shbt-precision` with
phase-locked boundary entanglement metrics, a 3D stress-energy audit, artificial
de-rendering events, and boundary thermodynamics.

## Architecture

- `src/boundary.rs` — visible `SU(2)_26 x SU(3)_8` register, unitary character
  excitation, framing-defect and central-charge ledgers.
- `src/projector.rs` — 3D Cartesian grids and the Fefferman–Graham slice shift
  vector `β_x(x) = -c exp(Δ_mod/2) f_SHBT(x, θ)`, plus the 4D metric tensor.
- `src/stress_energy.rs` — finite-difference Christoffel, Ricci, Einstein, and
  effective `T_μν`; automated NEC/WEC sampling over 100 random vectors.
- `src/derender.rs` — `DerenderingEngine` that decouples visible register
  weights into dark residual/completion channels while preserving bit budgets,
  plus a time-stepped re-rendering trajectory.
- `src/thermodynamics.rs` — entropy-debt integration, infinite maximum
  framing hold time, and a transient start-up engine for `Δ_mod(t)`.
- `src/causal_observer.rs` — flat Minkowski interior plateau and operational
  power scaling `P_op ≈ 142.08 MW` at `R_local = 10 m`.
- `src/lib.rs` — PyO3 `Simulation` class that runs every audit and returns a
  Python dictionary, now including time-stepping `transient` data.
- `python/shbt_warp/` — pure-Python package: `_core` PyO3 bindings,
  `shbt-warp-sim` CLI with parameter sweeps, `LaTeXMacroExporter`, and
  Matplotlib PDF plot generator.

## Build

A Python virtual environment is required for `maturin develop`.

```bash
python3 -m venv .venv
source .venv/bin/activate
pip install -r requirements.txt   # or: pip install numpy matplotlib pytest maturin
maturin develop
```

## Usage

Run the simulator from the command line:

```bash
shbt-warp-sim --radius 10.0 --domain-radius 30.0 --grid-points 1201 \
              --wall-steepness 0.8 --phase 0.421 \
              --tex-output sim_results.tex --figures-directory figures
```

This produces:

- `sim_results.tex` — LaTeX macros such as `\SimOutputWarpVelocity`,
  `\SimOutputOperationalPowerMW`, and `\SimOutputShiftFieldFormula`.
- `figures/warp_bubble_profile.pdf`
- `figures/shift_profile.pdf`
- `figures/stress_energy_audit.pdf`
- `figures/derendering_transition.pdf`
- `figures/entropy_gradient.pdf`

### Parameter sweeps

Sweep the vessel radius and/or phase-lock angle and produce a CSV of
operational power and entropy-debt scaling:

```bash
shbt-warp-sim --sweep-radius 5:15:2.5 --sweep-phase 0.1:0.8:0.1 \
              --sweep-output sweep_results.csv --grid-points 301
```

The sweep runs one simulation per `(radius, phase)` pair and reports
`v_eff/c`, `P_op (MW)`, entropy debt, and population shift.

### Interactive Jupyter notebook

Start Jupyter and open the exploration notebook:

```bash
jupyter notebook examples/warp_exploration.ipynb
```

The notebook uses `ipywidgets` sliders to adjust bubble radius, phase angle,
and wall steepness, then calls the Rust `Simulation` and displays
real-time Matplotlib visualisations of the stress-energy audit, shift
profile, and modular-register entanglement density.

### Python API

```python
import shbt_warp

result = shbt_warp.run_simulation(radius=10.0, grid_points=1201)
print(result["power_mw"])           # 142.08
print(result["c_dark_residual"])    # 834433 / 362670

# Time-stepping transient data is included by default:
print(result["transient"]["entropy_debt"])
print(result["derender"]["rerender_trajectory"]["origin_x"])
```

## Tests

```bash
cargo test
pytest
```

Both test suites verify the physics assertions from `main.pdf`, including the
exact `c_dark^residual` fraction, zero framing defect, warp-velocity formula,
NEC/WEC sampling, the 142.08 MW operational-power benchmark, and the new
parameter-sweep and time-stepping modules.

## Key outputs

| Macro | Meaning |
|-------|---------|
| `\SimOutputFramingDefect` | Scalar framing defect `Δ_fr` (identically zero) |
| `\SimOutputDarkResidual` | `c_dark^residual = 834433/362670` |
| `\SimOutputWarpVelocity` | `v_eff/c = exp(Δ_mod/2)` |
| `\SimOutputOperationalPowerMW` | `P_op ≈ 142.08 MW` at `R = 10 m` |
| `\SimOutputShiftFieldFormula` | LaTeX formula for the FG shift vector |
