use super::buffer::{BufferByteLength, BufferByteOffset};
use super::parameters::PassParameterName;
#[derive(Debug, Clone)]
pub struct UniformMember {
    pub name: PassParameterName,
    pub offset: BufferByteOffset,
    pub size: BufferByteLength,
}

#[derive(Debug, Clone)]
pub struct TextureBinding {
    pub name: ShaderBindingName,
    pub binding: BindingIndex,
    pub format: Option<wgpu::TextureFormat>,
    pub dimension: wgpu::TextureViewDimension,
}

#[derive(Debug, Clone)]
pub struct BufferBinding {
    pub name: ShaderBindingName,
    pub binding: BindingIndex,
    pub ty: wgpu::BufferBindingType,
}

#[derive(Debug, Clone, Default)]
pub struct BindGroupReflection {
    pub index: BindGroupIndex,
    pub uniform_members: Vec<UniformMember>,
    pub uniform_buffer_size: BufferByteLength,
    pub uniform_binding: Option<BindingIndex>,
    pub texture_bindings: Vec<TextureBinding>,
    pub sampler_bindings: Vec<SamplerBinding>,
    pub buffer_bindings: Vec<BufferBinding>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SamplerBinding {
    pub name: ShaderBindingName,
    pub binding: BindingIndex,
}

#[derive(Debug, Clone)]
pub struct ReflectionData {
    pub bind_groups: Vec<BindGroupReflection>,
    pub compute_entry_point: ShaderEntryPoint,
}

mod error;
mod source;
pub use error::ShaderParseError;
pub use source::{
    ShaderBindingMarker, ShaderBindingName, ShaderEntryPoint, ShaderLanguage, ShaderSource,
};

mod binding;
pub use binding::{BindGroupIndex, BindingIndex};
