//! Pattern-cut textile and mail carriers and articulated waist/neck defenses.
//!
//! These surfaces are authored independently of anatomical triangulation. Mail
//! represents the garment envelope; individual links belong to material detail.
use serde::{Deserialize, Serialize};

use crate::{DesignError, GenerateError, Millimeters, PartFrame, PartMesh, Permille};

#[path = "garment_armor_shapes.rs"]
mod shapes;
#[path = "garment_armor_shell.rs"]
mod shell;

pub const GARMENT_RING_SEGMENTS: usize = 48;
pub const GARMENT_AXIAL_SEGMENTS: usize = 16;
pub const GARMENT_PANEL_ACROSS: usize = 32;
pub const GARMENT_PANEL_ALONG: usize = 12;
pub const GARMENT_ARMPIT_ROW: usize = 9;
pub const GARMENT_SHOULDER_DEPTH_SEGMENTS: usize = 8;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum GarmentArmorKind {
    ArmingDoublet,
    Brigandine,
    JackOfPlates,
    Fauld,
    MailChausses,
    MailShirt,
    MailSkirt,
    MailSleeve,
    PaddedChausses,
    PaddedSkirt,
    QuiltedSleeve,
    Tassets,
    Gorget,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct GarmentArmorDesign {
    pub kind: GarmentArmorKind,
    pub clearance: Millimeters,
    pub wall_thickness: Millimeters,
    /// Axial length relative to the anatomical fitting frame.
    pub length: Permille,
    /// Additional lower-edge width, as a fraction of fitted width.
    pub flare: Permille,
    /// Waist width relative to fitted chest width for torso patterns.
    pub waist: Permille,
    /// Number of articulated plates for the fauld and each tasset.
    pub lame_count: u8,
}

impl GarmentArmorDesign {
    pub fn new(kind: GarmentArmorKind) -> Self {
        let padded = matches!(
            kind,
            GarmentArmorKind::ArmingDoublet
                | GarmentArmorKind::PaddedChausses
                | GarmentArmorKind::PaddedSkirt
                | GarmentArmorKind::QuiltedSleeve
        );
        Self {
            kind,
            clearance: Millimeters(if padded { 3 } else { 10 }),
            wall_thickness: Millimeters(if padded { 5 } else { 3 }),
            length: Permille(1_000),
            flare: Permille(
                if matches!(
                    kind,
                    GarmentArmorKind::MailSkirt | GarmentArmorKind::PaddedSkirt
                ) {
                    0
                } else {
                    180
                },
            ),
            waist: Permille(if kind == GarmentArmorKind::MailShirt {
                1_100
            } else if kind == GarmentArmorKind::JackOfPlates {
                960
            } else {
                900
            }),
            lame_count: 4,
        }
    }

    pub fn validate(&self) -> Result<(), GenerateError> {
        if !(1..=40).contains(&self.clearance.0)
            || !(1..=16).contains(&self.wall_thickness.0)
            || !(500..=1_300).contains(&self.length.0)
            || self.flare.0 > 500
            || !(800..=1_100).contains(&self.waist.0)
            || !(1..=8).contains(&self.lame_count)
        {
            return Err(DesignError::ParametricParameters.into());
        }
        Ok(())
    }
}

/// A torso frame spans waist to shoulder; skirts span their full hip/hem region;
/// sleeves and chausses use one limb frame. Tassets use the combined hip frame
/// and produce two separate fronts. A gorget frame bounds the neck itself.
pub fn generate_garment_armor(
    design: &GarmentArmorDesign,
    fit: &PartFrame,
) -> Result<PartMesh, GenerateError> {
    design.validate()?;
    fit.validate()?;
    let mesh = match design.kind {
        GarmentArmorKind::ArmingDoublet
        | GarmentArmorKind::Brigandine
        | GarmentArmorKind::JackOfPlates
        | GarmentArmorKind::MailShirt => shapes::torso(design, fit),
        GarmentArmorKind::Tassets => shapes::tassets(design, fit),
        GarmentArmorKind::Gorget => shapes::gorget(design, fit),
        _ => shapes::tube(design, fit),
    }?;
    Ok(mesh.transformed(fit))
}
