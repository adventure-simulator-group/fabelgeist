//! Helmet families have their own construction and style controls. A barbute's
//! integrated face opening is deliberately distinct from a burgonet's peak and
//! separate cheek plates; a visor is a separate solid with a real sight gap.

#[path = "helmets_close.rs"]
mod close;
#[path = "helmets_drape.rs"]
mod drape;
pub use drape::{
    COIF_DRAPE_SECTIONS, CoifDrapeProfile, CoifDrapeSection, CoifFlapDrape, CoifNeckDrape,
    generate_coif_with_drape,
};
#[path = "helmets_coif.rs"]
mod coif;
#[path = "helmets_geometry.rs"]
mod geometry;
#[path = "helmets_shapes.rs"]
mod shapes;

use serde::{Deserialize, Serialize};

use crate::parametric::{PartFrame, PartMesh};
use crate::{DesignError, GenerateError, Millimeters, Permille};

/// Shared fit controls in physical units. The frame describes the bare head.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct HelmetFit {
    pub clearance: Millimeters,
    pub wall_thickness: Millimeters,
    /// Skull dome height relative to the upper half of the wearer's head.
    pub crown_height: Permille,
}

impl Default for HelmetFit {
    fn default() -> Self {
        Self {
            clearance: Millimeters(12),
            wall_thickness: Millimeters(2),
            crown_height: Permille(1050),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct MorionDesign {
    pub fit: HelmetFit,
    pub brim_width: Millimeters,
    pub brim_sweep: Millimeters,
    pub comb_height: Millimeters,
}

impl Default for MorionDesign {
    fn default() -> Self {
        Self {
            fit: HelmetFit::default(),
            brim_width: Millimeters(35),
            brim_sweep: Millimeters(38),
            comb_height: Millimeters(75),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct KettleHatDesign {
    pub fit: HelmetFit,
    pub brim_width: Millimeters,
    pub brim_drop: Millimeters,
}

impl Default for KettleHatDesign {
    fn default() -> Self {
        Self {
            fit: HelmetFit::default(),
            brim_width: Millimeters(55),
            brim_drop: Millimeters(8),
        }
    }
}

/// Eye and mouth apertures are independent, allowing T and open Y outlines.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct BarbuteDesign {
    pub fit: HelmetFit,
    pub eye_opening: Permille,
    pub mouth_opening: Permille,
    pub cheek_depth: Permille,
}

impl Default for BarbuteDesign {
    fn default() -> Self {
        Self {
            fit: HelmetFit::default(),
            eye_opening: Permille(700),
            mouth_opening: Permille(210),
            cheek_depth: Permille(930),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct BurgonetDesign {
    pub fit: HelmetFit,
    pub peak_length: Millimeters,
    pub comb_height: Millimeters,
    pub cheek_depth: Permille,
    pub neck_flare: Millimeters,
}

impl Default for BurgonetDesign {
    fn default() -> Self {
        Self {
            fit: HelmetFit::default(),
            peak_length: Millimeters(38),
            comb_height: Millimeters(30),
            cheek_depth: Permille(1050),
            neck_flare: Millimeters(22),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SalletDesign {
    pub fit: HelmetFit,
    pub tail_length: Millimeters,
    pub tail_drop: Millimeters,
    pub brow_projection: Millimeters,
}

impl Default for SalletDesign {
    fn default() -> Self {
        Self {
            fit: HelmetFit::default(),
            tail_length: Millimeters(70),
            tail_drop: Millimeters(25),
            brow_projection: Millimeters(8),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct VisoredSalletDesign {
    pub skull: SalletDesign,
    pub visor_projection: Millimeters,
    pub sight_gap: Millimeters,
}

impl Default for VisoredSalletDesign {
    fn default() -> Self {
        Self {
            skull: SalletDesign::default(),
            visor_projection: Millimeters(12),
            sight_gap: Millimeters(8),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CloseHelmetDesign {
    pub fit: HelmetFit,
    pub visor_projection: Millimeters,
    pub sight_gap: Millimeters,
    pub comb_height: Millimeters,
    pub throat_flare: Millimeters,
    /// Upward sweep behind the neck, independently of the front throat lip.
    pub back_edge_lift: Millimeters,
}

impl Default for CloseHelmetDesign {
    fn default() -> Self {
        Self {
            fit: HelmetFit::default(),
            visor_projection: Millimeters(18),
            sight_gap: Millimeters(7),
            comb_height: Millimeters(15),
            throat_flare: Millimeters(28),
            back_edge_lift: Millimeters(32),
        }
    }
}

/// Visby construction: head enclosure with separate breast and back flaps.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CoifDesign {
    pub fit: HelmetFit,
    /// Coverage from the chin down to the anatomical neck base.
    pub neck_coverage: Permille,
    pub front_flap_length: Millimeters,
    pub back_flap_length: Millimeters,
    pub flap_width: Permille,
}

impl Default for CoifDesign {
    fn default() -> Self {
        Self {
            fit: HelmetFit {
                clearance: Millimeters(8),
                wall_thickness: Millimeters(4),
                crown_height: Permille(1000),
            },
            neck_coverage: Permille(1000),
            front_flap_length: Millimeters(110),
            back_flap_length: Millimeters(130),
            flap_width: Permille(1000),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum HelmetKind {
    Morion,
    KettleHat,
    Barbute,
    Burgonet,
    Sallet,
    VisoredSallet,
    CloseHelmet,
    ArmingCap,
    MailCoif,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum HelmetDesign {
    Morion(MorionDesign),
    KettleHat(KettleHatDesign),
    Barbute(BarbuteDesign),
    Burgonet(BurgonetDesign),
    Sallet(SalletDesign),
    VisoredSallet(VisoredSalletDesign),
    CloseHelmet(CloseHelmetDesign),
    ArmingCap(HelmetFit),
    MailCoif(CoifDesign),
}

impl HelmetDesign {
    pub fn catalog(kind: HelmetKind) -> Self {
        match kind {
            HelmetKind::Morion => Self::Morion(MorionDesign::default()),
            HelmetKind::KettleHat => Self::KettleHat(KettleHatDesign::default()),
            HelmetKind::Barbute => Self::Barbute(BarbuteDesign::default()),
            HelmetKind::Burgonet => Self::Burgonet(BurgonetDesign::default()),
            HelmetKind::Sallet => Self::Sallet(SalletDesign::default()),
            HelmetKind::VisoredSallet => Self::VisoredSallet(VisoredSalletDesign::default()),
            HelmetKind::CloseHelmet => Self::CloseHelmet(CloseHelmetDesign::default()),
            HelmetKind::ArmingCap => Self::ArmingCap(HelmetFit {
                clearance: Millimeters(5),
                wall_thickness: Millimeters(5),
                crown_height: Permille(1000),
            }),
            HelmetKind::MailCoif => Self::MailCoif(CoifDesign::default()),
        }
    }

    pub fn fit(&self) -> HelmetFit {
        match self {
            Self::Morion(d) => d.fit,
            Self::KettleHat(d) => d.fit,
            Self::Barbute(d) => d.fit,
            Self::Burgonet(d) => d.fit,
            Self::Sallet(d) => d.fit,
            Self::VisoredSallet(d) => d.skull.fit,
            Self::CloseHelmet(d) => d.fit,
            Self::ArmingCap(d) => *d,
            Self::MailCoif(d) => d.fit,
        }
    }

    pub fn validate(&self) -> Result<(), DesignError> {
        let fit = self.fit();
        let valid_fit = (3..=30).contains(&fit.clearance.0)
            && (1..=8).contains(&fit.wall_thickness.0)
            && (900..=1300).contains(&fit.crown_height.0);
        let valid_shape = match self {
            Self::Morion(d) => {
                (20..=65).contains(&d.brim_width.0)
                    && (15..=60).contains(&d.brim_sweep.0)
                    && (15..=85).contains(&d.comb_height.0)
            }
            Self::KettleHat(d) => {
                (30..=85).contains(&d.brim_width.0) && (5..=45).contains(&d.brim_drop.0)
            }
            Self::Barbute(d) => {
                (500..=850).contains(&d.eye_opening.0)
                    && (120..=400).contains(&d.mouth_opening.0)
                    && (750..=1050).contains(&d.cheek_depth.0)
            }
            Self::Burgonet(d) => {
                (20..=65).contains(&d.peak_length.0)
                    && (0..=60).contains(&d.comb_height.0)
                    && (750..=1150).contains(&d.cheek_depth.0)
                    && (5..=40).contains(&d.neck_flare.0)
            }
            Self::Sallet(d) => valid_sallet(d),
            Self::VisoredSallet(d) => {
                valid_sallet(&d.skull) && valid_visor(d.visor_projection, d.sight_gap)
            }
            Self::CloseHelmet(d) => {
                valid_visor(d.visor_projection, d.sight_gap)
                    && d.comb_height.0 <= 40
                    && (15..=40).contains(&d.throat_flare.0)
                    && (10..=45).contains(&d.back_edge_lift.0)
            }
            Self::ArmingCap(_) => true,
            Self::MailCoif(d) => {
                (800..=1100).contains(&d.neck_coverage.0)
                    && (60..=150).contains(&d.front_flap_length.0)
                    && (70..=170).contains(&d.back_flap_length.0)
                    && (750..=1200).contains(&d.flap_width.0)
            }
        };
        if valid_fit && valid_shape {
            Ok(())
        } else {
            Err(DesignError::ParametricParameters)
        }
    }
}

fn valid_sallet(d: &SalletDesign) -> bool {
    (35..=110).contains(&d.tail_length.0)
        && (10..=45).contains(&d.tail_drop.0)
        && d.brow_projection.0 <= 20
}

fn valid_visor(projection: Millimeters, gap: Millimeters) -> bool {
    (12..=50).contains(&projection.0) && (5..=15).contains(&gap.0)
}

pub fn generate_helmet(design: &HelmetDesign, fit: &PartFrame) -> Result<PartMesh, GenerateError> {
    design.validate()?;
    fit.validate()?;
    let local = shapes::generate(design, fit.half_extents)?;
    Ok(local.transformed(fit))
}
