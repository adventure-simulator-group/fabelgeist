//! Levels of detail: one groom, drawn from film to phone.
//!
//! A groom is prepared once ([`HairLods::build`]) into every form it can be
//! drawn in:
//!
//! * **Strand levels.** The strands at their full point count, and at a
//!   half, a quarter and a few points each -- each coarse strand kept off
//!   the head where its straight segments would cut through the curve of
//!   the scalp ([`keep_clear`]), or the hair would thin to bald patches
//!   with distance. Every level keeps the strands
//!   in one shuffled order (by a hash of each strand's ID), so the first
//!   `n` strands of any level are an even random sample of the whole head:
//!   drawing fewer strands is drawing a shorter prefix -- no rebuild, no
//!   popping, just a count. Fewer strands are drawn wider, by the inverse of
//!   the share drawn, so the head's coverage holds (Yuksel's and every
//!   game's strand culling).
//! * **Cards** ([`crate::cards`]): a few thousand strips following the
//!   locks, for distance and for mobile, in a fine and a coarse cut.
//! * **Occlusion** ([`crate::occlusion`]) baked into every point and card
//!   vertex, so a level with no shadow maps still has depth.
//! * **Scalp cover** ([`ScalpCover`]): how much hair covers the skin where
//!   it grows, which the head is shaded by -- at every level, so no level
//!   shows bare scalp between its strands.
//!
//! A [`LodPolicy`] then picks a form from how tall the head is on screen,
//! in pixels, with a shading [`Quality`] to go with it. Near the bottom of
//! each band the next coarser form fades in ([`LodPolicy::select_blended`]):
//! strands hand over to the coarser level strand by strand along the shared
//! order, cards dissolve in card by card as the strands thin out -- so moving
//! away never pops. The three policies
//! are the three targets the hair is for: [`LodPolicy::cinematic`] (every
//! strand, every point, deep opacity shadows, accumulated over jittered
//! frames), [`LodPolicy::desktop`] and [`LodPolicy::mobile`].

use fabelgeist_math::Vec3;
use rayon::prelude::*;

use crate::cards::{CardAtlas, CardMesh, CardSettings};
use crate::collider::{Collider, keep_clear};
use crate::cover::ScalpCover;
use crate::occlusion::{Occlusion, OcclusionSettings};
use crate::rng;
use crate::sim::SimLevel;
use crate::strands::{Strands, resample_into};

/// How far off the head a coarse strand's segments are kept, metres: about
/// where the groom's own strands lie over the scalp.
const STRAND_CLEARANCE: f32 = 0.0012;

/// A strand's per-strand data as the strand shader reads it.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct StrandInfo {
    /// Root radius, metres.
    pub radius: f32,
    /// 0 to 1, per strand.
    pub random: f32,
    pub uv: [f32; 2],
}

/// Every strand at one point count, in the shared shuffled order.
pub struct StrandLevel {
    pub points: usize,
    /// Position and baked sky visibility, strand-major.
    pub positions: Vec<[f32; 4]>,
    pub strands: Vec<StrandInfo>,
}

impl StrandLevel {
    pub fn len(&self) -> usize {
        self.strands.len()
    }

    pub fn is_empty(&self) -> bool {
        self.strands.is_empty()
    }

    pub fn bytes(&self) -> usize {
        self.positions.len() * 16 + self.strands.len() * 16
    }
}

pub struct LodSettings {
    /// Points per strand at each level after the first (which keeps the
    /// groom's own count); a level is dropped if it would not be coarser.
    pub strand_points: Vec<usize>,
    pub cards: Vec<CardSettings>,
    pub occlusion: OcclusionSettings,
    pub atlas_variants: u32,
    pub atlas_size: [u32; 2],
}

impl Default for LodSettings {
    fn default() -> Self {
        Self {
            strand_points: vec![24, 12, 6],
            cards: vec![
                CardSettings::default(),
                CardSettings {
                    size: 0.016,
                    segments: 6,
                    width_scale: 1.5,
                    min_strands: 4,
                },
            ],
            occlusion: OcclusionSettings::default(),
            atlas_variants: 8,
            atlas_size: [64, 256],
        }
    }
}

