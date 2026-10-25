//! The cached dispatch path.
//!
//! Kernels with one buffer-only bind group can use a dynamic uniform offset
//! into per-batch allocation chunks. Each dispatch writes its own slot and
//! creates a fresh bind group; the batch retains the chunks until submission.
//!
//! Fresh groups preserve the existing recording policy. The BVH refit test
//! exercises dependent dispatches, but does not establish that bind-group
//! identity controls barriers. Native synchronization remains wgpu's contract.
//!
//! Unsupported interfaces fall back to [`ComputePass::record`].

use crate::prelude::*;
pub(super) use arena::UniformArena;
use compile::CompiledLayout;
use fabelgeist_gpu::data::gpu::buffer::{Buffer, BufferByteLength, BufferByteOffset};
use std::sync::Arc;
use uniforms::write_uniform;

use fabelgeist_gpu::data::gpu::parameters::{PassParameter, PassParameters};
use fabelgeist_gpu::data::gpu::shader::{BufferBinding, UniformMember};

mod arena;
mod compile;
mod uniforms;

/// Capacity of the cached path's stack scratch buffer, in native uniform bytes.
/// Larger reflected blocks use the static recording path. Individual value
/// writers may still allocate while packing a block within this capacity.
const MAX_UNIFORM_BYTES: usize = 256;

pub(super) struct FastPath {
    pipeline: wgpu::ComputePipeline,
    layout: wgpu::BindGroupLayout,

    buffers: Vec<BufferBinding>,
    uniform_binding: Option<u32>,
    uniform_members: Vec<UniformMember>,
    uniform_size: BufferByteLength,
    /// `uniform_size` rounded up to the device's binding alignment.
    stride: BufferByteLength,
    device: Arc<wgpu::Device>,
}

struct BoundBuffer {
    descriptor: BufferBinding,
    buffer: Buffer,
}

/// Native dispatch resources admitted for one batch recording.
pub(super) struct Prepared {
    pipeline: wgpu::ComputePipeline,
    bind_group: wgpu::BindGroup,
    dynamic_offset: Option<BufferByteOffset>,
}

impl Prepared {
    pub(super) fn record(
        &self,
        pass: &mut wgpu::ComputePass<'_>,
        groups: WorkgroupGrid,
    ) -> Result<()> {
        let offset = self
            .dynamic_offset
            .map(|offset| u32::try_from(u64::from(offset)))
            .transpose()
            .map_err(|_| arena::UniformArenaError::DynamicOffsetOverflow)?;
        pass.set_pipeline(&self.pipeline);
        match offset {
            Some(offset) => pass.set_bind_group(0, &self.bind_group, &[offset]),
            None => pass.set_bind_group(0, &self.bind_group, &[]),
        }
        groups.record(pass);
        Ok(())
    }
}

impl FastPath {
    /// Build a fast path for this kernel, or `None` if its shape does not
    /// suit one.
    pub(super) fn new(
        context: &WgpuContext,
        pipeline: &ComputePipeline,
    ) -> Result<Option<Self>> {
        pipeline.ensure_device(context)?;
        let reflection = pipeline.reflection();
        // One bind group only. Everything the solver writes uses group 0, and
        // supporting more would mean a dynamic offset per group.
        if reflection.bind_groups.len() != 1 {
            return Ok(None);
        }
        let group = &reflection.bind_groups[0];
        if group.index != 0 {
            return Ok(None);
        }
        // Textures and samplers would each need their own cache key and their
        // own lifetime handling; nothing on the solver path uses them.
        if !group.texture_bindings.is_empty() || !group.sampler_bindings.is_empty() {
            return Ok(None);
        }

        let mut entries: Vec<wgpu::BindGroupLayoutEntry> = group
            .buffer_bindings
            .iter()
            .map(|binding| wgpu::BindGroupLayoutEntry {
                binding: binding.binding,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: binding.ty,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            })
            .collect();

        let uniform_size = group.uniform_buffer_size;
        if uniform_size > BufferByteLength::from(MAX_UNIFORM_BYTES) {
            return Ok(None);
        }
        let native_uniform_size = u64::from(uniform_size);
        let uniform_binding = group.uniform_binding.filter(|_| native_uniform_size > 0);
        if let Some(binding) = uniform_binding {
            entries.push(wgpu::BindGroupLayoutEntry {
                binding,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    // Each fresh group binds the batch's uniform chunk;
                    // the dynamic offset selects this dispatch's slot.
                    has_dynamic_offset: true,
                    min_binding_size: std::num::NonZeroU64::new(native_uniform_size),
                },
                count: None,
            });
        }
        entries.sort_by_key(|entry| entry.binding);

