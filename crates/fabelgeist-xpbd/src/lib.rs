//! Extended position-based dynamics.
//!
//! XPBD solves constraints by moving positions rather than by integrating
//! forces, and adds a compliance term so that a constraint's stiffness is a
//! material property rather than an artefact of the iteration count and the
//! timestep. That is what makes it usable for cloth: a fabric configured once
//! behaves the same whether the solver is running at ten substeps or forty.
//!
//! The pieces:
//!
//! * [`Particles`] -- positions, previous positions, velocities and inverse
//!   masses, on the GPU.
//! * [`ConstraintSet`] -- a kernel plus its constraints, graph-coloured so
//!   that each colour is a race-free parallel dispatch and the colours
//!   together are a Gauss-Seidel sweep.
//! * [`Solver`] -- the substep loop, with a hook where collisions go.
//! * [`wgsl`] -- the shared shader source: the compliance arithmetic, the
//!   particle bindings, and the distance-constraint kernel everything else is
//!   modelled on.
//!
//! Nothing here knows what a garment is. `fabelgeist-cloth` builds the fabric on
//! top, and `fabelgeist-physics` supplies the collision hook.

pub mod coloring;
pub mod constraint;
pub mod particles;
pub mod solver;
pub mod wgsl;

pub use coloring::{ColorCount, ColorRange, Coloring, ConstraintColor, ConstraintSlot};
pub mod incidence;
pub use constraint::{ConstraintCount, ConstraintIndex, ConstraintOccupancy, ConstraintSet};
pub use incidence::{
    ConstraintArity, ConstraintEdges, ConstraintIncidence, ConstraintLayoutError,
    ConstraintParticleCount, ParticleIndex,
};
pub use particles::{
    ParticlePositionRecord, ParticlePositions, ParticleVelocities, ParticleVelocityRecord,
    Particles,
};
pub use solver::{Solver, SolverSettings, SubstepHook};

pub mod dynamics;
pub use dynamics::*;

#[cfg(test)]
mod tests;
