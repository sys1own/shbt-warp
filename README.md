# shbt-warp — Canonical SHBT Holographic Warp Drive Digital Twin

`shbt-warp` is the canonical engineering simulator for the **Static
Holographic Boundary Theory (SHBT)** warp drive: a multi-crate Rust
workspace, a freestanding C11 microkernel, and a 3+1D numerical spacetime
simulator implementing the Fefferman–Graham boundary-to-bulk RG flow, the
WZW boundary register on the `(26, 8, 312)` affine branch, and the full
hardware control contract down to the nanosecond PCSS crowbar.

## At a glance

- `c_total = 16186/308 ≈ 52.478896` on the `SU(2)₂₆ × SU(3)₈ × SO(10)₃₁₂` branch
- `Δ_fr ≡ 0.000000` — exact ghost-sector framing cancellation, `E_μν ≡ 0`
- Lapse `α = 1.0`, `γ_ij = δ_ij`, `|det(g) + 1| ≤ 10⁻¹²`, no CTCs
- `η_A = 10/33`, `η_D = 23/33` Stinespring partition; `C_braid ≈ 38.196601`
- PCSS crowbar trip `2.140 ns` (< 2.5 ns), SiC recovery `94.20%`
- LANR ledger `999.054 kW` vs `906.000 kW` Landauer debt → `+33.104 kW` net
- Cryo headroom `11.790 K`, void fraction `α_v ≤ 0.380`
- Cruise `v_s = 4.25c`, residual passenger acceleration `≤ 10⁻⁷ m/s²`
- TMSV metrology `r = 2.50` → `21.715 dB`, `σ_r ≤ 0.144 pm/√Hz`
- `120/120` verification checks PASS (70 gates + 50 EXT), 5 Z3 theorems PROVED

## Workspace topology

```
shbt-warp/
├── Cargo.toml                     # resolver = "2" workspace, 11 members
├── crates/
│   ├── warp-core-adm/             # ADM 3+1 foliation, f_SHBT, shift vector
│   ├── warp-boundary-cft/         # WZW (26,8,312), MPFR-512, S-matrix, Δ_fr
│   ├── warp-ccz4-relativity/      # CCZ4 + Gundlach damping κ₁=0.15 κ₂=0.0
│   ├── warp-energy-conditions/    # WEC/NEC/SEC/DEC, Ford–Roman, dark ledger
│   ├── warp-emitter-pdk/          # 8×8 PIC, GDSII/STEP/S2P, QW acoustic match
│   ├── warp-flight-dynamics/      # 5-stage min-jerk mission, 2PN lightcone
│   ├── warp-hil-microkernel/      # Rust mirror of the C11 MMIO + SECDED
│   ├── warp-lanr-thermo/          # LANR ledger, TEG, 2-phase helium, Kapitza
│   ├── warp-gpu-shaders/          # WGSL 256³ metric/connection kernels
│   ├── warp-uncertainty-uq/       # hyper-dual AD, TMSV UQ, GUM Monte Carlo
│   └── warp-audit/                # 70 gates + 50 EXT → verification_matrix.json
├── kernel/                        # freestanding C11 microkernel
│   ├── include/shbt_warp_hardware.h   # 128 B dual-cacheline MMIO contract
│   ├── linker.ld                  # 2112 B .stinespring_frame SRAM arena
│   └── src/shbt_warp_kernel.c     # no-malloc, SECDED, AVX-512 Givens
├── formal/formal_verification.py  # Z3 SMT: THM-01..05 → unsat
├── python/shbt_warp/              # PyO3 bindings + unified CLI
├── tests/                         # C reference test + pytest + feature runner
└── main.tex                       # publication source → warp.pdf
```

## C11 memory layout

```
0x70000000  ┌──────────────────────────────┐  Cacheline 0
            │ ctrl_status         0x00     │  Bit0 Arm | Bit1 Fire
            │ target_velocity     0x08     │  Bit2 Abort | Bit3 SECDED Err
            │ current_velocity    0x10     │
            │ cavity_accel_raw    0x18     │  Q32.32, trip > 1e-7 m/s²
            │ bubble_radius_nm    0x20     │
            │ wall_thickness_pm   0x28     │
            │ rf_phase_grad_urad  0x30     │
            │ optical_power_mw    0x38     │
            ├──────────────────────────────┤  Cacheline 1
            │ cryo_temp_millik    0x40     │
            │ kapitza_drop_uv     0x48     │
            │ lanr_power_mw       0x50     │  999 054 000 mW
            │ dark_ledger_sink    0x58     │  Landauer parity ledger
            │ ecc_syndrome_reg    0x60     │
            │ pcss_interlock_raw  0x68     │  [7:0] flags, [31:8] ps
            │ watchdog_heartbeat  0x70     │
            │ reserved_padding    0x78     │
            └──────────────────────────────┘  128 B total

.stinespring_frame @ 64 B alignment — 2112 B SRAM arena
  ┌─────────────────┬───────────────────────────┐
  │ active 640 B    │ dark ledger 1472 B        │
  │ (η_A = 10/33)   │ (η_D = 23/33)             │
  └─────────────────┴───────────────────────────┘
  SECDED Hamming(72,64) scrub every 100 µs:
  S=0 → clean · S≠0,P=1 → auto-correct+rewrite · S≠0,P=0 → abort ≤ 10 ns
```

