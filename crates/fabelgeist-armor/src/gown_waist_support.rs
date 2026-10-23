//! Independent folded linen support at a skirt-to-bodice seam.
use crate::{ClothTape, DesignError, GenerateError, Millimeters};
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GownWaistSupport {
    /// Finished main band height and unfolded cloth gauge.
    pub main: ClothTape,
    /// Nominal canonical arc length of the independent front band.
    pub front_width: Millimeters,
    pub front_height: Millimeters,
    pub fasteners: Option<crate::WaistBandFasteners>,
}
impl GownWaistSupport {
    pub const FRONT_WIDTH_RANGE: std::ops::RangeInclusive<u16> = 20..=300;
    pub const FOLDED_PLY_COUNT: u16 = 4;
    pub fn validate(&self) -> Result<(), GenerateError> {
        self.main.validate()?;
        if let Some(fasteners) = &self.fasteners {
            fasteners.validate()?;
        }
        if !Self::FRONT_WIDTH_RANGE.contains(&self.front_width.0)
            || !ClothTape::WIDTH_RANGE.contains(&self.front_height.0)
            || u32::from(self.main.thickness.0) * u32::from(Self::FOLDED_PLY_COUNT)
                >= u32::from(self.main.width.0.min(self.front_height.0))
                    * crate::Micrometers::PER_MILLIMETRE
        {
            return Err(DesignError::ParametricParameters.into());
        }
        Ok(())
    }
}