/// A groom ready to draw at any distance.
pub struct HairLods {
    pub levels: Vec<StrandLevel>,
    pub cards: Vec<CardMesh>,
    pub atlas: CardAtlas,
    pub bounds: [Vec3; 2],
    /// The head's centre, for card orientation and framing.
    pub center: Vec3,
    /// How much hair covers the skin, for shading the head under it.
    pub cover: ScalpCover,
}

impl HairLods {
    pub fn build(
        strands: &Strands,
        center: Vec3,
        body: Option<&dyn Collider>,
        settings: &LodSettings,
    ) -> Self {
        let occlusion = Occlusion::bake(strands, body, &settings.occlusion);
        let mut order: Vec<usize> = (0..strands.len()).collect();
        order.sort_by_key(|&s| rng::hash(strands.id[s], 0x0D3E));

        let mut counts = vec![strands.points];
        for &p in &settings.strand_points {
            if p >= 2 && p < *counts.last().unwrap() {
                counts.push(p);
            }
        }
        let info: Vec<StrandInfo> = order
            .iter()
            .map(|&s| StrandInfo {
                radius: strands.radius[s],
                random: rng::unit(strands.id[s], 0x5EED),
                uv: strands.root_uv[s],
            })
            .collect();
        let levels = counts
            .iter()
            .map(|&points| {
                let positions: Vec<[f32; 4]> = order
                    .par_iter()
                    .flat_map_iter(|&s| {
                        let mut out = vec![Vec3::default(); points];
                        if points == strands.points {
                            out.copy_from_slice(strands.strand(s));
                        } else {
                            resample_into(strands.strand(s), &mut Vec::new(), &mut out);
                            if let Some(body) = body {
                                keep_clear(body, &mut out, STRAND_CLEARANCE, 4);
                            }
                        }
                        out.into_iter().map(|p| {
                            let [x, y, z] = p.to_array();
                            [x, y, z, occlusion.at(p)]
                        })
                    })
                    .collect();
                StrandLevel {
                    points,
                    positions,
                    strands: info.clone(),
                }
            })
            .collect();
        let cards = settings
            .cards
            .iter()
            .map(|c| {
                CardMesh::build(
                    strands,
                    center,
                    Some(&occlusion),
                    body,
                    c,
                    settings.atlas_variants,
                )
            })
            .collect();
        let [w, h] = settings.atlas_size;
        Self {
            levels,
            cards,
            atlas: CardAtlas::paint(settings.atlas_variants, w, h, 0xA71A5),
            bounds: strands.bounds(),
            center,
            cover: ScalpCover::new(strands),
        }
    }

    pub fn strand_count(&self) -> usize {
        self.levels.first().map_or(0, |l| l.len())
    }

    /// How many strands, levels and card levels there are: all a
    /// [`LodPolicy`] needs to choose among them.
    pub fn counts(&self) -> LodCounts {
        LodCounts {
            strands: self.strand_count(),
            levels: self.levels.len(),
            cards: self.cards.len(),
        }
    }
}

/// The sizes of a groom's levels of detail, wherever they were built: the
/// host's [`HairLods`] or the GPU's [`crate::gpu::GpuLods`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LodCounts {
    /// The strands of the finest level.
    pub strands: usize,
    /// The strand levels.
    pub levels: usize,
    /// The card levels.
    pub cards: usize,
}

/// How the hair is shaded.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Quality {
    /// Marschner's three lobes (Karis' fit) with multiple scattering, or
    /// Kajiya-Kay with two shifted highlights.
    pub marschner: bool,
    /// Deep opacity shadows from the key light; without them only the
    /// baked occlusion darkens.
    pub shadows: bool,
    /// Deep opacity map size.
    pub shadow_resolution: u32,
    /// Frames accumulated with sub-pixel jitter while the view holds still.
    pub accumulate: u32,
    /// Narrowest a strand is drawn, pixels; narrower strands fade instead.
    pub min_pixels: f32,
}

/// A coarser form fading in under the one a band draws: `amount` of the way
/// from the band's own form to `to`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Blend {
    pub to: Representation,
    pub amount: f32,
}

/// What to draw this frame.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Representation {
    /// The first `count` strands of `level`, `width_scale` times as wide.
    Strands {
        level: usize,
        count: usize,
        width_scale: f32,
    },
    Cards {
        level: usize,
    },
}

