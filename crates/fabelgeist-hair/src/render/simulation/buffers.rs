//! Explicit storage ranges and copy admission for renderer deformation.

use super::{RenderSimulationError, SimulationBufferRole};
use crate::gpu::StrandLayout;
use crate::gpu::error::Result;
use crate::parameters::StrandCount;
use crate::render::{DeformGpu, HairRenderer};
use fabelgeist_gpu::prelude::{Buffer, BufferByteLength};
use fabelgeist_math::Vec3;
use std::{num::NonZeroU64, sync::Arc};
use wgpu::util::DeviceExt;

/// A checked positive native binding, distinct from its logical data extent.
pub(in crate::render) struct SimulationBuffer {
    pub buffer: Arc<wgpu::Buffer>,
    bytes: BufferByteLength,
}

impl SimulationBuffer {
    pub fn from_native(
        renderer: &HairRenderer,
        role: SimulationBufferRole,
        buffer: &Buffer,
        data: BufferByteLength,
    ) -> Result<Self> {
        let prefix = buffer.prefix(data).map_err(RenderSimulationError::from)?;
        let bytes = binding_bytes(data);
        validate(
            renderer,
            role,
            &prefix.buffer,
            bytes,
            wgpu::BufferUsages::STORAGE,
        )?;
        Ok(Self {
            buffer: prefix.buffer,
            bytes,
        })
    }

    pub fn upload(
        renderer: &HairRenderer,
        role: SimulationBufferRole,
        data: &[u8],
        usage: wgpu::BufferUsages,
    ) -> Result<Self> {
        let bytes = binding_bytes(BufferByteLength::from(data.len()));
        storage_limit(renderer, role, bytes)?;
        // All inputs are vec4 arrays. An empty logical array still needs one
        // native storage record, which no zero-population invocation accesses.
        let empty = [0u8; size_of::<[f32; 4]>()];
        let data = if data.is_empty() { &empty[..] } else { data };
        let buffer = renderer
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(label(role)),
                contents: data,
                usage,
            });
        Ok(Self {
            buffer: Arc::new(buffer),
            bytes,
        })
    }

    pub fn bytes(&self) -> BufferByteLength {
        self.bytes
    }

    pub fn resource(&self) -> wgpu::BindingResource<'_> {
        wgpu::BindingResource::Buffer(wgpu::BufferBinding {
            buffer: &self.buffer,
            offset: 0,
            // Private constructors prove at least one vec4; this is Some.
            size: NonZeroU64::new(u64::from(self.bytes)),
        })
    }

    pub fn check_identity(&self, role: SimulationBufferRole, buffer: &Buffer) -> Result<()> {
        if !Arc::ptr_eq(&self.buffer, &buffer.buffer) {
            return Err(RenderSimulationError::ForeignBuffer { role }.into());
        }
        Ok(())
    }
}

/// Convert at the native vec4 upload boundary, preserving the previous w=1.
pub(super) fn pack_positions(positions: &[Vec3]) -> Result<Vec<[f32; 4]>> {
    let mut packed = Vec::new();
    packed
        .try_reserve_exact(positions.len())
        .map_err(RenderSimulationError::from)?;
    packed.extend(positions.iter().map(|p| [p.x, p.y, p.z, 1.0]));
    Ok(packed)
}

pub(super) fn preflight(renderer: &HairRenderer) -> Result<()> {
    use wgpu::BufferUsages as U;
    for (index, level) in renderer.levels.iter().enumerate() {
        let usage = if level.layout.native_samples() == 0 {
            U::STORAGE
        } else {
            U::STORAGE | U::COPY_SRC | U::COPY_DST
        };
        validate(
            renderer,
            SimulationBufferRole::LevelPositions,
            &level.positions,
            level.layout.positions_binding_bytes(),
            usage,
        )?;
        if let Some(rest) = renderer.dynamics.as_ref().and_then(|d| d.levels.get(index)) {
            validate(
                renderer,
                SimulationBufferRole::LevelRest,
                &rest.rest,
                level.layout.positions_binding_bytes(),
                U::STORAGE | U::COPY_SRC,
            )?;
        }
    }
    let limits = renderer.device.limits();
    limit(
        SimulationBufferRole::Parameters,
        BufferByteLength::from(size_of::<DeformGpu>()),
        BufferByteLength::from(
            limits
                .max_buffer_size
                .min(u64::from(limits.max_uniform_buffer_binding_size)),
        ),
    )
}

pub(super) fn host_limits(
    renderer: &HairRenderer,
    drawn: StrandCount,
    layout: StrandLayout,
) -> Result<()> {
    for role in [
        SimulationBufferRole::BindingIndices,
        SimulationBufferRole::BindingWeights,
    ] {
        storage_limit(
            renderer,
            role,
            binding_bytes(super::binding_vector_bytes(drawn)),
        )?;
    }
    for role in [
        SimulationBufferRole::GuideRest,
        SimulationBufferRole::GuidePositions,
    ] {
        storage_limit(renderer, role, layout.positions_binding_bytes())?;
    }
    storage_limit(
        renderer,
        SimulationBufferRole::Body,
        binding_bytes(BufferByteLength::default()),
    )
}

fn binding_bytes(data: BufferByteLength) -> BufferByteLength {
    data.max(BufferByteLength::from(size_of::<[f32; 4]>()))
}

fn storage_limit(
    renderer: &HairRenderer,
    role: SimulationBufferRole,
    bytes: BufferByteLength,
) -> Result<()> {
    let limits = renderer.device.limits();
    limit(
        role,
        bytes,
        BufferByteLength::from(
            limits
                .max_buffer_size
                .min(u64::from(limits.max_storage_buffer_binding_size)),
        ),
    )
}

fn limit(
    role: SimulationBufferRole,
    required: BufferByteLength,
    limit: BufferByteLength,
) -> Result<()> {
    if required > limit {
        return Err(RenderSimulationError::Limit {
            role,
            required,
            limit,
        }
        .into());
    }
    Ok(())
}

fn validate(
    renderer: &HairRenderer,
    role: SimulationBufferRole,
    buffer: &wgpu::Buffer,
    required: BufferByteLength,
    usage: wgpu::BufferUsages,
) -> Result<()> {
    storage_limit(renderer, role, required)?;
    let physical = BufferByteLength::from(buffer.size());
    if required > physical {
        return Err(RenderSimulationError::Extent {
            role,
            required,
            physical,
        }
        .into());
    }
    if !buffer.usage().contains(usage) {
        return Err(RenderSimulationError::Usage {
            role,
            required: usage,
            actual: buffer.usage(),
        }
        .into());
    }
    Ok(())
}

fn label(role: SimulationBufferRole) -> &'static str {
    match role {
        SimulationBufferRole::BindingIndices => "hair bind ids",
        SimulationBufferRole::BindingWeights => "hair bind weights",
        SimulationBufferRole::GuideRest => "hair guide rest",
        SimulationBufferRole::GuidePositions => "hair guide now",
        SimulationBufferRole::Body => "hair no body",
        SimulationBufferRole::LevelPositions => "hair positions",
        SimulationBufferRole::LevelRest => "hair rest pose",
        SimulationBufferRole::Parameters => "hair deform params",
    }
}
