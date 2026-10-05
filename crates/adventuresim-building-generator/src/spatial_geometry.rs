//! Metre geometry keeps its coordinate frame and arithmetic role at API boundaries.
//!
//! Native vectors belong to the arithmetic inside these owners and to explicit
//! mesh, serialization and framework adapters. Admission never uses a proximity
//! tolerance: finite signed coordinates and zero displacement remain valid.

mod bounds;
mod clearance;
pub use clearance::ClearanceVolume;
mod coordinates;
mod dimensions;
mod error;
mod length;
mod orientation;
pub use length::PositiveLength;

pub use bounds::SpatialBounds;
pub use coordinates::{Architectural, Displacement, Elevation, GeometryFrame, Position};
pub use dimensions::{CuboidDimensions, LeafDimensions, PlanDimensions, PlanExtents};
pub use error::{CoordinateAxis, GeometryError, GeometryRole};
pub use orientation::{PlanDirection, Radians, RigidRotation, SpatialDirection};

#[cfg(test)]
mod admission_tests;
#[cfg(test)]
mod tests;
