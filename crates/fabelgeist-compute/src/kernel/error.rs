//! Kernel build stages retain provider causes instead of formatted messages.
use fabelgeist_gpu::prelude::{
    ComputePipelineError, ComputeShaderError, ShaderEntryPoint, ShaderParseError, ShaderSource,
    WorkgroupShapeError,
};

#[derive(Debug)]
pub enum KernelBuildError {
    Parse(ShaderParseError),
    MissingComputeEntryPoint,
    ZeroWorkgroupDimension {
        entry: ShaderEntryPoint,
        cause: WorkgroupShapeError,
    },
    Shader(ComputeShaderError),
    Pipeline(ComputePipelineError),
}
impl std::fmt::Display for KernelBuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Parse(cause) => cause.fmt(f),
            Self::MissingComputeEntryPoint => f.write_str("Kernel: no compute entry point"),
            Self::ZeroWorkgroupDimension { entry, cause } => write!(f, "Kernel `{entry}`: {cause}"),
            Self::Shader(cause) => cause.fmt(f),
            Self::Pipeline(cause) => cause.fmt(f),
        }
    }
}
impl std::error::Error for KernelBuildError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Parse(cause) => Some(cause),
            Self::Shader(cause) => Some(cause),
            Self::Pipeline(cause) => Some(cause),
            Self::ZeroWorkgroupDimension { cause, .. } => Some(cause),
            Self::MissingComputeEntryPoint => None,
        }
    }
}
#[derive(Debug)]
pub enum KernelCacheError {
    ReadPoisoned,
    WritePoisoned,
    Compile {
        source: ShaderSource,
        cause: KernelBuildError,
    },
}
impl std::fmt::Display for KernelCacheError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ReadPoisoned => f.write_str("Failed to read kernel cache"),
            Self::WritePoisoned => f.write_str("Failed to write kernel cache"),
            Self::Compile { cause, .. } => cause.fmt(f),
        }
    }
}
impl std::error::Error for KernelCacheError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Compile { cause, .. } => Some(cause),
            Self::ReadPoisoned | Self::WritePoisoned => None,
        }
    }
}
