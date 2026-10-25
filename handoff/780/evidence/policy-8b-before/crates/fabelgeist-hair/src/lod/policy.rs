//! Screen-size selection and shading policies for a prepared groom.

use super::{CardLevelIndex, LodCounts, PixelSpan, Quality, StrandLevelIndex};
use crate::parameters::StrandCount;
use crate::sim::SimLevel;
use fabelgeist_math::Ratio;

pub type PolicyResult<T> = std::result::Result<T, LodPolicyError>;

/// A coarser form fading in under the one a band draws: `amount` of the way
/// from the band's own form to `to`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Blend {
    pub to: Representation,
    pub amount: Ratio,
}

/// The representation of the selected band and its coarser transition.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Selection {
    pub representation: Representation,
    pub blend: Option<Blend>,
}

/// A band of on-screen sizes and what is drawn in it.
#[derive(Clone, Copy, Debug)]
pub struct Band {
    /// The head is at least this many pixels tall.
    pub min_pixels: PixelSpan,
    pub form: Form,
}

#[derive(Clone, Debug)]
pub struct LodPolicy {
    pub name: &'static str,
    /// From the largest on screen down.
    pub bands: Vec<Band>,
    pub quality: Quality,
    /// Shadows are cast by this strand level and share, whatever is drawn.
    pub shadow_level: StrandLevelIndex,
    pub shadow_share: Ratio,
    /// Over how much of a band's lower edge, as a share of it, the next
    /// coarser form fades in: 0.4 fades from 1.4 times the edge down to it.
    pub blend: Ratio,
    /// How finely the hair is simulated, by the head's height on screen in
    /// pixels, largest first: each level from its height up.
    pub sim: Vec<SimBand>,
}

/// Simulation detail selected from an on-screen size threshold.
#[derive(Clone, Copy, Debug)]
pub struct SimBand {
    pub min_pixels: PixelSpan,
    pub level: SimLevel,
}

/// What to draw this frame.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Representation {
    /// No groom geometry has been prepared.
    Empty,
    /// The first `count` strands of `level`, `width_scale` times as wide.
    Strands {
        level: StrandLevelIndex,
        count: StrandCount,
        width_scale: Ratio,
    },
    Cards {
        level: CardLevelIndex,
    },
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Form {
    /// A share of the strands at a level.
    Strands {
        level: StrandLevelIndex,
        share: Ratio,
    },
    Cards {
        level: CardLevelIndex,
    },
}

/// A policy cannot select a representation from an empty band collection.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum LodPolicyError {
    #[error("a level-of-detail policy needs at least one band")]
    EmptyBands,
    #[error("the selected policy form needs a strand level")]
    MissingStrandLevels,
    #[error("the selected policy form needs a card level")]
    MissingCardLevels,
    #[error("a nonempty strand population needs a strand level")]
    StrandsWithoutLevels,
}

impl LodPolicy {
    /// How finely to simulate a head `pixels` tall.
    pub fn sim_level(&self, pixels: PixelSpan) -> SimLevel {
        self.sim
            .iter()
            .find(|band| pixels >= band.min_pixels)
            .or(self.sim.last())
            .map_or(SimLevel::MEDIUM, |band| band.level)
    }

    /// [`LodPolicy::sim_level`], staying at `current` until the head is a
    /// sixth past the edge it would cross: a level change rebuilds the
    /// simulation, and a head resting on an edge should not flicker across
    /// it.
    pub fn sim_level_from(&self, current: SimLevel, pixels: PixelSpan) -> SimLevel {
        let wanted = self.sim_level(pixels);
        if wanted == current {
            return current;
        }
        let rank = |level: SimLevel| self.sim.iter().position(|band| band.level == level);
        let (Some(at), Some(to)) = (rank(current), rank(wanted)) else {
            return wanted;
        };
        // Finer: past the finer level's edge by a sixth; coarser: below the
        // current level's edge by a sixth.
        let edge = if to < at {
            self.sim[to].min_pixels.pixels() * 7.0 / 6.0
        } else {
            self.sim[at].min_pixels.pixels() * 5.0 / 6.0
        };
        let past = if to < at {
            pixels.pixels() >= edge
        } else {
            pixels.pixels() < edge
        };
        if past { wanted } else { current }
    }

    /// One band throughout: the given form at every distance, with this
    /// policy's shading.
    pub fn fixed(&self, form: Form) -> Self {
        Self {
            bands: vec![Band {
                min_pixels: PixelSpan::from_pixels(0.0),
                form,
            }],
            ..self.clone()
        }
    }

