use crate::data::gpu::shader::ShaderParseError;
use std::sync::Arc;
#[derive(Debug)]
pub enum ComputePipelineError {
    Shader(super::ComputeShaderError),
    CachePoisoned,
    MissingModule,
    Interface(ShaderParseError),
    Reflection(ShaderParseError),
    MissingComputeEntryPoint,
    Device(Arc<wgpu::Error>),
}
impl std::fmt::Display for ComputePipelineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Shader(cause) => std::fmt::Display::fmt(cause, f),
            Self::CachePoisoned => f.write_str("Failed to lock pipeline cache"),
            Self::MissingModule => f.write_str("ComputePipeline: Shader Module missing"),
            Self::Interface(cause) => write!(f, "Compute Shader Parse Error: {cause}"),
            Self::Reflection(cause) => {
                write!(f, "Compute Shader Parse Error for Reflection: {cause}")
            }
            Self::MissingComputeEntryPoint => f.write_str("Compute Shader missing entry point"),
            Self::Device(cause) => write!(f, "ComputePipeline Creation Error: {cause}"),
        }
    }
}
impl std::error::Error for ComputePipelineError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Shader(cause) => Some(cause),
            Self::Interface(cause) | Self::Reflection(cause) => Some(cause),
            Self::Device(cause) => Some(cause.as_ref()),
            Self::CachePoisoned | Self::MissingModule | Self::MissingComputeEntryPoint => None,
        }
    }
}
