//! The cached dispatch path.
//!
//! [`ComputePass::record`] builds a uniform buffer and a bind group for every
//! dispatch. That is fine at a handful of dispatches per frame, which is what
//! the other primitives here do. A solver is different: an XPBD step is
//! substeps times constraint colours, comfortably five hundred dispatches in a
//! frame, and allocating two GPU objects for each of them costs more in CPU
//! time than the shaders cost on the GPU -- and, run at speed, exhausts the
//! allocator outright.
//!
//! So a [`Kernel`](super::Kernel) builds its own pipeline with a
//! **dynamic-offset** uniform and per-batch uniform slots, written by offset,
//! so the per-dispatch values cost a `write_buffer` into memory that already
//! exists rather than a fresh allocation.
//!
//! # The bind group is *not* cached, deliberately
//!
//! Caching it is the obvious next step and it is wrong. Reusing one bind group
//! across consecutive compute passes makes wgpu's resource tracker treat the
//! buffers as already in the right state, and it stops emitting the barrier
//! between them -- so a dispatch can start reading what the previous one has
//! not finished writing. It shows up as a solver that is subtly, intermittently
//! wrong: in the case that found this, a refit BVH that had lost a few
//! primitives, failing about two runs in three.
//!
//! Creating the bind group per dispatch re-registers the resources and the
//! barrier comes back. Measured on a full garment step, that costs six per
//! cent -- 4.7 ms a frame against 4.4 -- which is not a trade worth making.
//! The expensive part was never the bind group; it was allocating a uniform
//! *buffer* per dispatch, and the ring is what fixes that.
//!
//! Kernels whose shape this cannot serve -- more than one bind group, or any
//! texture or sampler binding -- have no fast path and fall back to
//! `ComputePass::record`, which still handles everything.

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

/// The largest uniform block the cached path packs without allocating.
///
/// Every kernel here passes a few scalars and at most a vector or two. A
/// kernel wanting more than this is refused a fast path rather than silently
/// allocating per dispatch, which is the cost this whole module exists to
/// avoid.
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
        module: &wgpu::ShaderModule,
        entry_point: &str,
        reflection: &fabelgeist_gpu::data::gpu::shader::ReflectionData,
    ) -> Result<Option<Self>> {
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

        let uniform_size = u64::from(group.uniform_buffer_size);
        if uniform_size as usize > MAX_UNIFORM_BYTES {
            return Ok(None);
        }
        let uniform_binding = group.uniform_binding.filter(|_| uniform_size > 0);
        if let Some(binding) = uniform_binding {
            entries.push(wgpu::BindGroupLayoutEntry {
                binding,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    // The whole point: one buffer, one bind group, and the
                    // per-dispatch values selected by offset.
                    has_dynamic_offset: true,
                    min_binding_size: std::num::NonZeroU64::new(uniform_size),
                },
                count: None,
            });
        }
        entries.sort_by_key(|entry| entry.binding);

        let compiled = CompiledLayout::new(context, &entries, module, entry_point)?;

        let alignment = context.device.limits().min_uniform_buffer_offset_alignment as u64;
        let stride = uniform_size.div_ceil(alignment.max(1)) * alignment.max(1);

        Ok(Some(Self {
            pipeline: compiled.pipeline,
            layout: compiled.layout,
            buffers: group.buffer_bindings.clone(),
            uniform_binding,
            uniform_members: group.uniform_members.clone(),
            uniform_size: uniform_size.into(),
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
        // Built fresh every dispatch. See the note at the top of this file:
        // reusing one costs wgpu the barrier between dependent passes.
        let mut entries: Vec<wgpu::BindGroupEntry> = Vec::with_capacity(self.buffers.len() + 1);
        let mut bound: Vec<BoundBuffer> = Vec::with_capacity(self.buffers.len());
        for binding in &self.buffers {
            let Some(PassParameter::Buffer(buffer)) = parameters.get(&binding.name) else {
                return Err(anyhow!(
                    "Kernel: parameter `{}` (binding {}) is missing or is not a buffer",
                    binding.name,
                    binding.binding
                ));
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
            .map_err(|_| arena::UniformArenaError::AddressOverflow)?;
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
