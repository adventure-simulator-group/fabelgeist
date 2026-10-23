//! Fitted cloth cut from the body, with separately inspectable sewn layers.
use serde::{Deserialize, Serialize};
use std::ops::RangeInclusive;

use crate::{DesignError, GenerateError, Millimeters, Permille, TextileAppearance};

/// Neck opening measured from the wearer's neck landmark. Width is a fraction
/// of shoulder breadth; drops are distances below the neck joint.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BodiceNeckline {
    pub shape: NecklineShape,
    pub width: Permille,
    pub front_drop: Millimeters,
    pub back_drop: Millimeters,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BodiceLining {
    pub fabric: TextileAppearance,
    pub thickness: Millimeters,
}

/// A separate supporting sheet, enclosed between face and lining.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BodiceInterlining {
    pub fabric: TextileAppearance,
    pub thickness: Millimeters,
    pub coverage: InterliningCoverage,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BodiceDesign {
    pub neckline: BodiceNeckline,
    /// Distance from each shoulder joint to the medial armhole edge.
    pub armhole_width: Millimeters,
    /// Distance below each shoulder joint at the lower armhole edge.
    pub armhole_depth: Millimeters,
    /// Hem height above the pelvis root in the canonical unposed body frame.
    pub hem_rise: Millimeters,
    pub opening: BodiceOpening,
    pub clearance: Millimeters,
    pub thickness: Millimeters,
    pub fabric: TextileAppearance,
    pub lining: Option<BodiceLining>,
    pub interlining: Option<BodiceInterlining>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum NecklineShape {
    Square,
    Round,
    /// A fourth-power rounded rectangle: broad bottom and softened corners.
    SoftSquare,
}

/// An opening traverses the complete bodice, including its lining. The two
/// widths independently control its waist and neckline ends.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum BodiceOpening {
    Closed,
    Front {
        waist: Millimeters,
        chest: Millimeters,
    },
    Back {
        waist: Millimeters,
        chest: Millimeters,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum InterliningCoverage {
    Full,
    /// Opening-edge strips and a facing around the opposite neckline.
    Facings {
        width: Millimeters,
    },
}

impl BodiceDesign {
    pub const NECK_WIDTH_RANGE: RangeInclusive<u16> = 100..=850;
    pub const FRONT_DROP_RANGE: RangeInclusive<u16> = 1..=350;
    pub const BACK_DROP_RANGE: RangeInclusive<u16> = 1..=200;
    pub const ARMHOLE_WIDTH_RANGE: RangeInclusive<u16> = 20..=100;
    pub const ARMHOLE_DEPTH_RANGE: RangeInclusive<u16> = 40..=200;
    pub const HEM_RISE_RANGE: RangeInclusive<u16> = 0..=300;
    pub const CLEARANCE_RANGE: RangeInclusive<u16> = 1..=50;
    pub const THICKNESS_RANGE: RangeInclusive<u16> = 1..=6;
    pub const LINING_THICKNESS_RANGE: RangeInclusive<u16> = 1..=4;
    pub const FACING_WIDTH_RANGE: RangeInclusive<u16> = 5..=100;
    pub const WAIST_OPENING_RANGE: RangeInclusive<u16> = 1..=300;
    pub const CHEST_OPENING_RANGE: RangeInclusive<u16> = 1..=350;

    pub fn validate(&self) -> Result<(), GenerateError> {
        let widths_valid = match self.opening {
            BodiceOpening::Closed => true,
            BodiceOpening::Front { waist, chest } | BodiceOpening::Back { waist, chest } => {
                Self::WAIST_OPENING_RANGE.contains(&waist.0)
                    && Self::CHEST_OPENING_RANGE.contains(&chest.0)
            }
        };
        if !widths_valid
            || !Self::NECK_WIDTH_RANGE.contains(&self.neckline.width.0)
            || !Self::FRONT_DROP_RANGE.contains(&self.neckline.front_drop.0)
            || !Self::BACK_DROP_RANGE.contains(&self.neckline.back_drop.0)
            || !Self::ARMHOLE_WIDTH_RANGE.contains(&self.armhole_width.0)
            || !Self::ARMHOLE_DEPTH_RANGE.contains(&self.armhole_depth.0)
            || !Self::HEM_RISE_RANGE.contains(&self.hem_rise.0)
            || !Self::CLEARANCE_RANGE.contains(&self.clearance.0)
            || !Self::THICKNESS_RANGE.contains(&self.thickness.0)
        {
            return Err(DesignError::ParametricParameters.into());
        }
        self.fabric.validate()?;
        if let Some(lining) = &self.lining {
            if !Self::LINING_THICKNESS_RANGE.contains(&lining.thickness.0) {
                return Err(DesignError::ParametricParameters.into());
            }
            lining.fabric.validate()?;
        }
        if let Some(interlining) = &self.interlining {
            if self.lining.is_none()
                || !Self::LINING_THICKNESS_RANGE.contains(&interlining.thickness.0)
            {
                return Err(DesignError::ParametricParameters.into());
            }
            if let InterliningCoverage::Facings { width } = interlining.coverage
                && (matches!(self.opening, BodiceOpening::Closed)
                    || !Self::FACING_WIDTH_RANGE.contains(&width.0))
            {
                return Err(DesignError::ParametricParameters.into());
            }
            interlining.fabric.validate()?;
        }
        Ok(())
    }
}
