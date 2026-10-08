//! Native shader frontend rejection and its diagnostic representation.

use std::{error::Error, fmt};

/// A shader frontend result, before validation or device compilation.
pub type ShaderParseResult<T> = Result<T, ShaderParseError>;

/// A rejection from the WebGPU Shading Language (WGSL) or OpenGL Shading
/// Language (GLSL) frontend, retaining the actual Naga cause.
#[derive(Debug)]
pub enum ShaderParseError {
    Wgsl {
        /// Native source-excerpt diagnostic, rendered while source is borrowed.
        diagnostic: String,
        cause: Box<wgpu::naga::front::wgsl::ParseError>,
    },
    Glsl {
        cause: Box<wgpu::naga::front::glsl::ParseErrors>,
    },
}

impl ShaderParseError {
    // Native frontend admission: the formatter needs source text for excerpts;
    // the owned native cause itself does not borrow that source.
    pub(super) fn from_wgsl(cause: wgpu::naga::front::wgsl::ParseError, source: &str) -> Self {
        let diagnostic = cause.emit_to_string(source);
        Self::Wgsl {
            diagnostic,
            cause: Box::new(cause),
        }
    }

    pub(super) fn from_glsl(cause: wgpu::naga::front::glsl::ParseErrors) -> Self {
        Self::Glsl {
            cause: Box::new(cause),
        }
    }
}

impl fmt::Display for ShaderParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Wgsl { diagnostic, .. } => write!(formatter, "WGSL Parse Error: {diagnostic}"),
            Self::Glsl { cause } => write!(formatter, "GLSL Parse Error: {cause:?}"),
        }
    }
}

impl Error for ShaderParseError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Wgsl { cause, .. } => Some(cause.as_ref()),
            Self::Glsl { cause } => Some(cause.as_ref()),
        }
    }
}
