//! The GPU algorithm library.
//!
//! Primitives built on `fabelgeist-gpu`: each takes buffers or textures, runs one
//! or more compute passes over them, and hands back buffers or textures. The
//! GPU layer they are built on -- the context, buffers, textures, samplers,
//! shaders, pipelines and passes -- is `fabelgeist-gpu`, and is not re-exported
//! from here: take it from the crate that owns it.
//!
//! * Shaped primitives -- [`Map`], [`Reduce`], [`Scan`], [`Gather`],
//!   [`Scatter`], [`Stream`], [`Stencil`], [`Broadcast`], [`Transpose`],
//!   [`Reshape`], [`RadixSort`].
//! * Fields and geometry -- [`DistanceField`], [`Advect`], [`Divergence`],
//!   [`Gradient`], marching cubes, dual contouring, advancing front, noise.
//! * Mesh searches -- [`MeshQuery`]: nearest point, closest point on a
//!   surface, and ray crossings, one query per invocation.
//! * [`Readback`] -- many buffers back to the host in one mapping.
//! * [`VertexNormalKernels`] -- area- or angle-weighted vertex normals,
//!   summed in triangle order so that they are deterministic.
//! * [`host_float`] -- WGSL arithmetic that rounds as the host's does, for
//!   kernels that must reproduce a host computation bit for bit.
//! * [`Kernel`] -- raw WGSL, storage buffers bound by name, for the passes
//!   none of the shapes above fit.
//! * [`stereo`] -- the geometry around a stereo matcher: rectification for
//!   pinhole, equirectangular and fisheye rigs, triangulation, temporal
//!   filtering, and depth put back on each eye's own picture.

pub mod advancing_front;
pub mod advect;
pub mod broadcast;
pub mod distance_field;
pub mod distance_field_jfa;
pub mod divergence;
pub mod dual_contouring;
pub mod gather;
pub mod gradient;
pub mod host_float;
pub mod kernel;
pub mod map;
pub mod marching_cubes;
pub mod matmul;
pub mod mesh_query;
pub mod perlin_noise;
pub mod prelude;
pub mod readback;
pub mod reduce;
pub mod reshape;
pub mod scan;
pub mod scatter;
pub mod simplex_noise;
pub mod sort;
pub mod stencil;
pub mod stereo;
pub mod stream;
pub mod transpose;
pub mod vertex_normals;

#[cfg(test)]
pub mod test_utils;

pub use advect::{Advect, AdvectDefinition};
pub use broadcast::*;
pub use distance_field::DistanceField;
pub use distance_field_jfa::DistanceFieldJfa;
pub use divergence::{Divergence, DivergenceDefinition};
pub use gather::*;
pub use gradient::{Gradient, GradientDefinition};
pub use kernel::*;
pub use map::*;
pub use matmul::*;
pub use mesh_query::{Crossing, Hit, MeshQuery, PointTargets, QueryHits, Rays, TriangleTargets};
pub use perlin_noise::RenderPerlin;
pub use readback::Readback;
pub use reduce::*;
pub use reshape::*;
pub use scan::*;
pub use scatter::*;
pub use simplex_noise::RenderSimplex;
pub use sort::{
    RadixSort, ScratchGrowth, SortDigit, SortDigits, SortItemCount, SortKeyWidth, SortPassCount,
    SortScratch,
};
pub use stencil::*;
pub use stream::*;
pub use transpose::*;
pub use vertex_normals::{
    DEGENERATE_TRIANGLE, NormalWeighting, VANISHED_NORMAL, VertexNormalKernels, VertexNormals,
};
