//! Overlapping horizontal torso courses, distinct from decorative fluting.
use crate::{DesignError, Millimeters, Permille};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum BreastplateConstruction {
    Solid,
    Anime(AnimeDesign),
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AnimeDesign {
    /// Number of lower courses beneath the larger upper plate.
    pub lame_count: u8,
    /// Fraction of the shortest torso column occupied by the lower courses.
    pub articulated_height: Permille,
    pub overlap: Millimeters,
    /// Rise of the front course edges per metre from the medial line.
    pub chevron_slope: Permille,
    pub rear_chevron_slope: Permille,
    /// Outward lift at each upper plate's lower overlapping edge.
    pub lap_lift: Millimeters,
}

impl Default for AnimeDesign {
    fn default() -> Self {
        Self {
            lame_count: 6,
            articulated_height: Permille(800),
            overlap: Millimeters(8),
            chevron_slope: Permille(300),
            rear_chevron_slope: Permille(120),
            lap_lift: Millimeters(7),
        }
    }
}

impl AnimeDesign {
    pub(crate) fn validate(&self, gauge: Millimeters) -> Result<(), DesignError> {
        if !(3..=10).contains(&self.lame_count)
            || !(450..=950).contains(&self.articulated_height.0)
            || !(4..=15).contains(&self.overlap.0)
            || self.chevron_slope.0 > 500
            || self.rear_chevron_slope.0 > 500
            || !(4..=12).contains(&self.lap_lift.0)
            || self.lap_lift.0 < gauge.0 * 2
        {
            return Err(DesignError::BreastplateShape);
        }
        Ok(())
    }
}
