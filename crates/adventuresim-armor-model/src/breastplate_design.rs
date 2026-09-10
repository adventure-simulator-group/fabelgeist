//! Shape controls for paired torso plates and their integral waist flanges.
use crate::{DesignError, Millimeters, Permille};
use serde::{Deserialize, Serialize};
use std::ops::RangeInclusive;

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BreastplateDesign {
    pub catalog_id: String,
    /// Scale of the neck opening around the wearer-derived default.
    pub neck_width: Permille,
    /// Scale of the front and rear neckline drop.
    pub neck_depth: Permille,
    /// Scale of the armscye depth.
    pub arm_opening_depth: Permille,
    /// Scale of the lower plate width around the wearer-derived default.
    pub waist_width: Permille,
    /// Scale of neck-to-waist plate length.
    pub plate_length: Permille,
    /// Scale of the side return toward the coronal torso plane.
    pub side_return: Permille,
    /// Scale of rear shell depth; fitting still encloses the wearer.
    pub back_depth: Permille,
    pub profile: BreastplateProfile,
    pub fluting: Option<BreastplateFluting>,
    /// Scale of the default short skirt length.
    pub skirt_length: Permille,
    /// Outward flare of the skirt's lower edge.
    pub skirt_flare: Millimeters,
    /// Inner-surface clearance for the front plate.
    pub front_clearance: Millimeters,
    /// Inner-surface clearance for the rear plate.
    pub back_clearance: Millimeters,
    pub wall_thickness: Millimeters,
}

impl Default for BreastplateDesign {
    fn default() -> Self {
        Self {
            catalog_id: "breastplate".into(),
            neck_width: Permille(1_000),
            neck_depth: Permille(1_000),
            arm_opening_depth: Permille(1_000),
            waist_width: Permille(1_000),
            plate_length: Permille(1_000),
            side_return: Permille(1_000),
            back_depth: Permille(1000),
            profile: BreastplateProfile::default(),
            fluting: None,
            skirt_length: Permille(1_000),
            skirt_flare: Millimeters(30),
            front_clearance: Millimeters(10),
            back_clearance: Millimeters(14),
            wall_thickness: Millimeters(4),
        }
    }
}

/// Independent sagittal fullness, medial ridge, and pointed waist controls.
/// Heights run from the waist (0) to the neckline (1000).
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BreastplateProfile {
    pub projection: Millimeters,
    /// Recession of the upper breast, independent of the lower projecting point.
    pub upper_chest_recession: Millimeters,
    /// Independent projection at the waist seam, carried through the flange.
    pub waist_projection: Millimeters,
    pub projection_height: Permille,
    /// Lateral extent of fullness; larger values produce a broader round chest.
    pub fullness: Permille,
    pub medial_ridge: Millimeters,
    /// Drop of the front waist point; the flange follows the same boundary.
    pub waist_point: Millimeters,
    /// Lateral extent of the dropped waist point, relative to the front half-width.
    pub waist_point_width: Permille,
}

impl Default for BreastplateProfile {
    fn default() -> Self {
        Self {
            projection: Millimeters(6),
            upper_chest_recession: Millimeters(0),
            waist_projection: Millimeters(0),
            projection_height: Permille(550),
            fullness: Permille(650),
            medial_ridge: Millimeters(0),
            waist_point: Millimeters(0),
            waist_point_width: Permille(800),
        }
    }
}

/// Raised, rounded flutes separated by smooth lands on the front plate.
/// Width is a fraction of flute pitch, independent of count. Dimensions are
/// in the authored carrier chart; physical widths scale with the wearer.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BreastplateFluting {
    pub count: FluteCount,
    pub width: Permille,
    pub depth: Millimeters,
    /// Fraction of front chart width occupied by the pattern at its top.
    pub spread: Permille,
    /// Lower pattern width relative to its top; 1000 adds no fan in the carrier chart.
    pub lower_spread: Permille,
    pub start: Permille,
    pub end: Permille,
    /// Fade length at each end, as a fraction of plate height.
    pub fade: Permille,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct FluteCount(pub u16);

impl Default for BreastplateFluting {
    fn default() -> Self {
        Self {
            count: FluteCount(16),
            width: Permille(850),
            depth: Millimeters(2),
            spread: Permille(850),
            lower_spread: Permille(900),
            start: Permille(150),
            end: Permille(800),
            fade: Permille(100),
        }
    }
}

impl BreastplateProfile {
    pub const UPPER_RECESSION_RANGE: RangeInclusive<u16> = 0..=50;
    pub const PROJECTION_RANGE: RangeInclusive<u16> = 0..=80;
    pub const PROJECTION_HEIGHT_RANGE: RangeInclusive<u16> = 150..=800;
    pub const FULLNESS_RANGE: RangeInclusive<u16> = 350..=1000;
    pub const RIDGE_RANGE: RangeInclusive<u16> = 0..=35;
    pub const WAIST_POINT_WIDTH_RANGE: RangeInclusive<u16> = 350..=1000;
    pub const WAIST_POINT_RANGE: RangeInclusive<u16> = 0..=100;

