//! Garment-independent colored fabric layers in a normalized cloth chart.
//!
//! Across runs around a garment; along runs from its proximal edge to its hem
//! or cuff. Regions use the whole chart; motif repeats use their region's chart.
use serde::{Deserialize, Serialize};

use crate::{DesignError, GenerateError, Permille, TextileColor};

pub const MAX_TEXTILE_LAYERS: usize = 12;

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TextileAppearance {
    pub color: TextileColor,
    pub finish: TextileFinish,
    pub layers: TextileLayers,
}

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct TextileLayers(pub Vec<TextileLayer>);

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TextileLayer {
    pub region: TextileRegion,
    pub fill: TextileFill,
    pub color: TextileColor,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum TextileFinish {
    Linen,
    Wool,
    Silk,
    Velvet,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum TextileAxis {
    Across,
    Along,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum TextileRegion {
    Whole,
    /// The interval is measured along the selected chart axis.
    Band {
        axis: TextileAxis,
        start: Permille,
        end: Permille,
    },
    /// A bounded border or inset on both chart axes.
    Panel {
        across: TextileSpan,
        along: TextileSpan,
    },
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TextileSpan {
    pub start: Permille,
    pub end: Permille,
}

/// Repetitions in one chart or motif feature; each use validates its limit.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct TextileRepeatCount(pub u8);

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum TextileFill {
    Solid,
    Stripes {
        axis: TextileAxis,
        repeats: TextileRepeatCount,
        width: Permille,
        phase: Permille,
    },
    Motifs {
        motif: TextileMotif,
        across: TextileRepeatCount,
        along: TextileRepeatCount,
        scale: Permille,
        rotation: Permille,
        stagger: Permille,
    },
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum TextileMotif {
    Lozenge,
    Rosette {
        petals: TextileRepeatCount,
    },
    /// Mirrored foliage and a crowned flower; a stylized woven motif, not an
    /// attribution to a particular historical textile.
    Palmette {
        leaf_pairs: TextileRepeatCount,
        spread: Permille,
    },
    /// Connected mirrored vines with curling branches and lobed leaves.
    Scrollwork {
        branches: TextileRepeatCount,
        curl: Permille,
    },
}

impl TextileAppearance {
    pub fn solid(color: TextileColor, finish: TextileFinish) -> Self {
        Self {
            color,
            finish,
            layers: TextileLayers::default(),
        }
    }

    pub fn validate(&self) -> Result<(), GenerateError> {
        if self.layers.0.len() > MAX_TEXTILE_LAYERS {
            return Err(DesignError::ParametricParameters.into());
        }
        for layer in &self.layers.0 {
            layer.validate()?;
        }
        Ok(())
    }

    pub fn material(&self) -> crate::ArmorComponentMaterial {
        let mut material = self.color.material();
        material.roughness = match self.finish {
            TextileFinish::Linen => 0.88,
            TextileFinish::Wool => 0.92,
            TextileFinish::Silk => 0.46,
            TextileFinish::Velvet => 0.78,
        };
        material
    }
}

impl TextileLayer {
    fn validate(&self) -> Result<(), GenerateError> {
        match self.region {
            TextileRegion::Whole => {}
            TextileRegion::Band { start, end, .. } => TextileSpan { start, end }.validate()?,
            TextileRegion::Panel { across, along } => {
                across.validate()?;
                along.validate()?;
            }
        }
        self.fill.validate()
    }
}

impl TextileSpan {
    fn validate(self) -> Result<(), GenerateError> {
        if self.start.0 >= self.end.0 || self.end.0 > Permille::ONE.0 {
            return Err(DesignError::ParametricParameters.into());
        }
        Ok(())
    }
}

impl TextileFill {
    fn validate(&self) -> Result<(), GenerateError> {
        let repeat = |n: TextileRepeatCount| (1..=64).contains(&n.0);
        let share = |n: Permille| n.0 <= Permille::ONE.0;
        let valid = match self {
            Self::Solid => true,
            Self::Stripes {
                repeats,
                width,
                phase,
                ..
            } => repeat(*repeats) && (1..=1000).contains(&width.0) && share(*phase),
            Self::Motifs {
                motif,
                across,
                along,
                scale,
                rotation,
                stagger,
            } => {
                motif.validate()?;
                repeat(*across)
                    && repeat(*along)
                    && (100..=1000).contains(&scale.0)
                    && share(*rotation)
                    && share(*stagger)
            }
        };
        if !valid {
            return Err(DesignError::ParametricParameters.into());
        }
        Ok(())
    }
}

impl TextileMotif {
    fn validate(&self) -> Result<(), GenerateError> {
        let valid = match self {
            Self::Lozenge => true,
            Self::Rosette { petals } => (3..=12).contains(&petals.0),
            Self::Palmette { leaf_pairs, spread } => {
                (1..=5).contains(&leaf_pairs.0) && (250..=1000).contains(&spread.0)
            }
            Self::Scrollwork { branches, curl } => {
                (1..=5).contains(&branches.0) && (250..=1000).contains(&curl.0)
            }
        };
        if !valid {
            return Err(DesignError::ParametricParameters.into());
        }
        Ok(())
    }
}
