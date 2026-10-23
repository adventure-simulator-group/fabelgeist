//! Chart-local needlework, folded edges and beads baked into color/height maps.
use crate::{
    DesignError, GenerateError, Micrometers, Millimeters, Permille, TextileChartSize, TextileColor,
    TextileRegion, TextileRepeatCount,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TextileNeedlework {
    /// Physical scale of the normalized chart; the fitted mesh supplies this
    /// when generated through the character creator or runtime.
    pub chart: TextileChartSize,
    pub strokes: Vec<TextileStroke>,
}
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TextileStroke {
    pub region: TextileRegion,
    pub across: TextileRepeatCount,
    pub along: TextileRepeatCount,
    /// Points within one repeat tile, independent of garment anatomy.
    pub points: Vec<TextilePathPoint>,
    pub closure: TextilePathClosure,
    pub style: TextileStrokeStyle,
}
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TextilePathPoint {
    pub across: Permille,
    pub along: Permille,
}
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum TextilePathClosure {
    Open,
    Closed,
}
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum TextileStrokeStyle {
    Cord {
        diameter: Micrometers,
        color: TextileColor,
    },
    Stitches {
        width: Micrometers,
        length: Millimeters,
        gap: Millimeters,
        relief: Micrometers,
        color: TextileColor,
    },
    Braid {
        width: Millimeters,
        pitch: Millimeters,
        relief: Micrometers,
        color: TextileColor,
    },
    Pearls {
        diameter: Millimeters,
        spacing: Millimeters,
        color: TextileColor,
    },
    Fold {
        width: Millimeters,
        relief: Micrometers,
        profile: TextileFoldProfile,
    },
}
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum TextileFoldProfile {
    Rolled,
    Felled,
}

impl TextileNeedlework {
    pub const MAXIMUM_STROKES: usize = 32;
    pub const REPEAT_RANGE: std::ops::RangeInclusive<u8> = 1..=32;
    pub const POINT_COUNT_RANGE: std::ops::RangeInclusive<usize> = 2..=32;
    pub fn validate(&self) -> Result<(), GenerateError> {
        if self.chart.across.0 == 0
            || self.chart.along.0 == 0
            || self.strokes.is_empty()
            || self.strokes.len() > Self::MAXIMUM_STROKES
        {
            return Err(DesignError::ParametricParameters.into());
        }
        for stroke in &self.strokes {
            stroke.validate()?;
        }
        Ok(())
    }
}
impl TextileStroke {
    fn validate(&self) -> Result<(), GenerateError> {
        self.region.validate()?;
        if !TextileNeedlework::POINT_COUNT_RANGE.contains(&self.points.len())
            || !TextileNeedlework::REPEAT_RANGE.contains(&self.across.0)
            || !TextileNeedlework::REPEAT_RANGE.contains(&self.along.0)
            || self
                .points
                .iter()
                .any(|p| p.across.0 > Permille::ONE.0 || p.along.0 > Permille::ONE.0)
            || self.points.windows(2).any(|p| p[0] == p[1])
            || (self.closure == TextilePathClosure::Closed
                && (self.points.len() < 3 || self.points.first() == self.points.last()))
        {
            return Err(DesignError::ParametricParameters.into());
        }
        self.style.validate()
    }
}
impl TextileStrokeStyle {
    pub(crate) const BRAID_STRANDS: u8 = 3;
    pub const THREAD_RANGE: std::ops::RangeInclusive<u16> = 50..=3000;
    pub const RELIEF_RANGE: std::ops::RangeInclusive<u16> = 50..=3000;
    pub const WIDTH_RANGE: std::ops::RangeInclusive<u16> = 1..=20;
    pub const PITCH_RANGE: std::ops::RangeInclusive<u16> = 1..=30;
    pub const PEARL_RANGE: std::ops::RangeInclusive<u16> = 1..=8;
    pub(crate) fn feature_metres(&self) -> f32 {
        match self {
            Self::Cord { diameter, .. } => diameter.metres(),
            Self::Stitches {
                width, length, gap, ..
            } => width.metres().min(length.metres()).min(gap.metres()),
            Self::Braid { width, pitch, .. } => {
                (width.metres() / f32::from(Self::BRAID_STRANDS)).min(pitch.metres() * 0.25)
            }
            Self::Pearls { diameter, .. } => diameter.metres(),
            Self::Fold { width, .. } => width.metres(),
        }
    }
    fn validate(&self) -> Result<(), GenerateError> {
        let valid = match self {
            Self::Cord { diameter, .. } => Self::THREAD_RANGE.contains(&diameter.0),
            Self::Stitches {
                width,
                length,
                gap,
                relief,
                ..
            } => {
                Self::THREAD_RANGE.contains(&width.0)
                    && Self::PITCH_RANGE.contains(&length.0)
                    && Self::PITCH_RANGE.contains(&gap.0)
                    && Self::RELIEF_RANGE.contains(&relief.0)
            }
            Self::Braid {
                width,
                pitch,
                relief,
                ..
            } => {
                Self::WIDTH_RANGE.contains(&width.0)
                    && Self::PITCH_RANGE.contains(&pitch.0)
                    && Self::RELIEF_RANGE.contains(&relief.0)
            }
            Self::Pearls {
                diameter, spacing, ..
            } => {
                Self::PEARL_RANGE.contains(&diameter.0)
                    && Self::PITCH_RANGE.contains(&spacing.0)
                    && spacing.0 >= diameter.0
            }
            Self::Fold { width, relief, .. } => {
                Self::WIDTH_RANGE.contains(&width.0) && Self::RELIEF_RANGE.contains(&relief.0)
            }
        };
        if valid {
            Ok(())
        } else {
            Err(DesignError::ParametricParameters.into())
        }
    }
}
