//! Separate lower joint plates, including the long terminal plate of a poleyn.
use crate::{DesignError, Millimeters, Milliradians, Permille};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct JointExtension {
    pub length: Millimeters,
    pub lame_count: u8,
    /// Share of the extension occupied by its lowest plate.
    pub terminal_share: Permille,
    /// Distal girth relative to the cup edge, shared across all lower plates.
    pub distal_taper: Permille,
    pub wrap: Milliradians,
    pub hem_rounding: Millimeters,
}

impl Default for JointExtension {
    fn default() -> Self {
        Self {
            length: Millimeters(130),
            lame_count: 3,
            terminal_share: Permille(650),
            distal_taper: Permille(1000),
            wrap: Milliradians(1350),
            hem_rounding: Millimeters(12),
        }
    }
}

impl JointExtension {
    pub fn validate(&self) -> Result<(), DesignError> {
        let terminal_length = f32::from(self.length.0)
            * if self.lame_count == 1 {
                1.0
            } else {
                self.terminal_share.unit()
            };
        if !(40..=180).contains(&self.length.0)
            || !(1..=4).contains(&self.lame_count)
            || !(350..=850).contains(&self.terminal_share.0)
            || !(700..=1100).contains(&self.distal_taper.0)
            || !(900..=1550).contains(&self.wrap.0)
            || self.hem_rounding.0 > 25
            || 4.0 * f32::from(self.hem_rounding.0) >= terminal_length
        {
            return Err(DesignError::ParametricParameters);
        }
        Ok(())
    }
}
