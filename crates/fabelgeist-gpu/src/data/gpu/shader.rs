pub use error::ShaderError;

use super::buffer::{BufferByteLength, BufferByteOffset};

mod error;

pub type ShaderResult<T> = std::result::Result<T, ShaderError>;

#[derive(Debug, Clone)]
pub struct UniformMember {
    pub name: String,
    pub offset: BufferByteOffset,
    pub size: BufferByteLength,
}

#[derive(Debug, Clone)]
pub struct TextureBinding {
    pub name: String,
    pub binding: u32,
    pub format: Option<wgpu::TextureFormat>,
    pub dimension: wgpu::TextureViewDimension,
}

#[derive(Debug, Clone)]
pub struct BufferBinding {
    pub name: String,
    pub binding: u32,
    pub ty: wgpu::BufferBindingType,
}

#[derive(Debug, Clone)]
pub struct SamplerBinding {
    pub name: String,
    pub binding: u32,
}

#[derive(Debug, Clone, Default)]
pub struct BindGroupReflection {
    pub index: u32,
    pub uniform_members: Vec<UniformMember>,
    pub uniform_buffer_size: BufferByteLength,
    pub uniform_binding: Option<u32>,
    pub texture_bindings: Vec<TextureBinding>,
    pub sampler_bindings: Vec<SamplerBinding>,
    pub buffer_bindings: Vec<BufferBinding>,
}

#[derive(Debug, Clone)]
pub struct ReflectionData {
    pub bind_groups: Vec<BindGroupReflection>,
    pub fragment_entry_point: String,
    pub vertex_entry_point: String,
}

/// The language selected at the shader source boundary.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShaderLanguage {
    Glsl,
    Wgsl,
}

pub fn detect_from_code(code: &str) -> ShaderLanguage {
    if code.contains("#version") {
        ShaderLanguage::Glsl
    } else {
        ShaderLanguage::Wgsl
    }
}

pub fn parse_naga(code: &str, stage: wgpu::naga::ShaderStage) -> ShaderResult<wgpu::naga::Module> {
    match detect_from_code(code) {
        ShaderLanguage::Glsl => {
            let mut frontend = wgpu::naga::front::glsl::Frontend::default();
            frontend
                .parse(
                    &wgpu::naga::front::glsl::Options {
                        stage,
                        defines: Default::default(),
                    },
                    code,
                )
                .map_err(|cause| ShaderError::GlslParse {
                    diagnostic: cause.emit_to_string(code),
                    cause: std::sync::Arc::new(cause),
                })
        }
        ShaderLanguage::Wgsl => {
            wgpu::naga::front::wgsl::parse_str(code).map_err(|cause| ShaderError::WgslParse {
                diagnostic: cause.emit_to_string(code),
                cause: std::sync::Arc::new(cause),
            })
        }
    }
}

/// Validate parsed source using the same policy for module and layout admission.
pub(crate) fn validate_naga(
    module: &wgpu::naga::Module,
    code: &str,
) -> ShaderResult<wgpu::naga::valid::ModuleInfo> {
    wgpu::naga::valid::Validator::new(
        wgpu::naga::valid::ValidationFlags::all(),
        wgpu::naga::valid::Capabilities::all(),
    )
    .validate(module)
    .map_err(|cause| ShaderError::Validation {
        diagnostic: cause.emit_to_string(code),
        cause: std::sync::Arc::new(cause),
    })
}
