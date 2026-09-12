//! Full shoulder defenses: a formed main plate, neck lames and upper-arm lames.
//!
//! Front and rear wings have independent reach and drop. The shoulder saddle
//! follows gravity while the descending lames follow the anatomical arm axis.
use serde::{Deserialize, Serialize};

use crate::{Millimeters, Milliradians, Permille, PlateFluting, PlateGauge};

#[path = "pauldron_mesh.rs"]
mod mesh;
pub(crate) use mesh::generate;
#[path = "pauldron_surface.rs"]
mod surface;
pub use surface::PauldronCarrier;

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct PauldronDesign {
    pub gauge: PlateGauge,
    /// Separation from the selected supporting torso plates.
    pub plate_clearance: Millimeters,
    /// Space occupied by the rerebrace beneath the descending lames.
    pub arm_allowance: Millimeters,
    pub fluting: Option<PlateFluting>,
    pub front_reach: Millimeters,
    pub rear_reach: Millimeters,
    pub front_drop: Millimeters,
    pub rear_drop: Millimeters,
    pub neck_reach: Millimeters,
    pub crown_height: Permille,
    pub arm_length: Millimeters,
    pub upper_lames: u8,
    pub lower_lames: u8,
    pub outline: PauldronOutline,
}

/// Boundaries of the hanging main wings and the narrower arm articulation.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PauldronOutline {
    /// Circumferential extent from the crown into each hanging main wing.
    pub front_return: Milliradians,
    pub rear_return: Milliradians,
    pub front_extension: Millimeters,
    pub rear_extension: Millimeters,
    /// Angle away from the shoulder crown at which the wing starts descending.
    pub wing_start: Milliradians,
    pub corner_rounding: Permille,
    /// Center of each hanging wing's lowest region, from arm to neck.
    pub front_wing_position: Permille,
    pub rear_wing_position: Permille,
    /// Rounded approach to each low region. Larger values narrow its flat
    /// interval; 1000 produces one continuously rounded low point.
    pub front_wing_rounding: Permille,
    pub rear_wing_rounding: Permille,
    /// Fraction of the shoulder saddle occupied by the upper lames.
    pub upper_span: Permille,
    /// Half-angle covered by the narrower lower arm lames.
    pub arm_wrap: Milliradians,
}

impl Default for PauldronOutline {
    fn default() -> Self {
        Self {
            front_return: Milliradians(2600),
            rear_return: Milliradians(2680),
            front_extension: Millimeters(0),
            rear_extension: Millimeters(0),
            wing_start: Milliradians(1100),
            corner_rounding: Permille(280),
            front_wing_position: Permille(500),
            rear_wing_position: Permille(500),
            front_wing_rounding: Permille(800),
            rear_wing_rounding: Permille(800),
            upper_span: Permille(260),
            arm_wrap: Milliradians(1680),
        }
    }
}

impl PauldronOutline {
    pub const CORNER_ROUNDING_RANGE: std::ops::RangeInclusive<u16> = 100..=650;
    pub const WING_POSITION_RANGE: std::ops::RangeInclusive<u16> = 250..=750;
    pub const WING_ROUNDING_RANGE: std::ops::RangeInclusive<u16> = 100..=1000;

    fn valid(&self) -> bool {
        (1800..=2800).contains(&self.front_return.0)
            && (1800..=2800).contains(&self.rear_return.0)
            && self.front_extension.0 <= 90
            && self.rear_extension.0 <= 90
            && (800..=1500).contains(&self.wing_start.0)
            && Self::CORNER_ROUNDING_RANGE.contains(&self.corner_rounding.0)
            && [self.front_wing_position, self.rear_wing_position]
                .iter()
                .all(|value| Self::WING_POSITION_RANGE.contains(&value.0))
            && [self.front_wing_rounding, self.rear_wing_rounding]
                .iter()
                .all(|value| Self::WING_ROUNDING_RANGE.contains(&value.0))
            && (150..=400).contains(&self.upper_span.0)
            && (1400..=1800).contains(&self.arm_wrap.0)
    }
}

impl Default for PauldronDesign {
    fn default() -> Self {
        Self {
            gauge: PlateGauge {
                clearance: Millimeters(12),
                thickness: Millimeters(2),
            },
            fluting: None,
            plate_clearance: Millimeters(4),
            arm_allowance: Millimeters(10),
            front_reach: Millimeters(105),
            rear_reach: Millimeters(95),
            front_drop: Millimeters(50),
            rear_drop: Millimeters(50),
            neck_reach: Millimeters(38),
            crown_height: Permille(1200),
            arm_length: Millimeters(96),
            upper_lames: 2,
            lower_lames: 4,
            outline: PauldronOutline::default(),
        }
    }
}

impl PauldronDesign {
    pub const PLATE_CLEARANCE_RANGE: std::ops::RangeInclusive<u16> = 2..=16;
    pub const FRONT_REACH_RANGE: std::ops::RangeInclusive<u16> = 40..=180;
    /// Supported sheet stock for this tightly returned shoulder construction.
    pub const THICKNESS_RANGE: std::ops::RangeInclusive<u16> = 1..=3;

    pub(crate) fn valid_shape(&self) -> bool {
        Self::THICKNESS_RANGE.contains(&self.gauge.thickness.0)
            && self.outline.valid()
            && Self::FRONT_REACH_RANGE.contains(&self.front_reach.0)
            && Self::PLATE_CLEARANCE_RANGE.contains(&self.plate_clearance.0)
            && self.arm_allowance.0 <= 20
            && (60..=145).contains(&self.rear_reach.0)
            && self.front_drop.0 <= 60
            && self.rear_drop.0 <= 75
            && (20..=60).contains(&self.neck_reach.0)
            && (1000..=1450).contains(&self.crown_height.0)
            && (65..=125).contains(&self.arm_length.0)
            && (1..=3).contains(&self.upper_lames)
            && (3..=7).contains(&self.lower_lames)
    }
}
