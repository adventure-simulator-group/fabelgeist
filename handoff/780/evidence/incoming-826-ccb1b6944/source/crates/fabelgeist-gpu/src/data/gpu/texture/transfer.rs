//! Checked tight upload rows and padded texture-to-buffer copy volumes.

use super::TextureFormat;
use crate::data::gpu::buffer::{BufferByteLength, BufferReadError};
use crate::globals::WgpuContext;
use std::{error::Error, fmt, num::TryFromIntError};

type TransferResult<T> = Result<T, TextureTransferError>;

#[derive(Clone, Copy)]
pub(super) struct TextureTransferLayout {
    extent: wgpu::Extent3d,
    row: BufferByteLength,
    bytes: BufferByteLength,
}

pub(super) struct TextureStagingLayout {
    transfer: TextureTransferLayout,
    pitch: BufferByteLength,
    bytes: BufferByteLength,
}

/// Texture rows or staging volumes cannot fit their native byte ranges.
#[derive(Debug)]
pub enum TextureTransferError {
    VolumeOverflow,
    NativeRow(TryFromIntError),
    EmptyStaging,
    StagingLimit {
        required: BufferByteLength,
        limit: BufferByteLength,
    },
}

impl TextureTransferLayout {
    pub fn new(extent: wgpu::Extent3d, format: TextureFormat) -> TransferResult<Self> {
        let row = u64::from(extent.width) * u64::from(format.pixel_size());
        u32::try_from(row).map_err(TextureTransferError::NativeRow)?;
        let bytes = Self::volume(extent, BufferByteLength::from(row))?;
        Ok(Self {
            extent,
            row: BufferByteLength::from(row),
            bytes,
        })
    }

    pub fn extent(self) -> wgpu::Extent3d {
        self.extent
    }

    pub fn bytes(self) -> BufferByteLength {
        self.bytes
    }

    /// Queue writes use tight rows; the encoder's row alignment is not required.
    pub fn write(
        self,
        context: &WgpuContext,
        texture: &wgpu::Texture,
        origin: wgpu::Origin3d,
        data: &[u8],
    ) -> anyhow::Result<()> {
        context.observe_validation(|| {
            context.queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture,
                    mip_level: 0,
                    origin,
                    aspect: wgpu::TextureAspect::All,
                },
                data,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    // Admission proves this native row field fits u32.
                    bytes_per_row: Some(u64::from(self.row) as u32),
                    rows_per_image: Some(self.extent.height),
                },
                self.extent,
            );
        })?;
        Ok(())
    }

    fn volume(extent: wgpu::Extent3d, pitch: BufferByteLength) -> TransferResult<BufferByteLength> {
        u64::from(pitch)
            .checked_mul(u64::from(extent.height))
            .and_then(|plane| plane.checked_mul(u64::from(extent.depth_or_array_layers)))
            .map(BufferByteLength::from)
            .ok_or(TextureTransferError::VolumeOverflow)
    }
}

impl TextureStagingLayout {
    pub fn new(transfer: TextureTransferLayout, device: &wgpu::Device) -> TransferResult<Self> {
        if u64::from(transfer.bytes) == 0 {
            return Err(TextureTransferError::EmptyStaging);
        }
        let alignment = u64::from(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT);
        let pitch = u64::from(transfer.row)
            .div_ceil(alignment)
            .checked_mul(alignment)
            .ok_or(TextureTransferError::VolumeOverflow)?;
        u32::try_from(pitch).map_err(TextureTransferError::NativeRow)?;
        let bytes = TextureTransferLayout::volume(transfer.extent, BufferByteLength::from(pitch))?;
        let limit = BufferByteLength::from(device.limits().max_buffer_size);
        if bytes > limit {
            return Err(TextureTransferError::StagingLimit {
                required: bytes,
                limit,
            });
        }
        Ok(Self {
            transfer,
            pitch: BufferByteLength::from(pitch),
            bytes,
        })
    }

    pub fn bytes(&self) -> BufferByteLength {
        self.bytes
    }

    pub fn buffer_layout(&self) -> wgpu::TexelCopyBufferLayout {
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            // Padded native row width was checked at staging admission.
            bytes_per_row: Some(u64::from(self.pitch) as u32),
            rows_per_image: Some(self.transfer.extent.height),
        }
    }

    /// Reserve packed host storage before a native mapped view is taken.
    pub fn packed_storage(&self) -> Result<Vec<u8>, BufferReadError> {
        usize::try_from(u64::from(self.bytes)).map_err(BufferReadError::HostAddress)?;
        let bytes = usize::try_from(u64::from(self.transfer.bytes))
            .map_err(BufferReadError::HostAddress)?;
        let mut packed = Vec::new();
        packed
            .try_reserve_exact(bytes)
            .map_err(BufferReadError::HostAllocation)?;
        Ok(packed)
    }

    pub fn unpack(&self, mapped: &[u8], packed: &mut Vec<u8>) -> Result<(), BufferReadError> {
        let bytes = usize::try_from(u64::from(self.bytes)).map_err(BufferReadError::HostAddress)?;
        let data = mapped.get(..bytes).ok_or(BufferReadError::Truncated {
            requested: self.bytes,
            available: BufferByteLength::from(mapped.len()),
        })?;
        // The positive pitch and row fit native u32, but host usize may differ.
        let pitch = usize::try_from(u64::from(self.pitch)).map_err(BufferReadError::HostAddress)?;
        let row =
            usize::try_from(u64::from(self.transfer.row)).map_err(BufferReadError::HostAddress)?;
        // Admission proves pitch >= row and the complete padded volume is an
        // exact number of rows. Their linear order is z-major, then row-major.
        for padded in data.chunks_exact(pitch) {
            packed.extend_from_slice(&padded[..row]);
        }
        Ok(())
    }
}

impl fmt::Display for TextureTransferError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::VolumeOverflow => formatter.write_str("Texture transfer byte volume overflows"),
            Self::NativeRow(cause) => write!(formatter, "Texture transfer native row: {cause}"),
            Self::EmptyStaging => formatter.write_str("Texture staging requires a nonempty volume"),
            Self::StagingLimit { required, limit } => write!(
                formatter,
                "Texture staging needs {required} bytes, device limit is {limit}"
            ),
        }
    }
}

impl Error for TextureTransferError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::NativeRow(cause) => Some(cause),
            _ => None,
        }
    }
}
