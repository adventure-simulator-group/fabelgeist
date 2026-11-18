//! Authored courses of a puff-and-slash treatment, distal to proximal.
use std::ops::RangeInclusive;

use serde::{Deserialize, Serialize};

use crate::{DesignError, GenerateError, Millimeters, Permille};

/// Open slashes cut through one puff course.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PuffSlashes {
    pub count: u8,
    /// Fraction of each circumferential repeat occupied by the opening.
    pub width: Permille,
    /// Fraction of the puff length occupied by open slashes.
    pub length: Permille,
    /// Where along its length a slash gapes most: 500 is the middle, less
    /// lies toward the distal end. Off the middle the slash is a teardrop.
    pub widest: Permille,
    /// Lean around the limb from one end of a slash to the other, as a share
    /// of one circumferential repeat.
    pub slant: Permille,
}

/// One course between constriction bands. Zero fullness keeps a fitted
/// section; a course without slashes keeps a closed surface.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PuffCourse {
    /// Axial share of the treatment, relative to the other courses.
    pub length: Permille,
    pub fullness: Millimeters,
    pub slashes: Option<PuffSlashes>,
    /// Fraction of this course reserved for fitted bands, half at each end.
    pub constriction_width: Permille,
}

/// Axial unit-interval extent of one course and of the puff inside its bands.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct PuffCourseSpan {
    pub start: f32,
    pub end: f32,
    pub half_band: f32,
}

impl PuffSlashes {
    pub const COUNT_RANGE: RangeInclusive<u8> = 3..=16;
    pub const WIDTH_RANGE: RangeInclusive<u16> = 50..=650;
    pub const LENGTH_RANGE: RangeInclusive<u16> = 300..=900;
    pub const WIDEST_RANGE: RangeInclusive<u16> = 200..=800;

    fn validate(&self) -> Result<(), GenerateError> {
        if !Self::COUNT_RANGE.contains(&self.count)
            || !Self::WIDTH_RANGE.contains(&self.width.0)
            || !Self::LENGTH_RANGE.contains(&self.length.0)
            || !Self::WIDEST_RANGE.contains(&self.widest.0)
            || self.slant.0 > Permille::ONE.0
        {
            return Err(DesignError::ParametricParameters.into());
        }
        Ok(())
    }
}

impl Default for PuffSlashes {
    fn default() -> Self {
        Self {
            count: 8,
            width: Permille(300),
            length: Permille(700),
            widest: Permille(500),
            slant: Permille(0),
        }
    }
}

impl PuffCourse {
    pub const LENGTH_RANGE: RangeInclusive<u16> = 50..=1000;
    pub const FULLNESS_RANGE: RangeInclusive<u16> = 0..=135;
    pub const CONSTRICTION_RANGE: RangeInclusive<u16> = 40..=600;

    pub(crate) fn validate(&self) -> Result<(), GenerateError> {
        if let Some(slashes) = &self.slashes {
            slashes.validate()?;
        }
        if !Self::LENGTH_RANGE.contains(&self.length.0)
            || !Self::FULLNESS_RANGE.contains(&self.fullness.0)
            || !Self::CONSTRICTION_RANGE.contains(&self.constriction_width.0)
        {
            return Err(DesignError::ParametricParameters.into());
        }
        Ok(())
    }
}

impl Default for PuffCourse {
    fn default() -> Self {
        Self {
            length: Permille::ONE,
            fullness: Millimeters(45),
            slashes: Some(PuffSlashes::default()),
            constriction_width: Permille(120),
        }
    }
}

impl PuffCourseSpan {
    /// Courses fill the unit interval in proportion to their authored shares.
    pub(crate) fn layout(courses: &[PuffCourse]) -> Vec<Self> {
        let total: f32 = courses.iter().map(|course| course.length.unit()).sum();
        let mut start = 0.;
        let mut spans = Vec::with_capacity(courses.len());
        for (index, course) in courses.iter().enumerate() {
            let end = if index + 1 == courses.len() {
                1.
            } else {
                start + course.length.unit() / total
            };
            spans.push(Self {
                start,
                end,
                half_band: (end - start) * course.constriction_width.unit() * 0.5,
            });
            start = end;
        }
        spans
    }

    /// The released cloth between this course's two band edges.
    pub(crate) fn puff(&self) -> [f32; 2] {
        [self.start + self.half_band, self.end - self.half_band]
    }
}
