//! Buildings, twice over: as a mesh to draw, and as the solids they are made
//! of, for whatever has to grow around them.
//!
//! * [`Solid`]: an upright box, cylinder, cone, gabled prism or square
//!   pyramid, turned about the vertical, with an exact signed distance --
//!   on the host ([`Solid::distance`]) and, the same arithmetic, in WGSL
//!   ([`SOLID_WGSL`]).
//! * [`SolidIndex`]: the distance to a union of solids, near a point, without
//!   visiting all of them.
//! * [`Mesh`]: flat-shaded triangles with a colour and a [`Material`] per
//!   vertex, for a shader to draw masonry, plaster, timber and roofs from.
//! * [`medieval`]: towers round and square, curtain walls with battlements,
//!   timber-framed houses, a castle and the village below it. Each comes as
//!   a [`Structure`]: its solids and its mesh, which agree.
//!
//! Lengths are metres, `+Y` is up.

pub mod index;
pub mod medieval;
pub mod mesh;
pub mod solid;
pub mod structure;

pub use index::SolidIndex;
pub use mesh::{Material, Mesh};
pub use solid::{SOLID_WGSL, Shape, Solid};
pub use structure::{Feature, FeatureKind, Structure};
