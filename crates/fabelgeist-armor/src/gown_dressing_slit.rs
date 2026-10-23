//! A real waist-down dressing opening continuous with a bodice opening.
use crate::{DesignError, GenerateError, Millimeters};
use serde::{Deserialize, Serialize};
/// Anatomical garment side, distinct from left/right edges of a local cloth gap.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DressingOpeningSide {
    Left,
    Right,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GownDressingSlit {
    /// Nominal canonical length below the sewn waist.
    pub length: Millimeters,
    /// Tailor-rest lip separation, tapering to the shared tip.
    pub gap: Millimeters,
    pub side: DressingOpeningSide,
    /// Integral finished overlap, tapering away at the cut tip.
    pub overlap: Millimeters,
}
impl GownDressingSlit {
    pub const OVERLAP_RANGE: std::ops::RangeInclusive<u16> = 0..=30;
    pub const LENGTH_RANGE: std::ops::RangeInclusive<u16> = 80..=400;
    pub fn validate(&self) -> Result<(), GenerateError> {
        if !Self::OVERLAP_RANGE.contains(&self.overlap.0)
            || !Self::LENGTH_RANGE.contains(&self.length.0)
            || !crate::ClothSlit::GAP_RANGE.contains(&self.gap.0)
        {
            return Err(DesignError::ParametricParameters.into());
        }
        Ok(())
    }
}
