use serde::{Deserialize, Serialize};

use crate::{GenerateError, Millimeters, Permille, TextileColor};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum PuffAndSlashKind {
    Sleeve,
    Hose,
}

/// Surface construction shared by sleeves, hose, skirts and dresses.
/// Counts are authored repeats; zero slashes keeps a closed puffed surface.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PuffAndSlashPattern {
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
    pub rotation: Permille,
    pub outer_color: TextileColor,
    pub undercloth_color: TextileColor,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PuffAndSlashDesign {
    pub kind: PuffAndSlashKind,
    pub pattern: PuffAndSlashPattern,
    pub length: Permille,
    /// Placement within the limb span: zero distal, 500 centered, 1000 proximal.
    pub proximal_position: Permille,
    pub clearance: Millimeters,
    pub thickness: Millimeters,
}

impl Default for PuffAndSlashPattern {
    fn default() -> Self {
        Self {
            puff_count: 3,
            puff_fullness: Millimeters(45),
            distal_fullness: Permille(650),
            puff_roundness: Permille(850),
            slash_count: 8,
            slash_width: Permille(300),
            slash_length: Permille(700),
            constriction_width: Permille(120),
            rotation: Permille(0),
            outer_color: TextileColor([150, 32, 28]),
            undercloth_color: TextileColor([224, 184, 58]),
        }
    }
}

impl Default for PuffAndSlashDesign {
    fn default() -> Self {
        Self {
            kind: PuffAndSlashKind::Sleeve,
            pattern: PuffAndSlashPattern::default(),
            length: Permille(950),
            proximal_position: Permille(750),
            clearance: Millimeters(4),
            thickness: Millimeters(2),
        }
    }
}

impl PuffAndSlashPattern {
    pub fn validate(&self) -> Result<(), GenerateError> {
        let valid = (1..=8).contains(&self.puff_count)
            && (5..=90).contains(&self.puff_fullness.0)
            && (250..=1500).contains(&self.distal_fullness.0)
            && (400..=2500).contains(&self.puff_roundness.0)
            && (self.slash_count == 0 || (3..=16).contains(&self.slash_count))
            && (50..=650).contains(&self.slash_width.0)
            && (300..=900).contains(&self.slash_length.0)
            && (40..=350).contains(&self.constriction_width.0)
            && self.rotation.0 <= Permille::ONE.0;
        if !valid {
            return Err(crate::DesignError::ParametricParameters.into());
        }
        Ok(())
    }

    /// Native shader words shared by each placement's coordinate kernel.
    pub(crate) fn shader_words(&self) -> [f32; 5] {
        [
            f32::from(self.puff_count),
            self.puff_fullness.metres(),
            self.distal_fullness.unit(),
            self.puff_roundness.unit(),
            self.constriction_width.unit(),
        ]
    }
}

impl PuffAndSlashDesign {
    pub fn validate(&self) -> Result<(), GenerateError> {
        self.pattern.validate()?;
        if !(350..=1000).contains(&self.length.0)
            || self.proximal_position.0 > Permille::ONE.0
            || !(1..=15).contains(&self.clearance.0)
            || !(1..=6).contains(&self.thickness.0)
        {
            return Err(crate::DesignError::ParametricParameters.into());
        }
        Ok(())
    }
}
