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

use crate::prelude::*;
use fabelgeist_gpu::data::gpu::buffer::Buffer;
use fabelgeist_gpu::data::gpu::parameters::{PassParameter, PassParameters};
use fabelgeist_gpu::data::gpu::shader::{BufferBinding, UniformMember};

/// Bytes in each buffer of a batch's [`UniformArena`].
const CHUNK_BYTES: u64 = 64 * 1024;

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
    uniform_size: u64,
    /// `uniform_size` rounded up to the device's binding alignment.
    stride: u64,
}

/// The uniform values of one batch's dispatches.
///
/// A kernel used to own a ring of uniform slots that every batch wrote into
/// in turn. A ring is only safe to wrap once the batch holding a slot has been
/// submitted, and with several threads recording at once nothing guarantees
/// that: a long batch still being recorded on one thread saw its slots
/// overwritten by other threads' dispatches, and ran with their counts. So
/// each batch now writes its uniforms into buffers of its own, which live
/// exactly as long as its commands need them.
#[derive(Default)]
pub(super) struct UniformArena {
    chunks: Vec<wgpu::Buffer>,
    /// Bytes used in the last chunk.
    used: u64,
}

impl UniformArena {
    /// Room for one dispatch's uniform block: the buffer and its offset.
    fn allocate(&mut self, context: &WgpuContext, stride: u64) -> (wgpu::Buffer, u64) {
        if self.chunks.is_empty() || self.used + stride > CHUNK_BYTES {
            // Created mapped, so that it is initialised from the start rather
            // than zero-filled lazily around the submission that reads it.
            let chunk = context.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Kernel Uniform Arena"),
                size: CHUNK_BYTES,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: true,
            });
            chunk.unmap();
            self.chunks.push(chunk);
            self.used = 0;
        }
        let offset = self.used;
        self.used += stride;
        (self.chunks.last().expect("allocated above").clone(), offset)
    }
}

/// What a prepared dispatch needs at record time.
pub(super) struct Prepared {
    pub pipeline: wgpu::ComputePipeline,
    pub bind_group: wgpu::BindGroup,
    pub dynamic_offset: Option<u32>,
}

impl FastPath {
    /// Build a fast path for this kernel, or `None` if its shape does not
    /// suit one.
    pub(super) fn new(
        context: &WgpuContext,
        code: &str,
        module: &wgpu::ShaderModule,
        entry_point: &str,
        reflection: &fabelgeist_gpu::data::gpu::shader::ReflectionData,
    ) -> Option<Self> {
        // One bind group only. Everything the solver writes uses group 0, and
        // supporting more would mean a dynamic offset per group.
        if reflection.bind_groups.len() != 1 {
            return None;
        }
        let group = &reflection.bind_groups[0];
        if group.index != 0 {
            return None;
        }
        // Textures and samplers would each need their own cache key and their
        // own lifetime handling; nothing on the solver path uses them.
        if !group.texture_bindings.is_empty() || !group.sampler_bindings.is_empty() {
            return None;
        }
        let _ = code;

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

        let uniform_size = group.uniform_buffer_size as u64;
        if uniform_size as usize > MAX_UNIFORM_BYTES {
            return None;
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
                entry_point: Some(entry_point),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                cache: None,
            });

        let alignment = context.device.limits().min_uniform_buffer_offset_alignment as u64;
        let stride = uniform_size.div_ceil(alignment.max(1)) * alignment.max(1);

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
    ) -> Result<Prepared> {
        let slot = (self.uniform_binding.is_some() && self.uniform_size > 0)
            .then(|| uniforms.allocate(context, self.stride));
        // Built fresh every dispatch. See the note at the top of this file:
        // reusing one costs wgpu the barrier between dependent passes.
        let mut entries: Vec<wgpu::BindGroupEntry> = Vec::with_capacity(self.buffers.len() + 1);
        let mut bound: Vec<(u32, Buffer)> = Vec::with_capacity(self.buffers.len());
        for binding in &self.buffers {
            let Some(PassParameter::Buffer(buffer)) = parameters.get(&binding.name) else {
                return Err(anyhow!(
                    "Kernel: parameter `{}` (binding {}) is missing or is not a buffer",
                    binding.name,
                    binding.binding
                ));
            };
            bound.push((binding.binding, buffer.clone()));
        }

        for (binding, buffer) in &bound {
            entries.push(wgpu::BindGroupEntry {
                binding: *binding,
                resource: if u64::from(buffer.size) < buffer.buffer.size() {
                    wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &buffer.buffer,
                        offset: 0,
                        size: std::num::NonZeroU64::new(u64::from(buffer.size)),
                    })
                } else {
                    buffer.buffer.as_entire_binding()
                },
            });
        }
        if let (Some(binding), Some((chunk, _))) = (self.uniform_binding, &slot) {
            entries.push(wgpu::BindGroupEntry {
                binding,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: chunk,
                    offset: 0,
                    size: std::num::NonZeroU64::new(self.uniform_size),
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
            Some((chunk, offset)) => {
                let offset = *offset;
                // A uniform block is a handful of words, so it is packed on
                // the stack rather than allocated per dispatch.
                let mut data = [0u8; MAX_UNIFORM_BYTES];
                let size = self.uniform_size as usize;
                let data = &mut data[..size];
                data.fill(0);
                write_uniform(&self.uniform_members, parameters, data)?;
                context.queue.write_buffer(chunk, offset, data);
                Some(offset as u32)
            }
        };

        Ok(Prepared {
            pipeline: self.pipeline.clone(),
            bind_group,
            dynamic_offset,
        })
    }
}

/// Pack the uniform block from named parameters, by reflected offsets.
///
/// Covers the scalar, vector and matrix kinds a solver passes. Anything else
/// is reported rather than silently skipped -- a uniform quietly left at zero
/// is a wrong simulation, not an error, and those are the expensive ones.
fn write_uniform(
    members: &[UniformMember],
    parameters: &PassParameters,
    out: &mut [u8],
) -> Result<()> {
    for member in members {
        let Some(value) = parameters.get(&member.name) else {
            return Err(anyhow!(
                "Kernel: uniform `{}` was not supplied",
                member.name
            ));
        };
        let offset = member.offset as usize;

        let mut put = |values: &[f32]| {
            let end = offset + values.len() * 4;
            if end <= out.len() {
                for (index, value) in values.iter().enumerate() {
                    out[offset + index * 4..offset + index * 4 + 4]
                        .copy_from_slice(&value.to_le_bytes());
                }
            }
        };

        match value {
            PassParameter::Number(n) => put(&[*n as f32]),
            PassParameter::Unsigned(u) => {
                if offset + 4 <= out.len() {
                    out[offset..offset + 4].copy_from_slice(&u.to_le_bytes());
                }
            }
            PassParameter::Vec2(v) => put(&[v.x, v.y]),
            PassParameter::Vec3(v) => put(&[v.x, v.y, v.z]),
            PassParameter::Vec4(v) => put(&[v.x, v.y, v.z, v.w]),
            PassParameter::Mat4(m) => {
                let mut values = Vec::with_capacity(16);
                for column in 0..4 {
                    for row in 0..4 {
                        values.push(m.columns[column][row]);
                    }
                }
                put(&values);
            }
            other => {
                return Err(anyhow!(
                    "Kernel: uniform `{}` is a {:?}, which the cached dispatch path does not pack; \
                     use ComputePass::record for this kernel",
                    member.name,
                    std::mem::discriminant(other)
                ));
            }
        }
    }
    Ok(())
}
