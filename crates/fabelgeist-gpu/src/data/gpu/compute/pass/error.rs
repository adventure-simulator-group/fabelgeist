use crate::data::gpu::buffer::{BufferByteLength, BufferByteOffset};
use crate::globals::{ValidationProgressError, ValidationScopeError, ValidationScopeResult};
use std::{error::Error, fmt, sync::Arc};

/// The reflected resource required by a shader binding.
#[derive(Clone, Copy, Debug)]
pub enum ComputeBindingKind {
    Buffer,
    Texture,
    Sampler,
    Uniform,
}

/// Failures in compute binding, recording, finalization and submission.
#[derive(Clone, Debug)]
pub enum ComputePassError {
    MissingParameter {
        name: String,
        kind: ComputeBindingKind,
    },
    WrongParameterKind {
        name: String,
        kind: ComputeBindingKind,
    },
    MissingSampler {
        name: String,
    },
    MissingLayout {
        index: u32,
    },
    UniformBufferTooLarge {
        requested: BufferByteLength,
        limit: BufferByteLength,
    },
    UnsupportedUniform {
        name: String,
    },
    UniformHostAddress(std::num::TryFromIntError),
    UniformPackingOverflow,
    UniformPackingOutOfBounds {
        offset: BufferByteOffset,
        size: BufferByteLength,
        capacity: BufferByteLength,
    },
    Native(Arc<wgpu::Error>),
    Progress(Arc<ValidationProgressError>),
    OperationAndObservation {
        operation: Arc<anyhow::Error>,
        observation: Arc<ValidationScopeError>,
    },
}

impl ComputePassError {
    /// Resolve an earlier operation after its observation scope has been drained.
    /// Both failures remain accessible; the operation stays the primary cause.
    pub fn resolve_observation<T>(
        operation: anyhow::Result<T>,
        observed: ValidationScopeResult<()>,
    ) -> anyhow::Result<T> {
        match (operation, observed) {
            (Ok(value), Ok(())) => Ok(value),
            (Err(operation), Ok(())) => Err(operation),
            (Ok(_), Err(observation)) => Err(Self::from(observation).into()),
            (Err(operation), Err(observation)) => Err(Self::OperationAndObservation {
                operation: Arc::new(operation),
                observation: Arc::new(observation),
            }
            .into()),
        }
    }
}

impl From<ValidationScopeError> for ComputePassError {
    fn from(cause: ValidationScopeError) -> Self {
        match cause {
            ValidationScopeError::Native(cause) => Self::Native(Arc::new(cause)),
            ValidationScopeError::Progress(cause) => Self::Progress(Arc::new(cause)),
        }
    }
}

impl fmt::Display for ComputePassError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingParameter { name, kind } => write!(
                formatter,
                "ComputePass: {kind:?} parameter '{name}' missing"
            ),
            Self::WrongParameterKind { name, kind } => write!(
                formatter,
                "ComputePass: parameter '{name}' is not a {kind:?}"
            ),
            Self::MissingSampler { name } => write!(
                formatter,
                "ComputePass: Sampler '{name}' has no WGPU resource"
            ),
            Self::MissingLayout { index } => {
                write!(formatter, "ComputePass: bind group layout {index} missing")
            }
            Self::UniformBufferTooLarge { requested, limit } => write!(
                formatter,
                "ComputePass: uniform buffer needs {requested} bytes, device limit is {limit}"
            ),
            Self::UnsupportedUniform { name } => write!(
                formatter,
                "Kernel: uniform '{name}' has a value the cached dispatch path does not pack"
            ),
            Self::UniformHostAddress(cause) => write!(
                formatter,
                "ComputePass: uniform layout does not fit a host byte address: {cause}"
            ),
            Self::UniformPackingOverflow => {
                formatter.write_str("ComputePass: uniform packing byte range overflow")
            }
            Self::UniformPackingOutOfBounds {
                offset,
                size,
                capacity,
            } => write!(
                formatter,
                "ComputePass: writing {size} uniform bytes at {offset} exceeds a {capacity}-byte block"
            ),
            Self::Progress(cause) => {
                write!(formatter, "ComputePass native validation error: {cause}")
            }
            Self::OperationAndObservation {
                operation,
                observation,
            } => write!(
                formatter,
                "{operation}; additional validation observation failure: {observation}"
            ),
            Self::Native(cause) => {
                write!(formatter, "ComputePass native validation error: {cause}")
            }
        }
    }
}

impl Error for ComputePassError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Native(cause) => Some(cause.as_ref()),
            Self::Progress(cause) => Some(cause.as_ref()),
            Self::OperationAndObservation { operation, .. } => Some(operation.as_ref().as_ref()),
            Self::UniformHostAddress(cause) => Some(cause),
            _ => None,
        }
    }
}
