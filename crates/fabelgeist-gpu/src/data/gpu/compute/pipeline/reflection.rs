//! Compute bindings are reflected before choosing a dispatch pipeline layout.
use super::*;
use crate::data::gpu::shader::{BindGroupReflection, BufferBinding, TextureBinding, UniformMember};
use std::collections::BTreeMap;

pub(super) struct ReflectedInterface {
    pub reflection: Arc<ReflectionData>,
    pub layouts: Vec<Option<Arc<wgpu::BindGroupLayout>>>,
}

struct GroupBuilder {
    reflection: BindGroupReflection,
    entries: Vec<wgpu::BindGroupLayoutEntry>,
}

impl ReflectedInterface {
    pub(super) fn new(context: &WgpuContext, code: &str) -> ComputePipelineResult<Self> {
        let module = parse_naga(code, wgpu::naga::ShaderStage::Compute)?;
        let entry = module
            .entry_points
            .iter()
            .find(|entry| entry.stage == wgpu::naga::ShaderStage::Compute)
            .ok_or(ComputePipelineError::MissingEntryPoint)?;
        let mut groups = BTreeMap::new();
        for (_, variable) in module.global_variables.iter() {
            let Some(binding) = &variable.binding else {
                continue;
            };
            groups
                .entry(binding.group)
                .or_insert_with(|| GroupBuilder::new(binding.group))
                .reflect(&module, variable, binding.binding);
        }
        let limit = context.device.limits().max_bind_groups;
        if let Some(&index) = groups.keys().find(|&&index| index >= limit) {
            return Err(ComputePipelineError::BindGroupLimit { index, limit });
        }
        #[cfg(not(target_arch = "wasm32"))]
        let scope = context.blocking_validation.then(|| {
            context
                .device
                .push_error_scope(wgpu::ErrorFilter::Validation)
        });
        let mut layouts = Vec::new();
        let mut reflected = Vec::new();
        for (index, mut group) in groups {
            while layouts.len() < index as usize {
                layouts.push(None);
            }
            group.entries.sort_by_key(|entry| entry.binding);
            layouts.push(Some(Arc::new(context.device.create_bind_group_layout(
                &wgpu::BindGroupLayoutDescriptor {
                    label: Some(&format!("ComputePipeline Bind Layout Group {index}")),
                    entries: &group.entries,
                },
            ))));
            reflected.push(group.reflection);
        }
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(scope) = scope {
            let _ = context.device.poll(wgpu::PollType::wait_indefinitely());
            if let Some(cause) = pollster::block_on(scope.pop()) {
                return Err(ComputePipelineError::Native(Arc::new(cause)));
            }
        }
        Ok(Self {
            reflection: Arc::new(ReflectionData {
                bind_groups: reflected,
                fragment_entry_point: String::new(),
                vertex_entry_point: entry.name.clone(),
            }),
            layouts,
        })
    }
}

impl GroupBuilder {
    fn new(index: u32) -> Self {
        Self {
            reflection: BindGroupReflection {
                index,
                ..Default::default()
            },
            entries: Vec::new(),
        }
    }

    fn reflect(
        &mut self,
        module: &wgpu::naga::Module,
        variable: &wgpu::naga::GlobalVariable,
        binding: u32,
    ) {
        let name = variable.name.clone().unwrap_or_default();
        match variable.space {
            wgpu::naga::AddressSpace::Uniform => {
                self.uniform(module, variable.ty, binding);
            }
            wgpu::naga::AddressSpace::Storage { access } => {
                let ty = wgpu::BufferBindingType::Storage {
                    read_only: !access.contains(wgpu::naga::StorageAccess::STORE),
                };
                self.reflection
                    .buffer_bindings
                    .push(BufferBinding { name, binding, ty });
                self.entry(
                    binding,
                    wgpu::BindingType::Buffer {
                        ty,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                );
            }
            wgpu::naga::AddressSpace::Handle => match &module.types[variable.ty].inner {
                wgpu::naga::TypeInner::Image {
                    dim,
                    class,
                    arrayed,
                } => {
                    self.image(name, binding, *dim, *arrayed, class);
                }
                wgpu::naga::TypeInner::Sampler { .. } => {
                    self.reflection.sampler_bindings.push((name, binding));
                    self.entry(
                        binding,
                        wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    );
                }
                _ => {}
            },
            _ => {}
        }
    }

    fn uniform(
        &mut self,
        module: &wgpu::naga::Module,
        ty: wgpu::naga::Handle<wgpu::naga::Type>,
        binding: u32,
    ) {
        let ty = &module.types[ty];
        self.reflection.uniform_binding = Some(binding);
        if let wgpu::naga::TypeInner::Struct { members, span } = &ty.inner {
            self.reflection.uniform_buffer_size = *span;
            for member in members {
                self.reflection.uniform_members.push(UniformMember {
                    name: member.name.clone().unwrap_or_default(),
                    offset: member.offset,
                    size: module.types[member.ty].inner.size(module.to_ctx()),
                });
            }
        } else {
            self.reflection.uniform_buffer_size = ty.inner.size(module.to_ctx());
        }
        self.entry(
            binding,
            wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
        );
    }

    fn image(
        &mut self,
        name: String,
        binding: u32,
        dimension: wgpu::naga::ImageDimension,
        arrayed: bool,
        class: &wgpu::naga::ImageClass,
    ) {
        let dimension = match dimension {
            wgpu::naga::ImageDimension::D1 => wgpu::TextureViewDimension::D1,
            wgpu::naga::ImageDimension::D2 => wgpu::TextureViewDimension::D2,
            wgpu::naga::ImageDimension::D3 => wgpu::TextureViewDimension::D3,
            wgpu::naga::ImageDimension::Cube => wgpu::TextureViewDimension::Cube,
        };
        let dimension = match (dimension, arrayed) {
            (wgpu::TextureViewDimension::D2, true) => wgpu::TextureViewDimension::D2Array,
            (wgpu::TextureViewDimension::Cube, true) => wgpu::TextureViewDimension::CubeArray,
            _ => dimension,
        };
        let mut format = None;
        let ty = match class {
            wgpu::naga::ImageClass::Storage {
                format: storage,
                access,
            } => {
                let storage = TextureFormat::naga_to_wgpu_format(*storage);
                format = Some(storage);
                wgpu::BindingType::StorageTexture {
                    access: if access.contains(wgpu::naga::StorageAccess::STORE) {
                        wgpu::StorageTextureAccess::WriteOnly
                    } else {
                        wgpu::StorageTextureAccess::ReadOnly
                    },
                    format: storage,
                    view_dimension: dimension,
                }
            }
            _ => wgpu::BindingType::Texture {
                multisampled: false,
                view_dimension: dimension,
                sample_type: sample_type(class),
            },
        };
        self.entry(binding, ty);
        self.reflection.texture_bindings.push(TextureBinding {
            name,
            binding,
            format,
            dimension,
        });
    }

    fn entry(&mut self, binding: u32, ty: wgpu::BindingType) {
        self.entries.push(wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty,
            count: None,
        });
    }
}

fn sample_type(class: &wgpu::naga::ImageClass) -> wgpu::TextureSampleType {
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
