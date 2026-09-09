//! Authored plate families fitted to anatomical frames, independently of body topology.
//!
//! Long plates, joint cups, lamed shoulders, mittens and footwear have separate
//! controls because their openings and articulation are structurally different.

use serde::{Deserialize, Serialize};

use crate::parametric::{PartFrame, PartMesh};
use crate::{DesignError, GenerateError, Millimeters, Permille};

#[path = "limb_armor_extremities.rs"]
mod extremities;
#[path = "limb_armor_mesh.rs"]
mod mesh;
#[path = "limb_armor_shapes.rs"]
mod shapes;

/// Padding clearance and actual wall gauge, in millimetres.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct PlateGauge {
    pub clearance: Millimeters,
    pub thickness: Millimeters,
}

impl Default for PlateGauge {
    fn default() -> Self {
        Self {
            clearance: Millimeters(10),
            thickness: Millimeters(2),
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct GreaveDesign {
    pub gauge: PlateGauge,
    pub length: Permille,
    /// Ankle radius relative to the maximum calf radius.
    pub ankle_taper: Permille,
    /// Longitudinal crown over the tibia; zero gives a rounded front.
    pub shin_ridge: Millimeters,
}

impl Default for GreaveDesign {
    fn default() -> Self {
        Self {
            gauge: PlateGauge::default(),
            length: Permille(970),
            ankle_taper: Permille(630),
            shin_ridge: Millimeters(5),
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct CuisseDesign {
    pub gauge: PlateGauge,
    pub length: Permille,
    pub knee_taper: Permille,
    /// Circumferential coverage: half the thigh to a deep three-quarter return.
    pub wrap: Permille,
}

impl Default for CuisseDesign {
    fn default() -> Self {
        Self {
            gauge: PlateGauge::default(),
            length: Permille(920),
            knee_taper: Permille(740),
            wrap: Permille(720),
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct RerebraceDesign {
    pub gauge: PlateGauge,
    pub length: Permille,
    pub distal_taper: Permille,
    pub wrap: Permille,
}

impl Default for RerebraceDesign {
    fn default() -> Self {
        Self {
            gauge: PlateGauge::default(),
            length: Permille(760),
            distal_taper: Permille(830),
            wrap: Permille(720),
        }
    }
}

/// A raised knee or elbow cop flowing into an integral lateral wing.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct JointCupDesign {
    pub gauge: PlateGauge,
    pub dome: Permille,
    pub wing: Permille,
    pub length: Permille,
}

impl Default for JointCupDesign {
    fn default() -> Self {
        Self::poleyn()
    }
}

impl JointCupDesign {
    pub fn poleyn() -> Self {
        Self {
            gauge: PlateGauge::default(),
            dome: Permille(650),
            wing: Permille(350),
            length: Permille(1000),
        }
    }

    pub fn couter() -> Self {
        Self {
            gauge: PlateGauge::default(),
            dome: Permille(850),
            wing: Permille(450),
            length: Permille(900),
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct SpaulderDesign {
    pub gauge: PlateGauge,
    pub length: Permille,
    pub crown: Permille,
    pub lame_count: u8,
}

impl Default for SpaulderDesign {
    fn default() -> Self {
        Self {
            gauge: PlateGauge::default(),
            length: Permille(1000),
            crown: Permille(1000),
            lame_count: 4,
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct GauntletDesign {
    pub gauge: PlateGauge,
    pub cuff_length: Permille,
    pub cuff_flare: Permille,
    pub finger_lames: u8,
}

impl Default for GauntletDesign {
    fn default() -> Self {
        Self {
            gauge: PlateGauge {
                clearance: Millimeters(4),
                thickness: Millimeters(2),
            },
            cuff_length: Permille(370),
            cuff_flare: Permille(1350),
            finger_lames: 4,
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct FootArmorDesign {
    pub gauge: PlateGauge,
    /// Toe box width; supports ordinary rounded and broad early-sixteenth-century forms.
    pub toe_width: Permille,
    pub toe_extension: Millimeters,
    pub lame_count: u8,
}

impl Default for FootArmorDesign {
    fn default() -> Self {
        Self {
            gauge: PlateGauge::default(),
            toe_width: Permille(1000),
            toe_extension: Millimeters(8),
            lame_count: 5,
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct BootDesign {
    pub gauge: PlateGauge,
    pub shaft_height: Millimeters,
    pub shaft_flare: Permille,
    pub toe_width: Permille,
}

impl Default for BootDesign {
    fn default() -> Self {
        Self {
            gauge: PlateGauge {
                clearance: Millimeters(6),
                thickness: Millimeters(3),
            },
            shaft_height: Millimeters(160),
            shaft_flare: Permille(1100),
            toe_width: Permille(1000),
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum LimbArmorDesign {
    Greave(GreaveDesign),
    Cuisse(CuisseDesign),
    Rerebrace(RerebraceDesign),
    Poleyn(JointCupDesign),
    Couter(JointCupDesign),
    Spaulder(SpaulderDesign),
    MittenGauntlet(GauntletDesign),
    Sabaton(FootArmorDesign),
    LeatherBoot(BootDesign),
}

impl LimbArmorDesign {
    pub fn validate(&self) -> Result<(), GenerateError> {
        let ratio = |p: Permille, low, high| (low..=high).contains(&p.0);
        let (gauge, valid) = match self {
            Self::Greave(d) => (
                d.gauge,
                ratio(d.length, 650, 1050)
                    && ratio(d.ankle_taper, 450, 850)
                    && d.shin_ridge.0 <= 15,
            ),
            Self::Cuisse(d) => (
                d.gauge,
                ratio(d.length, 600, 1050)
                    && ratio(d.knee_taper, 550, 1000)
                    && ratio(d.wrap, 500, 900),
            ),
            Self::Rerebrace(d) => (
                d.gauge,
                ratio(d.length, 600, 1050)
                    && ratio(d.distal_taper, 600, 1100)
                    && ratio(d.wrap, 550, 950),
            ),
            Self::Poleyn(d) | Self::Couter(d) => (
                d.gauge,
                ratio(d.dome, 350, 1100) && ratio(d.wing, 0, 650) && ratio(d.length, 650, 1250),
            ),
            Self::Spaulder(d) => (
                d.gauge,
                ratio(d.length, 650, 1250)
                    && ratio(d.crown, 850, 1350)
                    && (2..=6).contains(&d.lame_count),
            ),
            Self::MittenGauntlet(d) => (
                d.gauge,
                ratio(d.cuff_length, 150, 650)
                    && ratio(d.cuff_flare, 1050, 1600)
                    && (2..=6).contains(&d.finger_lames),
            ),
            Self::Sabaton(d) => (
                d.gauge,
                ratio(d.toe_width, 800, 1450)
                    && d.toe_extension.0 <= 50
                    && (3..=8).contains(&d.lame_count),
            ),
            Self::LeatherBoot(d) => (
                d.gauge,
                (40..=350).contains(&d.shaft_height.0)
                    && ratio(d.shaft_flare, 1000, 1500)
                    && ratio(d.toe_width, 800, 1300),
            ),
        };
        if !valid || !(2..=25).contains(&gauge.clearance.0) || !(1..=6).contains(&gauge.thickness.0)
        {
            return Err(DesignError::ParametricParameters.into());
        }
        Ok(())
    }
}

/// Generate in a local right-handed frame, then map into the supplied wearer frame.
///
/// `y` is proximal, `z` faces the cop/plate and `x` points toward the wing.
/// Feet use `y` up and `z` toward the toes. Mittens use `y` toward the wrist,
/// `z` dorsal and `x` away from the thumb. Extents describe the unarmored region.
pub fn generate_limb_armor(
    design: &LimbArmorDesign,
    fit: &PartFrame,
) -> Result<PartMesh, GenerateError> {
    design.validate()?;
    fit.validate()?;
    let mesh = match design {
        LimbArmorDesign::Greave(d) => shapes::greave(d, fit),
        LimbArmorDesign::Cuisse(d) => shapes::cuisse(d, fit),
        LimbArmorDesign::Rerebrace(d) => shapes::rerebrace(d, fit),
        LimbArmorDesign::Poleyn(d) | LimbArmorDesign::Couter(d) => shapes::joint_cup(d, fit),
        LimbArmorDesign::Spaulder(d) => shapes::spaulder(d, fit),
        LimbArmorDesign::MittenGauntlet(d) => extremities::gauntlet(d, fit),
        LimbArmorDesign::Sabaton(d) => extremities::sabaton(d, fit),
        LimbArmorDesign::LeatherBoot(d) => extremities::boot(d, fit),
    }?;
    Ok(mesh.transformed(fit))
}

/// Generate the mitten's separate thumb defense from its own anatomical frame.
/// The frame spans thumb root to tip; `y` points toward the root and `z` dorsal.
/// Append this to the main hand-frame mesh returned by `generate_limb_armor`.
pub fn generate_gauntlet_thumb(
    design: &GauntletDesign,
    fit: &PartFrame,
) -> Result<PartMesh, GenerateError> {
    LimbArmorDesign::MittenGauntlet(design.clone()).validate()?;
    fit.validate()?;
    Ok(extremities::gauntlet_thumb(design, fit)?.transformed(fit))
}
