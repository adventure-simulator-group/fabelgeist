//! Skull shape and temple-centered fan fluting, independent of lining clearance.
use crate::{DesignError, Millimeters, Permille, PlateFluting};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct HelmetCrown {
    /// Lower values retain more width toward the top of the bowl.
    pub fullness: Permille,
    /// A ridge formed into the bowl rather than an attached comb.
    pub ridge_height: Millimeters,
    pub fluting: Option<PlateFluting>,
}

impl Default for HelmetCrown {
    fn default() -> Self {
        Self {
            fullness: Permille(1000),
            ridge_height: Millimeters(0),
            fluting: None,
        }
    }
}

impl HelmetCrown {
    pub(crate) fn validate(&self) -> Result<(), DesignError> {
        if !(700..=1150).contains(&self.fullness.0) || self.ridge_height.0 > 18 {
            return Err(DesignError::ParametricParameters);
        }
        if let Some(pattern) = &self.fluting {
            pattern.validate()?;
        }
        Ok(())
    }
}
