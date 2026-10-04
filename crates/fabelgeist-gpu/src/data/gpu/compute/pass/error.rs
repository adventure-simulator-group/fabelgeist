//! Resource admission failures retain shader coordinates and provider causes.
use crate::data::{
    BindGroupIndex, BindingIndex, ShaderBindingName, TextureViewError, UniformPackingError,
};
use std::sync::Arc;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BindingKind {
    Buffer,
    Texture,
    Sampler,
}
impl std::fmt::Display for BindingKind {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Buffer => write!(formatter, "buffer"),
            Self::Texture => write!(formatter, "texture"),
            Self::Sampler => write!(formatter, "sampler"),
        }
    }
}
#[derive(Debug)]
pub enum ComputePassError {
    MissingPipeline,
    InvalidPipeline(Arc<wgpu::Error>),
    MissingReflection,
    MissingLayout(BindGroupIndex),
    MissingParameter {
        name: ShaderBindingName,
        binding: BindingIndex,
        kind: BindingKind,
    },
    WrongParameter {
        name: ShaderBindingName,
        expected: BindingKind,
    },
    TextureFormat {
        name: ShaderBindingName,
        expected: wgpu::TextureFormat,
        actual: wgpu::TextureFormat,
    },
    TextureDimension {
        name: ShaderBindingName,
        expected: wgpu::TextureViewDimension,
        actual: wgpu::TextureViewDimension,
    },
    MissingTextureView(ShaderBindingName),
    MissingSampler(ShaderBindingName),
    Uniform(UniformPackingError),
    View {
        name: ShaderBindingName,
        cause: TextureViewError,
    },
}
impl std::fmt::Display for ComputePassError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingPipeline => write!(
                f,
                "ComputePass: ComputePipeline missing actual WGPU pipeline."
            ),
            Self::InvalidPipeline(cause) => write!(
                f,
                "ComputePass: Cannot use invalid ComputePipeline: {cause}"
            ),
            Self::MissingReflection => {
                write!(f, "ComputePass: ComputePipeline missing ReflectionData")
            }
            Self::MissingLayout(index) => write!(
                f,
                "ComputePass: Bind group layout index {index} missing in pipeline"
            ),
            Self::MissingParameter {
                name,
                binding,
                kind,
            } => write!(
                f,
                "ComputePass: Parameter '{name}' ({kind} binding {binding}) not found"
            ),
            Self::WrongParameter { name, expected } => {
                let kinds = match expected {
                    BindingKind::Buffer => "a Buffer",
                    BindingKind::Texture => "a Texture2d, Texture3d, TextureCube, or TextureView",
                    BindingKind::Sampler => "a Sampler",
                };
                write!(f, "ComputePass: Parameter '{name}' is not {kinds}")
            }
            Self::TextureFormat {
                name,
                expected,
                actual,
            } => write!(
                f,
                "ComputePass: Texture '{name}' format mismatch. Expected {expected:?}, got {actual:?}"
            ),
            Self::TextureDimension {
                name,
                expected,
                actual,
            } => write!(
                f,
                "ComputePass: Texture '{name}' dimension mismatch. Expected {expected:?}, got {actual:?}"
            ),
            Self::MissingTextureView(name) => {
                write!(f, "ComputePass: Texture '{name}' has no view")
            }
            Self::MissingSampler(name) => {
                write!(f, "ComputePass: Sampler '{name}' has no WGPU resource")
            }
            Self::Uniform(cause) => cause.fmt(f),
            Self::View { cause, .. } => cause.fmt(f),
        }
    }
}
impl std::error::Error for ComputePassError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::InvalidPipeline(cause) => Some(cause.as_ref()),
            Self::Uniform(cause) => Some(cause),
            Self::View { cause, .. } => Some(cause),
            _ => None,
        }
    }
}
