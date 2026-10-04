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
//! **dynamic-offset** uniform and a ring of uniform slots, written by offset,
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

use super::KernelDispatchError;
use crate::prelude::*;
use fabelgeist_gpu::data::gpu::buffer::Buffer;
use fabelgeist_gpu::data::gpu::parameters::{PassParameter, PassParameters};
use fabelgeist_gpu::data::gpu::shader::{
    BindGroupIndex, BindingIndex, BufferBinding, UniformMember,
};

pub(super) use super::uniform_arena::UniformArena;
use super::uniform_arena::{UniformDynamicOffset, UniformStride};

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
    uniform_binding: Option<BindingIndex>,
    uniform_members: Vec<UniformMember>,
    uniform_size: BufferByteLength,
    /// `uniform_size` rounded up to the device's binding alignment.
    stride: UniformStride,
}

/// What a prepared dispatch needs at record time.
pub(super) struct Prepared {
    pub pipeline: wgpu::ComputePipeline,
    pub bind_group: wgpu::BindGroup,
    pub dynamic_offset: Option<UniformDynamicOffset>,
}

impl FastPath {
    /// Build a fast path for this kernel, or `None` if its shape does not
    /// suit one.
    pub(super) fn new(
        context: &WgpuContext,
        module: &wgpu::ShaderModule,
        entry_point: &ShaderEntryPoint,
        reflection: &fabelgeist_gpu::data::gpu::shader::ReflectionData,
    ) -> Option<Self> {
        // One bind group only. Everything the solver writes uses group 0, and
        // supporting more would mean a dynamic offset per group.
        if reflection.bind_groups.len() != 1 {
            return None;
        }
        let group = &reflection.bind_groups[0];
        if group.index != BindGroupIndex::FIRST {
            return None;
        }
        // Textures and samplers would each need their own cache key and their
        // own lifetime handling; nothing on the solver path uses them.
        if !group.texture_bindings.is_empty() || !group.sampler_bindings.is_empty() {
            return None;
        }

        let mut entries: Vec<wgpu::BindGroupLayoutEntry> = group
            .buffer_bindings
            .iter()
            .map(|binding: &BufferBinding| -> wgpu::BindGroupLayoutEntry {
                wgpu::BindGroupLayoutEntry {
                    binding: u32::from(binding.binding),
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: binding.ty,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            })
            .collect();

        let uniform_size = group.uniform_buffer_size;
        if uniform_size > BufferByteLength::from(MAX_UNIFORM_BYTES) {
            return None;
        }
        let uniform_binding = if u64::from(uniform_size) > 0 {
            group.uniform_binding
        } else {
            None
        };
        if let Some(binding) = uniform_binding {
            entries.push(wgpu::BindGroupLayoutEntry {
                binding: u32::from(binding),
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    // The whole point: one buffer, one bind group, and the
                    // per-dispatch values selected by offset.
                    has_dynamic_offset: true,
                    min_binding_size: std::num::NonZeroU64::new(u64::from(uniform_size)),
                },
                count: None,
            });
        }
        entries.sort_by_key(|entry| entry.binding);

        let layout = context
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Kernel Bind Layout"),
                entries: &entries,
            });
        let pipeline_layout =
            context
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Kernel Pipeline Layout"),
                    bind_group_layouts: &[Some(&layout)],
                    immediate_size: 0,
                });
        let pipeline = context
            .device
            .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("Kernel Pipeline"),
                layout: Some(&pipeline_layout),
                module,
                entry_point: Some(<&str>::from(entry_point)),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                cache: None,
            });

        let stride = UniformStride::new(context, uniform_size);

        Some(Self {
            pipeline,
            layout,
            buffers: group.buffer_bindings.clone(),
            uniform_binding,
            uniform_members: group.uniform_members.clone(),
            uniform_size,
            stride,
        })
    }

    /// Resolve the bind group and write this dispatch's uniform values.
    pub(super) fn prepare(
        &self,
        context: &WgpuContext,
        parameters: &PassParameters,
        uniforms: &mut UniformArena,
    ) -> std::result::Result<Prepared, KernelDispatchError> {
        let slot = (self.uniform_binding.is_some() && u64::from(self.uniform_size) > 0)
            .then(|| uniforms.allocate(context, self.stride));
        // Built fresh every dispatch. See the note at the top of this file:
        // reusing one costs wgpu the barrier between dependent passes.
        let mut entries: Vec<wgpu::BindGroupEntry> = Vec::with_capacity(self.buffers.len() + 1);
        let mut bound: Vec<(BindingIndex, Buffer)> = Vec::with_capacity(self.buffers.len());
        for binding in &self.buffers {
            let Some(PassParameter::Buffer(buffer)) = parameters.get(binding.name.parameter_name())
            else {
                return Err(KernelDispatchError::Buffer {
                    name: binding.name.clone(),
                    binding: binding.binding,
                });
            };
            bound.push((binding.binding, buffer.clone()));
        }

        for (binding, buffer) in &bound {
            entries.push(wgpu::BindGroupEntry {
                binding: u32::from(*binding),
                resource: if buffer.length() < BufferByteLength::from(buffer.buffer.size()) {
                    wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &buffer.buffer,
                        offset: 0,
                        size: std::num::NonZeroU64::new(u64::from(buffer.length())),
                    })
                } else {
                    buffer.buffer.as_entire_binding()
                },
            });
        }
        if let (Some(binding), Some(slot)) = (self.uniform_binding, &slot) {
            entries.push(wgpu::BindGroupEntry {
                binding: u32::from(binding),
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: slot.buffer(),
                    offset: 0,
                    size: std::num::NonZeroU64::new(u64::from(self.uniform_size)),
                }),
            });
        }
        entries.sort_by_key(|entry| entry.binding);

        let bind_group = context
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Kernel Bind Group"),
                layout: &self.layout,
                entries: &entries,
            });

        let dynamic_offset = match &slot {
            None => None,
            Some(slot) => {
                // Scalar and vector packing stays on the stack.
                let mut data = [0u8; MAX_UNIFORM_BYTES];
                let size =
                    usize::try_from(self.uniform_size).expect("bounded uniform size fits the host");
                let mut payload = UniformBytes::from(&mut data[..size]);
                payload
                    .pack(
                        &self.uniform_members,
                        parameters,
                        UniformPackingPolicy::Cached,
                    )
                    .map_err(KernelDispatchError::Uniform)?;
                payload.upload(context, slot.buffer(), slot.offset());
                Some(slot.dynamic_offset())
            }
        };

        Ok(Prepared {
            pipeline: self.pipeline.clone(),
            bind_group,
            dynamic_offset,
        })
    }
}
