//! A scene's surfaces as a field that shaders can query.
//!
//! A renderer draws a scene as separate meshes, and much of what makes it
//! look like one world happens where they meet or come close: a branch
//! growing out of its trunk, a rock sunk in the ground, moss gathering in
//! the crook of a root. This crate describes surfaces by simple [`Shape`]s,
//! bins them into a sparse grid ([`SurfaceIndex`]), and answers questions
//! about the field they make -- from the host, and from any shader ([`wgsl`]):
//!
//! * **nearest**: the nearest surface to a point, and which it is;
//! * **nearest four**: the few nearest, for blending three or more;
//! * **union**: the smooth union of every surface in reach -- a field value;
//! * **inside**: the surface a point lies deepest inside, if any;
//! * **your own**: [`wgsl::query`] writes the walk over a point's candidates
//!   around a WGSL snippet of the caller's, since WGSL has no closures.
//!
//! Every query takes a [`Filter`]: an object to leave out -- a surface does
//! not blend with its own pieces -- and layers to look at.
//!
//! # Blending separate meshes into one
//!
//! The polynomial smooth minimum of two surfaces' distances is their smooth
//! union, and it gives both halves of a seamless seam ([`union`]):
//!
//! * **Geometry.** A vertex near another surface moves out to the union's
//!   surface ([`union::lift`]), so both meshes grow into one fillet.
//! * **Attributes.** The union's mix factor ([`union::mix_factor`]) says how
//!   much of each surface a point is made of. Colour, normal, anything a
//!   material has, mixes by it -- the other surface's evaluated through the
//!   hit's payload. On the seam each side is half of each, so the two agree
//!   by construction.
//!
//! Unreal Engine blends meshes in the same spirit with its global distance
//! field and runtime virtual textures. Here the query is exact rather than
//! baked into a volume, leaves out the asking object, and says *which*
//! surface it found -- so any material can be blended into, and objects of
//! every size share one index.
//!
//! # Reach
//!
//! A shape is binned into every cell its surface, grown by its
//! [`Shape::reach`], touches, so a query reads a single cell's list. The
//! price is that queries are local: a shape is found only within its reach
//! of its surface. Rays and far queries need another traversal.

#[cfg(feature = "gpu")]
pub mod gpu;
pub mod index;
pub mod shape;
pub mod union;
pub mod wgsl;

pub use index::{Filter, Hit, SurfaceIndex, SurfaceParams};
pub use shape::{Shape, ShapeKind};
