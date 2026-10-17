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
    pub body_fit: GownBodyFit,
    pub sleeves: Option<SewnSleeves>,
}

/// A fitted upper sheet or loose cloth released from a bound neckline.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum GownBodyFit {
    Fitted,
    /// Radial ease beyond the hanging body enclosure, in addition to clearance.
    Loose {
        ease: Millimeters,
        /// Share of the neckline-to-suspension height over which the
        /// full hanging width is released. Cloth below it hangs continuously.
        release: Permille,
    },
}

/// A full sleeve sewn into an existing armhole. Fine gathering belongs in the
/// fabric relief; these measurements control the broad tailor-rest volume.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SewnSleeves {
    pub fabric: crate::TextileAppearance,
    /// Shoulder-to-wrist reach; 1000 ends at the wrist landmark.
    pub length: Permille,
    /// Uncompressed circumference of the full sleeve body.
    pub circumference: Millimeters,
    pub cuff_circumference: Millimeters,
    pub cuff_length: Millimeters,
}

impl SewnSleeves {
    pub const LENGTH_RANGE: std::ops::RangeInclusive<u16> = 400..=1100;
    pub const CIRCUMFERENCE_RANGE: std::ops::RangeInclusive<u16> = 250..=1000;
    pub const CUFF_CIRCUMFERENCE_RANGE: std::ops::RangeInclusive<u16> = 140..=400;
    pub const CUFF_LENGTH_RANGE: std::ops::RangeInclusive<u16> = 5..=80;

    pub fn validate(&self) -> Result<(), GenerateError> {
        self.fabric.validate()?;
        if !Self::LENGTH_RANGE.contains(&self.length.0)
            || !Self::CIRCUMFERENCE_RANGE.contains(&self.circumference.0)
            || !Self::CUFF_CIRCUMFERENCE_RANGE.contains(&self.cuff_circumference.0)
            || !Self::CUFF_LENGTH_RANGE.contains(&self.cuff_length.0)
            || self.cuff_circumference.0 > self.circumference.0
        {
            return Err(DesignError::ParametricParameters.into());
        }
        Ok(())
    }
}

impl GownDesign {
    pub const WAIST_BAND_RANGE: std::ops::RangeInclusive<u16> = 5..=60;
    pub const EASE_RANGE: std::ops::RangeInclusive<u16> = 0..=150;
    pub const RELEASE_RANGE: std::ops::RangeInclusive<u16> = 100..=1000;

    pub fn validate(&self) -> Result<(), GenerateError> {
        self.bodice.validate()?;
        self.skirt.validate()?;
        if let Some(sleeves) = &self.sleeves {
            sleeves.validate()?;
        }
        if let GownBodyFit::Loose { ease, release } = self.body_fit
            && (!Self::EASE_RANGE.contains(&ease.0)
                || !Self::RELEASE_RANGE.contains(&release.0)
                || !matches!(self.bodice.opening, crate::BodiceOpening::Closed)
                || self.bodice.lining.is_some()
                || self.bodice.interlining.is_some())
        {
            return Err(DesignError::ParametricParameters.into());
        }
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