    /// What to draw for a head `pixels` tall. Within a band of strands the
    /// share rises smoothly towards the next finer band's, so moving in
    /// grows the count rather than stepping it.
    pub fn select(&self, pixels: PixelSpan, lods: LodCounts) -> PolicyResult<Representation> {
        Ok(self.select_blended(pixels, lods)?.representation)
    }

    /// What to draw for a head `pixels` tall, and the coarser form fading
    /// in under it near the bottom of its band. At the band's lower edge
    /// the blend is all the coarser form, which is what the band below
    /// draws at its top: the hand-over is continuous.
    ///
    /// Returns [`LodPolicyError::EmptyBands`] when no band is configured.
    /// An absent requested geometry family is an error when another family
    /// exists; a groom with no levels selects [`Representation::Empty`].
    /// Nonempty strand counts require an actual strand level.
    /// With a nonempty collection, noncomparable pixel heights retain the
    /// last-band selection used by the numerical policy.
    pub fn select_blended(&self, pixels: PixelSpan, lods: LodCounts) -> PolicyResult<Selection> {
        let at = self.band(pixels).ok_or(LodPolicyError::EmptyBands)?;
        if lods.levels.count() == 0 && lods.strands.count() != 0 {
            return Err(LodPolicyError::StrandsWithoutLevels);
        }
        if lods.levels.count() == 0 && lods.cards.count() == 0 {
            return Ok(Selection {
                representation: Representation::Empty,
                blend: None,
            });
        }
        let form = self.form(at, pixels, lods)?;
        let lo = self.bands[at].min_pixels;
        if self.blend.ratio() <= 0.0 || at + 1 >= self.bands.len() || lo.pixels() <= 0.0 {
            return Ok(Selection {
                representation: form,
                blend: None,
            });
        }
        let amount = 1.0 - (pixels.pixels() - lo.pixels()) / (lo.pixels() * self.blend.ratio());
        if amount <= 0.0 {
            return Ok(Selection {
                representation: form,
                blend: None,
            });
        }
        let to = self.form(at + 1, lo, lods)?;
        Ok(Selection {
            representation: form,
            blend: Some(Blend {
                to,
                amount: Ratio::from_ratio(amount.min(1.0)),
            }),
        })
    }

    fn band(&self, pixels: PixelSpan) -> Option<usize> {
        self.bands
            .iter()
            .position(|b| pixels >= b.min_pixels)
            .or_else(|| self.bands.len().checked_sub(1))
    }

    /// Band `at`'s form for a head `pixels` tall.
    fn form(&self, at: usize, pixels: PixelSpan, lods: LodCounts) -> PolicyResult<Representation> {
        // The policy interpolation kernel counts strands in a float prefix;
        // convert the admitted population only for this arithmetic.
        let total = lods.strands.count();
        match self.bands[at].form {
            Form::Cards { level } => Ok(Representation::Cards {
                level: lods
                    .cards
                    .select(level)
                    .ok_or(LodPolicyError::MissingCardLevels)?,
            }),
            Form::Strands { level, share } => {
                let level = lods
                    .levels
                    .select(level)
                    .ok_or(LodPolicyError::MissingStrandLevels)?;
                let mut share = share.ratio();
                if at > 0
                    && let Form::Strands { share: finer, .. } = self.bands[at - 1].form
                {
                    let lo = self.bands[at].min_pixels.pixels();
                    let hi = self.bands[at - 1].min_pixels.pixels();
                    let t = ((pixels.pixels() - lo) / (hi - lo).max(1.0)).clamp(0.0, 1.0);
                    share += (finer.ratio() - share) * t;
                }
                let count = ((total as f32 * share).round() as usize).clamp(1.min(total), total);
                Ok(Representation::Strands {
                    level,
                    count: StrandCount::from_count(count),
                    width_scale: Ratio::from_ratio(total as f32 / count.max(1) as f32),
                })
            }
        }
    }

    /// The strands that cast shadows; empty when no strand level exists.
    pub fn shadow(&self, lods: LodCounts) -> Representation {
        let Some(level) = lods.levels.select(self.shadow_level) else {
            return Representation::Empty;
        };
        let total = lods.strands.count();
        let count = ((total as f32 * self.shadow_share.ratio()).round() as usize)
            .clamp(1.min(total), total);
        Representation::Strands {
            level,
            count: StrandCount::from_count(count),
            width_scale: Ratio::from_ratio(total as f32 / count.max(1) as f32),
        }
    }
}
