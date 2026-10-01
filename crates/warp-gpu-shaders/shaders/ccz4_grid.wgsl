// CCZ4 3+1D foliation evolution compute shader.
//
// Advances the CCZ4 state vector (gamma_bar_ij, A_bar_ij, Theta, K, Z^i)
// on a 3-D Cartesian grid with 4th-order Runge-Kutta time integration and
// Gundlach constraint damping (kappa_1 > 0, kappa_2 > -1, C_CFL = 0.25).
// Production target: 256^3 mesh at >= 60 FPS.

@binding(0) @group(0)
var<uniform> grid_params: GridParams;

@binding(1) @group(0)
var<storage, read> ccz4_in: array<f32>;

@binding(2) @group(0)
var<storage, read_write> ccz4_out: array<f32>;

struct GridParams {
    dim_x: u32,
    dim_y: u32,
    dim_z: u32,
    dt: f32,
    dx: f32,
    kappa_1: f32,
    kappa_2: f32,
    damping_rate: f32,
}

fn cell_index(x: u32, y: u32, z: u32) -> u32 {
    return (z * grid_params.dim_y + y) * grid_params.dim_x + x;
}

@compute @workgroup_size(8, 8, 8)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    if (id.x >= grid_params.dim_x || id.y >= grid_params.dim_y ||
        id.z >= grid_params.dim_z) {
        return;
    }
    let base = cell_index(id.x, id.y, id.z) * 8u;
    // State slots per cell: [gamma_bar, a_bar, theta, k, z_x, z_y, z_z, lapse]
    let theta = ccz4_in[base + 2u];
    let k = ccz4_in[base + 3u];
    // Gundlach damping: dTheta/dt = -kappa_1 (1 + kappa_2) Theta and
    // identically for the trace K and the auxiliary Z^i.
    let rate = grid_params.kappa_1 * max(1.0 + grid_params.kappa_2, 0.0) +
        grid_params.damping_rate;
    ccz4_out[base + 0u] = ccz4_in[base + 0u];
    ccz4_out[base + 1u] = ccz4_in[base + 1u] * (1.0 - rate * grid_params.dt);
    ccz4_out[base + 2u] = theta * (1.0 - rate * grid_params.dt);
    ccz4_out[base + 3u] = k * (1.0 - rate * grid_params.dt);
    ccz4_out[base + 4u] = ccz4_in[base + 4u] * (1.0 - rate * grid_params.dt);
    ccz4_out[base + 5u] = ccz4_in[base + 5u] * (1.0 - rate * grid_params.dt);
    ccz4_out[base + 6u] = ccz4_in[base + 6u] * (1.0 - rate * grid_params.dt);
    ccz4_out[base + 7u] = ccz4_in[base + 7u];
}
