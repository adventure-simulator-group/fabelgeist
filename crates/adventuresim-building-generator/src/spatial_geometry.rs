//! Metre geometry keeps its coordinate frame and arithmetic role at API boundaries.
//!
//! Native vectors belong to the arithmetic inside these owners and to explicit
//! mesh, serialization and framework adapters. Coordinate and displacement
//! admission requires finite values and permits signed coordinates and zero
//! displacement. Direction and rotation admission also checks normalization.

mod bounds;
mod clearance;
pub use clearance::ClearanceVolume;
mod coordinates;
mod dimensions;
mod error;
mod length;
mod measurement;
pub use measurement::{Area, SignedLength};
mod orientation;
pub use length::PositiveLength;

pub use bounds::SpatialBounds;
pub use coordinates::{Architectural, Displacement, Elevation, GeometryFrame, Position};
pub use dimensions::{CuboidDimensions, LeafDimensions, PlanDimensions, PlanExtents};
pub use error::{CoordinateAxis, GeometryError, GeometryResult, GeometryRole};
pub use orientation::{PlanDirection, Radians, RigidRotation, SpatialDirection};

#[cfg(test)]
mod admission_tests;
#[cfg(test)]
mod tests;
