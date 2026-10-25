//! Typed local failures at the neighborhood address-admission boundary.

use super::NeighborhoodAxis;
use crate::{BoundaryMode, stencil::NeighborhoodDimension};
use fabelgeist_gpu::data::gpu::signature::CoordinateShape;
use fabelgeist_gpu::prelude::BufferElementCount;

pub type NeighborhoodLayoutResult<T> = std::result::Result<T, NeighborhoodLayoutError>;

/// Actual resources do not fit the generated neighborhood's address contract.
#[derive(Clone, Debug, thiserror::Error)]
pub enum NeighborhoodLayoutError {
    #[error("neighborhood axis {axis:?} is empty")]
    EmptyAxis { axis: NeighborhoodAxis },
    #[error(
        "neighborhood axis {axis:?} extent {extent} exceeds {boundary_mode:?} signed arithmetic"
    )]
    SignedAxis {
        axis: NeighborhoodAxis,
        extent: u64,
        boundary_mode: BoundaryMode,
    },
    #[error("neighborhood population overflows its native address calculation")]
    PopulationOverflow,
    #[error("neighborhood population {population} exceeds signed native indexing")]
    NativeIndexOverflow { population: u64 },
    #[error("output axis {axis:?} requires {required} samples, neighborhood supplies {available}")]
    OutputCoverage {
        axis: NeighborhoodAxis,
        required: u64,
        available: u64,
    },
    #[error("input axis {axis:?} has {available} samples, neighborhood requires {required}")]
    InputCoverage {
        axis: NeighborhoodAxis,
        required: u64,
        available: u64,
    },
    #[error("output coordinate axis {axis:?} extent {extent} exceeds signed neighbor arithmetic")]
    SignedCoordinate { axis: NeighborhoodAxis, extent: u64 },
    #[error("flat neighborhood coordinate divisor {divisor} exceeds native unsigned arithmetic")]
    FlatDivisorOverflow { divisor: u64 },
    #[error("output population {required} exceeds neighborhood population {available}")]
    OutputPopulation { required: u64, available: u64 },
    #[error("neighborhood needs {required:?} input records, buffer has {available:?}")]
    InputCapacity {
        required: BufferElementCount,
        available: BufferElementCount,
    },
    #[error("{dimension:?} neighborhood does not support {shape:?} size coordinates")]
    UnsupportedSize {
        dimension: NeighborhoodDimension,
        shape: CoordinateShape,
    },
}
