// Fefferman-Graham slice metric compute shader.
//
// Evaluates the 4x4 covariant ADM metric on a 3-D Cartesian grid from a
// scalar shape-field buffer and a uniform direction vector n^i.
//
// ADM ansatz:
//   alpha   = 1.0
//   gamma_ij = delta_ij
//   beta^i  = -v_eff * xi * f_SHBT(x^k) * n^i
//   g_00    = -alpha^2 + |beta|^2
//   g_0i    = beta_i    (gamma_ij beta^j = beta^i because gamma = delta)
//   g_ij    = delta_ij

@binding(0) @group(0)
var<uniform> grid_params: GridParams;

@binding(1) @group(0)
var<storage, read> shape_field: array<f32>;

@binding(2) @group(0)
var<storage, read_write> metric_output: array<f32>;

struct GridParams {
    dim_x: u32,
    dim_y: u32,
    dim_z: u32,
    v_eff: f32,
    xi: f32,
    n_x: f32,
    n_y: f32,
    n_z: f32,
};

@compute @workgroup_size(8, 8, 8)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let x: u32 = global_id.x;
    let y: u32 = global_id.y;
    let z: u32 = global_id.z;

    if (x >= grid_params.dim_x || y >= grid_params.dim_y || z >= grid_params.dim_z) {
        return;
    }

    let dim_yz: u32 = grid_params.dim_y * grid_params.dim_z;
    let idx: u32 = x * dim_yz + y * grid_params.dim_z + z;
    let base: u32 = idx * 16u;

    let f_shbt: f32 = shape_field[idx];

    let n: vec3<f32> = normalize(vec3<f32>(grid_params.n_x, grid_params.n_y, grid_params.n_z));
    let b: f32 = -grid_params.v_eff * grid_params.xi * f_shbt;
    let beta: vec3<f32> = b * n;
    let beta_sq: f32 = dot(beta, beta);

    // g_00 = -alpha^2 + |beta|^2, with alpha = 1.0.
    let g00: f32 = -1.0 + beta_sq;

    // Row-major 4x4 covariant metric.
    metric_output[base + 0u]  = g00;
    metric_output[base + 1u]  = beta.x;
    metric_output[base + 2u]  = beta.y;
    metric_output[base + 3u]  = beta.z;

    metric_output[base + 4u]  = beta.x;
    metric_output[base + 5u]  = 1.0;
    metric_output[base + 6u]  = 0.0;
    metric_output[base + 7u]  = 0.0;

    metric_output[base + 8u]  = beta.y;
    metric_output[base + 9u]  = 0.0;
    metric_output[base + 10u] = 1.0;
    metric_output[base + 11u] = 0.0;

    metric_output[base + 12u] = beta.z;
    metric_output[base + 13u] = 0.0;
    metric_output[base + 14u] = 0.0;
    metric_output[base + 15u] = 1.0;
}
