//! Garment-independent colored fabric layers in a normalized cloth chart.
//!
//! Across runs around a garment; along runs from its proximal edge to its hem
//! or cuff. Regions clip the whole chart; motifs can repeat within each region
//! or through one continuous garment chart.
use serde::{Deserialize, Serialize};

use crate::{DesignError, GenerateError, Permille, TextileColor};

pub const MAX_TEXTILE_LAYERS: usize = 12;

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TextileAppearance {
    pub color: TextileColor,
    pub finish: TextileFinish,
    pub chart_boundary: TextileChartBoundary,
    /// Base cloth opacity; colored decorative layers remain opaque.
    pub opacity: Permille,
    pub layers: TextileLayers,
    pub relief: Option<Box<crate::TextilePleatRelief>>,
    pub needlework: Option<Box<crate::TextileNeedlework>>,
}

/// Texture addressing and height differences use the same physical boundary.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum TextileChartBoundary {
    CyclicAcross,
    Open,
}
impl TextileChartBoundary {
    pub(crate) fn neighbors(self, column: u32, width: u32) -> [u32; 2] {
        match self {
            Self::CyclicAcross => [(column + width - 1) % width, (column + 1) % width],
            Self::Open => [column.saturating_sub(1), (column + 1).min(width - 1)],
        }
    }
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

/// Translation in repeat-tile units, before alternating-row staggering.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TextileOffset {
    pub across: Permille,
    pub along: Permille,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum TextileFinish {
    Linen,
    Wool,
    Silk,
    Velvet,
    MetallicThread,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum TextileAxis {
    Across,
    Along,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum TextilePatternSpace {
    /// Rescale the ornament within its bounded region.
    Region,
    /// Clip one continuous fabric pattern through any number of regions.
    Garment,
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

/// Yarn periods across a normalized material chart, independent of motif repeats.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct TextileThreadCount(pub u16);

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum TextileFill {
    Solid,
    /// Crossing warp and weft yarns; the gaps retain the base cloth opacity.
    Weave {
        warp: TextileThreadCount,
        weft: TextileThreadCount,
        width: Permille,
    },
    Stripes {
        axis: TextileAxis,
        repeats: TextileRepeatCount,
        width: Permille,
        phase: Permille,
    },
    Motifs {
        space: TextilePatternSpace,
        motif: TextileMotif,
        across: TextileRepeatCount,
        along: TextileRepeatCount,
        scale: Permille,
        rotation: Permille,
        stagger: Permille,
        offset: TextileOffset,
    },
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum TextileMotif {
    Lozenge,
    Rosette {
        petals: TextileRepeatCount,
    },
    /// Concentric thread rings around a small lobed flower.
    Medallion {
        petals: TextileRepeatCount,
        rings: TextileRepeatCount,
        threads: TextileRepeatCount,
        stroke: Permille,
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
    /// Upright nested flowers and veined foliage on a curved stem.
    Damask {
        leaf_pairs: TextileRepeatCount,
        spread: Permille,
    },
}

impl TextileThreadCount {
    pub const RANGE: std::ops::RangeInclusive<u16> = 1..=1024;
}

impl TextileAppearance {
    pub const OPACITY_RANGE: std::ops::RangeInclusive<u16> = 0..=Permille::ONE.0;

    pub fn solid(color: TextileColor, finish: TextileFinish) -> Self {
        Self {
            color,
            finish,
            chart_boundary: TextileChartBoundary::CyclicAcross,
            opacity: Permille::ONE,
            layers: TextileLayers::default(),
            relief: None,
            needlework: None,
        }
    }

    pub fn validate(&self) -> Result<(), GenerateError> {
        if !Self::OPACITY_RANGE.contains(&self.opacity.0)
            || self.layers.0.len() > MAX_TEXTILE_LAYERS
        {
            return Err(DesignError::ParametricParameters.into());
        }
        if let (Some(relief), Some(needlework)) = (&self.relief, &self.needlework)
            && relief.chart != needlework.chart
        {
            return Err(DesignError::ParametricParameters.into());
        }
        if let Some(needlework) = &self.needlework {
            needlework.validate()?;
        }
        if let Some(relief) = &self.relief {
            relief.validate()?;
        }
        for layer in &self.layers.0 {
            layer.validate()?;
        }
        Ok(())
    }

    /// A conservative material classification; ornament can make covered
    /// texels opaque without making the underlying cloth opaque.
    pub fn is_translucent(&self) -> bool {
        self.opacity.0 < Permille::ONE.0
    }

    pub fn material(&self) -> crate::ArmorComponentMaterial {
        let mut material = self.color.material();
        material.base_color[3] = self.opacity.unit();
        material.roughness = match self.finish {
            TextileFinish::Linen => 0.88,
            TextileFinish::Wool => 0.92,
            TextileFinish::Silk => 0.46,
            TextileFinish::Velvet => 0.78,
            TextileFinish::MetallicThread => 0.65,
        };
        material.metallic = if self.finish == TextileFinish::MetallicThread {
            0.8
        } else {
            0.0
        };
        material
    }
}

impl TextileFinish {
    /// Native PBR adapter: velvet is a diffuse approximation without the
    /// default white glossy lobe. Other finishes retain standard dielectric
    /// specularity. Directional fiber sheen is not represented here.
    pub fn specular_factor(self) -> f32 {
        match self {
            Self::Velvet => 0.0,
            Self::Linen | Self::Wool | Self::Silk | Self::MetallicThread => 1.0,
        }
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

impl TextileRegion {
    pub(crate) fn validate(&self) -> Result<(), GenerateError> {
        match *self {
            Self::Whole => Ok(()),
            Self::Band { start, end, .. } => TextileSpan { start, end }.validate(),
            Self::Panel { across, along } => {
                across.validate()?;
                along.validate()
            }
        }
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
            Self::Weave { warp, weft, width } => {
                TextileThreadCount::RANGE.contains(&warp.0)
                    && TextileThreadCount::RANGE.contains(&weft.0)
                    && (1..=900).contains(&width.0)
            }
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
                offset,
                ..
            } => {
                motif.validate()?;
                repeat(*across)
                    && repeat(*along)
                    && (100..=1000).contains(&scale.0)
                    && share(*rotation)
                    && share(*stagger)
                    && share(offset.across)
                    && share(offset.along)
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
            Self::Medallion {
                petals,
                rings,
                threads,
                stroke,
            } => {
                (3..=12).contains(&petals.0)
                    && (1..=4).contains(&rings.0)
                    && (8..=64).contains(&threads.0)
                    && (5..=80).contains(&stroke.0)
            }
            Self::Palmette { leaf_pairs, spread } | Self::Damask { leaf_pairs, spread } => {
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
