use crate::engraving::{Engraving, EngravingError};
use serde::{Deserialize, Serialize};

pub use textures::{DepthMap, MetalTextureDataError, MetalTextureMap, MetalTextureMaps, MetalTextures};
pub use tile::{MetalTextureLayoutError, MetalTextureSize, MetalTextureSizeError};

mod textures;
mod tile;

/// Worked steel: a base color and roughness with a tiling map of its finish --
/// the undulation left by planishing, the grain of its polish, uneven gloss
/// and scratches -- and an optional engraving cut into the same tile. Every
/// rigid armor piece is shaded with it, whatever its construction.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Metal {
    pub color: [f32; 3],
    pub roughness: f32,
    pub scratch_density: u32,
    pub scratch_length: f32,
    pub scratch_width: f32,
    pub scratch_depth: f32,
    pub scratch_angle: f32,
    pub scratch_spread: f32,
    pub seed: u32,
    /// Peak-to-peak height of the planishing undulation, metres.
    pub waviness: f32,
    /// Strength of the polishing grain, 0 to 1.
    pub grain: f32,
    /// Roughness varied across the surface by handling.
    pub smudge: f32,
    pub engraving: Option<Engraving>,
}
/// An immutable material recipe admitted for native baking.
#[derive(Clone, Copy, Debug)]
pub(crate) struct ValidatedMetal<'a>(&'a Metal);

/// A procedural metal recipe fails its material admission contract.
#[derive(Debug, thiserror::Error)]
pub enum MetalError {
    #[error("Invalid metal parameters")]
    Parameters,
    #[error(transparent)]
    Engraving(#[from] EngravingError),
}

impl Default for Metal {
    /// Polished steel.
    fn default() -> Self {
        Self {
            color: [0.769, 0.776, 0.776],
            roughness: 0.20,
            scratch_density: 500,
            scratch_length: 0.03,
            scratch_width: 0.65,
            scratch_depth: 0.02,
            scratch_angle: 0.4,
            scratch_spread: 2.0,
            seed: 1544,
            waviness: 0.0004,
            grain: 0.35,
            smudge: 0.08,
            engraving: None,
        }
    }
}
impl Metal {
    /// Texture repeats per metre of armor surface.
    pub const TILES_PER_METRE: f32 = 4.0;
    pub const MAX_WAVINESS: f32 = 0.002;
    /// Roughness added at the bottom of the deepest scratch.
    pub(crate) const SCRATCH_ROUGHNESS: f32 = 0.45;

    pub fn validate(&self) -> Result<(), MetalError> {
        let values = [
            (self.roughness, 0.08, 0.9),
            (self.scratch_length, 0.005, 0.4),
            (self.scratch_width, 0.5, 3.0),
            (self.scratch_depth, 0.0, 1.0),
            (
                self.scratch_angle,
                -std::f32::consts::PI,
                std::f32::consts::PI,
            ),
            (self.scratch_spread, 0.0, std::f32::consts::PI),
            (self.waviness, 0.0, Self::MAX_WAVINESS),
            (self.grain, 0.0, 1.0),
            (self.smudge, 0.0, 0.4),
        ];
        if self.scratch_density > 3000
            || values
                .iter()
                .any(|(x, l, h)| !x.is_finite() || x < l || x > h)
            || self
                .color
                .iter()
                .any(|x| !x.is_finite() || !(0.0..=1.0).contains(x))
        {
            return Err(MetalError::Parameters);
        }
        if let Some(engraving) = &self.engraving {
            engraving.validate()?;
        }
        Ok(())
    }
}

impl<'a> ValidatedMetal<'a> {
    pub(crate) fn new(metal: &'a Metal) -> Result<Self, MetalError> {
        metal.validate()?;
        Ok(Self(metal))
    }

    /// Native parameter projection; the immutable borrow preserves admission.
    pub(crate) fn source(self) -> &'a Metal {
        self.0
    }
}
