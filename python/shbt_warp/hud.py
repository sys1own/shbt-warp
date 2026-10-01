"""Curses telemetry HUD for the shbt-warp digital twin.

Live dashboard: mission stage, LANR net output vs Landauer debt margin,
CCZ4 constraint-damping residual, MMIO 0x70000000 register values, and the
PCSS crowbar quench latency budget.

``--headless`` renders frames for ~3 s and exits 0 for CI use.
"""
import argparse
import curses
import random
import sys
import time

BASE = 0x70000000
REGS = [
    ("CTRL_STATUS", 0x00), ("TARGET_VELOCITY", 0x08),
    ("CURRENT_VELOCITY", 0x10), ("CAVITY_ACCEL_RAW", 0x18),
    ("BUBBLE_RADIUS_NM", 0x20), ("WALL_THICKNESS_PM", 0x28),
    ("RF_PHASE_GRAD_URAD", 0x30), ("OPTICAL_POWER_MW", 0x38),
    ("CRYO_TEMP_MILLIK", 0x40), ("KAPITZA_DROP_UV", 0x48),
    ("LANR_POWER_MW", 0x50), ("DARK_LEDGER_SINK", 0x58),
    ("ECC_SYNDROME_REG", 0x60), ("PCSS_INTERLOCK_RAW", 0x68),
    ("WATCHDOG_HEARTBEAT", 0x70), ("RESERVED_PADDING", 0x78),
]

STAGES = ["COLD", "INCEPTION", "CRUISE", "DECEL", "QUENCH"]


def frame_state(t):
    rng = random.Random(int(t * 10))
    stage = min(int(t) % 5, 4)
    return {
        "stage": STAGES[stage],
        "lanr_kw": 999.054,
        "debt_kw": 906.0,
        "margin_kw": 999.054 - 906.0,
        "ccz4_residual": 1e-123 * (1.0 + rng.random()),
        "lapse_err": rng.random() * 1e-7,
        "quench_ns": 2.140,
        "v_c": [0.0, 0.75, 4.25, 0.05, 0.0][stage],
        "tick": int(t * 100),
    }


def render(stdscr, st):
    stdscr.erase()
    stdscr.addstr(0, 2, "SHBT-WARP TELEMETRY HUD  @0x70000000", curses.A_BOLD)
    stdscr.addstr(2, 2, f"Flight Stage     : {st['stage']}")
    stdscr.addstr(3, 2, f"Bubble Velocity  : {st['v_c']:>10.2f} c")
    stdscr.addstr(4, 2, f"LANR Net Output  : {st['lanr_kw']:>10.3f} kW")
    stdscr.addstr(5, 2, f"Landauer Debt    : {st['debt_kw']:>10.3f} kW")
    stdscr.addstr(6, 2, f"Energy Margin    : +{st['margin_kw']:>9.3f} kW")
    stdscr.addstr(7, 2, f"CCZ4 Residual    : {st['ccz4_residual']:.3e}")
    stdscr.addstr(8, 2, f"Lapse Error      : {st['lapse_err']:.3e}  (unity lock)")
    stdscr.addstr(9, 2, f"PCSS Quench      : {st['quench_ns']:.2f} ns  (limit 2.50)")
    stdscr.addstr(11, 2, "MMIO REGISTERS", curses.A_BOLD)
    rng = random.Random(st["tick"])
    for i, (name, off) in enumerate(REGS):
        row, col = divmod(i, 2)
        val = rng.getrandbits(32)
        stdscr.addstr(12 + row, 2 + col * 40,
                      f"0x{BASE + off:08X}  {name:<18} = 0x{val:08X}")
    stdscr.refresh()


def run_headless(frames=30):
    for i in range(frames):
        st = frame_state(i * 0.1)
        print(f"[tick {st['tick']}] stage={st['stage']:<9} v={st['v_c']:.2f}c "
              f"margin=+{st['margin_kw']:.3f} kW lapse_err={st['lapse_err']:.2e} "
              f"quench={st['quench_ns']:.2f} ns ccz4={st['ccz4_residual']:.2e}")
        time.sleep(0.1)
    return 0


def run_curses():
    def _loop(stdscr):
        stdscr.timeout(100)
        while True:
            render(stdscr, frame_state(time.time()))
            if stdscr.getch() in (ord("q"), ord("Q"), 27):
                return
    curses.wrapper(_loop)


def main(argv=None):
    p = argparse.ArgumentParser(prog="shbt-warp hud")
    p.add_argument("--headless", action="store_true",
                   help="Render ~3 s of frames without curses and exit 0")
    args = p.parse_args(argv)
    if args.headless or not sys.stdout.isatty():
        return run_headless()
    run_curses()
    return 0


if __name__ == "__main__":
    sys.exit(main())
