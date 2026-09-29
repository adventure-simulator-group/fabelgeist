pub mod gpu;

// The maths is the vocabulary the GPU types themselves speak -- a buffer of
// `Vec3`, a `Mat4` uniform -- so this crate re-exports its own, under the
// paths everything downstream already uses. The animation model is *not* such
// a vocabulary and is no longer re-exported: take it from `fabelgeist-animation`.
pub use fabelgeist_math::{math, matrix, transform, vector};

pub use gpu::*;
pub use math::*;
pub use matrix::*;
pub use transform::*;
pub use vector::*;
