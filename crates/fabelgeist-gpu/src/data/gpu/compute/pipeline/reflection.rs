//! Naga binding metadata is translated once into wgpu layout descriptors.
use super::ComputePipelineError;
use crate::data::gpu::shader::{
    BindGroupIndex, BindGroupReflection, BindingIndex, BufferBinding, ReflectionData,
    SamplerBinding, ShaderBindingName, ShaderEntryPoint, ShaderSource, TextureBinding,
    UniformMember,
};
use crate::data::{BufferByteLength, BufferByteOffset, PassParameterName, TextureFormat};
use std::collections::BTreeMap;
use std::sync::Arc;

pub(super) struct ReflectedComputeModule {
    entry: ShaderEntryPoint,
    groups: BTreeMap<BindGroupIndex, ReflectedBindGroup>,
}
impl ReflectedComputeModule {
    pub fn new(source: &ShaderSource) -> Result<Self, ComputePipelineError> {
        let module = source
            .parse(wgpu::naga::ShaderStage::Compute)
            .map_err(ComputePipelineError::Reflection)?;
        let entry = module
            .entry_points
            .iter()
            .find(|ep: &&wgpu::naga::EntryPoint| -> bool {
                ep.stage == wgpu::naga::ShaderStage::Compute
            })
            .map(|ep: &wgpu::naga::EntryPoint| -> ShaderEntryPoint {
                ShaderEntryPoint::from(ep.name.clone())
            })
            .ok_or(ComputePipelineError::MissingComputeEntryPoint)?;
        let mut groups = BTreeMap::new();
        for (_, variable) in module.global_variables.iter() {
            if let Some(binding) = &variable.binding {
                groups
                    .entry(BindGroupIndex::from(binding.group))
                    .or_insert_with(|| -> ReflectedBindGroup { ReflectedBindGroup::new(binding) })
                    .include(&module, variable, binding);
            }
        }
        for group in groups.values_mut() {
            group
                .entries
                .sort_by_key(|entry: &wgpu::BindGroupLayoutEntry| -> u32 { entry.binding });
        }
        Ok(Self { entry, groups })
    }
    pub fn create_layouts(&self, device: &wgpu::Device) -> Vec<Arc<wgpu::BindGroupLayout>> {
        self.groups
            .values()
            .map(|group: &ReflectedBindGroup| -> Arc<wgpu::BindGroupLayout> {
                Arc::new(
                    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                        label: Some(&format!(
                            "ComputePipeline Bind Layout Group {}",
                            group.data.index
                        )),
                        entries: &group.entries,
                    }),
                )
            })
            .collect()
    }
    pub fn into_data(self) -> ReflectionData {
        ReflectionData {
            bind_groups: self
                .groups
                .into_values()
                .map(|group: ReflectedBindGroup| -> BindGroupReflection { group.data })
                .collect(),
            compute_entry_point: self.entry,
        }
    }
}
struct ReflectedBindGroup {
    data: BindGroupReflection,
    entries: Vec<wgpu::BindGroupLayoutEntry>,
}
impl ReflectedBindGroup {
    fn new(binding: &wgpu::naga::ResourceBinding) -> Self {
        Self {
            data: BindGroupReflection {
                index: BindGroupIndex::from(binding.group),
                ..Default::default()
            },
            entries: Vec::new(),
        }
    }
    fn include(
        &mut self,
        module: &wgpu::naga::Module,
        variable: &wgpu::naga::GlobalVariable,
        binding: &wgpu::naga::ResourceBinding,
    ) {
        match variable.space {
            wgpu::naga::AddressSpace::Uniform => self.include_uniform(module, variable, binding),
            wgpu::naga::AddressSpace::Storage { access } => {
                let ty = wgpu::BufferBindingType::Storage {
                    read_only: !access.contains(wgpu::naga::StorageAccess::STORE),
                };
                self.data.buffer_bindings.push(BufferBinding {
                    name: ShaderBindingName::from(variable.name.clone().unwrap_or_default()),
                    binding: BindingIndex::from(binding.binding),
                    ty,
                });
                self.entries.push(wgpu::BindGroupLayoutEntry {
                    binding: binding.binding,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                });
            }
            wgpu::naga::AddressSpace::Handle => self.include_handle(module, variable, binding),
            _ => {}
        }
    }
    fn include_uniform(
        &mut self,
        module: &wgpu::naga::Module,
        variable: &wgpu::naga::GlobalVariable,
        binding: &wgpu::naga::ResourceBinding,
    ) {
        let ty = &module.types[variable.ty];
        self.data.uniform_binding = Some(BindingIndex::from(binding.binding));
        if let wgpu::naga::TypeInner::Struct { members, span } = &ty.inner {
            self.data.uniform_buffer_size = BufferByteLength::from(*span);
            for member in members {
                self.data.uniform_members.push(UniformMember {
                    name: PassParameterName::from(member.name.clone().unwrap_or_default()),
                    offset: BufferByteOffset::from(member.offset),
                    size: BufferByteLength::from(
                        module.types[member.ty].inner.size(module.to_ctx()),
                    ),
                });
            }
        } else {
            self.data.uniform_buffer_size = BufferByteLength::from(ty.inner.size(module.to_ctx()));
        }
        self.entries.push(wgpu::BindGroupLayoutEntry {
            binding: binding.binding,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        });
    }
    fn include_handle(
        &mut self,
        module: &wgpu::naga::Module,
        variable: &wgpu::naga::GlobalVariable,
        binding: &wgpu::naga::ResourceBinding,
    ) {
        let ty = &module.types[variable.ty];
        match &ty.inner {
            wgpu::naga::TypeInner::Image { .. } => {
                let reflected = ReflectedTexture::new(variable, binding, &ty.inner);
                self.entries.push(reflected.layout);
                self.data.texture_bindings.push(reflected.data);
            }
            wgpu::naga::TypeInner::Sampler { .. } => {
                self.data.sampler_bindings.push(SamplerBinding {
                    name: ShaderBindingName::from(variable.name.clone().unwrap_or_default()),
                    binding: BindingIndex::from(binding.binding),
                });
                self.entries.push(wgpu::BindGroupLayoutEntry {
                    binding: binding.binding,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                });
            }
            _ => {}
        }
    }
}
struct ReflectedTexture {
    data: TextureBinding,
    layout: wgpu::BindGroupLayoutEntry,
}
impl ReflectedTexture {
    fn new(
        variable: &wgpu::naga::GlobalVariable,
        binding: &wgpu::naga::ResourceBinding,
        ty: &wgpu::naga::TypeInner,
    ) -> Self {
        let wgpu::naga::TypeInner::Image { dim, class, .. } = ty else {
            unreachable!("only images reach texture reflection")
        };
        let dimension = match dim {
            wgpu::naga::ImageDimension::D1 => wgpu::TextureViewDimension::D1,
            wgpu::naga::ImageDimension::D2 => wgpu::TextureViewDimension::D2,
            wgpu::naga::ImageDimension::D3 => wgpu::TextureViewDimension::D3,
            wgpu::naga::ImageDimension::Cube => wgpu::TextureViewDimension::Cube,
        };
        let (format, ty) = if let wgpu::naga::ImageClass::Storage { format, access } = class {
            let format = TextureFormat::naga_to_wgpu_format(*format);
            (
                Some(format),
                wgpu::BindingType::StorageTexture {
                    access: if access.contains(wgpu::naga::StorageAccess::STORE) {
                        wgpu::StorageTextureAccess::WriteOnly
                    } else {
                        wgpu::StorageTextureAccess::ReadOnly
                    },
                    format,
                    view_dimension: dimension,
                },
            )
        } else {
            (
                None,
                wgpu::BindingType::Texture {
                    multisampled: false,
                    view_dimension: dimension,
                    sample_type: texture_sample_type(class),
                },
            )
        };
        Self {
            data: TextureBinding {
                name: ShaderBindingName::from(variable.name.clone().unwrap_or_default()),
                binding: BindingIndex::from(binding.binding),
                format,
                dimension,
            },
            layout: wgpu::BindGroupLayoutEntry {
                binding: binding.binding,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty,
                count: None,
            },
        }
    }
}
fn texture_sample_type(class: &wgpu::naga::ImageClass) -> wgpu::TextureSampleType {
    match class {
        wgpu::naga::ImageClass::Sampled { kind, .. } => match kind {
            wgpu::naga::ScalarKind::Uint => wgpu::TextureSampleType::Uint,
            wgpu::naga::ScalarKind::Sint => wgpu::TextureSampleType::Sint,
            _ => wgpu::TextureSampleType::Float { filterable: true },
        },
        wgpu::naga::ImageClass::Depth { .. } => wgpu::TextureSampleType::Depth,
        _ => wgpu::TextureSampleType::Float { filterable: true },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reflection_retains_binding_order_uniform_layout_and_resource_classes() {
        let source = ShaderSource::from(
            r#"
struct Uniforms { count: u32, scale: f32, offset: vec2<f32> }
@group(0) @binding(5) var<uniform> params: Uniforms;
@group(0) @binding(3) var<storage, read> input: array<f32>;
@group(0) @binding(1) var<storage, read_write> output: array<f32>;
@group(0) @binding(6) var sampled: texture_2d<f32>;
@group(0) @binding(7) var filtering: sampler;
@group(0) @binding(8) var stored: texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(9) var integer: texture_2d<u32>;
@group(0) @binding(10) var depth: texture_depth_cube;
@group(2) @binding(0) var<uniform> scalar: u32;
@compute @workgroup_size(1) fn selected() {}
"#,
        );
        let reflected = ReflectedComputeModule::new(&source).unwrap();
        assert_eq!(reflected.entry, ShaderEntryPoint::from("selected"));
        assert_eq!(
            reflected.groups.keys().copied().collect::<Vec<_>>(),
            [BindGroupIndex::from(0), BindGroupIndex::from(2)]
        );
        let group = &reflected.groups[&BindGroupIndex::from(0)];
        for (entry, binding) in group.entries.iter().zip([1, 3, 5, 6, 7, 8, 9, 10]) {
            assert_eq!(entry.binding, binding);
        }
        assert_eq!(
            group.data.uniform_buffer_size,
            BufferByteLength::from(16u32)
        );
        assert_eq!(group.data.uniform_binding, Some(BindingIndex::from(5)));
        for (member, (offset, size)) in
            group
                .data
                .uniform_members
                .iter()
                .zip([(0u32, 4u32), (4, 4), (8, 8)])
        {
            assert_eq!(member.offset, BufferByteOffset::from(offset));
            assert_eq!(member.size, BufferByteLength::from(size));
        }
        assert_eq!(
            group.data.buffer_bindings[0].ty,
            wgpu::BufferBindingType::Storage { read_only: true }
        );
        assert_eq!(
            group.data.buffer_bindings[1].ty,
            wgpu::BufferBindingType::Storage { read_only: false }
        );
        assert_eq!(
            group.data.sampler_bindings,
            [SamplerBinding {
                name: ShaderBindingName::from("filtering"),
                binding: BindingIndex::from(7)
            }]
        );
        assert!(matches!(
            group.entries[3].ty,
            wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                ..
            }
        ));
        assert!(matches!(
            group.entries[5].ty,
            wgpu::BindingType::StorageTexture {
                access: wgpu::StorageTextureAccess::WriteOnly,
                format: wgpu::TextureFormat::Rgba8Unorm,
                ..
            }
        ));
        assert!(matches!(
            group.entries[6].ty,
            wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Uint,
                ..
            }
        ));
        assert!(matches!(
            group.entries[7].ty,
            wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Depth,
                view_dimension: wgpu::TextureViewDimension::Cube,
                ..
            }
        ));
        let scalar = &reflected.groups[&BindGroupIndex::from(2)].data;
        assert_eq!(scalar.uniform_buffer_size, BufferByteLength::from(4u32));
        assert!(scalar.uniform_members.is_empty());
    }
}
