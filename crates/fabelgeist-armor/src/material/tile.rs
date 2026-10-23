//! Checked square metal tiles and their native storage populations.

use fabelgeist_gpu::prelude::{BufferElementCount, PassParameter};

const SUPPORTED_SIDES: std::ops::RangeInclusive<u32> = 32..=1024;

/// Pixels along either side of a supported square metal bake.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MetalTextureSize(u32);

#[derive(Clone, Copy, Debug, thiserror::Error)]
#[error("Texture size must be 32–1024")]
pub struct MetalTextureSizeError;

/// A native word layout cannot address the requested tile storage.
#[derive(Clone, Copy, Debug, thiserror::Error)]
#[error("metal texture word layout exceeds native addresses")]
pub struct MetalTextureLayoutError;

impl MetalTextureSize {
    /// Preview and export density: about a quarter millimetre per texel.
    pub const PREVIEW: Self = Self(1024);

    pub fn texels(self) -> BufferElementCount {
        // The admitted side is at most 1024, so its square fits native u32.
        BufferElementCount::from(self.0 * self.0)
    }

    /// Native storage words per texel, such as paired slopes or finish lanes.
    pub(crate) fn words(
        self,
        words_per_texel: u32,
    ) -> Result<BufferElementCount, MetalTextureLayoutError> {
        self.texels()
            .checked_mul(words_per_texel)
            .ok_or(MetalTextureLayoutError)
    }
}

impl TryFrom<u32> for MetalTextureSize {
    type Error = MetalTextureSizeError;

    fn try_from(side: u32) -> Result<Self, Self::Error> {
        if !SUPPORTED_SIDES.contains(&side) {
            return Err(MetalTextureSizeError);
        }
        Ok(Self(side))
    }
}

impl From<MetalTextureSize> for u32 {
    fn from(size: MetalTextureSize) -> Self {
        size.0
    }
}

impl From<MetalTextureSize> for f32 {
    fn from(size: MetalTextureSize) -> Self {
        size.0 as f32
    }
}

impl From<MetalTextureSize> for PassParameter {
    fn from(size: MetalTextureSize) -> Self {
        Self::Unsigned(size.0)
    }
}
