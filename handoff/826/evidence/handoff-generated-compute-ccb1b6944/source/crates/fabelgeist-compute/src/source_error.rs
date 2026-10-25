//! Local admission of generated algorithm source and its resource roles.

use fabelgeist_gpu::data::gpu::resource::ResourceType;
use fabelgeist_gpu::data::gpu::signature::CoordinateShape;
use std::{error::Error, fmt};

pub type GeneratedProgramResult<T> = Result<T, GeneratedProgramError>;

/// The reserved source function an algorithm requires.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GeneratedFunction {
    Map,
    Gather,
    Scatter,
    Stream,
    Stencil,
}

/// A built-in fluid recipe rather than an authored source-function protocol.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FluidOperation {
    Advect,
    Gradient,
    Divergence,
}

/// The native binding whose kind or coordinates failed algorithm admission.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GeneratedBinding {
    Input,
    Output,
    Velocity,
    Quantity,
    Pressure,
}

/// A local source or resource-policy rejection in a generated algorithm.
/// Regex, coordinate, texture-value, compiler and device errors remain separate.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GeneratedProgramError {
    MissingFunction {
        function: GeneratedFunction,
    },
    MissingReturn {
        function: GeneratedFunction,
    },
    InvalidParameter {
        function: GeneratedFunction,
        declaration: String,
    },
    DuplicateParameter {
        function: GeneratedFunction,
        name: String,
    },
    MissingScatterValue,
    InvalidStreamOffset {
        native_type: String,
    },
    MultipleNeighborhoods {
        count: usize,
    },
    MultipleCenters {
        count: usize,
    },
    InvalidNeighborhoodType {
        parameter: String,
        native_type: String,
    },
    UnsupportedNeighborhoodType {
        parameter: String,
        native_type: String,
    },
    UnsupportedCoordinate {
        function: GeneratedFunction,
        shape: CoordinateShape,
        binding: GeneratedBinding,
        actual: ResourceType,
    },
    UnsupportedFluidResource {
        operation: FluidOperation,
        binding: GeneratedBinding,
        actual: ResourceType,
    },
    ExpectedTexture {
        operation: FluidOperation,
        binding: GeneratedBinding,
        actual: ResourceType,
    },
}

impl GeneratedFunction {
    pub fn name(self) -> &'static str {
        match self {
            Self::Map => "map",
            Self::Gather => "gather",
            Self::Scatter => "scatter",
            Self::Stream => "stream",
            Self::Stencil => "stencil",
        }
    }

    fn title(self) -> &'static str {
        match self {
            Self::Map => "Map",
            Self::Gather => "Gather",
            Self::Scatter => "Scatter",
            Self::Stream => "Stream",
            Self::Stencil => "Stencil",
        }
    }

    fn unsupported_coordinate_message(self, shape: CoordinateShape) -> &'static str {
        match (self, shape) {
            (Self::Map, CoordinateShape::Plane) => "vec2 index only supported for Textures",
            (_, CoordinateShape::Plane) => "vec2 index not supported for Buffer",
            (_, CoordinateShape::Volume) => "vec3 index only supported for Texture3d",
            (_, CoordinateShape::Scalar) => "scalar index is not supported for this resource",
        }
    }
}

impl FluidOperation {
    fn title(self) -> &'static str {
        match self {
            Self::Advect => "Advect",
            Self::Gradient => "Gradient",
            Self::Divergence => "Divergence",
        }
    }
}

impl GeneratedBinding {
    fn title(self) -> &'static str {
        match self {
            Self::Input => "Input",
            Self::Output => "Output",
            Self::Velocity => "Velocity",
            Self::Quantity => "Quantity",
            Self::Pressure => "Pressure",
        }
    }
}

impl fmt::Display for GeneratedProgramError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingFunction {
                function: GeneratedFunction::Stream,
            } => formatter
                .write_str("Compute shader must define a 'stream' function: fn stream(...)"),
            Self::MissingFunction { function } => {
                let shader = if *function == GeneratedFunction::Stencil {
                    "Stencil"
                } else {
                    "Compute"
                };
                write!(
                    formatter,
                    "{shader} shader must define a '{}' function",
                    function.name()
                )
            }
            Self::MissingReturn { function } => write!(
                formatter,
                "{} function must return a type.",
                function.title()
            ),
            Self::InvalidParameter {
                function,
                declaration,
            } => write!(
                formatter,
                "Invalid {} parameter format: {declaration}",
                function.name()
            ),
            Self::DuplicateParameter { function, name } => {
                write!(formatter, "Duplicate {} parameter: {name}", function.name())
            }
            Self::MissingScatterValue => formatter
                .write_str("Scatter function must take a value parameter from the input domain."),
            Self::InvalidStreamOffset { .. } => formatter.write_str("Offset must be u32"),
            Self::MultipleNeighborhoods { .. } => {
                formatter.write_str("Stencil accepts one neighborhood parameter")
            }
            Self::MultipleCenters { .. } => {
                formatter.write_str("Stencil accepts one explicit center or val parameter")
            }
            Self::InvalidNeighborhoodType { native_type, .. } => {
                write!(formatter, "Invalid neighborhood type: {native_type}")
            }
            Self::UnsupportedNeighborhoodType { native_type, .. } => {
                write!(formatter, "Unsupported neighborhood type: {native_type}")
            }
            Self::UnsupportedCoordinate {
                function, shape, ..
            } => formatter.write_str(function.unsupported_coordinate_message(*shape)),
            Self::UnsupportedFluidResource { operation, .. } => write!(
                formatter,
                "{} only supports Texture2d or Texture3d",
                operation.title()
            ),
            Self::ExpectedTexture { binding, .. } => {
                write!(formatter, "{} must be a texture", binding.title())
            }
        }
    }
}

impl Error for GeneratedProgramError {}