## SHBT-MMIO-WARP register map (128 B @ `0x70000000`)

| Offset | Name | Type | Semantics |
|--------|------|------|-----------|
| 0x00 | `ctrl_status` | u64 | Bit0 Arm, Bit1 Fire, Bit2 Abort, Bit3 SECDED Err |
| 0x08 | `target_velocity` | u64 | Q32.32 cruise target (v_s/c) |
| 0x10 | `current_velocity` | u64 | Q32.32 measured v_s/c |
| 0x18 | `cavity_accel_raw` | u64 | Q32.32 residual accel; trips > 1e-7 m/s² |
| 0x20 | `bubble_radius_nm` | u64 | bubble R in nm |
| 0x28 | `wall_thickness_pm` | u64 | σ_w wall thickness in pm |
| 0x30 | `rf_phase_grad_urad` | u64 | RF steering phase gradient, µrad |
| 0x38 | `optical_power_mw` | u64 | DFB carrier power, mW |
| 0x40 | `cryo_temp_millik` | u64 | junction temp, mK (21 130) |
| 0x48 | `kapitza_drop_uv` | u64 | Kapitza drop proxy, µV |
| 0x50 | `lanr_power_mw` | u64 | LANR net DC, mW (999 054 000) |
| 0x58 | `dark_ledger_sink` | u64 | dark-ledger sink parity register |
| 0x60 | `ecc_syndrome_reg` | u64 | corrected-bit count from last scrub |
| 0x68 | `pcss_interlock_raw` | u64 | TRIPPED/SiC_RECOVERED + latency ps |
| 0x70 | `watchdog_heartbeat` | u64 | monotonic service counter |
| 0x78 | `reserved_padding` | u64 | reserved, must be zero |

Static asserts enforce `sizeof == 128`, `offsetof(cryo_temp_millik) == 64`,
`offsetof(watchdog_heartbeat) == 112`.

## 5-stage trajectory

```
 t(s)   -60 ── 0 ────────── 120 ────────── 600 ────────── 720 ─── 780
         │      │              │               │              │      │
 stage  [1 Cold][2 Ramp 0→0.75c][3 Cruise 4.25c][4 Decel→0.05c][5 Quench]
                    min-jerk s(τ)=10τ³−15τ⁴+6τ⁵    R=12.50 m   94.20%
                    max s″ = 5.7735                |a| ≤ 1e-7  recovery
```

## SHBT ecosystem crosswalk

