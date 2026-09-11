//! Authored plate families fitted to anatomical frames, independently of body topology.
//!
//! Long plates, joint cups, lamed shoulders, mittens and footwear have separate
//! controls because their openings and articulation are structurally different.

use serde::{Deserialize, Serialize};

use crate::parametric::{PartFrame, PartMesh};
use crate::{DesignError, GenerateError, Millimeters, Permille, PlateFluting};

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
    pub ankle_extension: Millimeters,
    pub fluting: Option<PlateFluting>,
    pub calf_height: Permille,
    pub knee_taper: Permille,

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
            ankle_extension: Millimeters(15),
            fluting: None,
            calf_height: Permille(660),
            knee_taper: Permille(880),
            gauge: PlateGauge::default(),
            length: Permille(970),
            ankle_taper: Permille(630),
            shin_ridge: Millimeters(5),
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct CuisseDesign {
    pub fluting: Option<PlateFluting>,
    pub center_ridge: Millimeters,
    pub upper_edge_slope: Millimeters,

    pub gauge: PlateGauge,
    pub length: Permille,
    pub knee_taper: Permille,
    /// Circumferential coverage: half the thigh to a deep three-quarter return.
    pub wrap: Permille,
}

impl Default for CuisseDesign {
    fn default() -> Self {
        Self {
            fluting: None,
            center_ridge: Millimeters(0),
            upper_edge_slope: Millimeters(0),
            gauge: PlateGauge::default(),
            length: Permille(920),
            knee_taper: Permille(740),
            wrap: Permille(720),
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct RerebraceDesign {
    pub fluting: Option<PlateFluting>,
    pub center_ridge: Millimeters,
    pub section_depth: Permille,

    pub gauge: PlateGauge,
    pub length: Permille,
    pub distal_taper: Permille,
    pub wrap: Permille,
}

impl Default for RerebraceDesign {
    fn default() -> Self {
        Self {
            fluting: None,
            center_ridge: Millimeters(0),
            section_depth: Permille(1000),
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
    /// Flare of the proximal rim over the adjacent upper-limb plate.
    pub proximal_flare: Millimeters,
    /// Rounds the free end of the fan while retaining its attachment to the cup.
    pub wing_roundness: Permille,
    pub wing_notch: Permille,
    pub fluting: Option<PlateFluting>,
    pub wing_height: Permille,
    /// Distal fan-lobe height relative to the proximal lobe.
    pub distal_wing_scale: Permille,
    pub center_ridge: Millimeters,

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
            proximal_flare: Millimeters(6),
            wing_roundness: Permille(1000),
            fluting: None,
            wing_height: Permille(1000),
            distal_wing_scale: Permille(1000),
            wing_notch: Permille(0),
            center_ridge: Millimeters(0),
            gauge: PlateGauge::default(),
            dome: Permille(200),
            wing: Permille(350),
            length: Permille(1000),
        }
    }

    pub fn couter() -> Self {
        Self {
            proximal_flare: Millimeters(6),
            wing_roundness: Permille(500),
            fluting: None,
            wing_height: Permille(1000),
            distal_wing_scale: Permille(1000),
            wing_notch: Permille(0),
            center_ridge: Millimeters(0),
            gauge: PlateGauge::default(),
            dome: Permille(850),
            wing: Permille(450),
            length: Permille(900),
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct SpaulderDesign {
    /// Front and rear rim flare over the adjoining upper-arm plate.
    pub lower_flare: Millimeters,
    /// Neckward reach of the closed crown apex.
    pub crown_reach: Permille,
    pub fluting: Option<PlateFluting>,
    pub wrap: Permille,
    pub rear_extension: Permille,

    pub gauge: PlateGauge,
    pub length: Permille,
    pub crown: Permille,
    pub lame_count: u8,
}

impl Default for SpaulderDesign {
    fn default() -> Self {
        Self {
            lower_flare: Millimeters(6),
            crown_reach: Permille(800),
            fluting: None,
            wrap: Permille(560),
            rear_extension: Permille(1000),
            gauge: PlateGauge::default(),
            length: Permille(1000),
            crown: Permille(1000),
            lame_count: 4,
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct GauntletDesign {
    /// Additional cuff room for a vambrace beneath the gauntlet.
    pub cuff_clearance: Millimeters,
    pub fluting: Option<PlateFluting>,
    pub knuckle_width: Permille,
    pub knuckle_ridge: Millimeters,

    pub gauge: PlateGauge,
    pub cuff_length: Permille,
    pub cuff_flare: Permille,
    pub finger_lames: u8,
}

impl Default for GauntletDesign {
    fn default() -> Self {
        Self {
            cuff_clearance: Millimeters(6),
            fluting: None,
            knuckle_width: Permille(1000),
            knuckle_ridge: Millimeters(0),
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
    /// Forward trim of the ankle opening to accommodate a greave's lower edge.
    pub ankle_cutaway: Millimeters,
    pub fluting: Option<PlateFluting>,
    pub instep_height: Permille,
    pub toe_roundness: Permille,

    pub gauge: PlateGauge,
    /// Toe box width; supports ordinary rounded and broad early-sixteenth-century forms.
    pub toe_width: Permille,
    pub toe_extension: Millimeters,
    pub lame_count: u8,
}

impl Default for FootArmorDesign {
    fn default() -> Self {
        Self {
            ankle_cutaway: Millimeters(0),
            fluting: None,
            instep_height: Permille(1000),
            toe_roundness: Permille(1000),
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
    Pauldron(crate::PauldronDesign),
    MittenGauntlet(GauntletDesign),
    Sabaton(FootArmorDesign),
    LeatherBoot(BootDesign),
}

impl LimbArmorDesign {
    pub fn fluting_mut(&mut self) -> Option<&mut Option<PlateFluting>> {
        match self {
            Self::Greave(d) => Some(&mut d.fluting),
            Self::Cuisse(d) => Some(&mut d.fluting),
            Self::Rerebrace(d) => Some(&mut d.fluting),
            Self::Poleyn(d) => Some(&mut d.fluting),
            Self::Couter(d) => Some(&mut d.fluting),
            Self::Spaulder(d) => Some(&mut d.fluting),
            Self::Pauldron(d) => Some(&mut d.fluting),
            Self::MittenGauntlet(d) => Some(&mut d.fluting),
            Self::Sabaton(d) => Some(&mut d.fluting),
            Self::LeatherBoot(_) => None,
        }
    }
    pub fn fluting(&self) -> Option<&PlateFluting> {
        match self {
            Self::Greave(d) => d.fluting.as_ref(),
            Self::Cuisse(d) => d.fluting.as_ref(),
            Self::Rerebrace(d) => d.fluting.as_ref(),
            Self::Poleyn(d) => d.fluting.as_ref(),
            Self::Couter(d) => d.fluting.as_ref(),
            Self::Spaulder(d) => d.fluting.as_ref(),
            Self::Pauldron(d) => d.fluting.as_ref(),
            Self::MittenGauntlet(d) => d.fluting.as_ref(),
            Self::Sabaton(d) => d.fluting.as_ref(),
            Self::LeatherBoot(_) => None,
        }
    }
    pub fn validate(&self) -> Result<(), GenerateError> {
        if let Some(pattern) = self.fluting() {
            pattern.validate()?;
        }
        let ratio = |p: Permille, low, high| (low..=high).contains(&p.0);
        let (gauge, valid) = match self {
            Self::Greave(d) => (
                d.gauge,
                d.ankle_extension.0 <= 30
                    && ratio(d.length, 650, 1050)
                    && ratio(d.ankle_taper, 450, 850)
                    && d.shin_ridge.0 <= 15
                    && ratio(d.calf_height, 450, 800)
                    && ratio(d.knee_taper, 700, 1050),
            ),
            Self::Cuisse(d) => (
                d.gauge,
                ratio(d.length, 600, 1050)
                    && ratio(d.knee_taper, 550, 1000)
                    && ratio(d.wrap, 500, 900)
                    && d.center_ridge.0 <= 15
                    && d.upper_edge_slope.0 <= 40,
            ),
            Self::Rerebrace(d) => (
                d.gauge,
                ratio(d.length, 600, 1050)
                    && ratio(d.distal_taper, 600, 1100)
                    && ratio(d.wrap, 550, 950)
                    && d.center_ridge.0 <= 12
                    && ratio(d.section_depth, 850, 1200),
            ),
            Self::Poleyn(d) | Self::Couter(d) => (
                d.gauge,
                ratio(d.dome, 100, 1100)
                    && ratio(d.wing, 0, 650)
                    && ratio(d.length, 650, 1250)
                    && d.wing_notch.0 <= 600
                    && d.wing_roundness.0 <= 1000
                    && d.proximal_flare.0 <= 15
                    && ratio(d.wing_height, 600, 1400)
                    && ratio(d.distal_wing_scale, 500, 2000)
                    && d.center_ridge.0 <= 12,
            ),
            Self::Spaulder(d) => (
                d.gauge,
                ratio(d.length, 650, 1250)
                    && ratio(d.crown, 850, 1350)
                    && ratio(d.crown_reach, 500, 900)
                    && d.lower_flare.0 <= 15
                    && (2..=7).contains(&d.lame_count)
                    && ratio(d.wrap, 450, 700)
                    && ratio(d.rear_extension, 800, 1400),
            ),
            Self::Pauldron(d) => (d.gauge, d.valid_shape()),
            Self::MittenGauntlet(d) => (
                d.gauge,
                d.cuff_clearance.0 <= 15
                    && ratio(d.cuff_length, 150, 650)
                    && ratio(d.cuff_flare, 1050, 1600)
                    && (2..=6).contains(&d.finger_lames)
                    && ratio(d.knuckle_width, 850, 1200)
                    && d.knuckle_ridge.0 <= 8,
            ),
            Self::Sabaton(d) => (
                d.gauge,
                ratio(d.toe_width, 800, 1450)
                    && d.toe_extension.0 <= 100
                    && d.ankle_cutaway.0 <= 30
                    && ratio(d.instep_height, 850, 1250)
                    && ratio(d.toe_roundness, 500, 1800)
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
        LimbArmorDesign::Pauldron(d) => return crate::pauldron::generate(d, fit),
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
