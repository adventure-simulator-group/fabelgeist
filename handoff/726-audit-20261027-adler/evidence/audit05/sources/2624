//! Shape controls specific to exposed neck and waist plate constructions.
use crate::{DesignError, GarmentArmorKind, Millimeters, Permille};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum GarmentPlateShape {
    None,
    WrappedTassets(crate::WrappedTassetDesign),
    Fauld {
        waist_rise: Millimeters,
        front_arch: Permille,
        /// Half-width of the groin opening relative to the skirt half-width.
        front_arch_width: Permille,
        /// Rise per metre away from the front medial hem.
        chevron_slope: Permille,
    },
    Tassets {
        inner_cutaway: Permille,
        hem_point: Permille,
        width: Permille,
        gap: Permille,
        hem_roundness: Permille,
    },
    Gorget {
        neck_clearance: Millimeters,
        collar_slope: Permille,
        collar_height: Permille,
        hem_flatness: Permille,
        rear_hem_flatness: Permille,
        rear_sweep: Permille,
        front_depth: Permille,
        back_depth: Permille,
        front_width: Permille,
        back_width: Permille,
    },
}

impl GarmentPlateShape {
    pub fn for_kind(kind: GarmentArmorKind) -> Self {
        match kind {
            GarmentArmorKind::Fauld => Self::Fauld {
                waist_rise: Millimeters(50),
                front_arch: Permille(0),
                front_arch_width: Permille(500),
                chevron_slope: Permille(0),
            },
            GarmentArmorKind::Tassets => Self::Tassets {
                inner_cutaway: Permille(0),
                hem_point: Permille(0),
                width: Permille(1000),
                gap: Permille(160),
                hem_roundness: Permille(80),
            },
            GarmentArmorKind::Gorget => Self::Gorget {
                neck_clearance: Millimeters(6),
                collar_slope: Permille(1000),
                collar_height: Permille(1000),
                hem_flatness: Permille(0),
                rear_hem_flatness: Permille(0),
                rear_sweep: Permille(0),
                front_depth: Permille(1000),
                back_depth: Permille(1000),
                front_width: Permille(1000),
                back_width: Permille(1000),
            },
            _ => Self::None,
        }
    }

    pub(crate) fn validate(self, kind: GarmentArmorKind) -> Result<(), DesignError> {
        let valid = match self {
            Self::WrappedTassets(shape) => {
                shape.validate()?;
                kind == GarmentArmorKind::Tassets
            }
            Self::Fauld {
                waist_rise,
                front_arch,
                front_arch_width,
                chevron_slope,
            } => {
                kind == GarmentArmorKind::Fauld
                    && waist_rise.0 <= 120
                    && front_arch.0 <= 1200
                    && (250..=800).contains(&front_arch_width.0)
                    && chevron_slope.0 <= 500
            }
            Self::Tassets {
                inner_cutaway,
                hem_point,
                width,
                gap,
                hem_roundness,
            } => {
                kind == GarmentArmorKind::Tassets
                    && inner_cutaway.0 <= 400
                    && hem_point.0 <= 200
                    && (700..=1150).contains(&width.0)
                    && (100..=400).contains(&gap.0)
                    && hem_roundness.0 <= 300
            }
            Self::Gorget {
                neck_clearance,
                collar_slope,
                collar_height,
                hem_flatness,
                rear_hem_flatness,
                rear_sweep,
                front_depth,
                back_depth,
                front_width,
                back_width,
            } => {
                kind == GarmentArmorKind::Gorget
                    && (2..=15).contains(&neck_clearance.0)
                    && collar_slope.0 <= 1000
                    && (500..=2000).contains(&collar_height.0)
                    && hem_flatness.0 <= 1000
                    && rear_hem_flatness.0 <= 1000
                    && rear_sweep.0 <= 1000
                    && (600..=1500).contains(&front_depth.0)
                    && (600..=1500).contains(&back_depth.0)
                    && (500..=1300).contains(&front_width.0)
                    && (500..=1300).contains(&back_width.0)
            }
            Self::None => !matches!(
                kind,
                GarmentArmorKind::Fauld | GarmentArmorKind::Tassets | GarmentArmorKind::Gorget
            ),
        };
        if valid {
            Ok(())
        } else {
            Err(DesignError::ParametricParameters)
        }
    }
}