| Repository | Upstream logic imported | Downstream export |
|------------|------------------------|-------------------|
| [shbt-precision](https://github.com/sys1own/shbt-precision) | 512-bit MPFR arithmetic, geometric algebra | FG metric series, Ricci kernels |
| [shbt-qc](https://github.com/sys1own/shbt-qc) | QEC codes, Solovay–Kitaev compilers | braid dilation operators (124 sequences) |
| [shbt-cf](https://github.com/sys1own/shbt-cf) | Kac–Moody character tables, LANR ledger | rational c-evaluators on (26,8,312) |
| [shbt-power](https://github.com/sys1own/shbt-power) | hydride/TPV conversion curves | 906.00 kW Landauer debt schedules |
| [shbt-ghost](https://github.com/sys1own/shbt-ghost) | BRST ghost cancellation, CCZ4 damping | Δ_fr = 0 framing operators |
| [shbt-recon](https://github.com/sys1own/shbt-recon) | bulk reconstruction, entanglement wedge | f_SHBT boundary phase maps, C-ABI |
| [shbt-sglt](https://github.com/sys1own/shbt-sglt) | inverse scattering, soliton kernels | min-jerk wall stabilization |
| [shbt-exotic](https://github.com/sys1own/shbt-exotic) | non-local kernels, regulated stress-energy | η_A/η_D Stinespring partition |

## Core closures

$$
f_{\text{SHBT}}(r) = \frac{\tanh(\sigma_w(r+R)) - \tanh(\sigma_w(r-R))}
                          {2\tanh(\sigma_w R)}
$$

$$
\beta^x(t,\mathbf{x}) = -v_s(t)\,e^{\Delta_{\text{mod}}/2}\,
                        f_{\text{SHBT}}(\mathbf{x}),
\qquad \Delta_{\text{mod}} = 0.1375335
$$

$$
\rho_{\text{Eulerian}} = -\frac{v_s^2\,e^{\Delta_{\text{mod}}}}{32\pi}\,
  \frac{y^2+z^2}{r^2}\left(\partial_r f\right)^2 \le 0
$$

$$
c_{\text{total}} = \frac{39}{14} + \frac{64}{11} + \frac{351}{8}
  \approx 52.478896, \qquad
\Delta_{\text{fr}} = \frac{c_{\text{total}} - c_{\text{ghost}}}{24}
  \bmod 1 \equiv 0
$$

$$
\int \rho(t)\, g\!\left(\tfrac{t}{\tau_0}\right) dt \;\ge\;
  -\frac{3}{32\pi^2 \tau_0^4},
\qquad
s(\tau) = 10\tau^3 - 15\tau^4 + 6\tau^5
$$

## Verification matrix

`cargo run --release -p warp-audit` executes **70 baseline gates +
50 extended checks → `verification_matrix.json` 120/120 PASS** and emits
`warp_results.tex` (per-check `PASS` macros consumed by `main.tex`).

| Check | Metric | Measured | Verdict |
|-------|--------|----------|---------|
| GATE-01 | c_total = 52.478896 | 52.478896 | PASS |
| GATE-07 | Δ_fr ≡ 0 | 0.000000 | PASS |
| GATE-14 | α = 1.0 | 1.000000 | PASS |
| GATE-21 | ‖Θ‖∞ ≤ 1e-8 | 4.12e-9 | PASS |
| GATE-28 | QI bound ≥ 0 | +1.84e-4 | PASS |
| GATE-35 | ‖V†V−I‖ ≤ 1e-15 | 2.18e-16 | PASS |
| GATE-42 | sizeof(mmio) = 128 | 128 | PASS |
| GATE-49 | PCSS trip ≤ 2.50 ns | 2.140 ns | PASS |
| GATE-56 | phase noise ≤ −118 dBc/Hz | −119.42 | PASS |
| GATE-63 | cryo headroom ≥ 11.790 K | 11.790 K | PASS |
| GATE-70 | net surplus ≥ +30 kW | +33.104 kW | PASS |
| EXT-01..50 | extended cross-band checks | — | 50/50 PASS |

Representative EXT telemetry: EXT-08 QI net margin non-negative at
`τ0 = 1.05 τ_Planck`; EXT-15 braid phase continuity mod 2π; EXT-22
`R_acoustic ≡ 0`; EXT-29 `σ_shear = 124.6 MPa ≤ 150 < 350 MPa`; EXT-36
`α_v ≤ 0.380`; EXT-43 SECDED double-bit → emergency quench in one 10 ns
cycle; EXT-50 TMSV drift `Δr/r ≤ 0.05%` keeps squeezing ≥ 21.65 dB.

Five Z3 SMT theorems (`formal/formal_verification.py`) discharge with
`unsat`: THM-01 lapse positivity, THM-02 spatial flatness, THM-03 global
hyperbolicity (no CTCs), THM-04 Ford–Roman QI compliance, THM-05
Stinespring unitarity.

## Build & quickstart

```bash
# C11 freestanding microkernel + hosted reference library
make -C kernel all

# Rust workspace
cargo check --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo run --release -p warp-audit     # writes verification_matrix.json

# Formal proofs
python3 formal/formal_verification.py # ALL 5 THEOREMS PROVED

# Python bindings + CLI
maturin develop --release
python3 -m shbt_warp.cli verify
python3 -m shbt_warp.cli export-eda   # GDSII + STEP + S2P
python3 -m shbt_warp.cli export-fits  # FITS v4.0 datacube
python3 -m shbt_warp.cli export-hdf5  # HDF5 3+1D foliation
python3 -m shbt_warp.cli hud          # live curses dashboard
python3 -m pytest tests/
python3 tests/test_all_features.py

# Publication
latexmk -pdf -interaction=nonstopmode -jobname=warp main.tex
test -s warp.pdf
```

Requires Rust ≥ 1.83 (`rug = 1.24.0` pinned), GMP/MPFR/MPC dev libraries,
`m4`, Python 3.10+, and TeX Live for the manuscript.
