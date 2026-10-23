//! Draw roles and their native strand-prefix/card-coverage projections.

use super::{DrawGpu, HairRenderer};
use crate::lod::{CardLevelIndex, PixelSpan, Representation, Selection, StrandLevelIndex};
use fabelgeist_math::Ratio;
use std::ops::Range;

pub(super) struct DrawPlan {
    pub strands: Vec<StrandDraw>,
    pub cards: Vec<CardDraw>,
}

pub(super) struct StrandDraw {
    pub level: StrandLevelIndex,
    pub role: DrawRole,
    /// Native instance range, bounded by the resident strand population.
    pub instances: Range<u32>,
}

pub(super) struct CardDraw {
    pub level: CardLevelIndex,
    pub coverage: CardCoverage,
}

#[derive(Clone, Copy)]
pub(super) enum DrawRole {
    Main,
    Shadow,
    Blend,
}

pub(super) enum CardCoverage {
    Outgoing { amount: Ratio },
    Incoming { amount: Ratio },
}

impl DrawPlan {
    pub fn new(renderer: &HairRenderer, selection: Selection, min_pixels: PixelSpan) -> Self {
        let Selection {
            representation,
            blend,
        } = selection;
        let population = |representation| {
            renderer
                .draw_gpu(representation, min_pixels)
                .map_or(0, |draw| draw.count)
        };
        let amount = blend.map_or(Ratio::from_ratio(0.0), |blend| blend.amount);
        let handed = |count: u32| ((count as f32) * (1.0 - amount.ratio())).round() as u32;
        let mut plan = Self {
            strands: Vec::new(),
            cards: Vec::new(),
        };
        match representation {
            Representation::Strands { level, .. } => {
                let count = population(representation);
                plan.strands.push(StrandDraw {
                    level,
                    role: DrawRole::Main,
                    instances: 0..handed(count),
                });
            }
            Representation::Cards { level } => plan.cards.push(CardDraw {
                level,
                coverage: CardCoverage::Outgoing { amount },
            }),
            Representation::Empty => {}
        }
        if let Some(blend) = blend {
            match (representation, blend.to) {
                (Representation::Strands { .. }, Representation::Strands { level, .. }) => {
                    let count = population(representation);
                    plan.strands.push(StrandDraw {
                        level,
                        role: DrawRole::Blend,
                        instances: handed(count)..count,
                    });
                }
                (Representation::Cards { .. }, Representation::Strands { level, .. }) => {
                    let count = population(blend.to);
                    plan.strands.push(StrandDraw {
                        level,
                        role: DrawRole::Blend,
                        instances: 0..count - handed(count),
                    });
                }
                (_, Representation::Cards { level }) => plan.cards.push(CardDraw {
                    level,
                    coverage: CardCoverage::Incoming { amount },
                }),
                (_, Representation::Empty)
                | (Representation::Empty, Representation::Strands { .. }) => {}
            }
        }
        plan
    }
}

impl DrawRole {
    /// Index the native main, shadow and transition uniform/bind-group slots.
    pub fn index(self) -> usize {
        match self {
            Self::Main => 0,
            Self::Shadow => 1,
            Self::Blend => 2,
        }
    }
}

impl CardCoverage {
    /// The native shader keeps a card when its [0, 1] random value lies in
    /// this half-open interval. The outer endpoints retain the old shader's
    /// complete outgoing/incoming coverage at transition endpoints.
    pub fn keep(&self) -> [f32; 4] {
        match *self {
            Self::Outgoing { amount } => [amount.ratio(), 2.0, 0.0, 0.0],
            Self::Incoming { amount } => [-1.0, amount.ratio(), 0.0, 0.0],
        }
    }
}

impl HairRenderer {
    pub(super) fn draw_gpu(
        &self,
        representation: Representation,
        min_pixels: PixelSpan,
    ) -> Option<DrawGpu> {
        let Representation::Strands {
            level,
            count,
            width_scale,
        } = representation
        else {
            return None;
        };
        let resident = self.levels.get(level.index())?;
        Some(DrawGpu {
            points: resident.points,
            // Clamp to an existing u32 native population before narrowing.
            count: count.count().min(resident.count as usize) as u32,
            pad: [0; 2],
            width_scale: width_scale.ratio(),
            min_pixels: min_pixels.pixels(),
            pad2: [0.0; 2],
        })
    }
}
