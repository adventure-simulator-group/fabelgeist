//! The fixed scalp recipe's positive mip dimensions and padded native addresses.

use anyhow::{Context, Result};
use fabelgeist_gpu::prelude::{
    BufferByteLength, BufferByteOffset, BufferElementCount, BufferElementStride, PassParameter,
};
use std::num::NonZeroU32;

/// About an eighth of a millimetre per texel on the fitted scalp.
const SIZE: u32 = 2048;
pub(super) const TEXEL_BYTES: u32 = size_of::<u32>() as u32;
const ROW_TEXELS: u32 = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT / TEXEL_BYTES;

/// A positive span in scalp texels, including a shadow block or blur radius.
#[derive(Clone, Copy, Debug)]
pub(in crate::hair) struct ScalpTexelSpan(NonZeroU32);

/// The admitted four-byte storage layout of one fixed scalp mip chain.
#[derive(Debug)]
pub struct ScalpTextureLayout {
    stride: BufferElementStride,
    levels: Vec<ScalpMipLevel>,
}

/// One positive square mip's byte start in its padded storage chain.
#[derive(Clone, Copy, Debug)]
pub struct ScalpMipLevel {
    side: ScalpTexelSpan,
    offset: BufferByteOffset,
}

impl ScalpTexelSpan {
    /// Admit a native integer span, clamped to at least one texel.
    pub(super) fn at_least_one(texels: u32) -> Self {
        Self(NonZeroU32::new(texels).unwrap_or(NonZeroU32::MIN))
    }

    /// Native texel arithmetic or shader/image dimensions.
    pub(in crate::hair) fn native_texels(self) -> u32 {
        self.0.get()
    }
}

impl From<ScalpTexelSpan> for PassParameter {
    fn from(span: ScalpTexelSpan) -> Self {
        Self::Unsigned(span.native_texels())
    }
}

impl ScalpTextureLayout {
    pub(super) fn new() -> Result<Self> {
        let stride = BufferElementStride::try_from(TEXEL_BYTES)?;
        let mut side = ScalpTexelSpan(NonZeroU32::new(SIZE).context("empty scalp texture recipe")?);
        let mut levels = Vec::new();
        levels.try_reserve_exact((SIZE.ilog2() + 1) as usize)?;
        let mut offset = 0u64;
        loop {
            levels.push(ScalpMipLevel {
                side,
                offset: BufferByteOffset::from(offset),
            });
            let row = side.native_texels().max(ROW_TEXELS);
            offset += u64::from(row) * u64::from(side.native_texels()) * u64::from(TEXEL_BYTES);
            let Some(next) = NonZeroU32::new(side.native_texels() / 2) else {
                break;
            };
            side = ScalpTexelSpan(next);
        }
        // The complete chain bounds every level's host and native word address.
        usize::try_from(offset)?;
        isize::try_from(offset)?;
        u32::try_from(offset / u64::from(TEXEL_BYTES))?;
        Ok(Self { stride, levels })
    }

    /// Every mip, largest first, ending at one texel per side.
    pub fn levels(&self) -> &[ScalpMipLevel] {
        &self.levels
    }

    /// The factory always admits a positive base and at least one level.
    pub fn base(&self) -> &ScalpMipLevel {
        &self.levels[0]
    }

    /// Exact padded storage bytes, including the final mip's row padding.
    pub fn bytes(&self) -> BufferByteLength {
        let last = &self.levels[self.levels.len() - 1];
        let end = u64::from(last.offset)
            + u64::from(self.row_bytes(last)) * u64::from(last.side_texels());
        BufferByteLength::from(end)
    }

    /// Native copy row pitch; the fixed power-of-two sides preserve alignment.
    pub fn row_bytes(&self, level: &ScalpMipLevel) -> BufferByteLength {
        level.row_elements().byte_length(self.stride)
    }

    pub(super) fn element_bytes(&self, elements: BufferElementCount) -> BufferByteLength {
        elements.byte_length(self.stride)
    }

    /// Project the admitted byte address to the shader's packed-u32 array ABI.
    pub(super) fn native_word_offset(&self, level: &ScalpMipLevel) -> u32 {
        // All starts are word-aligned and bounded by new()'s checked full extent.
        (u64::from(level.offset) / u64::from(self.stride.byte_length())) as u32
    }
}

impl ScalpMipLevel {
    /// Native image or shader side, in texels.
    pub fn side_texels(&self) -> u32 {
        self.side.native_texels()
    }

    pub(in crate::hair) fn side(&self) -> ScalpTexelSpan {
        self.side
    }

    pub fn offset(&self) -> BufferByteOffset {
        self.offset
    }

    /// One native storage record per visible texel.
    pub fn elements(&self) -> BufferElementCount {
        // Private levels descend from 2048: the square fits u32 addresses.
        BufferElementCount::from(self.side_texels() * self.side_texels())
    }

    pub(super) fn row_elements(&self) -> BufferElementCount {
        BufferElementCount::from(self.side_texels().max(ROW_TEXELS))
    }
}
