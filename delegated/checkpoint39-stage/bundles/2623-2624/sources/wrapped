//! Long lamed thigh defenses suspended at the waist, with an open rear.
use crate::{DesignError, Millimeters, Permille};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WrappedTassetDesign {
    /// Inner and outer returns, each as a fraction of a half circumference.
    pub inner_wrap: Permille,
    pub outer_wrap: Permille,
    /// Minimum distance of either inner edge from the body's medial plane.
    pub inner_gap: Millimeters,
    /// Rise of the upper suspension edge per metre from the medial plane.
    pub upper_edge_slope: Permille,
    /// Fraction of hip-to-knee distance reached by the lower hem.
    pub knee_reach: Permille,
    pub inner_cutaway: Millimeters,
    pub hem_rounding: Millimeters,
    /// Course at which a removable lower section begins; zero is unsplit.
    pub section_break: u8,
    pub section_gap: Millimeters,
}

impl Default for WrappedTassetDesign {
    fn default() -> Self {
        Self {
            inner_wrap: Permille(280),
            outer_wrap: Permille(620),
            inner_gap: Millimeters(10),
            upper_edge_slope: Permille(0),
            knee_reach: Permille(880),
            inner_cutaway: Millimeters(18),
            hem_rounding: Millimeters(18),
            section_break: 6,
            section_gap: Millimeters(3),
        }
    }
}

impl WrappedTassetDesign {
    pub(crate) fn validate(&self) -> Result<(), DesignError> {
        if !(150..=450).contains(&self.inner_wrap.0)
            || !(450..=750).contains(&self.outer_wrap.0)
            || !(650..=1100).contains(&self.knee_reach.0)
            || self.upper_edge_slope.0 > 500
            || self.inner_gap.0 > 80
            || self.inner_cutaway.0 > 50
            || self.hem_rounding.0 > 35
            || self.section_gap.0 > 8
            || self.section_break > 11
        {
            return Err(DesignError::ParametricParameters);
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug)]
pub enum TassetSide {
    Left,
    Right,
}
