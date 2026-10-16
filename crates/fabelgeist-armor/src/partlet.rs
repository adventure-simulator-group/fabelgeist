//! Separate open-sided shoulder/chest covering, with an optional standing collar.
use crate::{
    ClothTies, DesignError, GenerateError, Micrometers, Millimeters, Permille, TextileAppearance,
    TextileRepeatCount,
};
use serde::{Deserialize, Serialize};
use std::ops::RangeInclusive;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PartletDesign {
    pub hem_rise: Millimeters,
    pub neck_rise: Millimeters,
    /// Downward anterior slope of the neck seam; 1000 means one metre per metre.
    pub neckline_slope: Permille,
    /// Vertical depth of the connected cloth over each shoulder.
    pub shoulder_bridge: Millimeters,
    /// Angular share of a quarter turn removed at each flank.
    pub side_opening: Permille,
    /// Measured down from the neckline; the slit must end above the hem.
    pub back_slit: Millimeters,
    pub back_gap: Millimeters,
    pub collar: Option<StandingClothCollar>,
    pub clearance: Millimeters,
    pub thickness: Micrometers,
    pub fabric: TextileAppearance,
    pub binding: Option<ClothBinding>,
    pub ties: Option<ClothTies>,
}

/// Folded finishing tape, independently opaque even on a sheer carrier.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClothBinding {
    pub width: Millimeters,
    pub hem_width: Millimeters,
    pub thickness: Micrometers,
    pub fabric: TextileAppearance,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StandingClothCollar {
    pub height: Millimeters,
    /// Nominal top scale relative to the sewn base; body support can expand it.
    pub top_scale: Permille,
    pub pleats: Option<CollarPleats>,
    pub neckband: Option<CollarBand>,
    pub trim: Option<CollarBand>,
}

/// Separate smooth tape anchored to the same open neck seam as the collar.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CollarBand {
    pub bottom: Millimeters,
    pub height: Millimeters,
    pub thickness: Micrometers,
    pub fabric: TextileAppearance,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CollarPleats {
    pub count: TextileRepeatCount,
    pub depth: Millimeters,
}

impl PartletDesign {
    pub const HEM_RISE_RANGE: RangeInclusive<u16> = 100..=350;
    pub const NECK_RISE_RANGE: RangeInclusive<u16> = 0..=100;
    pub const NECKLINE_SLOPE_RANGE: RangeInclusive<u16> = 0..=1000;
    pub const SHOULDER_BRIDGE_RANGE: RangeInclusive<u16> = 10..=80;
    pub const SIDE_OPENING_RANGE: RangeInclusive<u16> = 100..=700;
    pub const BACK_SLIT_RANGE: RangeInclusive<u16> = 20..=280;
    pub const BACK_GAP_RANGE: RangeInclusive<u16> = 1..=10;
    pub const CLEARANCE_RANGE: RangeInclusive<u16> = 1..=20;
    pub const THICKNESS_RANGE: RangeInclusive<u16> = 100..=1000;

    pub fn validate(&self) -> Result<(), GenerateError> {
        self.fabric.validate()?;
        if !Self::HEM_RISE_RANGE.contains(&self.hem_rise.0)
            || !Self::NECK_RISE_RANGE.contains(&self.neck_rise.0)
            || !Self::NECKLINE_SLOPE_RANGE.contains(&self.neckline_slope.0)
            || !Self::SHOULDER_BRIDGE_RANGE.contains(&self.shoulder_bridge.0)
            || !Self::SIDE_OPENING_RANGE.contains(&self.side_opening.0)
            || !Self::BACK_SLIT_RANGE.contains(&self.back_slit.0)
            || !Self::BACK_GAP_RANGE.contains(&self.back_gap.0)
            || !Self::CLEARANCE_RANGE.contains(&self.clearance.0)
            || !Self::THICKNESS_RANGE.contains(&self.thickness.0)
        {
            return Err(DesignError::ParametricParameters.into());
        }
        if let Some(collar) = &self.collar {
            collar.validate()?;
        }
        if let Some(binding) = &self.binding {
            binding.validate()?;
        }
        if let Some(ties) = &self.ties {
            ties.validate()?;
        }
        Ok(())
    }
}

impl ClothBinding {
    pub const WIDTH_RANGE: RangeInclusive<u16> = 1..=30;
    pub const THICKNESS_RANGE: RangeInclusive<u16> = 100..=1000;

    pub fn validate(&self) -> Result<(), GenerateError> {
        self.fabric.validate()?;
        if !Self::WIDTH_RANGE.contains(&self.width.0)
            || !Self::WIDTH_RANGE.contains(&self.hem_width.0)
            || !Self::THICKNESS_RANGE.contains(&self.thickness.0)
        {
            return Err(DesignError::ParametricParameters.into());
        }
        Ok(())
    }
}

impl StandingClothCollar {
    pub const HEIGHT_RANGE: RangeInclusive<u16> = 5..=70;
    pub const TOP_SCALE_RANGE: RangeInclusive<u16> = 500..=1500;
    pub const PLEAT_COUNT_RANGE: RangeInclusive<u8> = 8..=64;
    pub const PLEAT_DEPTH_RANGE: RangeInclusive<u16> = 1..=6;

    fn validate(&self) -> Result<(), GenerateError> {
        if !Self::HEIGHT_RANGE.contains(&self.height.0)
            || !Self::TOP_SCALE_RANGE.contains(&self.top_scale.0)
            || self.pleats.as_ref().is_some_and(|pleats| {
                !Self::PLEAT_COUNT_RANGE.contains(&pleats.count.0)
                    || !Self::PLEAT_DEPTH_RANGE.contains(&pleats.depth.0)
            })
        {
            return Err(DesignError::ParametricParameters.into());
        }
        for band in [&self.neckband, &self.trim].into_iter().flatten() {
            band.fabric.validate()?;
            if band.height.0 == 0
                || u32::from(band.bottom.0) + u32::from(band.height.0) > u32::from(self.height.0)
                || !ClothBinding::THICKNESS_RANGE.contains(&band.thickness.0)
            {
                return Err(DesignError::ParametricParameters.into());
            }
        }
        Ok(())
    }
}
