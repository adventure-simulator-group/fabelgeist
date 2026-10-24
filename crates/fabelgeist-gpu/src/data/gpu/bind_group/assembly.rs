use super::{ComputeBindGroup, uniform_buffer};
use crate::data::{BindGroupReflection, ComputePipeline, PassParameter, PassParameters};
use crate::data::{ComputeBindingKind, ComputePassError};
#[cfg(not(target_arch = "wasm32"))]
use crate::globals::ValidationScope;
use crate::globals::WgpuContext;
use anyhow::{Context, Result};
use std::sync::Arc;

struct BoundTexture {
    binding: u32,
    view: Arc<wgpu::TextureView>,
}

/// Assemble the pipeline's admitted static interface, preserving logical buffer
/// prefixes and selected texture format, dimension and creating device.
pub fn create_bind_groups(
    context: &WgpuContext,
    pipeline: &ComputePipeline,
    parameters: &PassParameters,
) -> Result<Vec<ComputeBindGroup>> {
    pipeline.ensure_device(context)?;
    pipeline
        .reflection()
        .bind_groups
        .iter()
        .map(|reflection| {
            let layout = pipeline
                .bind_group_layouts()
                .get(reflection.index as usize)
                .and_then(Option::as_deref)
                .ok_or(ComputePassError::MissingLayout {
                    index: reflection.index,
                })?;
            let group = create_group(context, layout, reflection, parameters)?;
            Ok(ComputeBindGroup {
                index: reflection.index,
                group,
            })
        })
        .collect()
}

fn create_group(
    context: &WgpuContext,
    layout: &wgpu::BindGroupLayout,
    reflection: &BindGroupReflection,
    parameters: &PassParameters,
) -> Result<wgpu::BindGroup> {
    let uniform_buffer = uniform_buffer::create(context, reflection, parameters)?;
    let mut entries = Vec::new();
    if let (Some(buffer), Some(binding)) = (&uniform_buffer, reflection.uniform_binding) {
        entries.push(wgpu::BindGroupEntry {
            binding,
            resource: buffer.buffer.as_entire_binding(),
        });
    }
    add_buffers(&mut entries, reflection, parameters)?;
    let textures = select_textures(context, reflection, parameters)?;
    for texture in &textures {
        entries.push(wgpu::BindGroupEntry {
            binding: texture.binding,
            resource: wgpu::BindingResource::TextureView(&texture.view),
        });
    }
    add_samplers(&mut entries, reflection, parameters)?;
    #[cfg(not(target_arch = "wasm32"))]
    let scope = ValidationScope::new(context);
    let group = context
        .device
        .create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ComputePass Bind Group"),
            layout,
            entries: &entries,
        });
    #[cfg(not(target_arch = "wasm32"))]
    if let Some(scope) = scope {
        scope.finish().map_err(ComputePassError::from)?;
    }
    Ok(group)
}

fn parameter<'a>(
    parameters: &'a PassParameters,
    name: &str,
    kind: ComputeBindingKind,
) -> Result<&'a PassParameter, ComputePassError> {
    parameters
        .get(name)
        .ok_or_else(|| ComputePassError::MissingParameter {
            name: name.to_owned(),
            kind,
        })
}

fn add_buffers<'a>(
    entries: &mut Vec<wgpu::BindGroupEntry<'a>>,
    reflection: &BindGroupReflection,
    parameters: &'a PassParameters,
) -> Result<()> {
    for binding in &reflection.buffer_bindings {
        let name = &binding.name;
        let PassParameter::Buffer(buffer) =
            parameter(parameters, name, ComputeBindingKind::Buffer)?
        else {
            return Err(ComputePassError::WrongParameterKind {
                name: name.clone(),
                kind: ComputeBindingKind::Buffer,
            }
            .into());
        };
        entries.push(wgpu::BindGroupEntry {
            binding: binding.binding,
            resource: buffer.binding_resource(name)?,
        });
    }
    Ok(())
}

fn select_textures(
    context: &WgpuContext,
    reflection: &BindGroupReflection,
    parameters: &PassParameters,
) -> Result<Vec<BoundTexture>> {
    reflection
        .texture_bindings
        .iter()
        .map(|binding| {
            let selected = parameter(parameters, &binding.name, ComputeBindingKind::Texture)?
                .texture_binding(context, binding)
                .map_err(anyhow::Error::new)
                .with_context(|| format!("ComputePass: Texture '{}'", binding.name))?;
            Ok(BoundTexture {
                binding: binding.binding,
                view: selected.view,
            })
        })
        .collect()
}

fn add_samplers<'a>(
    entries: &mut Vec<wgpu::BindGroupEntry<'a>>,
    reflection: &BindGroupReflection,
    parameters: &'a PassParameters,
) -> Result<()> {
    for binding in &reflection.sampler_bindings {
        let name = &binding.name;
        let PassParameter::Sampler(sampler) =
            parameter(parameters, name, ComputeBindingKind::Sampler)?
        else {
            return Err(ComputePassError::WrongParameterKind {
                name: name.clone(),
                kind: ComputeBindingKind::Sampler,
            }
            .into());
        };
        let sampler = sampler
            .sampler
            .as_ref()
            .ok_or_else(|| ComputePassError::MissingSampler { name: name.clone() })?;
        entries.push(wgpu::BindGroupEntry {
            binding: binding.binding,
            resource: wgpu::BindingResource::Sampler(sampler),
        });
    }
    Ok(())
}
