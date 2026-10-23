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

pub use counts::{CardLevelCount, CardLevelIndex, LodCounts, StrandLevelCount, StrandLevelIndex};
pub use policy::{
    Band, Blend, Form, LodPolicy, LodPolicyError, PolicyResult, Representation, Selection, SimBand,
};
pub use quality::{
    AccumulationFrames, HairShading, PixelSpan, Quality, ShadowMapSize, ShadowQuality,
};

use fabelgeist_math::Vec3;
use rayon::prelude::*;

use crate::cards::{CardAtlas, CardMesh, CardSettings};
use crate::collider::{Collider, keep_clear};
use crate::cover::ScalpCover;
use crate::occlusion::{Occlusion, OcclusionSettings};
use crate::rng;
use crate::strands::{Strands, resample_into};

mod counts;
mod policy;
mod presets;
mod quality;

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
        order.sort_by_key(|&s| rng::hash(strands.id[s].word(), 0x0D3E));

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
                random: rng::unit(strands.id[s].word(), 0x5EED),
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
