use serde::{Deserialize, Serialize};

use crate::{ArmorComponentMaterial, GenerateError, Millimeters, Permille};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum PuffAndSlashKind {
    Sleeve,
    Hose,
}

/// Serialized sRGB textile color.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct TextileColor(pub [u8; 3]);

impl TextileColor {
    pub fn material(self) -> ArmorComponentMaterial {
        ArmorComponentMaterial {
            base_color: [
                f32::from(self.0[0]) / 255.0,
                f32::from(self.0[1]) / 255.0,
                f32::from(self.0[2]) / 255.0,
                1.0,
            ],
            metallic: 0.0,
            roughness: 0.92,
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PuffAndSlashDesign {
    pub kind: PuffAndSlashKind,
    pub puff_count: u8,
    pub puff_fullness: Millimeters,
    /// Fullness at the distal end relative to the proximal end.
    pub distal_fullness: Permille,
    /// Bulb profile exponent: 1000 is a sine profile; lower values broaden it.
    pub puff_roundness: Permille,
    pub slash_count: u8,
    /// Fraction of each circumferential repeat occupied by the opening.
    pub slash_width: Permille,
    /// Fraction of each puff length occupied by open slashes.
    pub slash_length: Permille,
    /// Fraction of each puff course reserved for fitted constriction bands.
    pub constriction_width: Permille,
    pub length: Permille,
    /// Placement within the limb span: zero distal, 500 centered, 1000 proximal.
    pub proximal_position: Permille,
    pub rotation: Permille,
    pub clearance: Millimeters,
    pub thickness: Millimeters,
    pub outer_color: TextileColor,
    pub undercloth_color: TextileColor,
}

impl Default for PuffAndSlashDesign {
    fn default() -> Self {
        Self {
            kind: PuffAndSlashKind::Sleeve,
            puff_count: 3,
            puff_fullness: Millimeters(45),
            distal_fullness: Permille(650),
            puff_roundness: Permille(850),
            slash_count: 8,
            slash_width: Permille(300),
            slash_length: Permille(700),
            constriction_width: Permille(120),
            length: Permille(950),
            proximal_position: Permille(750),
            rotation: Permille(0),
            clearance: Millimeters(4),
            thickness: Millimeters(2),
            outer_color: TextileColor([150, 32, 28]),
            undercloth_color: TextileColor([224, 184, 58]),
        }
    }
}

impl PuffAndSlashDesign {
    pub fn validate(&self) -> Result<(), GenerateError> {
        let valid = (1..=8).contains(&self.puff_count)
            && (5..=90).contains(&self.puff_fullness.0)
            && (250..=1_500).contains(&self.distal_fullness.0)
            && (400..=2_500).contains(&self.puff_roundness.0)
            && (self.slash_count == 0 || (3..=16).contains(&self.slash_count))
            && (50..=650).contains(&self.slash_width.0)
            && (300..=900).contains(&self.slash_length.0)
            && (40..=350).contains(&self.constriction_width.0)
            && (350..=1_000).contains(&self.length.0)
            && self.proximal_position.0 <= 1_000
            && self.rotation.0 <= 1_000
            && (1..=15).contains(&self.clearance.0)
            && (1..=6).contains(&self.thickness.0);
        if !valid {
            return Err(crate::DesignError::ParametricParameters.into());
        }
        Ok(())
    }
}
