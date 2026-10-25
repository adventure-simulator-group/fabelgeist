//! Admission of actual neighborhood axes at the signed WGSL address boundary.

use super::{NeighborhoodDimension, NeighborhoodParameter};
use crate::prelude::*;
use crate::stencil::signature::StencilSignature;
use fabelgeist_gpu::data::gpu::resource::GpuResource;
use fabelgeist_gpu::data::gpu::signature::CoordinateShape;
use fabelgeist_gpu::prelude::BufferElementCount;

pub type NeighborhoodLayoutResult<T> = std::result::Result<T, NeighborhoodLayoutError>;

/// The existing fallback side for a neighborhood without a declared shape.
pub(super) const DEFAULT_NEIGHBORHOOD_SIDE: u32 = 64;

/// Checked positive extents in the native shader's x, y, z and fourth axes.
/// Unused higher axes are one. These values prove signed sample arithmetic;
/// they are independent of the workgroup launch shape.
struct NeighborhoodLayout {
    axes: [u64; 4],
    population: u64,
}

/// The native sample axis whose extent cannot satisfy the generated program.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NeighborhoodAxis {
    X,
    Y,
    Z,
    Fourth,
}

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

impl NeighborhoodLayout {
    fn admit(axes: [u64; 4], boundary_mode: BoundaryMode) -> NeighborhoodLayoutResult<Self> {
        let mut population = 1u64;
        for (axis, extent) in NeighborhoodAxis::ALL.into_iter().zip(axes) {
            if extent == 0 {
                return Err(NeighborhoodLayoutError::EmptyAxis { axis });
            }
            // Wrap evaluates (coordinate % extent + extent) before its final
            // remainder. A right-hand neighbor may first equal extent.
            let largest = match boundary_mode {
                BoundaryMode::Clamp => extent,
                BoundaryMode::Wrap => extent
                    .checked_mul(2)
                    .and_then(|sum| sum.checked_sub(1))
                    .ok_or(NeighborhoodLayoutError::PopulationOverflow)?,
            };
            if largest > i32::MAX as u64 {
                return Err(NeighborhoodLayoutError::SignedAxis {
                    axis,
                    extent,
                    boundary_mode,
                });
            }
            population = population
                .checked_mul(extent)
                .ok_or(NeighborhoodLayoutError::PopulationOverflow)?;
        }
        Ok(Self { axes, population })
    }

    fn cover_output(
        &self,
        dimension: NeighborhoodDimension,
        output: &GpuResource,
        output_axes: [u64; 4],
    ) -> NeighborhoodLayoutResult<()> {
        if matches!(output, GpuResource::Buffer(_)) || dimension == NeighborhoodDimension::Four {
            let required = output_axes
                .into_iter()
                .try_fold(1u64, |product, axis| product.checked_mul(axis))
                .ok_or(NeighborhoodLayoutError::PopulationOverflow)?;
            if required > self.population {
                return Err(NeighborhoodLayoutError::OutputPopulation {
                    required,
                    available: self.population,
                });
            }
            return Ok(());
        }
        for ((axis, required), available) in NeighborhoodAxis::ALL
            .into_iter()
            .zip(output_axes)
            .zip(self.axes)
            .take(dimension.axis_count())
        {
            if required > available {
                return Err(NeighborhoodLayoutError::OutputCoverage {
                    axis,
                    required,
                    available,
                });
            }
        }
        Ok(())
    }

    fn cover_input(
        &self,
        input: &GpuResource,
        input_axes: [u64; 4],
    ) -> NeighborhoodLayoutResult<()> {
        if let GpuResource::Buffer(_) = input {
            // Every signed flattening intermediate is nonnegative and at most
            // the last record index in this complete grid.
            if self.population - 1 > i32::MAX as u64 {
                return Err(NeighborhoodLayoutError::NativeIndexOverflow {
                    population: self.population,
                });
            }
            let required = BufferElementCount::from(self.population as u32);
            let available = BufferElementCount::from(input_axes[0] as u32);
            if self.population > input_axes[0] {
                return Err(NeighborhoodLayoutError::InputCapacity {
                    required,
                    available,
                });
            }
            return Ok(());
        }
        let projected_axes = match input {
            GpuResource::Texture2d(_) => 2,
            GpuResource::Texture3d(_) => 3,
            GpuResource::Buffer(_) => 0,
        };
        for ((axis, required), available) in NeighborhoodAxis::ALL
            .into_iter()
            .zip(self.axes)
            .zip(input_axes)
            .take(projected_axes)
        {
            if required > available {
                return Err(NeighborhoodLayoutError::InputCoverage {
                    axis,
                    required,
                    available,
                });
            }
        }
        Ok(())
    }
}

impl NeighborhoodAxis {
    const ALL: [Self; 4] = [Self::X, Self::Y, Self::Z, Self::Fourth];
}

impl NeighborhoodDimension {
    fn axis_count(self) -> usize {
        match self {
            Self::One => 1,
            Self::Two => 2,
            Self::Three => 3,
            Self::Four => 4,
        }
    }
}

