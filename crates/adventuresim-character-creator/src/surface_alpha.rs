//! Shared opacity modes for preview materials and exported surface maps.
use bevy::prelude::AlphaMode;

const MASK_ALPHA_CUTOFF: f32 = 0.5;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SurfaceAlpha {
    Opaque,
    Mask,
    Blend,
}

impl From<SurfaceAlpha> for AlphaMode {
    fn from(alpha: SurfaceAlpha) -> Self {
        match alpha {
            SurfaceAlpha::Opaque => Self::Opaque,
            SurfaceAlpha::Mask => Self::Mask(MASK_ALPHA_CUTOFF),
            SurfaceAlpha::Blend => Self::Blend,
        }
    }
}