        let compiled = CompiledLayout::new(
            context,
            &entries,
            pipeline.shader().module(),
            &reflection.entry_point,
        )?;

        let alignment = context.device.limits().min_uniform_buffer_offset_alignment as u64;
        let stride = native_uniform_size.div_ceil(alignment.max(1)) * alignment.max(1);

        Ok(Some(Self {
            pipeline: compiled.pipeline,
            layout: compiled.layout,
            buffers: group.buffer_bindings.clone(),
            uniform_binding,
            uniform_members: group.uniform_members.clone(),
            uniform_size,
            stride: stride.into(),
            device: context.device.clone(),
        }))
    }

    /// Resolve the bind group and write this dispatch's uniform values.
    pub(super) fn prepare(
        &self,
        context: &WgpuContext,
        parameters: &PassParameters,
        uniforms: &mut UniformArena,
    ) -> Result<Prepared> {
        if self.device.as_ref() != context.device.as_ref() {
            return Err(ComputePipelineError::WrongDevice.into());
        }
        // Preserve fresh groups for each dispatch, including dependent kernels.
        let mut entries: Vec<wgpu::BindGroupEntry> = Vec::with_capacity(self.buffers.len() + 1);
        let mut bound: Vec<BoundBuffer> = Vec::with_capacity(self.buffers.len());
        for binding in &self.buffers {
            let value = parameters.get(&binding.name).ok_or_else(|| {
                ComputePassError::MissingParameter {
                    name: binding.name.clone(),
                    kind: ComputeBindingKind::Buffer,
                }
            })?;
            let PassParameter::Buffer(buffer) = value else {
                return Err(ComputePassError::WrongParameterKind {
                    name: binding.name.clone(),
                    kind: ComputeBindingKind::Buffer,
                }
                .into());
            };
            bound.push(BoundBuffer {
                descriptor: binding.clone(),
                buffer: buffer.clone(),
            });
        }

        for bound in &bound {
            entries.push(wgpu::BindGroupEntry {
                binding: bound.descriptor.binding,
                resource: bound.buffer.binding_resource(&bound.descriptor.name)?,
            });
        }
        let mut data = [0u8; MAX_UNIFORM_BYTES];
        let size = usize::try_from(u64::from(self.uniform_size))
            .map_err(ComputePassError::UniformHostAddress)?;
        let data = data
            .get_mut(..size)
            .ok_or(arena::UniformArenaError::StrideExceedsChunk)?;
        if self.uniform_binding.is_some() {
            write_uniform(&self.uniform_members, parameters, data)?;
        }
        let slot = self
            .uniform_binding
            .map(|_| uniforms.allocate(context, self.stride))
            .transpose()?;
        if let (Some(binding), Some(slot)) = (self.uniform_binding, &slot) {
            entries.push(wgpu::BindGroupEntry {
                binding,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &slot.buffer,
                    offset: 0,
                    size: std::num::NonZeroU64::new(u64::from(self.uniform_size)),
                }),
            });
        }
        entries.sort_by_key(|entry| entry.binding);

        #[cfg(not(target_arch = "wasm32"))]
        let scope = context.blocking_validation.then(|| {
            context
                .device
                .push_error_scope(wgpu::ErrorFilter::Validation)
        });
        let bind_group = context
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Kernel Bind Group"),
                layout: &self.layout,
                entries: &entries,
            });

        let dynamic_offset = slot.as_ref().map(|slot| {
            context
                .queue
                .write_buffer(&slot.buffer, u64::from(slot.offset), data);
            slot.offset
        });
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(scope) = scope {
            let _ = context.device.poll(wgpu::PollType::wait_indefinitely());
            if let Some(cause) = pollster::block_on(scope.pop()) {
                return Err(ComputePassError::Native(Arc::new(cause)).into());
            }
        }

        Ok(Prepared {
            pipeline: self.pipeline.clone(),
            bind_group,
            dynamic_offset,
        })
    }
}
