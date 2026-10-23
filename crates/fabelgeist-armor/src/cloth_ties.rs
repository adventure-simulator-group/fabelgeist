//! Reusable narrow cloth tapes tied in an even bow at a garment opening.
use crate::{DesignError, GenerateError, Micrometers, Millimeters, TextileAppearance};
use serde::{Deserialize, Serialize};
use std::ops::RangeInclusive;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClothTies {
    pub width: Millimeters,
    pub tail_length: Millimeters,
    pub bow_span: Millimeters,
    pub thickness: Micrometers,
    pub fabric: TextileAppearance,
}

impl ClothTies {
    pub const WIDTH_RANGE: RangeInclusive<u16> = 4..=20;
    pub const TAIL_LENGTH_RANGE: RangeInclusive<u16> = 30..=300;
    pub const BOW_SPAN_RANGE: RangeInclusive<u16> = 15..=60;
    pub const THICKNESS_RANGE: RangeInclusive<u16> = 100..=800;

    pub fn validate(&self) -> Result<(), GenerateError> {
        self.fabric.validate()?;
        if !Self::WIDTH_RANGE.contains(&self.width.0)
            || !Self::TAIL_LENGTH_RANGE.contains(&self.tail_length.0)
            || !Self::BOW_SPAN_RANGE.contains(&self.bow_span.0)
            || !Self::THICKNESS_RANGE.contains(&self.thickness.0)
            || u32::from(self.width.0) * 3 > u32::from(self.bow_span.0)
        {
            return Err(DesignError::ParametricParameters.into());
        }
        Ok(())
    }
}
