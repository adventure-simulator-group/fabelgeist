//! Compute bindings are reflected before choosing a dispatch pipeline layout.
use super::*;
use crate::data::gpu::shader::{BindGroupReflection, BufferBinding, SamplerBinding, validate_naga};
use std::collections::BTreeMap;

mod images;
mod uniforms;

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
        let info = validate_naga(&module, code)?;
        let entry = module
            .entry_points
            .iter()
            .find(|entry| entry.stage == wgpu::naga::ShaderStage::Compute)
            .ok_or(ComputePipelineError::MissingEntryPoint)?;
        let mut groups = BTreeMap::new();
        for (handle, variable) in module.global_variables.iter() {
            let used = module
                .entry_points
                .iter()
                .enumerate()
                .any(|(index, entry)| {
                    entry.stage == wgpu::naga::ShaderStage::Compute
                        && !info.get_entry_point(index)[handle].is_empty()
                });
            if !used {
                continue;
            }
            if variable.space == wgpu::naga::AddressSpace::Immediate {
                return Err(ComputePipelineError::UnsupportedImmediate {
                    name: variable.name.clone().unwrap_or_default(),
                });
            }
            let Some(binding) = &variable.binding else {
                continue;
            };
            // One retained layout serves every cached compute entry point.
            // A float image needs filtering if any of those entries samples it.
            let filterable = module
                .entry_points
                .iter()
                .enumerate()
                .any(|(index, entry)| {
                    entry.stage == wgpu::naga::ShaderStage::Compute
                        && info
                            .get_entry_point(index)
                            .sampling_set
                            .iter()
                            .any(|pair| pair.image == handle)
                });
            groups
                .entry(binding.group)
                .or_insert_with(|| GroupBuilder::new(binding.group))
                .reflect(&module, variable, binding.binding, filterable)?;
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
        filterable: bool,
    ) -> ComputePipelineResult<()> {
        if matches!(
            &module.types[variable.ty].inner,
            wgpu::naga::TypeInner::BindingArray { .. }
        ) {
            return Err(self.unsupported(binding, UnsupportedBindingKind::BindingArray));
        }
        let name = variable.name.clone().unwrap_or_default();
        match variable.space {
            wgpu::naga::AddressSpace::Uniform => {
                self.uniform(module, variable.ty, binding, name)?;
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
                    self.image(name, binding, *dim, *arrayed, class, filterable)?;
                }
                wgpu::naga::TypeInner::Sampler { comparison } => {
                    self.reflection
                        .sampler_bindings
                        .push(SamplerBinding { name, binding });
                    self.entry(
                        binding,
                        wgpu::BindingType::Sampler(if *comparison {
                            wgpu::SamplerBindingType::Comparison
                        } else {
                            wgpu::SamplerBindingType::Filtering
                        }),
                    );
                }
                _ => return Err(self.unsupported(binding, UnsupportedBindingKind::Handle)),
            },
            _ => return Err(self.unsupported(binding, UnsupportedBindingKind::AddressSpace)),
        }
        Ok(())
    }

    fn unsupported(&self, binding: u32, kind: UnsupportedBindingKind) -> ComputePipelineError {
        ComputePipelineError::UnsupportedBinding {
            group: self.reflection.index,
            binding,
            kind,
        }
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
