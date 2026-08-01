import shbt_warp
import numpy as np


def run_velocity_sweep():
    print("--- Running SHBT Warp Velocity Sweep ---")
    register = shbt_warp.BoundaryRegister(k_l=26, k_q=8, K=312)
    projector = shbt_warp.FGSliceProjector(register)

    velocities = np.linspace(0.5, 3.0, 6)
    for v in velocities:
        metric = projector.project_bulk_slice(
            radius_m=10.0, target_velocity_c=float(v)
        )
        print(
            f"Velocity: {v:.2f}c | "
            f"Power: {metric.operational_power_mw:.2f} MW | "
            f"WEC Valid: {metric.wec_satisfied}"
        )


if __name__ == "__main__":
    run_velocity_sweep()
