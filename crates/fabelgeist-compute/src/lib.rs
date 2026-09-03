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
//! * [`Kernel`] -- raw WGSL, storage buffers bound by name, for the passes
//!   none of the shapes above fit.

pub mod advancing_front;
pub mod advect;
pub mod broadcast;
pub mod distance_field;
pub mod distance_field_jfa;
pub mod divergence;
pub mod dual_contouring;
pub mod gather;
pub mod gradient;
pub mod kernel;
pub mod map;
pub mod marching_cubes;
pub mod matmul;
pub mod perlin_noise;
pub mod prelude;
pub mod reduce;
pub mod reshape;
pub mod scan;
pub mod scatter;
pub mod simplex_noise;
pub mod sort;
pub mod stencil;
pub mod stream;
pub mod surface_extraction;
pub mod transpose;

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
pub use perlin_noise::RenderPerlin;
pub use reduce::*;
pub use reshape::*;
pub use scan::*;
pub use scatter::*;
pub use simplex_noise::RenderSimplex;
pub use sort::{RadixSort, SortScratch};
pub use stencil::*;
pub use stream::*;
pub use surface_extraction::*;
pub use transpose::*;
