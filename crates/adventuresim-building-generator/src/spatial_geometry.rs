//! Metre geometry keeps its coordinate frame and arithmetic role at API boundaries.
//!
//! Native vectors belong to the arithmetic inside these owners and to explicit
//! mesh, serialization and framework adapters. Coordinate and displacement
//! admission requires finite values and permits signed coordinates and zero
//! displacement. Direction and rotation admission also checks normalization.

pub use fabelgeist_geometry::{
    Area, CoordinateAxis, CuboidDimensions, DirectionError, GeometryError, GeometryResult,
    GeometryRole, PositiveLength, SignedLength,
};

pub use bounds::SpatialBounds;
pub use clearance::ClearanceVolume;
pub use coordinates::{Architectural, Displacement, Elevation, GeometryFrame, Position};
pub use dimensions::{LeafDimensions, PlanDimensions, PlanExtents};
pub use orientation::{PlanDirection, Radians, RigidRotation, SpatialDirection};

mod bounds;
mod clearance;
mod coordinates;
mod dimensions;
mod error;
mod orientation;

#[cfg(test)]
mod admission_tests;
#[cfg(test)]
mod tests;
