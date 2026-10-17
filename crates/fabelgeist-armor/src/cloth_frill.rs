//! A free flared hem beyond a gathered or supported cloth band.
use crate::{ClothPleats, DesignError, GenerateError, Millimeters};
use serde::{Deserialize, Serialize};
use std::ops::RangeInclusive;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClothFrill {
    /// Axial distance from the gathered band to the free hem.
    pub length: Millimeters,
    /// Radial expansion from the band to the hem. Fine folds use fabric relief.
    pub flare: Millimeters,
    /// Broad, evenly spaced folds affecting the free edge's silhouette.
    /// Finer gathering remains part of the fabric's height field.
    pub folds: Option<ClothPleats>,
}
impl ClothFrill {
    pub const LENGTH_RANGE: RangeInclusive<u16> = 5..=60;
    pub const FLARE_RANGE: RangeInclusive<u16> = 0..=40;
    pub const FOLD_COUNT_RANGE: RangeInclusive<u8> = 8..=16;
    pub const FOLD_DEPTH_RANGE: RangeInclusive<u16> = 1..=20;
    pub fn validate(&self) -> Result<(), GenerateError> {
        if !Self::LENGTH_RANGE.contains(&self.length.0)
            || !Self::FLARE_RANGE.contains(&self.flare.0)
        {
            return Err(DesignError::ParametricParameters.into());
        }
        if let Some(folds) = &self.folds
            && (!Self::FOLD_COUNT_RANGE.contains(&folds.count.0)
                || !Self::FOLD_DEPTH_RANGE.contains(&folds.depth.0)
                || folds.depth.0 > self.flare.0)
        {
            return Err(DesignError::ParametricParameters.into());
        }
        Ok(())
    }
}
