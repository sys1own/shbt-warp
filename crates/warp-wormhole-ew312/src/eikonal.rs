//! crates/warp-wormhole-ew312/src/eikonal.rs
//! Progressive Eikonal Foliation Kinematics alias module.
#![forbid(unsafe_code)]

pub use crate::corridor::{
    EikonalKinematicState, ProgressiveEikonalFoliation,
    MAX_DERENDER_VELOCITY_MPS, MAX_DIMENSIONLESS_ACCEL,
    DMA_RING_BPS, DMA_RING_Z0_OHMS, SYSTEM_TICK_HZ, SYSTEM_TICK_SEC,
};
