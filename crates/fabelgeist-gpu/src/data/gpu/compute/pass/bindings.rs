//! Construction and admission of one reflected resource group.
use super::{BindingKind, ComputePassError};
use crate::data::{
    BindGroupReflection, BindingIndex, BufferBinding, PassParameter, PassParameters,
    SamplerBinding, ShaderBindingName, UniformBytes, UniformPackingPolicy,
};
use crate::globals::WgpuContext;
use wgpu::util::DeviceExt;

pub(super) struct BoundGroup {
    pub native: wgpu::BindGroup,
}
impl BoundGroup {
    pub fn new(
        context: &WgpuContext,
        reflected: &BindGroupReflection,
        layout: &wgpu::BindGroupLayout,
        parameters: &PassParameters,
    ) -> Result<Self, ComputePassError> {
        let mut entries = Vec::new();
        let uniform = reflected.create_uniform(context, parameters)?;
        if let Some(buffer) = &uniform
            && let Some(binding) = reflected.uniform_binding
        {
            entries.push(wgpu::BindGroupEntry {
                binding: u32::from(binding),
                resource: buffer.as_entire_binding(),
            });
        }
        for binding in &reflected.buffer_bindings {
            entries.push(wgpu::BindGroupEntry {
                binding: u32::from(binding.binding),
                resource: binding.resolve(parameters)?,
            });
        }
        let mut views = Vec::new();
        for binding in &reflected.texture_bindings {
            let value = Self::parameter(
                parameters,
                &binding.name,
                binding.binding,
                BindingKind::Texture,
            )?;
            views.push((binding.binding, binding.resolve(context, value)?));
        }
        for (binding, view) in &views {
            entries.push(wgpu::BindGroupEntry {
                binding: u32::from(*binding),
                resource: wgpu::BindingResource::TextureView(view),
            });
        }
        for binding in &reflected.sampler_bindings {
            let native = binding.resolve(parameters)?;
            entries.push(wgpu::BindGroupEntry {
                binding: u32::from(binding.binding),
                resource: wgpu::BindingResource::Sampler(native),
            });
        }
        Ok(Self {
            native: context
                .device
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some(&format!("ComputePass Group {} Bind Group", reflected.index)),
                    layout,
                    entries: &entries,
                }),
        })
    }
    fn parameter<'a>(
        parameters: &'a PassParameters,
        name: &ShaderBindingName,
        binding: BindingIndex,
        kind: BindingKind,
    ) -> Result<&'a PassParameter, ComputePassError> {
        parameters
            .get(name.parameter_name())
            .ok_or_else(|| -> ComputePassError {
                ComputePassError::MissingParameter {
                    name: name.clone(),
                    binding,
                    kind,
                }
            })
    }
}

impl BindGroupReflection {
    fn create_uniform(
        &self,
        context: &WgpuContext,
        parameters: &PassParameters,
    ) -> Result<Option<wgpu::Buffer>, ComputePassError> {
        if u64::from(self.uniform_buffer_size) > 0 {
            let mut bytes = vec![0u8; u64::from(self.uniform_buffer_size) as usize];
            UniformBytes::from(bytes.as_mut_slice())
                .pack(
                    &self.uniform_members,
                    parameters,
                    UniformPackingPolicy::General,
                )
                .map_err(ComputePassError::Uniform)?;
            Ok(Some(context.device.create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some(&format!("ComputePass Group {} Uniforms", self.index)),
                    contents: &bytes,
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                },
            )))
        } else {
            Ok(None)
        }
    }
}
impl BufferBinding {
    fn resolve<'a>(
        &self,
        parameters: &'a PassParameters,
    ) -> Result<wgpu::BindingResource<'a>, ComputePassError> {
        let value =
            BoundGroup::parameter(parameters, &self.name, self.binding, BindingKind::Buffer)?;
        let PassParameter::Buffer(buffer) = value else {
            return Err(ComputePassError::WrongParameter {
                name: self.name.clone(),
                expected: BindingKind::Buffer,
            });
        };
        Ok(
            if buffer.length()
                < super::super::super::buffer::BufferByteLength::from(buffer.buffer.size())
            {
                wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &buffer.buffer,
                    offset: 0,
                    size: Some(std::num::NonZeroU64::new(u64::from(buffer.length())).unwrap()),
                })
            } else {
                buffer.buffer.as_entire_binding()
            },
        )
    }
}
impl SamplerBinding {
    fn resolve<'a>(
        &self,
        parameters: &'a PassParameters,
    ) -> Result<&'a wgpu::Sampler, ComputePassError> {
        let value =
            BoundGroup::parameter(parameters, &self.name, self.binding, BindingKind::Sampler)?;
        let PassParameter::Sampler(sampler) = value else {
            return Err(ComputePassError::WrongParameter {
                name: self.name.clone(),
                expected: BindingKind::Sampler,
            });
        };
        sampler
            .sampler
            .as_deref()
            .ok_or_else(|| -> ComputePassError {
                ComputePassError::MissingSampler(self.name.clone())
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resource_admission_retains_kind_coordinate_and_uninitialized_state() {
        let buffer = BufferBinding {
            name: "input".into(),
            binding: BindingIndex::from(7),
            ty: wgpu::BufferBindingType::Storage { read_only: true },
        };
        let mut parameters = PassParameters::new();
        let error = buffer.resolve(&parameters).err().unwrap();
        assert!(
            matches!(&error, ComputePassError::MissingParameter { name, binding, kind: BindingKind::Buffer } if *name == buffer.name && *binding == buffer.binding)
        );
        assert_eq!(
            error.to_string(),
            "ComputePass: Parameter 'input' (buffer binding 7) not found"
        );
        parameters.insert("input".into(), 4u32.into());
        assert!(matches!(
            buffer.resolve(&parameters),
            Err(ComputePassError::WrongParameter {
                expected: BindingKind::Buffer,
                ..
            })
        ));
        let sampler = SamplerBinding {
            name: "filtering".into(),
            binding: BindingIndex::from(9),
        };
        assert!(matches!(
            sampler.resolve(&parameters),
            Err(ComputePassError::MissingParameter {
                kind: BindingKind::Sampler,
                ..
            })
        ));
        parameters.insert("filtering".into(), 4u32.into());
        assert!(matches!(
            sampler.resolve(&parameters),
            Err(ComputePassError::WrongParameter {
                expected: BindingKind::Sampler,
                ..
            })
        ));
        parameters.insert("filtering".into(), crate::data::Sampler::default().into());
        assert!(matches!(
            sampler.resolve(&parameters),
            Err(ComputePassError::MissingSampler(_))
        ));
    }
}
