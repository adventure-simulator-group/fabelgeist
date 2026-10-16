//! A fitted bodice sewn into a skirt, retaining independently inspectable layers.
use serde::{Deserialize, Serialize};

use crate::{
    BodiceDesign, DesignError, DressBodice, GenerateError, Millimeters, Permille, SkirtDesign,
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GownDesign {
    pub bodice: BodiceDesign,
    /// The skirt supplies its own fabric and construction; its minimal upper
    /// band must be absent because this garment supplies a complete bodice.
    pub skirt: SkirtDesign,
    pub transition: Permille,
    /// Uncut cloth below a front/back opening keeps a closed sewn waist.
    /// Fastenings and a skirt slit are separate construction features.
    pub waist_band: Millimeters,
}

impl GownDesign {
    pub const WAIST_BAND_RANGE: std::ops::RangeInclusive<u16> = 5..=60;

    pub fn validate(&self) -> Result<(), GenerateError> {
        self.bodice.validate()?;
        self.skirt.validate()?;
        if self.skirt.bodice.is_some()
            || self.skirt.puff_and_slash.is_some()
            || self.bodice.hem_rise != self.skirt.waist_rise
            || self.bodice.thickness != self.skirt.thickness
            || !DressBodice::TRANSITION_RANGE.contains(&self.transition.0)
            || !Self::WAIST_BAND_RANGE.contains(&self.waist_band.0)
        {
            return Err(DesignError::ParametricParameters.into());
        }
        Ok(())
    }

    pub fn first_skirt_row(&self) -> usize {
        (self.transition.unit() * crate::SKIRT_AXIAL_SEGMENTS as f32).ceil() as usize
    }
}
