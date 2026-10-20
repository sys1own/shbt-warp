#!/usr/bin/env python3
"""
formal/verify_warp_battery.py
Unified Z3 SMT Formal Verification Suite for SHBT-MAR-312 Battery-Reactor & IBR-312 Magazine.
Discharges THM-MAR-01..03 and THM-IBR-01..02 (all UNSAT).
"""

import sys
import subprocess
from pathlib import Path

REPO = Path(__file__).resolve().parents[1]

def main():
    print("========================================================================")
    print(" SHBT-WARP BATTERY & MAGAZINE FORMAL Z3 SMT VERIFICATION SUITE ")
    print("========================================================================")

    scripts = [
        ("MAR-312 Graser & Battery Theorems", REPO / "formal/verify_isomer_graser.py"),
        ("IBR-312 Reload Magazine Theorems", REPO / "formal/verify_ibr312.py"),
        ("Coset Spinor Synthesis Theorems", REPO / "formal/verify_warp_synthesis.py"),
    ]

    for name, script in scripts:
        print(f"\n--- Running {name} ({script.name}) ---")
        proc = subprocess.run([sys.executable, str(script)], cwd=REPO, capture_output=True, text=True)
        if proc.returncode != 0:
            print(proc.stdout)
            print(proc.stderr)
            print(f"[FAIL] {name} failed with code {proc.returncode}")
            return 1
        print(proc.stdout.strip())

    print("\n========================================================================")
    print("Result: ALL SHBT-WARP BATTERY FORMAL THEOREMS PROVED (UNSAT)")
    print("========================================================================")
    return 0

if __name__ == "__main__":
    sys.exit(main())
