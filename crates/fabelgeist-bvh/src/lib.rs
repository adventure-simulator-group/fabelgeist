//! Bounding volume hierarchies, on the host and on the GPU.
//!
//! Two builds of the same idea, for two different jobs:
//!
//! * [`cpu`] -- a binned surface-area-heuristic tree. Better quality, host
//!   memory, and the oracle the GPU build is checked against.
//! * [`gpu`] -- a linear (Morton-order) tree built entirely in compute passes,
//!   which is what a per-frame rebuild over deforming geometry needs. It also
//!   publishes its traversal as WGSL source, so a solver kernel can walk it
//!   without a round trip.
//!
//! Both speak [`Aabb`], and both report primitives by the index they had in
//! the array of boxes they were built from.

pub mod aabb;
pub mod cpu;
pub mod morton;

#[cfg(feature = "gpu")]
pub mod gpu;

pub use aabb::{Aabb, Ray, closest_point_on_triangle, ray_triangle};
pub use cpu::{Bvh, Node, TriangleBvh, triangle_bounds};

#[cfg(test)]
mod tests;
