//! Provider failures retain source, stage and the original structured cause.
use super::ShaderSource;

#[derive(Debug)]
pub enum ShaderParseError {
    Wgsl {
        source: ShaderSource,
        stage: wgpu::naga::ShaderStage,
        cause: Box<wgpu::naga::front::wgsl::ParseError>,
    },
    Glsl {
        source: ShaderSource,
        stage: wgpu::naga::ShaderStage,
        cause: Box<wgpu::naga::front::glsl::ParseErrors>,
    },
}
impl std::fmt::Display for ShaderParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Wgsl { source, cause, .. } => write!(
                f,
                "WGSL Parse Error: {}",
                cause.emit_to_string(<&str>::from(source))
            ),
            Self::Glsl { cause, .. } => write!(f, "GLSL Parse Error: {cause:?}"),
        }
    }
}
impl std::error::Error for ShaderParseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Wgsl { cause, .. } => Some(cause.as_ref()),
            Self::Glsl { cause, .. } => Some(cause.as_ref()),
        }
    }
}
