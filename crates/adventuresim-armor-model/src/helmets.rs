//! Helmet families have their own construction and style controls. A barbute's
//! integrated face opening is deliberately distinct from a burgonet's peak and
//! separate cheek plates; a visor is a separate solid with a real sight gap.

#[path = "helmets_close.rs"]
mod close;
#[path = "helmets_close_design.rs"]
mod close_design;
pub use close_design::CloseHelmetDesign;
#[path = "visor_breaths.rs"]
mod breaths;
pub use breaths::{SlotInclination, VentSides, VisorBreaths};
#[path = "helmets_close_profile.rs"]
mod close_profile;
pub use close_profile::{CloseHelmetProfile, generate_close_helmet};
#[path = "helmets_drape.rs"]
mod drape;
pub use drape::{
    COIF_DRAPE_SECTIONS, CoifDrapeProfile, CoifDrapeSection, CoifFlapDrape, CoifNeckDrape,
    generate_coif_with_drape,
};
#[path = "helmets_barbute.rs"]
mod barbute;
#[path = "helmets_buffe.rs"]
mod buffe;
#[path = "helmets_burgonet.rs"]
mod burgonet;
pub use buffe::{BuffeCourses, BuffeDesign};
#[path = "helmets_cheek.rs"]
mod cheek;
#[path = "helmets_coif.rs"]
mod coif;
#[path = "helmets_crown_mesh.rs"]
mod crown_mesh;
#[path = "helmets_geometry.rs"]
mod geometry;
#[path = "helmets_sallet.rs"]
mod sallet;
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
    pub front_reach: Permille,
    pub back_reach: Permille,
    pub back_sweep: Permille,
    pub crown: crate::HelmetCrown,
    pub fit: HelmetFit,
    pub brim_width: Millimeters,
    pub brim_sweep: Millimeters,
    pub comb_height: Millimeters,
}

