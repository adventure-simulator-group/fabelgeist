//! Checked RGBA8 decoding from the renderer's private output staging buffer.

use super::HairRenderer;
use crate::gpu::error::{HairGpuError, Result};
#[cfg(target_arch = "wasm32")]
use fabelgeist_gpu::globals::await_browser_callback;
use fabelgeist_gpu::prelude::{BufferByteLength, CallbackProgressError};
use std::{collections::TryReserveError, num::TryFromIntError};

type LayoutResult<T> = std::result::Result<T, RenderReadbackError>;

struct ReadbackLayout {
    extent: wgpu::Extent3d,
    row: u32,
    padded: u32,
    channels: Channels,
}

#[derive(Clone, Copy)]
enum Channels {
    Rgba,
    Bgra,
}

/// A rendered image could not be represented as tightly packed RGBA8 pixels.
#[derive(Debug, thiserror::Error)]
pub enum RenderReadbackError {
    #[error("RGBA8 readback does not support output format {format:?}")]
    UnsupportedFormat { format: wgpu::TextureFormat },
    #[error("RGBA8 readback cannot represent image extent {extent:?}")]
    Extent { extent: wgpu::Extent3d },
    #[error("RGBA8 readback exceeds host addresses")]
    HostAddress(#[source] TryFromIntError),
    #[error("RGBA8 readback host allocation failed")]
    Allocation(#[source] TryReserveError),
    #[error("RGBA8 mapping callback was dropped")]
    Callback(#[source] futures_channel::oneshot::Canceled),
}

impl ReadbackLayout {
    fn new(extent: wgpu::Extent3d, format: wgpu::TextureFormat) -> LayoutResult<Self> {
        let channels = match format {
            wgpu::TextureFormat::Rgba8Unorm | wgpu::TextureFormat::Rgba8UnormSrgb => Channels::Rgba,
            wgpu::TextureFormat::Bgra8Unorm | wgpu::TextureFormat::Bgra8UnormSrgb => Channels::Bgra,
            _ => return Err(RenderReadbackError::UnsupportedFormat { format }),
        };
        let bytes_per_pixel = u32::from(<image::Rgba<u8> as image::Pixel>::CHANNEL_COUNT);
        let row = extent
            .width
            .checked_mul(bytes_per_pixel)
            .ok_or(RenderReadbackError::Extent { extent })?;
        let padded = row
            .div_ceil(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT)
            .checked_mul(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT)
            .ok_or(RenderReadbackError::Extent { extent })?;
        Ok(Self {
            extent,
            row,
            padded,
            channels,
        })
    }

    fn image(&self) -> LayoutResult<image::RgbaImage> {
        let bytes = usize::try_from(u64::from(self.row) * u64::from(self.extent.height))
            .map_err(RenderReadbackError::HostAddress)?;
        let mut pixels = Vec::new();
        pixels
            .try_reserve_exact(bytes)
            .map_err(RenderReadbackError::Allocation)?;
        pixels.resize(bytes, 0);
        image::RgbaImage::from_raw(self.extent.width, self.extent.height, pixels).ok_or(
            RenderReadbackError::Extent {
                extent: self.extent,
            },
        )
    }

    fn staging_bytes(&self) -> BufferByteLength {
        // Both factors are native u32 extents, so their product fits u64.
        BufferByteLength::from(u64::from(self.padded) * u64::from(self.extent.height))
    }
}

impl From<RenderReadbackError> for HairGpuError {
    fn from(cause: RenderReadbackError) -> Self {
        Self::Backend(cause.into())
    }
}

impl From<CallbackProgressError> for HairGpuError {
    fn from(cause: CallbackProgressError) -> Self {
        match cause {
            CallbackProgressError::Poll(cause) => Self::Poll(cause),
            CallbackProgressError::Canceled(cause) => RenderReadbackError::Callback(cause).into(),
        }
    }
}

impl HairRenderer {
    /// Read the last submitted frame as tightly packed RGBA8 pixels.
    ///
    /// Resize alone does not produce a frame. RGBA8 and BGRA8 unorm/sRGB
    /// outputs are supported; other formats remain usable for rendering but
    /// cannot be decoded by this operation. BGRA channels are reordered.
    ///
    /// Native progress blocks explicitly. Browser progress yields to the
    /// event loop while driving providers that require polling. Native
    /// creation, copy and submission errors follow this device's configured
    /// error handling; successful mapping does not certify global GPU health.
    pub async fn read_rgba(&self) -> Result<image::RgbaImage> {
        let Some(targets) = self
            .targets
            .as_ref()
            .filter(|_| self.last.is_some() && self.frames > 0)
        else {
            return Err(HairGpuError::NothingRendered);
        };
        let layout = ReadbackLayout::new(targets.output.size(), self.output_format)?;
        // Complete fallible host allocation before taking the mapped view.
        // The SDK may separately allocate its read-only WebGPU mapping copy.
        let mut image = layout.image()?;
        let row = usize::try_from(layout.row).map_err(RenderReadbackError::HostAddress)?;
        let padded = usize::try_from(layout.padded).map_err(RenderReadbackError::HostAddress)?;
        let buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("hair readback"),
            size: u64::from(layout.staging_bytes()),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut encoder = self.device.create_command_encoder(&Default::default());
        encoder.copy_texture_to_buffer(
            targets.output.as_image_copy(),
            wgpu::TexelCopyBufferInfo {
                buffer: &buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(layout.padded),
                    rows_per_image: Some(layout.extent.height),
                },
            },
            layout.extent,
        );
        self.queue.submit([encoder.finish()]);
        Self::map_readback(&self.device, &buffer).await?;
        let mapped = buffer.slice(..).get_mapped_range();
        let pixels: &mut [u8] = &mut image;
        for (destination, source) in pixels
            .chunks_exact_mut(row)
            .zip(mapped.chunks_exact(padded))
        {
            destination.copy_from_slice(&source[..row]);
        }
        drop(mapped);
        buffer.unmap();
        if matches!(layout.channels, Channels::Bgra) {
            for pixel in image.pixels_mut() {
                pixel.0.swap(0, 2);
            }
        }
        Ok(image)
    }

    async fn map_readback(device: &wgpu::Device, buffer: &wgpu::Buffer) -> Result<()> {
        let (sender, receiver) = futures_channel::oneshot::channel();
        buffer
            .slice(..)
            .map_async(wgpu::MapMode::Read, move |result| {
                let _ = sender.send(result);
            });
        #[cfg(not(target_arch = "wasm32"))]
        {
            device.poll(wgpu::PollType::wait_indefinitely())?;
            receiver.await.map_err(RenderReadbackError::Callback)??;
            Ok(())
        }
        #[cfg(target_arch = "wasm32")]
        {
            let mut receiver = receiver;
            await_browser_callback(device, &mut receiver).await??;
            Ok(())
        }
    }
}
