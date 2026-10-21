//! Checked scalar geometry shared by simulation, generation and authoring.
//!
//! Scalars and their portable errors have no mandatory rendering or engine
//! dependency. Native vector/direction adapters remain with their framework;
//! opaque scalar reflection is available through the optional `reflect` feature.

pub use error::{CoordinateAxis, DirectionError, GeometryError, GeometryResult, GeometryRole};
pub use length::PositiveLength;
pub use measurement::{Area, SignedLength};

mod error;
mod length;
mod measurement;