impl Default for MorionDesign {
    fn default() -> Self {
        Self {
            front_reach: Permille(1250),
            back_reach: Permille(1250),
            back_sweep: Permille(1000),
            crown: crate::HelmetCrown::default(),
            fit: HelmetFit::default(),
            brim_width: Millimeters(35),
            brim_sweep: Millimeters(38),
            comb_height: Millimeters(75),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct KettleHatDesign {
    pub crown: crate::HelmetCrown,
    pub fit: HelmetFit,
    pub brim_width: Millimeters,
    pub brim_drop: Millimeters,
}

impl Default for KettleHatDesign {
    fn default() -> Self {
        Self {
            crown: crate::HelmetCrown::default(),
            fit: HelmetFit::default(),
            brim_width: Millimeters(55),
            brim_drop: Millimeters(8),
        }
    }
}

/// Eye and mouth apertures are independent, allowing T and open Y outlines.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct BarbuteDesign {
    pub rear_edge_lift: Millimeters,
    pub eye_height: Millimeters,
    pub opening_roundness: Permille,
    pub chin_taper: Permille,
    pub nape_flare: Millimeters,
    pub crown: crate::HelmetCrown,
    pub fit: HelmetFit,
    pub eye_opening: crate::Milliradians,
    pub mouth_opening: crate::Milliradians,
    pub cheek_depth: Permille,
}

impl Default for BarbuteDesign {
    fn default() -> Self {
        Self {
            rear_edge_lift: Millimeters(20),
            eye_height: Millimeters(28),
            opening_roundness: Permille(650),
            chin_taper: Permille(840),
            nape_flare: Millimeters(3),
            crown: crate::HelmetCrown::default(),
            fit: HelmetFit::default(),
            eye_opening: crate::Milliradians(700),
            mouth_opening: crate::Milliradians(210),
            cheek_depth: Permille(930),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct BurgonetDesign {
    pub buffe: Option<BuffeDesign>,
    /// Fraction of the nape covered by a separate overlapping neck guard.
    pub neck_guard_fraction: Permille,
    pub chin_tab: Millimeters,
    pub nape_depth: Permille,
    pub nape_taper: Permille,
    /// Fore-aft neck constriction, independent of lateral neck width.
    pub nape_recession: Millimeters,
    pub cheek_fluting: Option<crate::PlateFluting>,
    pub cheek_width: Permille,
    pub cheek_taper: Permille,
    pub peak_drop: Millimeters,
    /// Central rise of the peak, independent of its downward pitch.
    pub peak_rise: Millimeters,
    pub crown: crate::HelmetCrown,
    pub fit: HelmetFit,
    pub peak_length: Millimeters,
    pub comb_height: Millimeters,
    pub cheek_depth: Permille,
    pub neck_flare: Millimeters,
}

impl Default for BurgonetDesign {
    fn default() -> Self {
        Self {
            buffe: None,
            neck_guard_fraction: Permille(400),
            chin_tab: Millimeters(20),
            nape_depth: Permille(920),
            nape_taper: Permille(880),
            nape_recession: Millimeters(12),
            cheek_fluting: None,
            cheek_width: Permille(1000),
            cheek_taper: Permille(900),
            peak_drop: Millimeters(6),
            peak_rise: Millimeters(0),
            crown: crate::HelmetCrown::default(),
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
    /// Raises the posterior skirt independently of the face-side coverage.
    pub rear_edge_lift: Millimeters,
    /// Half of the frontal opening angle, in thousandths of a radian.
    pub opening_width: crate::Milliradians,
    pub opening_sweep: Permille,
    pub cheek_depth: Permille,
    pub tail_width: Permille,
    pub crown: crate::HelmetCrown,
    pub fit: HelmetFit,
    pub tail_length: Millimeters,
    pub tail_drop: Millimeters,
    pub brow_projection: Millimeters,
}

impl Default for SalletDesign {
    fn default() -> Self {
        Self {
            rear_edge_lift: Millimeters(0),
            cheek_depth: Permille(550),
            opening_width: crate::Milliradians(1047),
            opening_sweep: Permille(600),
            tail_width: Permille(1000),
            crown: crate::HelmetCrown::default(),
            fit: HelmetFit::default(),
            tail_length: Millimeters(70),
            tail_drop: Millimeters(25),
            brow_projection: Millimeters(8),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct VisoredSalletDesign {
    pub visor_height: Millimeters,
    /// Height retained by the side panel below the rising pivot attachment.
    pub side_panel: Permille,
    pub pivot_rise: Millimeters,
    pub skull: SalletDesign,
    pub visor_projection: Millimeters,
    pub sight_gap: Millimeters,
}

impl Default for VisoredSalletDesign {
    fn default() -> Self {
        Self {
            visor_height: Millimeters(55),
            side_panel: Permille(800),
            pivot_rise: Millimeters(45),
            skull: SalletDesign::default(),
            visor_projection: Millimeters(12),
            sight_gap: Millimeters(8),
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

    pub fn crown(&self) -> Option<&crate::HelmetCrown> {
        match self {
            Self::Morion(d) => Some(&d.crown),
            Self::KettleHat(d) => Some(&d.crown),
            Self::Barbute(d) => Some(&d.crown),
            Self::Burgonet(d) => Some(&d.crown),
            Self::Sallet(d) => Some(&d.crown),
            Self::VisoredSallet(d) => Some(&d.skull.crown),
            Self::CloseHelmet(d) => Some(&d.crown),
            Self::ArmingCap(_) | Self::MailCoif(_) => None,
        }
    }

    pub fn validate(&self) -> Result<(), DesignError> {
        if let Some(crown) = self.crown() {
            crown.validate()?;
        }
        let fit = self.fit();
        let valid_fit = (3..=30).contains(&fit.clearance.0)
            && (1..=8).contains(&fit.wall_thickness.0)
            && (900..=1300).contains(&fit.crown_height.0);
        let valid_shape = match self {
            Self::Morion(d) => {
                (800..=2000).contains(&d.front_reach.0)
                    && (800..=2000).contains(&d.back_reach.0)
                    && (500..=1500).contains(&d.back_sweep.0)
                    && (20..=65).contains(&d.brim_width.0)
                    && (15..=60).contains(&d.brim_sweep.0)
                    && (15..=85).contains(&d.comb_height.0)
            }
            Self::KettleHat(d) => {
                (30..=85).contains(&d.brim_width.0) && (5..=45).contains(&d.brim_drop.0)
            }
            Self::Barbute(d) => {
                (14..=45).contains(&d.eye_height.0)
                    && d.opening_roundness.0 <= 1000
                    && (750..=1000).contains(&d.chin_taper.0)
                    && d.nape_flare.0 <= 12
                    && (500..=850).contains(&d.eye_opening.0)
                    && (120..=400).contains(&d.mouth_opening.0)
                    && d.rear_edge_lift.0 <= 60
                    && (750..=1400).contains(&d.cheek_depth.0)
            }
            Self::Burgonet(d) => {
                if let Some(buffe) = &d.buffe {
                    buffe.validate()?;
                }
                if let Some(pattern) = &d.cheek_fluting {
                    pattern.validate()?;
                }
                (650..=1400).contains(&d.nape_depth.0)
                    && d.chin_tab.0 <= 35
                    && d.nape_recession.0 <= 30
                    && (550..=1000).contains(&d.nape_taper.0)
                    && (750..=1250).contains(&d.cheek_width.0)
                    && (800..=1050).contains(&d.cheek_taper.0)
                    && d.peak_drop.0 <= 20
                    && d.peak_rise.0 <= 20
                    && (20..=65).contains(&d.peak_length.0)
                    && (0..=60).contains(&d.comb_height.0)
                    && (750..=1150).contains(&d.cheek_depth.0)
                    && (200..=550).contains(&d.neck_guard_fraction.0)
                    && (5..=40).contains(&d.neck_flare.0)
            }
            Self::Sallet(d) => valid_sallet(d),
            Self::VisoredSallet(d) => {
                (35..=75).contains(&d.visor_height.0)
                    && (300..=1000).contains(&d.side_panel.0)
                    && (25..=60).contains(&d.pivot_rise.0)
                    && valid_sallet(&d.skull)
                    && valid_visor(d.visor_projection, d.sight_gap)
            }
            Self::CloseHelmet(d) => {
                d.validate_shape()?;
                true
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
    (550..=1250).contains(&d.opening_width.0)
        && d.opening_sweep.0 <= 2000
        && d.rear_edge_lift.0 <= 60
        && (350..=1400).contains(&d.cheek_depth.0)
        && (650..=1400).contains(&d.tail_width.0)
        && (35..=110).contains(&d.tail_length.0)
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
