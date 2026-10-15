//! Skirts authored independently of either leg's surface topology.

use serde::{Deserialize, Serialize};
use std::ops::RangeInclusive;

use crate::{DesignError, GenerateError, Millimeters, Permille, TextileColor};

/// Circumferential samples; shared by construction and the fitted hull profile.
pub const SKIRT_RING_SEGMENTS: usize = 96;
/// Axial samples include both the waist and hem.
pub const SKIRT_AXIAL_SEGMENTS: usize = 32;

/// Skirt placement, silhouette and optional construction. Structure and surface
/// treatments remain independent; this carrier needs no cloth simulation.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SkirtDesign {
    /// A body-conforming upper section joined to the hanging cloth.
    pub bodice: Option<DressBodice>,
    pub pleats: Option<crate::SkirtPleats>,
    pub support: Option<crate::SkirtSupport>,
    pub puff_and_slash: Option<crate::PuffAndSlashPattern>,
    /// Height above the pelvis landmark in the canonical fitting frame.
    pub waist_rise: Millimeters,
    /// Hip-to-ankle reach: 1000 ends at the ankle landmark.
    pub length: Permille,
    /// Share of the skirt over which the waist opens into its hanging profile.
    pub fitted_fraction: Permille,
    pub profile: SkirtProfile,
    pub clearance: Millimeters,
    pub thickness: Millimeters,
    pub color: TextileColor,
}

/// Minimal continuous dress upper section. Necklines, sleeves and corsetry
/// belong to separate construction work.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DressBodice {
    /// Share of the distance from the waist to the upper chest landmark.
    pub height: Permille,
    /// Share of skirt length used to release the fitted waist into the skirt.
    pub transition: Permille,
}

/// Width and depth gains are relative to the enclosing hip/leg profile.
/// The straight variant has no irrelevant flare controls.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum SkirtProfile {
    Straight,
    Flared {
        width: Permille,
        depth: Permille,
    },
    Bell {
        width: Permille,
        depth: Permille,
        roundness: Permille,
    },
}

impl Default for DressBodice {
    fn default() -> Self {
        Self {
            height: Permille(800),
            transition: Permille(150),
        }
    }
}

impl DressBodice {
    pub const HEIGHT_RANGE: RangeInclusive<u16> = 400..=950;
    pub const TRANSITION_RANGE: RangeInclusive<u16> = 100..=300;

    /// The first ordinary skirt ring, after the body-to-skirt transition.
    pub fn first_skirt_row(&self) -> usize {
        (self.transition.unit() * SKIRT_AXIAL_SEGMENTS as f32).ceil() as usize
    }

    fn validate(&self) -> Result<(), GenerateError> {
        if !Self::HEIGHT_RANGE.contains(&self.height.0)
            || !Self::TRANSITION_RANGE.contains(&self.transition.0)
        {
            return Err(DesignError::ParametricParameters.into());
        }
        Ok(())
    }
}

impl Default for SkirtDesign {
    fn default() -> Self {
        Self {
            bodice: None,
            pleats: None,
            support: None,
            puff_and_slash: None,
            waist_rise: Millimeters(100),
            length: Permille(850),
            fitted_fraction: Permille(220),
            profile: SkirtProfile::Flared {
                width: Permille(1600),
                depth: Permille(1600),
            },
            clearance: Millimeters(8),
            thickness: Millimeters(2),
            color: TextileColor([82, 57, 103]),
        }
    }
}

impl SkirtDesign {
    pub const WAIST_RISE_RANGE: RangeInclusive<u16> = 0..=200;
    pub const LENGTH_RANGE: RangeInclusive<u16> = 100..=1000;
    pub const FITTED_FRACTION_RANGE: RangeInclusive<u16> = 100..=500;
    pub const CLEARANCE_RANGE: RangeInclusive<u16> = 1..=30;
    pub const THICKNESS_RANGE: RangeInclusive<u16> = 1..=6;

    pub fn validate(&self) -> Result<(), GenerateError> {
        if let Some(pattern) = &self.puff_and_slash {
            pattern.validate()?;
        }
        if let Some(pleats) = &self.pleats {
            pleats.validate()?;
        }
        if let Some(support) = &self.support {
            support.validate()?;
        }
        if let Some(bodice) = &self.bodice {
            bodice.validate()?;
        }
        if !Self::WAIST_RISE_RANGE.contains(&self.waist_rise.0)
            || !Self::LENGTH_RANGE.contains(&self.length.0)
            || !Self::FITTED_FRACTION_RANGE.contains(&self.fitted_fraction.0)
            || !Self::CLEARANCE_RANGE.contains(&self.clearance.0)
            || !Self::THICKNESS_RANGE.contains(&self.thickness.0)
        {
            return Err(DesignError::ParametricParameters.into());
        }
        self.profile.validate()
    }
}

impl SkirtProfile {
    pub const HEM_GAIN_RANGE: RangeInclusive<u16> = 1000..=3500;
    pub const BELL_ROUNDNESS_RANGE: RangeInclusive<u16> = 400..=2400;

    fn validate(&self) -> Result<(), GenerateError> {
        let valid_gain = |gain: Permille| Self::HEM_GAIN_RANGE.contains(&gain.0);
        let valid = match *self {
            Self::Straight => true,
            Self::Flared { width, depth } => valid_gain(width) && valid_gain(depth),
            Self::Bell {
                width,
                depth,
                roundness,
            } => {
                valid_gain(width)
                    && valid_gain(depth)
                    && Self::BELL_ROUNDNESS_RANGE.contains(&roundness.0)
            }
        };
        if !valid {
            return Err(DesignError::ParametricParameters.into());
        }
        Ok(())
    }

    /// Native shader words: profile discriminator, width, depth and exponent.
    pub(crate) fn shader_words(&self) -> [f32; 4] {
        match *self {
            Self::Straight => [0.0, 1.0, 1.0, 1.0],
            Self::Flared { width, depth } => [1.0, width.unit(), depth.unit(), 1.0],
            Self::Bell {
                width,
                depth,
                roundness,
            } => [2.0, width.unit(), depth.unit(), roundness.unit()],
        }
    }
}