/// A band of on-screen sizes and what is drawn in it.
#[derive(Clone, Copy, Debug)]
pub struct Band {
    /// The head is at least this many pixels tall.
    pub min_pixels: f32,
    pub form: Form,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Form {
    /// A share of the strands at a level.
    Strands {
        level: usize,
        share: f32,
    },
    Cards {
        level: usize,
    },
}

#[derive(Clone, Debug)]
pub struct LodPolicy {
    pub name: &'static str,
    /// From the largest on screen down.
    pub bands: Vec<Band>,
    pub quality: Quality,
    /// Shadows are cast by this strand level and share, whatever is drawn.
    pub shadow_level: usize,
    pub shadow_share: f32,
    /// Over how much of a band's lower edge, as a share of it, the next
    /// coarser form fades in: 0.4 fades from 1.4 times the edge down to it.
    pub blend: f32,
    /// How finely the hair is simulated, by the head's height on screen in
    /// pixels, largest first: each level from its height up.
    pub sim: Vec<(f32, SimLevel)>,
}

impl LodPolicy {
    /// Every strand at every point, always.
    pub fn cinematic() -> Self {
        Self {
            name: "Cinematic",
            bands: vec![Band {
                min_pixels: 0.0,
                form: Form::Strands {
                    level: 0,
                    share: 1.0,
                },
            }],
            quality: Quality {
                marschner: true,
                shadows: true,
                shadow_resolution: 2048,
                accumulate: 64,
                min_pixels: 0.5,
            },
            shadow_level: 0,
            shadow_share: 1.0,
            blend: 0.0,
            sim: vec![(0.0, SimLevel::HIGH)],
        }
    }

    /// Strands close up, thinning with distance, cards far away. Every
    /// strand until the head is under 300 pixels tall -- a head in a
    /// medium shot at 1080p -- so what is coarser only shows where too
    /// little of it is on screen to tell.
    pub fn desktop() -> Self {
        Self {
            name: "Desktop",
            bands: vec![
                Band {
                    min_pixels: 300.0,
                    form: Form::Strands {
                        level: 0,
                        share: 1.0,
                    },
                },
                Band {
                    min_pixels: 150.0,
                    form: Form::Strands {
                        level: 1,
                        share: 0.5,
                    },
                },
                Band {
                    min_pixels: 70.0,
                    form: Form::Strands {
                        level: 2,
                        share: 0.2,
                    },
                },
                Band {
                    min_pixels: 32.0,
                    form: Form::Cards { level: 0 },
                },
                Band {
                    min_pixels: 0.0,
                    form: Form::Cards { level: 1 },
                },
            ],
            quality: Quality {
                marschner: true,
                shadows: true,
                shadow_resolution: 1024,
                accumulate: 8,
                min_pixels: 1.0,
            },
            shadow_level: 2,
            shadow_share: 0.25,
            blend: 0.4,
            sim: vec![
                (260.0, SimLevel::HIGH),
                (110.0, SimLevel::MEDIUM),
                (0.0, SimLevel::LOW),
            ],
        }
    }

    /// A fifth of the strands close up, a tenth further, cards far away;
    /// cheap shading and only baked shadowing.
    pub fn mobile() -> Self {
        Self {
            name: "Mobile",
            bands: vec![
                Band {
                    min_pixels: 300.0,
                    form: Form::Strands {
                        level: 2,
                        share: 0.2,
                    },
                },
                Band {
                    min_pixels: 140.0,
                    form: Form::Strands {
                        level: 3,
                        share: 0.1,
                    },
                },
                Band {
                    min_pixels: 55.0,
                    form: Form::Cards { level: 0 },
                },
                Band {
                    min_pixels: 0.0,
                    form: Form::Cards { level: 1 },
                },
            ],
            quality: Quality {
                marschner: false,
                shadows: false,
                shadow_resolution: 512,
                accumulate: 1,
                min_pixels: 1.0,
            },
            shadow_level: 3,
            shadow_share: 0.1,
            blend: 0.4,
            sim: vec![(220.0, SimLevel::MEDIUM), (0.0, SimLevel::LOW)],
        }
    }

    /// How finely to simulate a head `pixels` tall.
    pub fn sim_level(&self, pixels: f32) -> SimLevel {
        self.sim
            .iter()
            .find(|(min, _)| pixels >= *min)
            .or(self.sim.last())
            .map_or(SimLevel::MEDIUM, |(_, level)| *level)
    }

