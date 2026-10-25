//! Raw texture bytes share the buffer mapping and typed host-copy owners.

use super::transfer::{TextureStagingLayout, TextureTransferError, TextureTransferLayout};
use crate::data::ComputePassError;
use crate::data::gpu::buffer::{
    BufferByteLength, BufferReadError, MappingExtent, MappingRequest, Readback,
};
use crate::globals::WgpuContext;
use std::sync::Arc;
#[cfg(not(target_arch = "wasm32"))]
use std::time::Duration;

#[cfg(not(target_arch = "wasm32"))]
const TEXTURE_MAPPING_POLL_INTERVAL: Duration = Duration::from_millis(1);

pub(super) struct TextureReadSource<'a> {
    pub texture: &'a wgpu::Texture,
    pub origin: wgpu::Origin3d,
    pub layout: TextureTransferLayout,
    pub kind: TextureReadKind,
}

pub(super) enum TextureReadKind {
    Plane,
    Volume,
    View,
}

impl TextureReadSource<'_> {
    pub async fn read<T: bytemuck::AnyBitPattern>(
        &self,
        context: &WgpuContext,
    ) -> anyhow::Result<Vec<T>> {
        if u64::from(self.layout.bytes()) == 0 {
            return Ok(Readback::<T>::new(self.layout.bytes())?.copy_from(&[])?);
        }
        let layout = TextureStagingLayout::new(self.layout, &context.device)?;
        let host = Readback::<T>::new(self.layout.bytes())?;
        let mut packed = layout.packed_storage()?;
        let staging = context
            .observe_validation(|| {
                context.device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some(self.kind.staging_label()),
                    size: u64::from(layout.bytes()),
                    usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                })
            })
            .map_err(BufferReadError::Native)?;
        let staging = Arc::new(staging);
        let mut encoder = context
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some(self.kind.encoder_label()),
            });
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: self.texture,
                mip_level: 0,
                origin: self.origin,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &staging,
                layout: layout.buffer_layout(),
            },
            self.layout.extent(),
        );
        context
            .submit(encoder)
            .map_err(|cause| ComputePassError::Native(Arc::new(cause)))?;
        let extent =
            MappingExtent::for_prefix(layout.bytes(), BufferByteLength::from(staging.size()))?
                .ok_or(TextureTransferError::EmptyStaging)?;
        let mut mapping = MappingRequest::request(context, staging, extent)?;
        #[cfg(not(target_arch = "wasm32"))]
        while mapping.poll_complete(context)?.is_none() {
            fabelgeist_timer::sleep(TEXTURE_MAPPING_POLL_INTERVAL).await;
        }
        #[cfg(target_arch = "wasm32")]
        mapping.complete(context).await?;
        let mapped = mapping.mapped_range();
        let unpacked = layout.unpack(&mapped, &mut packed);
        drop(mapped);
        unpacked?;
        mapping.release(context)?;
        Ok(host.copy_from(&packed)?)
    }
}

impl TextureReadKind {
    fn staging_label(&self) -> &'static str {
        match self {
            Self::Plane => "Texture2d Staging Buffer",
            Self::Volume => "Texture3d Staging Buffer",
            Self::View => "TextureView Staging Buffer",
        }
    }

    fn encoder_label(&self) -> &'static str {
        match self {
            Self::Plane => "Texture2d Read Encoder",
            Self::Volume => "Texture3d Read Encoder",
            Self::View => "TextureView Read Encoder",
        }
    }
}
