---
name: Testing the SHBT-Warp simulator
description: How to build, test, and verify the shbt-warp Rust/Python simulator from a fresh checkout.
---

# Testing the SHBT-Warp simulator

## Devin Secrets Needed

None.

## Repository & branch

- Typical checkout: `/home/ubuntu/repos/shbt-warp`
- Working branch for the custom-sim interface: `devin/custom-sim-interface`

## One-time environment setup

1. Ensure system TeX Live packages are installed:
   ```bash
   sudo apt-get install -y texlive-latex-base texlive-latex-recommended \
     texlive-latex-extra texlive-publishers texlive-fonts-recommended poppler-utils
   ```

2. Create the Python 3.10 virtualenv and install dependencies:
   ```bash
   cd /home/ubuntu/repos/shbt-warp
   python3 -m venv .venv
   . .venv/bin/activate
   pip install -q --upgrade pip
   pip install -q -r requirements.txt
   ```

3. Build the Rust/PyO3 extension:
   ```bash
   . .venv/bin/activate
   maturin develop --release
   ```
   The executable entry points `shbt-warp-sim` and `shbt-cad-sim` are created in `.venv/bin/`.

## Key verification commands

- Custom CLI audit at 15 m / 2.0c:
  ```bash
  python -m shbt_warp.cli --radius 15.0 --velocity 2.0
  ```
  Expect exit 0, all audits `PASS`, `Effective warp velocity (c): 2.0...`, `Operational power (MW): ~6.36e+02`.

- Installed executable with canonical velocity:
  ```bash
  .venv/bin/shbt-warp-sim --radius 10.0 --velocity 1.0711863512293989
  ```
  Expect exit 0, all `PASS`, power `1.4208e+02` MW.

- Velocity sweep example:
  ```bash
  python examples/custom_simulation.py
  ```
  Expect exit 0 and six lines from 0.50c to 3.00c. `WEC Valid: False` is expected because the `BulkSliceMetric.wec_satisfied` flag reads the `wec_passed` energy-condition result, which is negative for non-canonical warp velocities.

- Pytest suite:
  ```bash
  pytest tests/ -q
  ```
  Expect `33 passed`.

- Full paper pipeline:
  ```bash
  make clean && make
  ```
  Expect exit 0, `main.pdf` 19 pages, `main.log` with no `!`, `LaTeX Error`, or `Undefined control sequence` lines. The log may still contain `LaTeX Warning: Reference ... undefined` for missing cross-reference labels.

- New Python API smoke test:
  ```python
  import shbt_warp
  r = shbt_warp.BoundaryRegister(k_l=26, k_q=8, K=312)
  p = shbt_warp.FGSliceProjector(r)
  m = p.project_bulk_slice(radius_m=12.0, target_velocity_c=2.0)
  print(m.operational_power_mw, m.wec_satisfied)
  ```
  At R=12 m and v=2.0c, `operational_power_mw` should be approximately 994.5 MW.

## Common gotchas

- `python -m shbt_warp.cli` prints a `runpy.RuntimeWarning` about `shbt_warp.cli` being in `sys.modules` before execution. This is because `shbt_warp/__init__.py` imports `main` from `cli`; it is harmless.
- The CLI default (`--audit`) writes `sim_results.tex` and PDF figures in the working directory.
- `make` runs `maturin develop --release`, `pytest`, `python -m shbt_warp.cli`, `shbt-cad-sim`, and then `pdflatex main.tex` twice.
- `main.log` warnings about undefined references are warnings, not fatal errors, and do not affect the `make` exit code.
- Generated files (`sim_results.tex`, `cad_sim_results.tex`, `figures/*.pdf`, auxiliary files) are ignored by `git` and should be regenerated locally with `make`.

## Regression scope

- Focus on the custom velocity/delta_mod paths (`Simulation`, `FGSliceProjector`, `Metric3DCalculator`, `CausalObserver`, `ThermodynamicRateEngine`) and the new Python API (`BoundaryRegister`, `FGSliceProjector`, `BulkSliceMetric`).
- The `Stress-energy` CLI audit section reports `PASS` based on norm residuals and finite geometry; `wec_satisfied` from `BulkSliceMetric` may be `False` for custom velocities because the model produces negative WEC energy density in the warp bubble wall.
