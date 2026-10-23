//! Screen-space quantities and the shading choices carried into a renderer.

const MIN_SHADOW_MAP_TEXELS: u32 = 16;

/// A fractional distance on screen, in pixels. Numerical policy calculations
/// retain signed and non-finite input semantics; render admission checks the
/// effective finite values before publishing a frame.
#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd)]
pub struct PixelSpan(f32);

/// Number of samples in a temporal running mean. Zero is an unrendered mean;
/// a requested target always requires at least one sample.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct AccumulationFrames(u32);

/// Width and height in texels of a square deep-opacity map. The existing
/// renderer minimum is retained; the native device owns upper size limits.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ShadowMapSize(u32);

/// How the hair is shaded and accumulated.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Quality {
    pub shading: HairShading,
    pub shadows: ShadowQuality,
    pub accumulate: AccumulationFrames,
    /// Narrower strands fade instead of becoming thinner.
    pub min_pixels: PixelSpan,
}

/// The real-time hair reflectance model.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HairShading {
    /// Marschner's three lobes with multiple scattering, using Karis' fit.
    Marschner,
    /// Kajiya-Kay with two shifted highlights.
    KajiyaKay,
}

/// Live key-light shadows, or only the groom's baked occlusion.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShadowQuality {
    Baked,
    DeepOpacity { size: ShadowMapSize },
}

impl PixelSpan {
    pub const fn from_pixels(pixels: f32) -> Self {
        Self(pixels)
    }

    /// Scalar for a screen-space numerical kernel or native shader uniform.
    pub const fn pixels(self) -> f32 {
        self.0
    }
}

impl AccumulationFrames {
    pub const fn from_count(count: u32) -> Self {
        Self(count)
    }

    /// Sample count for output or the temporal numerical kernel.
    pub const fn count(self) -> u32 {
        self.0
    }

    pub fn target(self) -> Self {
        Self(self.0.max(1))
    }

    pub fn next(self) -> Self {
        Self(self.0.saturating_add(1))
    }

    pub fn next_weight(self) -> f32 {
        1.0 / (u64::from(self.0) + 1) as f32
    }
}

impl ShadowMapSize {
    /// Keep the renderer's minimum of sixteen texels per edge.
    pub fn from_texels(texels: u32) -> Self {
        Self(texels.max(MIN_SHADOW_MAP_TEXELS))
    }

    /// Edge extent at the native texture or numerical-kernel boundary.
    pub const fn texels(self) -> u32 {
        self.0
    }
}

impl ShadowQuality {
    pub const fn size(self) -> Option<ShadowMapSize> {
        match self {
            Self::Baked => None,
            Self::DeepOpacity { size } => Some(size),
        }
    }

    /// Native shader and pass-enable boundary.
    pub const fn enabled(self) -> bool {
        matches!(self, Self::DeepOpacity { .. })
    }
}
