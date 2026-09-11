//! Full shoulder defenses: a formed main plate, neck lames and upper-arm lames.
//!
//! Front and rear wings have independent reach and drop. The shoulder saddle
//! follows gravity while the descending lames follow the anatomical arm axis.
use serde::{Deserialize, Serialize};

use crate::{Millimeters, Permille, PlateFluting, PlateGauge};

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
        }
    }
}

impl PauldronDesign {
    /// Supported sheet stock for this tightly returned shoulder construction.
    pub const THICKNESS_RANGE: std::ops::RangeInclusive<u16> = 1..=3;

    pub(crate) fn valid_shape(&self) -> bool {
        Self::THICKNESS_RANGE.contains(&self.gauge.thickness.0)
            && (40..=120).contains(&self.front_reach.0)
            && (2..=12).contains(&self.plate_clearance.0)
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
