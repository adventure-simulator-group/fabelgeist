//! How a hairstyle is varied from its preset.
//!
//! A preset is the hair crate's whole look: its follicles' growth pattern,
//! its cut, its arrangement, and how the strands curl, wave, frizz and
//! clump. A [`HairShape`] changes it -- another cut, arrangement or growth
//! pattern, and the rest scaled or shifted from the preset's own -- so the
//! default shape grows the preset as it is.

use fabelgeist_hair::{Arrangement, Cut, Growth, Haircut, Hairstyle, Look, Preset};
use serde::{Deserialize, Serialize};

/// The range of each of a shape's values: the length's scale, the volume
/// and the part's shift, metres, the curl and clumping added, the waves
/// added, metres, and the frizz's scale.
pub const LENGTH: std::ops::RangeInclusive<f32> = 0.3..=2.0;
pub const VOLUME: std::ops::RangeInclusive<f32> = -0.01..=0.04;
pub const PART: std::ops::RangeInclusive<f32> = -0.06..=0.06;
pub const CURL: std::ops::RangeInclusive<f32> = -1.0..=1.0;
pub const WAVE: std::ops::RangeInclusive<f32> = -0.01..=0.02;
pub const FRIZZ: std::ops::RangeInclusive<f32> = 0.0..=4.0;
pub const CLUMPING: std::ops::RangeInclusive<f32> = -1.0..=1.0;

/// What hair grows: a preset, varied by a shape.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Groom {
    pub preset: Preset,
    pub shape: HairShape,
}

impl Groom {
    /// The hair crate's look for this groom.
    pub(super) fn look(&self) -> Look {
        let mut look = Look::new(self.preset);
        self.shape.apply(&mut look);
        look
    }
}

/// How a hairstyle differs from its preset.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, try_from = "ShapeValues")]
pub struct HairShape {
    /// The cut, the arrangement and the growth pattern, when not the
    /// preset's own.
    cut: Option<Cut>,
    arrangement: Option<Arrangement>,
    growth: Option<Growth>,
    /// Every length of the cut scaled, its hemline as far from the crown as
    /// the hair is long.
    length: f32,
    /// How far further the hair stands off the head at the crown, metres.
    volume: f32,
    /// How far the part moves towards the character's left, metres.
    part: f32,
    /// Added to how much the strands curl, 0 to 1.
    curl: f32,
    /// Added to the waves either side, metres.
    wave: f32,
    /// The frizz and the flyaways, scaled.
    frizz: f32,
    /// Added to how much the strands gather into locks, 0 to 1.
    clumping: f32,
}

/// A shape's values, before their ranges are checked.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ShapeValues {
    cut: Option<Cut>,
    arrangement: Option<Arrangement>,
    growth: Option<Growth>,
    length: f32,
    volume: f32,
    part: f32,
    curl: f32,
    wave: f32,
    frizz: f32,
    clumping: f32,
}

/// A shape's value outside its range.
#[derive(Debug, thiserror::Error)]
#[error("a hair shape's values must lie within their ranges")]
pub struct ShapeRangeError;

impl TryFrom<ShapeValues> for HairShape {
    type Error = ShapeRangeError;

    fn try_from(values: ShapeValues) -> Result<Self, Self::Error> {
        let within = LENGTH.contains(&values.length)
            && VOLUME.contains(&values.volume)
            && PART.contains(&values.part)
            && CURL.contains(&values.curl)
            && WAVE.contains(&values.wave)
            && FRIZZ.contains(&values.frizz)
            && CLUMPING.contains(&values.clumping);
        if !within {
            return Err(ShapeRangeError);
        }
        Ok(Self {
            cut: values.cut,
            arrangement: values.arrangement,
            growth: values.growth,
            length: values.length,
            volume: values.volume,
            part: values.part,
            curl: values.curl,
            wave: values.wave,
            frizz: values.frizz,
            clumping: values.clumping,
        })
    }
}

impl Default for HairShape {
    /// The preset as it is.
    fn default() -> Self {
        Self {
            cut: None,
            arrangement: None,
            growth: None,
            length: 1.0,
            volume: 0.0,
            part: 0.0,
            curl: 0.0,
            wave: 0.0,
            frizz: 1.0,
            clumping: 0.0,
        }
    }
}

/// A shape's sliding values, each clamped into its range when set.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ShapeKnobs {
    pub length: f32,
    pub volume: f32,
    pub part: f32,
    pub curl: f32,
    pub wave: f32,
    pub frizz: f32,
    pub clumping: f32,
}

impl HairShape {
    pub fn cut(&self) -> Option<Cut> {
        self.cut
    }

    pub fn arrangement(&self) -> Option<Arrangement> {
        self.arrangement
    }

    pub fn growth(&self) -> Option<Growth> {
        self.growth
    }

    pub fn with_cut(self, cut: Option<Cut>) -> Self {
        Self { cut, ..self }
    }

    pub fn with_arrangement(self, arrangement: Option<Arrangement>) -> Self {
        Self {
            arrangement,
            ..self
        }
    }

    pub fn with_growth(self, growth: Option<Growth>) -> Self {
        Self { growth, ..self }
    }

    pub fn knobs(&self) -> ShapeKnobs {
        ShapeKnobs {
            length: self.length,
            volume: self.volume,
            part: self.part,
            curl: self.curl,
            wave: self.wave,
            frizz: self.frizz,
            clumping: self.clumping,
        }
    }

    /// This shape with `knobs`, each clamped into its range.
    pub fn with_knobs(self, knobs: ShapeKnobs) -> Self {
        let within = |value: f32, range: std::ops::RangeInclusive<f32>| {
            value.clamp(*range.start(), *range.end())
        };
        Self {
            length: within(knobs.length, LENGTH),
            volume: within(knobs.volume, VOLUME),
            part: within(knobs.part, PART),
            curl: within(knobs.curl, CURL),
            wave: within(knobs.wave, WAVE),
            frizz: within(knobs.frizz, FRIZZ),
            clumping: within(knobs.clumping, CLUMPING),
            ..self
        }
    }

    /// Change `look`, a preset's, by this shape.
    fn apply(&self, look: &mut Look) {
        let hairdo = &mut look.hairdo;
        if let Some(cut) = self.cut {
            hairdo.haircut = Haircut::new(cut);
        }
        if let Some(arrangement) = self.arrangement {
            hairdo.hairstyle = Hairstyle::new(arrangement);
        }
        if let Some(growth) = self.growth {
            hairdo.follicles.growth = growth;
        }
        hairdo.haircut = hairdo.haircut.scaled(self.length);
        let style = &mut hairdo.hairstyle;
        style.volume = (style.volume + self.volume).max(0.0);
        style.part += self.part;
        look.curl = (look.curl + self.curl).clamp(0.0, 1.0);
        look.wave = (look.wave + self.wave).max(0.0);
        look.frizz *= self.frizz;
        look.flyaways = (look.flyaways * self.frizz).min(1.0);
        look.clump = (look.clump + self.clumping).clamp(0.0, 1.0);
        // Only some arrangements hang under gravity.
        look.drape = look.hairdo.drapes();
    }
}
