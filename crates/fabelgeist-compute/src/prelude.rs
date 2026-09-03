//! The algorithms, in one import.
//!
//! The GPU layer is deliberately *not* re-exported to the outside:
//! `WgpuContext`, `Buffer`, `Texture2d`, `ComputePipeline` and the rest come
//! from `fabelgeist_gpu::prelude`, which is the crate that owns them. The
//! `pub(crate)` line below is this crate's own shorthand for that vocabulary,
//! which every algorithm here is written in.

pub use crate::*;
pub(crate) use fabelgeist_gpu::data::gpu::*;
pub(crate) use fabelgeist_gpu::globals::*;
// The maths the primitives speak in, from the crate that owns it.
#[allow(unused_imports)]
pub(crate) use anyhow::anyhow;
pub use anyhow::{Error, Result};
pub(crate) use fabelgeist_math::vector::*;
