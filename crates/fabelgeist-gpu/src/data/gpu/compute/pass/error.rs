use crate::data::gpu::buffer::BufferByteLength;
use std::{error::Error, fmt, sync::Arc};

/// The reflected resource required by a shader binding.
#[derive(Clone, Copy, Debug)]
pub enum ComputeBindingKind {
    Buffer,
    Texture,
    Sampler,
}

/// Failures at the native compute binding and recording boundary.
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
    Native(Arc<wgpu::Error>),
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
            _ => None,
        }
    }
}
