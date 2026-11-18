use std::ops::RangeInclusive;

use serde::{Deserialize, Serialize};

use crate::puff_course::{PuffCourse, PuffCourseSpan};
use crate::sleeve_cuff::SleeveCuff;
use crate::{GenerateError, Millimeters, Permille, TextileAppearance, TextileColor, TextileFinish};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum PuffAndSlashKind {
    Sleeve,
    Hose,
}

/// Surface construction shared by sleeves, hose, skirts and dresses.
/// Courses run distal to proximal, the direction of the textile chart.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PuffAndSlashPattern {
    pub courses: Vec<PuffCourse>,
    /// Bulb profile exponent: 1000 is a sine profile; lower values broaden it.
    pub puff_roundness: Permille,
    pub rotation: Permille,
    pub proximal_edge: PuffEdge,
    pub outer_fabric: TextileAppearance,
    /// Fitted bands between and beyond the courses.
    pub band_fabric: TextileAppearance,
    pub undercloth_fabric: TextileAppearance,
    /// Share of each course's fullness that the undercloth takes. Less sets
    /// it back inside the slashes, under the edges of the panes.
    pub lining_fullness: Permille,
}

/// Cloth of the fitted ring that closes the proximal course.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum PuffEdge {
    /// A band like those between the courses.
    Band,
    /// The outer fabric itself, as where a sleeve head is set into a seam.
    Seam,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PuffAndSlashDesign {
    pub kind: PuffAndSlashKind,
    pub pattern: PuffAndSlashPattern,
    pub length: Permille,
    /// Placement within the limb span: zero distal, 500 centered, 1000 proximal.
    pub proximal_position: Permille,
    /// Sleeves only: how far the head sits below the shoulder joint, in limb
    /// radii. A low inset reaches an armhole; a high one clears torso armor.
    pub proximal_inset: Permille,
    /// Sleeves only: how much lower the head ends under the arm than over
    /// the shoulder, in limb radii. A sleeve head is cut on a curve: with no
    /// drop its top is level, and a head set high on the shoulder runs into
    /// the body under the arm.
    pub underarm_drop: Permille,
    pub clearance: Millimeters,
    pub thickness: Millimeters,
    /// Sleeves only: a flared cuff below the distal band.
    pub cuff: Option<Box<SleeveCuff>>,
}

impl Default for PuffAndSlashPattern {
    fn default() -> Self {
        let outer = TextileAppearance::solid(TextileColor([150, 32, 28]), TextileFinish::Wool);
        Self {
            courses: [650, 825, 1000]
                .map(|scale| PuffCourse {
                    fullness: Millimeters(45 * scale / 1000),
                    ..Default::default()
                })
                .to_vec(),
            puff_roundness: Permille(850),
            rotation: Permille(0),
            proximal_edge: PuffEdge::Band,
            band_fabric: outer.clone(),
            outer_fabric: outer,
            undercloth_fabric: TextileAppearance::solid(
                TextileColor([224, 184, 58]),
                TextileFinish::Silk,
            ),
            lining_fullness: Permille::ONE,
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
            proximal_inset: Permille(1500),
            underarm_drop: Permille(0),
            clearance: Millimeters(4),
            thickness: Millimeters(2),
            cuff: None,
        }
    }
}

impl PuffAndSlashPattern {
    pub const MAX_COURSES: usize = 8;
    pub const ROUNDNESS_RANGE: RangeInclusive<u16> = 400..=2500;
    pub const LINING_FULLNESS_RANGE: RangeInclusive<u16> = 300..=1000;
    /// Native course-table layout: count, roundness, then each course's
    /// axial end, fullness in metres and axial half band.
    pub(crate) const SHADER_WORDS: usize = 2 + 3 * Self::MAX_COURSES;

    pub fn validate(&self) -> Result<(), GenerateError> {
        self.outer_fabric.validate()?;
        self.band_fabric.validate()?;
        self.undercloth_fabric.validate()?;
        for course in &self.courses {
            course.validate()?;
        }
        if self.courses.is_empty()
            || self.courses.len() > Self::MAX_COURSES
            || !Self::ROUNDNESS_RANGE.contains(&self.puff_roundness.0)
            || self.rotation.0 > Permille::ONE.0
            || !Self::LINING_FULLNESS_RANGE.contains(&self.lining_fullness.0)
        {
            return Err(crate::DesignError::ParametricParameters.into());
        }
        Ok(())
    }

    pub(crate) fn spans(&self) -> Vec<PuffCourseSpan> {
        PuffCourseSpan::layout(&self.courses)
    }

    /// Native shader words shared by each placement's coordinate kernel.
    pub(crate) fn shader_words(&self) -> [f32; Self::SHADER_WORDS] {
        let mut words = [0.; Self::SHADER_WORDS];
        words[0] = self.courses.len() as f32;
        words[1] = self.puff_roundness.unit();
        for ((course, span), row) in self
            .courses
            .iter()
            .zip(self.spans())
            .zip(words[2..].as_chunks_mut::<3>().0)
        {
            *row = [span.end, course.fullness.metres(), span.half_band];
        }
        words
    }
}

impl PuffAndSlashDesign {
    /// Share of the proximal course over which a sleeve head set at the
    /// armhole loses its fullness.
    const HEAD_TAPER_COURSES: f32 = 0.8;

    /// Axial extent below the armhole where fullness tapers; zero keeps it.
    /// Each cloth layer tapers its own release, so the stack stays ordered.
    pub(crate) fn head_taper(&self) -> f32 {
        let at_armhole =
            self.kind == PuffAndSlashKind::Sleeve && self.proximal_position == Permille::ONE;
        match self.pattern.spans().last() {
            Some(span) if at_armhole => (span.end - span.start) * Self::HEAD_TAPER_COURSES,
            _ => 0.,
        }
    }

    pub const PROXIMAL_INSET_RANGE: RangeInclusive<u16> = 0..=2000;
    pub const UNDERARM_DROP_RANGE: RangeInclusive<u16> = 0..=3000;

    /// Share of the garment's length, from its distal end, at which its
    /// proximal course begins.
    pub fn head_start(&self) -> f32 {
        self.pattern.spans().last().map_or(1., |span| span.start)
    }

    pub fn validate(&self) -> Result<(), GenerateError> {
        self.pattern.validate()?;
        if let Some(cuff) = &self.cuff {
            cuff.validate()?;
        }
        if (self.cuff.is_some() && self.kind != PuffAndSlashKind::Sleeve)
            || !(350..=1000).contains(&self.length.0)
            || self.proximal_position.0 > Permille::ONE.0
            || !Self::PROXIMAL_INSET_RANGE.contains(&self.proximal_inset.0)
            || !Self::UNDERARM_DROP_RANGE.contains(&self.underarm_drop.0)
            || !(1..=15).contains(&self.clearance.0)
            || !(1..=6).contains(&self.thickness.0)
        {
            return Err(crate::DesignError::ParametricParameters.into());
        }
        Ok(())
    }
}
