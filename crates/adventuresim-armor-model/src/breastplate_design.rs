//! Shape controls for paired torso plates and their integral waist flanges.
use crate::{DesignError, Millimeters, Permille, PlateFluting};
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
    pub fluting: Option<PlateFluting>,
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
            fluting: Some(PlateFluting::default()),
            ..Self::globose()
        }
    }
}
