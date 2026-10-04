//! Collision for the XPBD solver: what a particle is not allowed to be inside.
//!
//! Everything here is a *positional* response. A particle that has ended up
//! inside something is moved out, and the substep's own velocity update turns
//! that move into the velocity change it implies -- which is why there are no
//! impulses, restitution coefficients or contact caches anywhere in this
//! crate.
//!
//! * [`Collider`] -- planes, spheres, capsules and boxes, tested in closed
//!   form. A body approximated by capsules collides in a few microseconds.
//! * [`MeshCollider`] -- a triangle mesh with a GPU hierarchy over it, refit
//!   every frame as the mesh deforms. Exact, and what a garment that has to
//!   follow a shoulder blade needs.
//! * [`Collisions`] -- the [`fabelgeist_xpbd::SubstepHook`] that resolves both.
//!
//! Friction is Coulomb, applied positionally: the tangential part of the
//! particle's own movement this substep is cancelled, up to a limit that
//! scales with the penetration depth. Depth stands in for normal force,
//! exactly as it does in the rigid-body formulation.

pub mod collider;
pub mod contact;
pub mod mesh;
pub mod wgsl;

pub use collider::{Collider, Shape};
pub use contact::{
    ColliderCapacity, ColliderCount, ColliderUpdateError, CollisionBuildError, CollisionKernel,
    CollisionRecordError, Collisions,
};
pub use mesh::{MeshCollider, MeshSurface};

#[cfg(test)]
mod tests;