impl NeighborhoodParameter {
    pub(in crate::stencil) fn admit_resources(
        &self,
        signature: &StencilSignature,
        boundary_mode: BoundaryMode,
        program: &ResourcePipeline,
        input: &GpuResource,
        output: &GpuResource,
    ) -> Result<()> {
        let output_axes = resource_axes(output, program, DispatchResource::Output)?;
        if output_axes.contains(&0) {
            return Ok(());
        }
        let input_axes = resource_axes(input, program, DispatchResource::Input)?;
        let axes = declared_axes(
            self.dimension(),
            signature,
            input,
            output,
            input_axes,
            output_axes,
        )?;
        let layout = NeighborhoodLayout::admit(axes, boundary_mode)?;
        let divisor = match (self.dimension(), output) {
            (NeighborhoodDimension::Four, _) => population([axes[0], axes[1], axes[2], 1])?,
            (NeighborhoodDimension::Three, GpuResource::Buffer(_)) => {
                population([axes[0], axes[1], 1, 1])?
            }
            _ => 1,
        };
        if divisor > u32::MAX as u64 {
            return Err(NeighborhoodLayoutError::FlatDivisorOverflow { divisor }.into());
        }
        layout.cover_output(self.dimension(), output, output_axes)?;
        layout.cover_input(input, input_axes)?;
        Ok(())
    }
}

pub(super) fn admit_size(
    dimension: NeighborhoodDimension,
    signature: &StencilSignature,
    input: &ResourceDescriptor,
) -> NeighborhoodLayoutResult<()> {
    if let Some(parameter) = signature.size() {
        let shape = parameter.shape();
        if (dimension == NeighborhoodDimension::Four && shape == CoordinateShape::Plane)
            || (dimension == NeighborhoodDimension::One
                && matches!(input, ResourceDescriptor::Buffer(_))
                && shape != CoordinateShape::Scalar)
        {
            return Err(NeighborhoodLayoutError::UnsupportedSize { dimension, shape });
        }
    }
    Ok(())
}

fn resource_axes(
    resource: &GpuResource,
    program: &ResourcePipeline,
    binding: DispatchResource,
) -> Result<[u64; 4]> {
    match resource {
        GpuResource::Buffer(buffer) => {
            let stride = program.storage_stride_for(binding)?;
            let count = BufferElementCount::from_extent(buffer.size, stride)?;
            Ok([usize::from(count) as u64, 1, 1, 1])
        }
        GpuResource::Texture2d(texture) => {
            let extent = texture.extent();
            Ok([extent.width.into(), extent.height.into(), 1, 1])
        }
        GpuResource::Texture3d(texture) => {
            let extent = texture.extent();
            Ok([
                extent.width.into(),
                extent.height.into(),
                extent.depth_or_array_layers.into(),
                1,
            ])
        }
    }
}

fn declared_axes(
    dimension: NeighborhoodDimension,
    signature: &StencilSignature,
    input: &GpuResource,
    output: &GpuResource,
    input_axes: [u64; 4],
    output_axes: [u64; 4],
) -> NeighborhoodLayoutResult<[u64; 4]> {
    let size = signature.size().map(|parameter| parameter.shape());
    if dimension == NeighborhoodDimension::Four {
        return match size {
            Some(CoordinateShape::Volume) => {
                Ok([output_axes[0], output_axes[1], output_axes[2], 1])
            }
            Some(CoordinateShape::Scalar) => Ok([population(output_axes)?, 1, 1, 1]),
            Some(shape) => Err(NeighborhoodLayoutError::UnsupportedSize { dimension, shape }),
            None => Ok([DEFAULT_NEIGHBORHOOD_SIDE.into(); 4]),
        };
    }
    if !matches!(input, GpuResource::Buffer(_)) {
        let mut axes = input_axes;
        for axis in axes.iter_mut().skip(dimension.axis_count()) {
            *axis = 1;
        }
        return Ok(axes);
    }
    if let Some(shape) = size {
        return match (dimension, shape) {
            (NeighborhoodDimension::One, CoordinateShape::Plane | CoordinateShape::Volume) => {
                Err(NeighborhoodLayoutError::UnsupportedSize { dimension, shape })
            }
            (_, CoordinateShape::Scalar) => Ok([population(output_axes)?, 1, 1, 1]),
            (NeighborhoodDimension::Two, _) | (_, CoordinateShape::Plane) => {
                Ok([output_axes[0], output_axes[1], 1, 1])
            }
            (_, CoordinateShape::Volume) => Ok([output_axes[0], output_axes[1], output_axes[2], 1]),
        };
    }
    Ok(match dimension {
        NeighborhoodDimension::One => [input_axes[0], 1, 1, 1],
        NeighborhoodDimension::Two if !matches!(output, GpuResource::Buffer(_)) => {
            [output_axes[0], output_axes[1], 1, 1]
        }
        NeighborhoodDimension::Three if matches!(output, GpuResource::Texture3d(_)) => output_axes,
        NeighborhoodDimension::Two => [
            DEFAULT_NEIGHBORHOOD_SIDE.into(),
            DEFAULT_NEIGHBORHOOD_SIDE.into(),
            1,
            1,
        ],
        NeighborhoodDimension::Three => [
            DEFAULT_NEIGHBORHOOD_SIDE.into(),
            DEFAULT_NEIGHBORHOOD_SIDE.into(),
            DEFAULT_NEIGHBORHOOD_SIDE.into(),
            1,
        ],
        NeighborhoodDimension::Four => [DEFAULT_NEIGHBORHOOD_SIDE.into(); 4],
    })
}

fn population(axes: [u64; 4]) -> NeighborhoodLayoutResult<u64> {
    axes.into_iter()
        .try_fold(1u64, |product, axis| product.checked_mul(axis))
        .ok_or(NeighborhoodLayoutError::PopulationOverflow)
}
