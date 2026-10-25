//! Read a complete admitted scalp chain on its actual creating device.

use super::layout::TEXEL_BYTES;
use super::{HairGpu, ScalpTextureChain};
use crate::hair::AtlasLevel;
use anyhow::Result;
use fabelgeist_hair::gpu::error::HairGpuError;

impl ScalpTextureChain {
    pub(super) async fn read(&self, gpu: &HairGpu) -> Result<Vec<AtlasLevel>> {
        let context = gpu.context();
        if &self.device != context.device.as_ref() {
            return Err(HairGpuError::DeviceMismatch.into());
        }
        let logical = self.buffer.prefix(self.layout.bytes())?;
        let texels: Vec<u32> = logical.read(context).await?;
        let bytes: &[u8] = bytemuck::cast_slice(&texels);
        self.layout
            .levels()
            .iter()
            .map(|level| {
                let side = level.side_texels() as usize;
                let offset = usize::try_from(u64::from(level.offset()))?;
                let row_bytes = usize::try_from(u64::from(self.layout.row_bytes(level)))?;
                let rgba = (0..side)
                    .flat_map(|row| {
                        let start = offset + row * row_bytes;
                        bytes[start..start + side * TEXEL_BYTES as usize]
                            .iter()
                            .copied()
                    })
                    .collect();
                Ok(AtlasLevel {
                    width: level.side_texels(),
                    height: level.side_texels(),
                    rgba,
                })
            })
            .collect()
    }
}
