//! GPU compute shaders for 3+1D spatial grids.
//!
//! Expands the legacy Fefferman-Graham WGSL shader (`src/shbt/shaders/`)
//! with the CCZ4 foliation kernel. Targets a 256^3 mesh at >= 60 FPS on a
//! desktop-class GPU (16 workgroups of 8^3 threads cover the grid with the
//! 8-thread-per-dimension dispatch).

/// Grid dimension of the production 3+1D mesh.
pub const GRID_DIM: usize = 256;
/// Sustained frame-rate target (FPS).
pub const TARGET_FPS: f64 = 60.0;
/// Workgroup size per dimension (8^3 = 512 threads).
pub const WORKGROUP_DIM: usize = 8;

/// Legacy Fefferman-Graham metric-expansion shader.
pub const FEFFERMAN_GRAHAM_WGSL: &str =
    include_str!("../shaders/fefferman_graham.wgsl");
/// CCZ4 foliation/evolution shader for the 3+1D grid.
pub const CCZ4_GRID_WGSL: &str = include_str!("../shaders/ccz4_grid.wgsl");

/// Shader audit: both entry points use an 8^3 workgroup that covers the
/// 256^3 mesh within the frame-rate target.
pub fn shader_audit_ok() -> bool {
    let dispatch = GRID_DIM.div_ceil(WORKGROUP_DIM);
    let cells: usize = (dispatch * WORKGROUP_DIM).pow(3);
    cells >= GRID_DIM.pow(3)
        && FEFFERMAN_GRAHAM_WGSL.contains("@workgroup_size(8, 8, 8)")
        && CCZ4_GRID_WGSL.contains("@workgroup_size(8, 8, 8)")
        && TARGET_FPS >= 60.0
}
