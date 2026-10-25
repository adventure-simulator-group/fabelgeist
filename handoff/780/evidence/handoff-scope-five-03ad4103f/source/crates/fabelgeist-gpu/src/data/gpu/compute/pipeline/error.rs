use crate::data::gpu::shader::ShaderError;
use crate::globals::{ValidationProgressError, ValidationScopeError};
use std::{error::Error, fmt, sync::Arc};

pub type ComputePipelineResult<T> = Result<T, ComputePipelineError>;

/// Resource payloads that the named pass-parameter interface cannot represent.
#[derive(Clone, Copy, Debug)]
pub enum UnsupportedBindingKind {
    BindingArray,
    ExternalTexture,
    Handle,
    AddressSpace,
}

/// A pipeline's rejection remains available to every clone and cache lookup.
#[derive(Clone, Debug)]
pub enum ComputePipelineError {
    Shader(ShaderError),
    WrongDevice,
    MissingEntryPoint {
        requested: Option<String>,
    },
    NotCompiled,
    Native(Arc<wgpu::Error>),
    Progress(Arc<ValidationProgressError>),
    CachePoisoned,
    BindGroupLimit {
        index: u32,
        limit: u32,
    },
    MultipleUniformBindings {
        group: u32,
        first: u32,
        additional: u32,
    },
    UnsupportedBinding {
        group: u32,
        binding: u32,
        kind: UnsupportedBindingKind,
    },
    UnsupportedImmediate {
        name: String,
    },
}

impl From<ValidationScopeError> for ComputePipelineError {
    fn from(cause: ValidationScopeError) -> Self {
        match cause {
            ValidationScopeError::Native(cause) => Self::Native(Arc::new(cause)),
            ValidationScopeError::Progress(cause) => Self::Progress(Arc::new(cause)),
        }
    }
}

impl From<ShaderError> for ComputePipelineError {
    fn from(cause: ShaderError) -> Self {
        Self::Shader(cause)
    }
}

impl fmt::Display for ComputePipelineError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Shader(cause) => fmt::Display::fmt(cause, formatter),
            Self::WrongDevice => {
                formatter.write_str("ComputePipeline belongs to a different GPU device")
            }
            Self::MissingEntryPoint {
                requested: Some(name),
            } => {
                write!(formatter, "Compute shader missing entry point `{name}`")
            }
            Self::MissingEntryPoint { requested: None } => {
                formatter.write_str("Compute shader missing a compute entry point")
            }
            Self::NotCompiled => {
                formatter.write_str("ComputePass: ComputePipeline missing actual WGPU pipeline")
            }
            Self::Progress(cause) => write!(formatter, "ComputePipeline Creation Error: {cause}"),
            Self::Native(cause) => write!(formatter, "ComputePipeline Creation Error: {cause}"),
            Self::BindGroupLimit { index, limit } => write!(
                formatter,
                "Compute bind group {index} exceeds device group limit {limit}"
            ),
            Self::MultipleUniformBindings {
                group,
                first,
                additional,
            } => write!(
                formatter,
                "Compute group {group} has uniform bindings {first} and {additional}; named parameters support one block per group"
            ),
            Self::UnsupportedBinding {
                group,
                binding,
                kind,
            } => write!(
                formatter,
                "Compute group {group} binding {binding}: {kind:?} is not supported by named pass parameters"
            ),
            Self::UnsupportedImmediate { name } => write!(
                formatter,
                "Compute immediate '{name}' is not supported by named pass parameters"
            ),
            Self::CachePoisoned => formatter.write_str("Failed to lock pipeline cache"),
        }
    }
}

impl Error for ComputePipelineError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Shader(cause) => Some(cause),
            Self::Native(cause) => Some(cause.as_ref()),
            Self::Progress(cause) => Some(cause.as_ref()),
            _ => None,
        }
    }
}
