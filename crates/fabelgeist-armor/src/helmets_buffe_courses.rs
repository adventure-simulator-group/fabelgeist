use crate::{DesignError, Millimeters, Permille};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BuffeCourses {
    pub plate_count: u8,
    /// First seam's height, measured upward from chin to sight edge.
    pub lower_boundary: Permille,
    /// Second seam's height; used when there are three faceplates.
    pub upper_boundary: Permille,
    /// Physical vertical overlap at each descending edge.
    pub overlap: Millimeters,
    /// Additional air between courses, beyond twice the plate gauge.
    pub lap_clearance: Millimeters,
    /// Downward median point at the internal seams; outer edges stay fixed.
    pub boundary_drop: Millimeters,
}

impl Default for BuffeCourses {
    fn default() -> Self {
        Self {
            plate_count: 3,
            lower_boundary: Permille(350),
            upper_boundary: Permille(700),
            overlap: Millimeters(5),
            lap_clearance: Millimeters(1),
            boundary_drop: Millimeters(12),
        }
    }
}

impl BuffeCourses {
    pub fn validate(&self) -> Result<(), DesignError> {
        if !(2..=3).contains(&self.plate_count)
            || !(200..=700).contains(&self.lower_boundary.0)
            || !(450..=850).contains(&self.upper_boundary.0)
            || (self.plate_count == 3 && self.upper_boundary.0 < self.lower_boundary.0 + 180)
            || !(2..=12).contains(&self.overlap.0)
            || self.lap_clearance.0 > 4
            || self.boundary_drop.0 > 25
        {
            return Err(DesignError::ParametricParameters);
        }
        Ok(())
    }
}