    /// [`LodPolicy::sim_level`], staying at `current` until the head is a
    /// sixth past the edge it would cross: a level change rebuilds the
    /// simulation, and a head resting on an edge should not flicker across
    /// it.
    pub fn sim_level_from(&self, current: SimLevel, pixels: f32) -> SimLevel {
        let wanted = self.sim_level(pixels);
        if wanted == current {
            return current;
        }
        let rank = |level: SimLevel| self.sim.iter().position(|(_, l)| *l == level);
        let (Some(at), Some(to)) = (rank(current), rank(wanted)) else {
            return wanted;
        };
        // Finer: past the finer level's edge by a sixth; coarser: below the
        // current level's edge by a sixth.
        let edge = if to < at {
            self.sim[to].0 * 7.0 / 6.0
        } else {
            self.sim[at].0 * 5.0 / 6.0
        };
        let past = if to < at {
            pixels >= edge
        } else {
            pixels < edge
        };
        if past { wanted } else { current }
    }

    /// One band throughout: the given form at every distance, with this
    /// policy's shading.
    pub fn fixed(&self, form: Form) -> Self {
        Self {
            bands: vec![Band {
                min_pixels: 0.0,
                form,
            }],
            ..self.clone()
        }
    }

    /// What to draw for a head `pixels` tall. Within a band of strands the
    /// share rises smoothly towards the next finer band's, so moving in
    /// grows the count rather than stepping it.
    pub fn select(&self, pixels: f32, lods: LodCounts) -> Representation {
        self.select_blended(pixels, lods).0
    }

    /// What to draw for a head `pixels` tall, and the coarser form fading
    /// in under it near the bottom of its band. At the band's lower edge
    /// the blend is all the coarser form, which is what the band below
    /// draws at its top: the hand-over is continuous.
    pub fn select_blended(&self, pixels: f32, lods: LodCounts) -> (Representation, Option<Blend>) {
        let at = self.band(pixels);
        let form = self.form(at, pixels, lods);
        let lo = self.bands[at].min_pixels;
        if self.blend <= 0.0 || at + 1 >= self.bands.len() || lo <= 0.0 {
            return (form, None);
        }
        let amount = 1.0 - (pixels - lo) / (lo * self.blend);
        if amount <= 0.0 {
            return (form, None);
        }
        let to = self.form(at + 1, lo, lods);
        (
            form,
            Some(Blend {
                to,
                amount: amount.min(1.0),
            }),
        )
    }

    fn band(&self, pixels: f32) -> usize {
        self.bands
            .iter()
            .position(|b| pixels >= b.min_pixels)
            .unwrap_or(self.bands.len() - 1)
    }

    /// Band `at`'s form for a head `pixels` tall.
    fn form(&self, at: usize, pixels: f32, lods: LodCounts) -> Representation {
        let total = lods.strands;
        match self.bands[at].form {
            Form::Cards { level } => Representation::Cards {
                level: level.min(lods.cards.saturating_sub(1)),
            },
            Form::Strands { level, share } => {
                let mut share = share;
                if at > 0
                    && let Form::Strands { share: finer, .. } = self.bands[at - 1].form
                {
                    let lo = self.bands[at].min_pixels;
                    let hi = self.bands[at - 1].min_pixels;
                    let t = ((pixels - lo) / (hi - lo).max(1.0)).clamp(0.0, 1.0);
                    share += (finer - share) * t;
                }
                let count = ((total as f32 * share).round() as usize).clamp(1.min(total), total);
                Representation::Strands {
                    level: level.min(lods.levels.saturating_sub(1)),
                    count,
                    width_scale: total as f32 / count.max(1) as f32,
                }
            }
        }
    }

    /// The strands that cast shadows: the policy's shadow level and share.
    pub fn shadow(&self, lods: LodCounts) -> Representation {
        let total = lods.strands;
        let count =
            ((total as f32 * self.shadow_share).round() as usize).clamp(1.min(total), total);
        Representation::Strands {
            level: self.shadow_level.min(lods.levels.saturating_sub(1)),
            count,
            width_scale: total as f32 / count.max(1) as f32,
        }
    }
}
