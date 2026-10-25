use crate::globals::{ValidationProgressError, ValidationScopeError};
use std::{error::Error, fmt, sync::Arc};

/// Shader admission failures retain the SDK cause and any source diagnostic.
#[derive(Clone, Debug)]
pub enum ShaderError {
    WgslParse {
        cause: Arc<wgpu::naga::front::wgsl::ParseError>,
        diagnostic: String,
    },
    GlslParse {
        cause: Arc<wgpu::naga::front::glsl::ParseErrors>,
        diagnostic: String,
    },
    Validation {
        cause: Arc<wgpu::naga::WithSpan<wgpu::naga::valid::ValidationError>>,
        diagnostic: String,
    },
    Translation(Arc<wgpu::naga::back::wgsl::Error>),
    Native(Arc<wgpu::Error>),
    Progress(Arc<ValidationProgressError>),
}

impl From<ValidationScopeError> for ShaderError {
    fn from(cause: ValidationScopeError) -> Self {
        match cause {
            ValidationScopeError::Native(cause) => Self::Native(Arc::new(cause)),
            ValidationScopeError::Progress(cause) => Self::Progress(Arc::new(cause)),
        }
    }
}

impl fmt::Display for ShaderError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WgslParse { diagnostic, .. } => {
                write!(formatter, "WGSL Parse Error: {diagnostic}")
            }
            Self::GlslParse { diagnostic, .. } => {
                write!(formatter, "GLSL Parse Error: {diagnostic}")
            }
            Self::Validation { diagnostic, .. } => {
                write!(formatter, "Compute Shader Validation Error: {diagnostic}")
            }
            Self::Translation(cause) => {
                write!(formatter, "Failed to convert GLSL to WGSL: {cause}")
            }
            Self::Progress(cause) => write!(formatter, "Compute Shader Validation Error: {cause}"),
            Self::Native(cause) => write!(formatter, "Compute Shader Validation Error: {cause}"),
        }
    }
}

impl Error for ShaderError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(match self {
            Self::WgslParse { cause, .. } => cause.as_ref(),
            Self::GlslParse { cause, .. } => cause.as_ref(),
            Self::Validation { cause, .. } => cause.as_ref(),
            Self::Translation(cause) => cause.as_ref(),
            Self::Native(cause) => cause.as_ref(),
            Self::Progress(cause) => cause.as_ref(),
        })
    }
}
