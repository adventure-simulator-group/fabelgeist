pub use error::ShaderError;

mod error;

pub type ShaderResult<T> = std::result::Result<T, ShaderError>;

#[derive(Debug, Clone)]
pub struct UniformMember {
    pub name: String,
    pub offset: u32,
    pub size: u32,
    pub shape: UniformValueShape,
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
    pub uniform_buffer_size: u32,
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

/// Declared host value roles supported by the named uniform packers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UniformValueShape {
    Float,
    /// Native 32-bit integer representation, including the existing i32 adapter.
    IntegerWord,
    Vec2,
    Vec3,
    Vec4,
    Mat2,
    Mat3,
    Mat4,
}

impl UniformValueShape {
    pub fn accepts(self, value: &crate::data::PassParameter) -> bool {
        use crate::data::PassParameter;
        matches!(
            (self, value),
            (Self::Float, PassParameter::Number(_))
                | (Self::IntegerWord, PassParameter::Unsigned(_))
                | (Self::Vec2, PassParameter::Vec2(_))
                | (Self::Vec3, PassParameter::Vec3(_))
                | (Self::Vec4, PassParameter::Vec4(_))
                | (Self::Mat2, PassParameter::Mat2(_))
                | (Self::Mat3, PassParameter::Mat3(_))
                | (
                    Self::Mat4,
                    PassParameter::Mat4(_) | PassParameter::Transform(_)
                )
        )
    }
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
