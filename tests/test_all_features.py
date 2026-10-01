#!/usr/bin/env python3
"""Full feature-verification runner for the shbt-warp digital twin.

Exercises every CLI subcommand end to end and asserts that each expected
artifact exists with non-zero size, that the verification matrix reports
120/120 PASS (70 gates + 50 EXT), and that the Z3 formal suite discharges
all five theorems. Exits 0 when everything passes.

Usage:  python3 tests/test_all_features.py
"""
import json
import os
import subprocess
import sys
import tempfile
from pathlib import Path

REPO = Path(__file__).resolve().parents[1]


def run(cmd, timeout=900):
    proc = subprocess.run(cmd, cwd=REPO, capture_output=True, text=True,
                          timeout=timeout)
    return proc


def check(name, ok, detail=""):
    print(f"[{'PASS' if ok else 'FAIL'}] {name} {detail}")
    if not ok:
        FAILURES.append(name)


FAILURES = []


def main():
    # 1. C11 microkernel build + C reference test.
    p = run(["make", "-C", "kernel", "all"], timeout=300)
    check("build kernel", p.returncode == 0, p.stderr.strip()[-200:] if p.returncode else "")
    check("kernel .so artifacts",
          (REPO / "kernel/build/shbt_warp_reference.so").stat().st_size > 0
          and (REPO / "kernel/build/libshbt_warp_reference.so").stat().st_size > 0)
    (REPO / "build").mkdir(exist_ok=True)
    p = run(["gcc", "-std=c11", "-Wall", "-Wextra", "-pedantic",
             "tests/reference_test.c", "-Ikernel/include",
             "-Lkernel/build", "-lshbt_warp_reference", "-lm",
             "-o", "build/test_c_abi"], timeout=120)
    check("gcc reference build", p.returncode == 0, p.stderr[-200:])
    env = dict(os.environ, LD_LIBRARY_PATH=str(REPO / "kernel/build"))
    p = subprocess.run(["./build/test_c_abi"], cwd=REPO, env=env,
                       capture_output=True, text=True, timeout=60)
    check("reference test run", p.returncode == 0 and "all checks passed" in p.stdout)

    # 2. Formal Z3 suite.
    p = run([sys.executable, "formal/formal_verification.py"], timeout=300)
    check("Z3 THM-01..05 PROVED", p.returncode == 0 and "ALL 5 THEOREMS PROVED" in p.stdout)

    # 3. CLI subcommands.
    p = run([sys.executable, "-m", "shbt_warp.cli", "build-kernel"], timeout=300)
    check("cli build-kernel", p.returncode == 0)
    p = run([sys.executable, "-m", "shbt_warp.cli", "verify"], timeout=900)
    matrix_ok = False
    mpath = REPO / "verification_matrix.json"
    if mpath.exists():
        m = json.loads(mpath.read_text())
        matrix_ok = m["passed"] == 120 == m["total"]
    check("cli verify 120/120", p.returncode == 0 and matrix_ok)
    check("warp_results.tex", (REPO / "warp_results.tex").stat().st_size > 0)

    p = run([sys.executable, "-m", "shbt_warp.cli", "sim"], timeout=300)
    check("cli sim", p.returncode == 0)
    p = run([sys.executable, "-m", "shbt_warp.cli", "hud", "--headless"], timeout=60)
    check("cli hud", p.returncode == 0)

    with tempfile.TemporaryDirectory() as td:
        p = run([sys.executable, "-m", "shbt_warp.cli", "export-eda",
                 "--out-dir", td], timeout=300)
        ok = p.returncode == 0 and all(
            (Path(td) / n).stat().st_size > 0
            for n in ("pic8x8.gds", "waveguide.step", "interposer.s2p"))
        check("cli export-eda", ok)
    with tempfile.TemporaryDirectory() as td:
        f = Path(td) / "warp_adm.fits"
        p = run([sys.executable, "-m", "shbt_warp.cli", "export-fits",
                 "--out", str(f)], timeout=300)
        check("cli export-fits", p.returncode == 0 and f.stat().st_size > 0)
        h = Path(td) / "warp_mission.h5"
        p = run([sys.executable, "-m", "shbt_warp.cli", "export-hdf5",
                 "--out", str(h)], timeout=300)
        check("cli export-hdf5", p.returncode == 0 and h.stat().st_size > 0)

    # 4. Publication artifact.
    p = run(["latexmk", "-pdf", "-interaction=nonstopmode",
             "-jobname=warp", "main.tex"], timeout=600)
    check("warp.pdf compiles", p.returncode == 0 and (REPO / "warp.pdf").stat().st_size > 0)

    print()
    if FAILURES:
        print(f"{len(FAILURES)} failures: {FAILURES}")
        return 1
    print("ALL FEATURE CHECKS PASSED")
    return 0


if __name__ == "__main__":
    sys.exit(main())
