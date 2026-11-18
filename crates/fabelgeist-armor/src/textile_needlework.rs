//! Chart-local needlework, folded edges and beads baked into color/height maps.
use crate::{
    DesignError, GenerateError, Micrometers, Millimeters, Permille, TextileChartSize, TextileColor,
    TextileRegion, TextileRepeatCount, TextileSurface, TextileThread,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TextileNeedlework {
    /// Physical scale of the normalized chart; the fitted mesh supplies this
    /// when generated through the character creator or runtime.
    pub chart: TextileChartSize,
    /// Where threads cross, the one that stands higher shows. Threads of
    /// equal height join, and the later stroke supplies the color there.
    pub strokes: Vec<TextileStroke>,
    /// Measured size of regions that carry ornament; the fitted mesh
    /// supplies these with the chart. Cloth is not equally wide everywhere
    /// in its chart, least of all round an armhole, and ornament sized by
    /// the whole chart would be stretched where the cloth is wider. A
    /// region not listed takes its share of the whole chart.
    pub regions: Vec<TextileRegionSize>,
}
/// The size on the cloth of one region of a chart.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TextileRegionSize {
    pub region: TextileRegion,
    pub size: TextileChartSize,
}
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TextileStroke {
    pub region: TextileRegion,
    pub across: TextileRepeatCount,
    pub along: TextileRepeatCount,
    /// Points in one repeat tile, independent of garment anatomy. A path may
    /// run on into the next tile; its repeats then overlap and join.
    pub points: Vec<TextilePathPoint>,
    pub closure: TextilePathClosure,
    pub course: TextilePathCourse,
    /// How one repeat differs from the next, as hand work does.
    pub variation: Option<TextileRepeatVariation>,
    pub style: TextileStrokeStyle,
}
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TextilePathPoint {
    pub across: Permille,
    pub along: Permille,
    /// Height of the thread's middle line above the cloth at this point. A
    /// path that rises and falls passes over and under what it crosses, so
    /// a ring leaning out of the cloth links with its neighbours as real
    /// rings do. A thread lying on the cloth has no lift.
    pub lift: Micrometers,
}
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum TextilePathClosure {
    Open,
    Closed,
}
/// How a path runs from one of its points to the next.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum TextilePathCourse {
    /// Straight from point to point, with a corner at each.
    Straight,
    /// One smooth curve through every point.
    Curved,
}
/// Each repeat of a path is turned about the middle of the path and shifted
/// by its own amount. Repeats of a closed chart still join around it.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TextileRepeatVariation {
    /// Largest turn either way; 1000 is a quarter turn.
    pub turn: Permille,
    /// Largest shift along each axis, as a share of the shorter tile side.
    pub shift: Permille,
    /// Strokes with the same seed, tiling and path middle vary together, so
    /// a motif drawn with several strokes stays in one piece.
    pub seed: u8,
}
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum TextileStrokeStyle {
    Cord {
        diameter: Micrometers,
        color: TextileColor,
        thread: TextileThread,
    },
    Stitches {
        width: Micrometers,
        length: Millimeters,
        gap: Millimeters,
        relief: Micrometers,
        color: TextileColor,
        thread: TextileThread,
    },
    /// Flat plait of round strands. Each strand passes alternately over and
    /// under the others: two strands twist, three plait, more interlace.
    Braid {
        strands: TextileRepeatCount,
        /// Strand diameter as a share of an edge-to-edge packing. Lower
        /// values open the plait into separate interlaced cords.
        cover: Permille,
        width: Millimeters,
        /// Length along the path in which one strand crosses over and back.
        pitch: Millimeters,
        relief: Micrometers,
        color: TextileColor,
        thread: TextileThread,
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
    /// Metres of cloth across and along `region`.
    pub(crate) fn extent(&self, region: &TextileRegion) -> [f32; 2] {
        if let Some(measured) = self.regions.iter().find(|entry| &entry.region == region) {
            return [measured.size.across.metres(), measured.size.along.metres()];
        }
        let span = region.span();
        [
            self.chart.across.metres() * span[0],
            self.chart.along.metres() * span[1],
        ]
    }

    /// The regions that carry ornament, each once, without the whole chart.
    pub fn ornamented_regions(&self) -> Vec<TextileRegion> {
        let mut regions = Vec::<TextileRegion>::new();
        for region in self.strokes.iter().map(|stroke| &stroke.region) {
            if *region != TextileRegion::Whole && !regions.contains(region) {
                regions.push(region.clone());
            }
        }
        regions
    }

    pub const MAXIMUM_STROKES: usize = 64;
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
    /// Farthest a path point may lie along either tile axis: one tile on.
    pub const POINT_REACH: Permille = Permille(2000);
    /// Highest a thread's middle line may stand above the cloth.
    pub const MAXIMUM_LIFT: Micrometers = Micrometers(40_000);
    /// Tiles, relative to the sampled one, whose path can reach into it.
    pub(crate) const NEIGHBOR_TILES: std::ops::RangeInclusive<i8> = -2..=1;

    fn validate(&self) -> Result<(), GenerateError> {
        self.region.validate()?;
        if !TextileNeedlework::POINT_COUNT_RANGE.contains(&self.points.len())
            || !TextileNeedlework::REPEAT_RANGE.contains(&self.across.0)
            || !TextileNeedlework::REPEAT_RANGE.contains(&self.along.0)
            || self.variation.is_some_and(|variation| {
                variation.turn.0 > Permille::ONE.0
                    || variation.shift.0 > TextileRepeatVariation::MAXIMUM_SHIFT.0
            })
            || self.points.iter().any(|p| {
                p.across.0 > Self::POINT_REACH.0
                    || p.along.0 > Self::POINT_REACH.0
                    || p.lift.0 > Self::MAXIMUM_LIFT.0
            })
            || self
                .points
                .windows(2)
                .any(|p| (p[0].across, p[0].along) == (p[1].across, p[1].along))
            || (self.closure == TextilePathClosure::Closed
                && (self.points.len() < 3
                    || self.points.first().map(|p| (p.across, p.along))
                        == self.points.last().map(|p| (p.across, p.along))))
        {
            return Err(DesignError::ParametricParameters.into());
        }
        self.style.validate()
    }
}
impl TextileRepeatVariation {
    pub const MAXIMUM_SHIFT: Permille = Permille(500);
    /// Odd multipliers and shift of a multiply-xorshift integer mix.
    const MIX: [u32; 2] = [0x9E37_79B1, 0x85EB_CA6B];
    const MIX_SHIFT: u32 = 15;

    /// A fixed value from -1 to 1 for one repeat tile and one use of it.
    pub(crate) fn offset(self, tile: [u8; 2], channel: u8) -> f32 {
        let mut state = u32::from_le_bytes([self.seed, tile[0], tile[1], channel]);
        for multiplier in Self::MIX {
            state = (state ^ (state >> Self::MIX_SHIFT)).wrapping_mul(multiplier);
        }
        let [low, high, ..] = (state ^ (state >> Self::MIX_SHIFT)).to_le_bytes();
        f32::from(u16::from_le_bytes([low, high])) / f32::from(u16::MAX) * 2. - 1.
    }
}
impl TextileStrokeStyle {
    pub const STRAND_RANGE: std::ops::RangeInclusive<u8> = 2..=8;
    pub const COVER_RANGE: std::ops::RangeInclusive<u16> = 200..=1000;
    pub const BRAID_PITCH_RANGE: std::ops::RangeInclusive<u16> = 1..=80;
    pub const BRAID_WIDTH_RANGE: std::ops::RangeInclusive<u16> = 1..=40;
    pub const THREAD_RANGE: std::ops::RangeInclusive<u16> = 50..=3000;
    /// Couched cord runs from fine thread to heavy rope.
    pub const CORD_RANGE: std::ops::RangeInclusive<u16> = 50..=12000;
    pub const RELIEF_RANGE: std::ops::RangeInclusive<u16> = 50..=3000;
    pub const WIDTH_RANGE: std::ops::RangeInclusive<u16> = 1..=20;
    pub const PITCH_RANGE: std::ops::RangeInclusive<u16> = 1..=30;
    pub const PEARL_RANGE: std::ops::RangeInclusive<u16> = 1..=8;
    /// How the style's thread or beads take the light. A fold is the cloth
    /// itself and has no surface of its own.
    pub(crate) fn surface(&self) -> Option<TextileSurface> {
        match self {
            Self::Cord { thread, .. }
            | Self::Stitches { thread, .. }
            | Self::Braid { thread, .. } => Some(thread.surface()),
            Self::Pearls { .. } => Some(TextileSurface::PEARL),
            Self::Fold { .. } => None,
        }
    }
    pub(crate) fn feature_metres(&self) -> f32 {
        match self {
            Self::Cord { diameter, .. } => diameter.metres(),
            Self::Stitches {
                width, length, gap, ..
            } => width.metres().min(length.metres()).min(gap.metres()),
            Self::Braid {
                strands,
                cover,
                width,
                pitch,
                ..
            } => (width.metres() * cover.unit() / f32::from(strands.0)).min(pitch.metres() * 0.25),
            Self::Pearls { diameter, .. } => diameter.metres(),
            Self::Fold { width, .. } => width.metres(),
        }
    }
    /// Half the width of the mark the style leaves beside its path.
    pub(crate) fn reach_metres(&self) -> f32 {
        match self {
            Self::Cord { diameter, .. } => diameter.metres() * 0.5,
            Self::Stitches { width, .. } => width.metres() * 0.5,
            Self::Braid { width, .. } | Self::Fold { width, .. } => width.metres() * 0.5,
            Self::Pearls { diameter, .. } => diameter.metres() * 0.5,
        }
    }
    fn validate(&self) -> Result<(), GenerateError> {
        let valid = match self {
            Self::Cord { diameter, .. } => Self::CORD_RANGE.contains(&diameter.0),
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
                strands,
                cover,
                width,
                pitch,
                relief,
                ..
            } => {
                Self::STRAND_RANGE.contains(&strands.0)
                    && Self::COVER_RANGE.contains(&cover.0)
                    && Self::BRAID_WIDTH_RANGE.contains(&width.0)
                    && Self::BRAID_PITCH_RANGE.contains(&pitch.0)
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
