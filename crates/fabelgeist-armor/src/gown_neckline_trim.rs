//! Raised band applied along a closed gown neckline.
use std::ops::RangeInclusive;

use serde::{Deserialize, Serialize};

use crate::{DesignError, GenerateError, Millimeters, TextileAppearance};

/// Braid or guard laid on the face cloth from the neckline edge inward. The
/// band takes its front width where it hangs below the front of the opening
/// and its shoulder width up the sides, over the shoulders and at the back.
/// Its chart runs once around the neckline; across, one unit is the front
/// width, so a repeated ornament keeps its scale where the band narrows.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GownNecklineTrim {
    pub shoulder_width: Millimeters,
    pub front_width: Millimeters,
    /// Height of the band's crown above the face cloth.
    pub relief: Millimeters,
    pub fabric: TextileAppearance,
}

impl GownNecklineTrim {
    pub const WIDTH_RANGE: RangeInclusive<u16> = 5..=160;
    pub const RELIEF_RANGE: RangeInclusive<u16> = 1..=12;

    pub(crate) fn validate(&self) -> Result<(), GenerateError> {
        self.fabric.validate()?;
        if !Self::WIDTH_RANGE.contains(&self.shoulder_width.0)
            || !Self::WIDTH_RANGE.contains(&self.front_width.0)
            || self.front_width.0 < self.shoulder_width.0
            || !Self::RELIEF_RANGE.contains(&self.relief.0)
        {
            return Err(DesignError::ParametricParameters.into());
        }
        Ok(())
    }
}
