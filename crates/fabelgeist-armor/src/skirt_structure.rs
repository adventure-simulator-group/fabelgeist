//! Optional folds and physical support members for a skirt carrier.

use serde::{Deserialize, Serialize};
use std::ops::RangeInclusive;

use crate::{DesignError, GenerateError, Millimeters, Permille, TextileColor};

/// Number of repeated members; the containing construction validates its range.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct SkirtRepeatCount(pub u8);

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SkirtPleats {
    pub count: SkirtRepeatCount,
    pub depth: Millimeters,
    pub style: PleatStyle,
    /// Rotation as a share of one repeat, rather than of the whole skirt.
    pub rotation: Permille,
    /// Front fold depth relative to the side and rear folds. Zero leaves a
    /// quiet center front; one preserves equal depth around the skirt.
    pub front_depth: Permille,
    /// Fold depth at the gathered waist, independent of lower-skirt depth.
    pub waist_depth: Millimeters,
    /// Fine gathers near the waist, separate from the broader hanging falls.
    pub waist_count: SkirtRepeatCount,
    /// Skirt reach over which fine waist gathers relax into broad falls.
    pub gather_falloff: Permille,
    /// Fine fold depth retained below the waist-gather relaxation region.
    pub gather_retention: Permille,
    /// Circumference reserved for a quiet central front panel.
    pub front_panel: Permille,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SkirtSupport {
    pub stays: Option<SkirtStays>,
    pub hoops: Option<SkirtHoops>,
    pub color: TextileColor,
}

/// Longitudinal strips along the hanging skirt section toward the hem.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SkirtStays {
    pub count: SkirtRepeatCount,
    pub width: Millimeters,
    pub thickness: Millimeters,
    pub length: Permille,
}

/// Circumferential strips spaced evenly between two skirt descents.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SkirtHoops {
    pub count: SkirtRepeatCount,
    /// Nominal band width, capped to a member's share of a tiny hanging span.
    pub width: Millimeters,
    pub thickness: Millimeters,
    /// Centers within the hanging skirt span; full bands stay within its ends.
    pub start: Permille,
    pub end: Permille,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum PleatStyle {
    Rounded,
    Accordion,
}

impl Default for SkirtPleats {
    fn default() -> Self {
        Self {
            count: SkirtRepeatCount(12),
            depth: Millimeters(35),
            style: PleatStyle::Rounded,
            rotation: Permille(0),
            front_depth: Permille::ONE,
            waist_depth: Millimeters(6),
            waist_count: SkirtRepeatCount(36),
            gather_falloff: Permille(400),
            gather_retention: Permille(0),
            front_panel: Permille(0),
        }
    }
}

impl SkirtPleats {
    pub const COUNT_RANGE: RangeInclusive<u8> = 4..=24;
    pub const DEPTH_RANGE: RangeInclusive<u16> = 5..=160;
    pub const FRONT_DEPTH_RANGE: RangeInclusive<u16> = 0..=1000;
    pub const WAIST_DEPTH_RANGE: RangeInclusive<u16> = 0..=30;
    pub const WAIST_COUNT_RANGE: RangeInclusive<u8> = 8..=48;
    pub const GATHER_FALLOFF_RANGE: RangeInclusive<u16> = 100..=700;
    pub const FRONT_PANEL_RANGE: RangeInclusive<u16> = 0..=500;
    pub const GATHER_RETENTION_RANGE: RangeInclusive<u16> = 0..=1000;

    pub fn validate(&self) -> Result<(), GenerateError> {
        if !Self::COUNT_RANGE.contains(&self.count.0)
            || !Self::DEPTH_RANGE.contains(&self.depth.0)
            || self.rotation.0 > Permille::ONE.0
            || !Self::FRONT_DEPTH_RANGE.contains(&self.front_depth.0)
            || !Self::WAIST_DEPTH_RANGE.contains(&self.waist_depth.0)
            || !Self::WAIST_COUNT_RANGE.contains(&self.waist_count.0)
            || !Self::GATHER_FALLOFF_RANGE.contains(&self.gather_falloff.0)
            || !Self::GATHER_RETENTION_RANGE.contains(&self.gather_retention.0)
            || !Self::FRONT_PANEL_RANGE.contains(&self.front_panel.0)
        {
            return Err(DesignError::ParametricParameters.into());
        }
        Ok(())
    }

    /// Native GPU design words: repeats, depth in metres, profile and phase.
    pub(crate) fn shader_words(&self) -> [f32; 4] {
        [
            f32::from(self.count.0),
            self.depth.metres(),
            match self.style {
                PleatStyle::Rounded => 0.0,
                PleatStyle::Accordion => 1.0,
            },
            self.rotation.unit(),
        ]
    }
}

impl Default for SkirtSupport {
    fn default() -> Self {
        Self {
            stays: Some(SkirtStays::default()),
            hoops: None,
            color: TextileColor([173, 145, 103]),
        }
    }
}

impl SkirtSupport {
    pub const WIDTH_RANGE: RangeInclusive<u16> = 4..=20;
    pub const THICKNESS_RANGE: RangeInclusive<u16> = 1..=5;

    pub fn validate(&self) -> Result<(), GenerateError> {
        if self.stays.is_none() && self.hoops.is_none() {
            return Err(DesignError::ParametricParameters.into());
        }
        if let Some(stays) = &self.stays {
            if !SkirtStays::COUNT_RANGE.contains(&stays.count.0)
                || !SkirtStays::LENGTH_RANGE.contains(&stays.length.0)
            {
                return Err(DesignError::ParametricParameters.into());
            }
            Self::validate_gauge(stays.width, stays.thickness)?;
        }
        if let Some(hoops) = &self.hoops {
            if !SkirtHoops::COUNT_RANGE.contains(&hoops.count.0)
                || !SkirtHoops::DESCENT_RANGE.contains(&hoops.start.0)
                || !SkirtHoops::DESCENT_RANGE.contains(&hoops.end.0)
                || hoops.start.0 >= hoops.end.0
            {
                return Err(DesignError::ParametricParameters.into());
            }
            Self::validate_gauge(hoops.width, hoops.thickness)?;
        }
        Ok(())
    }

    fn validate_gauge(width: Millimeters, thickness: Millimeters) -> Result<(), GenerateError> {
        if !Self::WIDTH_RANGE.contains(&width.0) || !Self::THICKNESS_RANGE.contains(&thickness.0) {
            return Err(DesignError::ParametricParameters.into());
        }
        Ok(())
    }
}

impl Default for SkirtStays {
    fn default() -> Self {
        Self {
            count: SkirtRepeatCount(8),
            width: Millimeters(10),
            thickness: Millimeters(2),
            length: Permille(950),
        }
    }
}

impl SkirtStays {
    pub const COUNT_RANGE: RangeInclusive<u8> = 4..=24;
    pub const LENGTH_RANGE: RangeInclusive<u16> = 500..=1000;
}

impl Default for SkirtHoops {
    fn default() -> Self {
        Self {
            count: SkirtRepeatCount(3),
            width: Millimeters(12),
            thickness: Millimeters(3),
            start: Permille(350),
            end: Permille(950),
        }
    }
}

impl SkirtHoops {
    pub const COUNT_RANGE: RangeInclusive<u8> = 1..=8;
    pub const DESCENT_RANGE: RangeInclusive<u16> = 100..=980;
}
