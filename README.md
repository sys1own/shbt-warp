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
  weights into dark residual/completion channels while preserving bit budgets.
- `src/thermodynamics.rs` — entropy-debt integration and infinite maximum
  framing hold time for the topologically protected closed-defect state.
- `src/causal_observer.rs` — flat Minkowski interior plateau and operational
  power scaling `P_op ≈ 142.08 MW` at `R_local = 10 m`.
- `src/lib.rs` — PyO3 `Simulation` class that runs every audit and returns a
  Python dictionary.
- `python/shbt_warp/` — pure-Python package: `_core` PyO3 bindings,
  `shbt-warp-sim` CLI, `LaTeXMacroExporter`, and Matplotlib PDF plot generator.

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
- `figures/stress_energy_audit.pdf`
- `figures/derendering_transition.pdf`

You can also call the Python API directly:

```python
import shbt_warp

result = shbt_warp.run_simulation(radius=10.0, grid_points=1201)
print(result["power_mw"])           # 142.08
print(result["c_dark_residual"])    # 834433 / 362670
```

## Tests

```bash
cargo test
pytest
```

Both test suites verify the physics assertions from `main.pdf`, including the
exact `c_dark^residual` fraction, zero framing defect, warp-velocity formula,
NEC/WEC sampling, and the 142.08 MW operational-power benchmark.

## Key outputs

| Macro | Meaning |
|-------|---------|
| `\SimOutputFramingDefect` | Scalar framing defect `Δ_fr` (identically zero) |
| `\SimOutputDarkResidual` | `c_dark^residual = 834433/362670` |
| `\SimOutputWarpVelocity` | `v_eff/c = exp(Δ_mod/2)` |
| `\SimOutputOperationalPowerMW` | `P_op ≈ 142.08 MW` at `R = 10 m` |
| `\SimOutputShiftFieldFormula` | LaTeX formula for the FG shift vector |
