//! Cloth: a garment made of flat panels, sewn together and draped on a body.
//!
//! Built on `fabelgeist-xpbd` for the solver and `fabelgeist-physics` for collision.
//! What this crate adds is everything specific to fabric:
//!
//! * [`triangulate`] -- a flat outline becomes a roughly uniform triangle
//!   mesh. Uniform matters: the stretch constraints are the mesh's edges, and
//!   a sliver is a constraint far stiffer than its neighbours.
//! * [`topology`] -- the edges and the triangle pairs that hinge, derived from
//!   the triangles alone.
//! * [`garment`] -- panels, seams, and the build that places each panel in
//!   space and turns the seam list into constraints.
//! * [`Fabric`] -- density, compliance, thickness and friction, with presets.
//! * [`SelfCollision`] -- a uniform spatial hash, rebuilt every substep.
//! * [`gpu_contact`] -- swept face and edge contacts between substeps, on the
//!   device. [`surface_contact`] is the same solve on the host.
//! * [`Cloth`] -- all of it together, plus the step.
//!
//! Nothing here knows about any particular pattern library. A caller converts
//! its own format into [`garment::Panel`]s and [`garment::Seam`]s.

mod ccd;
pub mod cloth;
pub mod fabric;
pub mod garment;
pub mod gpu_contact;
pub mod outer_layer;
pub mod selfcollision;
pub mod surface_contact;
pub mod topology;
pub mod triangulate;
pub mod wgsl;

pub use cloth::{Cloth, HostContactSchedule};
pub use fabric::Fabric;
pub use garment::{GarmentMesh, Panel, Placement, Seam, SeamSide, build};
pub use selfcollision::SelfCollision;
pub use topology::{BendQuad, Topology};
pub use triangulate::{PanelMesh, triangulate};