    pub(crate) fn validate(&self) -> Result<(), DesignError> {
        if !Self::PROJECTION_RANGE.contains(&self.projection.0)
            || !Self::UPPER_RECESSION_RANGE.contains(&self.upper_chest_recession.0)
            || !Self::PROJECTION_RANGE.contains(&self.waist_projection.0)
            || !Self::PROJECTION_HEIGHT_RANGE.contains(&self.projection_height.0)
            || !Self::FULLNESS_RANGE.contains(&self.fullness.0)
            || !Self::RIDGE_RANGE.contains(&self.medial_ridge.0)
            || !Self::WAIST_POINT_RANGE.contains(&self.waist_point.0)
            || !Self::WAIST_POINT_WIDTH_RANGE.contains(&self.waist_point_width.0)
        {
            return Err(DesignError::BreastplateShape);
        }
        Ok(())
    }
}

impl BreastplateFluting {
    pub const COUNT_RANGE: RangeInclusive<u16> = 2..=24;
    pub const WIDTH_RANGE: RangeInclusive<u16> = 350..=850;
    pub const DEPTH_RANGE: RangeInclusive<u16> = 1..=4;
    pub const SPREAD_RANGE: RangeInclusive<u16> = 400..=850;
    pub const LOWER_SPREAD_RANGE: RangeInclusive<u16> = 500..=1000;
    pub const FADE_RANGE: RangeInclusive<u16> = 100..=250;
    pub const MIN_START: u16 = 50;
    pub const MAX_END: u16 = 950;

    pub(crate) fn validate(&self) -> Result<(), DesignError> {
        if !Self::COUNT_RANGE.contains(&self.count.0)
            || !Self::WIDTH_RANGE.contains(&self.width.0)
            || !Self::DEPTH_RANGE.contains(&self.depth.0)
            || !Self::SPREAD_RANGE.contains(&self.spread.0)
            || !Self::LOWER_SPREAD_RANGE.contains(&self.lower_spread.0)
            || self.start.0 < Self::MIN_START
            || self.end.0 > Self::MAX_END
            || self.end.0 <= self.start.0
            || !Self::FADE_RANGE.contains(&self.fade.0)
            || self.end.0 - self.start.0 < self.fade.0 * 2
        {
            return Err(DesignError::BreastplateFluting);
        }
        Ok(())
    }
}

impl BreastplateDesign {
    /// Early sixteenth-century rounded silhouette; no separate plackart.
    pub fn globose() -> Self {
        Self {
            profile: BreastplateProfile {
                projection: Millimeters(25),
                projection_height: Permille(540),
                fullness: Permille(950),
                ..BreastplateProfile::default()
            },
            waist_width: Permille(800),
            plate_length: Permille(700),
            skirt_length: Permille(700),
            skirt_flare: Millimeters(10),
            back_depth: Permille(800),
            arm_opening_depth: Permille(1100),
            back_clearance: Millimeters(10),
            wall_thickness: Millimeters(2),
            ..Self::default()
        }
    }

    /// Mid-sixteenth-century medial ridge with fullness below the sternum.
    pub fn tapul() -> Self {
        Self {
            profile: BreastplateProfile {
                projection: Millimeters(45),
                upper_chest_recession: Millimeters(35),
                waist_projection: Millimeters(0),
                projection_height: Permille(300),
                fullness: Permille(700),
                medial_ridge: Millimeters(24),
                waist_point: Millimeters(22),
                waist_point_width: Permille(900),
            },
            waist_width: Permille(800),
            plate_length: Permille(700),
            skirt_length: Permille(700),
            skirt_flare: Millimeters(10),
            back_depth: Permille(800),
            arm_opening_depth: Permille(1100),
            back_clearance: Millimeters(10),
            wall_thickness: Millimeters(2),
            ..Self::default()
        }
    }

    /// Late-sixteenth-century low projecting belly and pointed waist.
    pub fn peascod() -> Self {
        Self {
            profile: BreastplateProfile {
                projection: Millimeters(10),
                upper_chest_recession: Millimeters(0),
                waist_projection: Millimeters(45),
                projection_height: Permille(250),
                fullness: Permille(650),
                medial_ridge: Millimeters(14),
                waist_point: Millimeters(95),
                waist_point_width: Permille(700),
            },
            waist_width: Permille(800),
            plate_length: Permille(700),
            skirt_length: Permille(700),
            skirt_flare: Millimeters(10),
            back_depth: Permille(800),
            arm_opening_depth: Permille(1100),
            back_clearance: Millimeters(10),
            wall_thickness: Millimeters(2),
            ..Self::default()
        }
    }

    pub fn fluted() -> Self {
        Self {
            fluting: Some(BreastplateFluting::default()),
            ..Self::globose()
        }
    }
}
