use anyhow::anyhow;
pub use language::ShaderLanguage;
pub use source::ShaderSource;

mod language;
mod source;

#[derive(Debug, Clone)]
pub struct UniformMember {
    pub name: String,
    pub offset: u32,
    pub size: u32,
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

#[derive(Debug, Clone, Default)]
pub struct BindGroupReflection {
    pub index: u32,
    pub uniform_members: Vec<UniformMember>,
    pub uniform_buffer_size: u32,
    pub uniform_binding: Option<u32>,
    pub texture_bindings: Vec<TextureBinding>,
    pub sampler_bindings: Vec<(String, u32)>, // name, binding index
    pub buffer_bindings: Vec<BufferBinding>,
}

#[derive(Debug, Clone)]
pub struct ReflectionData {
    pub bind_groups: Vec<BindGroupReflection>,
    pub fragment_entry_point: String,
    pub vertex_entry_point: String,
}

pub fn parse_naga(
    code: &ShaderSource<'_>,
    stage: wgpu::naga::ShaderStage,
) -> anyhow::Result<wgpu::naga::Module> {
    match code.language() {
        ShaderLanguage::Glsl => {
            let mut frontend = wgpu::naga::front::glsl::Frontend::default();
            frontend
                .parse(
                    &wgpu::naga::front::glsl::Options {
                        stage,
                        defines: Default::default(),
                    },
                    code.as_str(),
                )
                .map_err(|e| anyhow!("GLSL Parse Error: {:?}", e))
        }
        ShaderLanguage::Wgsl => wgpu::naga::front::wgsl::parse_str(code.as_str()).map_err(|e| {
            let message = e.emit_to_string(code.as_str());
            anyhow!("WGSL Parse Error: {}", message)
        }),
    }
}
