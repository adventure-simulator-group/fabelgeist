//! Finishing tape shared by bound garment openings and sewn edges.
use crate::{DesignError, GenerateError, Micrometers, Millimeters, TextileAppearance};
use serde::{Deserialize, Serialize};
use std::ops::RangeInclusive;

/// Finished width and gauge of each folded face of a cloth tape.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClothTape {
    pub width: Millimeters,
    pub thickness: Micrometers,
    pub fabric: TextileAppearance,
}
impl ClothTape {
    pub const WIDTH_RANGE: RangeInclusive<u16> = 1..=30;
    pub const THICKNESS_RANGE: RangeInclusive<u16> = 100..=1000;
    pub fn validate(&self) -> Result<(), GenerateError> {
        self.fabric.validate()?;
        if !Self::WIDTH_RANGE.contains(&self.width.0)
            || !Self::THICKNESS_RANGE.contains(&self.thickness.0)
        {
            return Err(DesignError::ParametricParameters.into());
        }
        Ok(())
    }
}
