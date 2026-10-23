//! Separate open-sided shoulder/chest covering, with an optional standing collar.
use crate::{
    ClothPleats, ClothTape, ClothTies, DesignError, GenerateError, Micrometers, Millimeters,
    Permille, TextileAppearance,
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
    pub front_curve: Option<ClothNeckCurve>,
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

/// Smooth additional anterior drop below the nominal neckline. The curve is
/// level at the neck anchor and reaches the authored drop one reach forward.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClothNeckCurve {
    pub reach: Millimeters,
    pub drop: Millimeters,
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
    /// Fabric above the transition on an oval stand; the fitted section uses
    /// this fabric from the sewn seam. The partlet panel covers the oval base.
    pub fabric: TextileAppearance,
    /// Circumference relative to the sewn base. Fitted support can expand it.
    pub top_scale: Permille,
    /// Height over which the sewn base reaches its upper circumference.
    pub taper_height: Millimeters,
    pub section: CollarSection,
    /// Outward turn above the highest supporting collar band.
    pub flare: Millimeters,
    pub pleats: Option<ClothPleats>,
    pub neckband: Option<CollarBand>,
    pub trim: Option<CollarBand>,
    /// Rear fasteners, measured from the sewn neck seam to the collar top.
    pub closures: Option<crate::ClothClosures>,
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

/// A fitted surface or an oval tailor-rest stand. The oval does not chase
/// local jaw and shoulder bulges. Its smooth seat is fitted above the unposed
/// shoulder surface; detailed body collision and posed settling remain separate.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum CollarSection {
    Fitted,
    Oval { depth_to_width: Permille },
}

impl CollarSection {
    pub const DEPTH_TO_WIDTH_RANGE: RangeInclusive<u16> = 500..=1500;
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
        if let Some(curve) = &self.front_curve {
            curve.validate()?;
        }
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
            let free_height = collar.height.metres() - collar.free_edge_start_metres();
            // The quartic turn has maximum second derivative 12*flare/height².
            if collar.flare.0 > 0
                && self.thickness.metres()
                    >= free_height * free_height / (12. * collar.flare.metres())
            {
                return Err(DesignError::ParametricParameters.into());
            }
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

impl ClothNeckCurve {
    pub const REACH_RANGE: RangeInclusive<u16> = 20..=150;
    pub const DROP_RANGE: RangeInclusive<u16> = 1..=100;

    fn validate(&self) -> Result<(), GenerateError> {
        if !Self::REACH_RANGE.contains(&self.reach.0) || !Self::DROP_RANGE.contains(&self.drop.0) {
            return Err(DesignError::ParametricParameters.into());
        }
        Ok(())
    }
}

impl ClothBinding {
    pub fn validate(&self) -> Result<(), GenerateError> {
        self.fabric.validate()?;
        if !ClothTape::WIDTH_RANGE.contains(&self.width.0)
            || !ClothTape::WIDTH_RANGE.contains(&self.hem_width.0)
            || !ClothTape::THICKNESS_RANGE.contains(&self.thickness.0)
        {
            return Err(DesignError::ParametricParameters.into());
        }
        Ok(())
    }
}

impl StandingClothCollar {
    pub const HEIGHT_RANGE: RangeInclusive<u16> = 5..=70;
    pub const TOP_SCALE_RANGE: RangeInclusive<u16> = 500..=1500;
    pub const FLARE_RANGE: RangeInclusive<u16> = 0..=20;
    pub const PLEAT_COUNT_RANGE: RangeInclusive<u8> = 8..=192;
    pub const PLEAT_DEPTH_RANGE: RangeInclusive<u16> = 1..=6;

    /// Native collar kernel boundary; the returned height is in metres.
    pub fn free_edge_start_metres(&self) -> f32 {
        [&self.neckband, &self.trim]
            .into_iter()
            .flatten()
            .map(|band| Millimeters(band.bottom.0 + band.height.0).metres())
            .fold(0., f32::max)
    }

    fn validate(&self) -> Result<(), GenerateError> {
        self.fabric.validate()?;
        if let Some(closures) = &self.closures {
            closures.validate()?;
        }
        if !Self::HEIGHT_RANGE.contains(&self.height.0)
            || !Self::TOP_SCALE_RANGE.contains(&self.top_scale.0)
            || self.taper_height.0 == 0
            || self.taper_height.0 > self.height.0
            || matches!(self.section, CollarSection::Oval { depth_to_width } if !CollarSection::DEPTH_TO_WIDTH_RANGE.contains(&depth_to_width.0))
            || !Self::FLARE_RANGE.contains(&self.flare.0)
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
                || !ClothTape::THICKNESS_RANGE.contains(&band.thickness.0)
            {
                return Err(DesignError::ParametricParameters.into());
            }
        }
        Ok(())
    }
}
