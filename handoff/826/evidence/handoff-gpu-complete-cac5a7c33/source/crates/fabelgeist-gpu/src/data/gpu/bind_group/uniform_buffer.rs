//! Static uniform extents are checked before packing; allocation precedes upload.

use super::uniforms;
use crate::data::{
    BindGroupReflection, Buffer, BufferByteLength, BufferDefinition, BufferLabel, BufferUpload,
    BufferUse, ComputePassError, PassParameters,
};
#[cfg(not(target_arch = "wasm32"))]
use crate::globals::ValidationScope;
use crate::globals::WgpuContext;
use anyhow::Result;

pub(super) fn create(
    context: &WgpuContext,
    reflection: &BindGroupReflection,
    parameters: &PassParameters,
) -> Result<Option<Buffer>> {
    if reflection.uniform_buffer_size == BufferByteLength::default() {
        return Ok(None);
    }
    // DeviceExt previously padded the initialized allocation to this native
    // copy alignment. Keep those trailing zero bytes and the whole binding.
    let bytes = u64::from(reflection.uniform_buffer_size).div_ceil(wgpu::COPY_BUFFER_ALIGNMENT)
        * wgpu::COPY_BUFFER_ALIGNMENT;
    let limits = context.device.limits();
    let limit = limits
        .max_buffer_size
        .min(u64::from(limits.max_uniform_buffer_binding_size));
    let requested = BufferByteLength::from(bytes);
    if bytes > limit {
        return Err(ComputePassError::UniformBufferTooLarge {
            requested,
            limit: BufferByteLength::from(limit),
        }
        .into());
    }
    let mut data = uniforms::pack_uniforms(reflection, parameters)?;
    data.resize(
        usize::try_from(bytes).map_err(ComputePassError::UniformHostAddress)?,
        0,
    );
    #[cfg(not(target_arch = "wasm32"))]
    let scope = ValidationScope::new(context);
    let buffer = Buffer::new(
        context,
        requested,
        BufferDefinition::uniform()
            .with_usage(BufferUse::CopyDestination)
            .with_label(BufferLabel::from("ComputePass Uniforms")),
    );
    #[cfg(not(target_arch = "wasm32"))]
    let buffer = ComputePassError::resolve_observation(
        buffer,
        match scope {
            Some(scope) => scope.finish(),
            None => Ok(()),
        },
    );
    let buffer = buffer?;
    #[cfg(not(target_arch = "wasm32"))]
    let scope = ValidationScope::new(context);
    let written = buffer.write(context, BufferUpload::from_elements(&data));
    #[cfg(not(target_arch = "wasm32"))]
    let written = ComputePassError::resolve_observation(
        written,
        match scope {
            Some(scope) => scope.finish(),
            None => Ok(()),
        },
    );
    written?;
    Ok(Some(buffer))
}
