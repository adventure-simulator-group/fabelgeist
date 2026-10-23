//! A hem opening in a wrapped cloth surface.
use crate::{DesignError, GenerateError, Millimeters, Permille};
use serde::{Deserialize, Serialize};
use std::ops::RangeInclusive;
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClothSlit {
    /// Length upward from the free hem.
    pub length: Millimeters,
    /// Separation of the two edges at the hem, tapering to a closed tip.
    pub gap: Millimeters,
    /// Position around the cloth tube in the carrier's angular frame.
    pub rotation: Permille,
}
impl ClothSlit {
    pub const LENGTH_RANGE: RangeInclusive<u16> = 5..=400;
    pub const GAP_RANGE: RangeInclusive<u16> = 1..=20;
    pub const ROTATION_RANGE: RangeInclusive<u16> = 0..=1000;
    pub fn validate(&self) -> Result<(), GenerateError> {
        if !Self::LENGTH_RANGE.contains(&self.length.0)
            || !Self::GAP_RANGE.contains(&self.gap.0)
            || !Self::ROTATION_RANGE.contains(&self.rotation.0)
        {
            return Err(DesignError::ParametricParameters.into());
        }
        Ok(())
    }
}
