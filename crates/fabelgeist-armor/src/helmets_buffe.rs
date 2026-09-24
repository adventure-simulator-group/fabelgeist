//! A removable face defense under a burgonet's peak, separate from its skull.
use crate::{DesignError, Millimeters, Milliradians, Permille, VisorBreaths};
use serde::{Deserialize, Serialize};
#[path = "helmets_buffe_courses.rs"]
mod courses;
pub use courses::BuffeCourses;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BuffeDesign {
    pub breaths: Option<VisorBreaths>,
    pub courses: Option<BuffeCourses>,
    pub sight_gap: Millimeters,
    pub face_projection: Millimeters,
    pub chin_width: Permille,
    pub throat_depth: Permille,
    pub neck_drop: Millimeters,
    pub side_wrap: Milliradians,
    pub medial_ridge: Millimeters,
    pub ridge_sharpness: Permille,
    pub chin_point: Millimeters,
}

impl Default for BuffeDesign {
    fn default() -> Self {
        Self {
            breaths: None,
            courses: None,
            sight_gap: Millimeters(8),
            face_projection: Millimeters(18),
            chin_width: Permille(850),
            throat_depth: Permille(650),
            neck_drop: Millimeters(12),
            side_wrap: Milliradians(1800),
            medial_ridge: Millimeters(10),
            ridge_sharpness: Permille(0),
            chin_point: Millimeters(35),
        }
    }
}

impl BuffeDesign {
    pub(crate) fn validate(&self) -> Result<(), DesignError> {
        if !(5..=20).contains(&self.sight_gap.0)
            || self.face_projection.0 > 40
            || !(500..=1100).contains(&self.chin_width.0)
            || !(550..=1000).contains(&self.throat_depth.0)
            || self.neck_drop.0 > 35
            || !(1500..=1950).contains(&self.side_wrap.0)
            || self.medial_ridge.0 > 25
            || self.chin_point.0 > 60
            || self.ridge_sharpness.0 > 1000
        {
            return Err(DesignError::ParametricParameters);
        }
        if let Some(courses) = &self.courses {
            courses.validate()?;
        }
        if let Some(breaths) = &self.breaths {
            breaths.validate(50..=950)?;
        }
        Ok(())
    }
}
