//! Build strips along source locks without narrowing their indexed layout.

use super::mesh::CardTopology;
use super::{AtlasVariantCount, CardCount, CardMesh, CardResult, CardSegments, CardSettings,
    CardStripLayout, CardVertex};
use crate::collider::{Collider, keep_clear, push_out};
use crate::occlusion::Occlusion;
use crate::ops::GuideMap;
use crate::rng;
use crate::strands::{Strands, safe_normalize};
use fabelgeist_math::Vec3;
use rayon::prelude::*;

#[derive(Clone, Copy)]
struct CardBake<'a> {
    strands: &'a Strands,
    center: Vec3,
    occlusion: Option<&'a Occlusion>,
    body: Option<&'a dyn Collider>,
    settings: &'a CardSettings,
    variants: AtlasVariantCount,
    segments: CardSegments,
}

#[derive(Clone, Copy)]
struct CardFrame {
    tangent: Vec3,
    normal: Vec3,
    side: Vec3,
}

/// The three row arrays are built together at the same admitted point count.
struct CardRows {
    spine: Vec<Vec3>,
    frames: Vec<CardFrame>,
    widths: Vec<f32>,
}

impl CardMesh {
    pub fn build(
        strands: &Strands,
        center: Vec3,
        occlusion: Option<&Occlusion>,
        body: Option<&dyn Collider>,
        settings: &CardSettings,
        variants: AtlasVariantCount,
    ) -> CardResult<Self> {
        let segments = settings.segments.max(CardSegments::from_count(1));
        // Validate a complete strip before its row allocations or native indices.
        CardStripLayout::new(CardCount::from_count(1), segments)?;
        let map = GuideMap::new(strands, settings.size, 1.0, 0xCA4D);
        let mut members: Vec<Vec<u32>> = vec![Vec::new(); strands.len()];
        for (strand, &guide) in map.guide.iter().enumerate() {
            members[guide as usize].push(strand as u32);
        }
        let bake = CardBake { strands, center, occlusion, body, settings, variants, segments };
        let cards: Vec<Vec<CardVertex>> = map.guides.par_iter()
            .filter_map(|&guide| bake.lock(guide, &members[guide as usize])).collect();
        let strips = CardStripLayout::new(CardCount::from_count(cards.len()), segments)?;
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        vertices.try_reserve_exact(usize::from(strips.layout().vertices()))?;
        indices.try_reserve_exact(usize::from(strips.layout().corners()))?;
        for card in cards {
            // The complete indexed population was admitted before concatenation.
            let base = vertices.len() as u32;
            for segment in 0..segments.count() as u32 {
                let (a, b, c, d) = (
                    base + 2 * segment,
                    base + 2 * segment + 1,
                    base + 2 * segment + 2,
                    base + 2 * segment + 3,
                );
                indices.extend_from_slice(&[a, c, b, b, c, d]);
            }
            vertices.extend(card);
        }
        Ok(Self { vertices, indices, topology: CardTopology::Strips(strips) })
    }
}

impl CardBake<'_> {
    fn lock(&self, g: u32, lock: &[u32]) -> Option<Vec<CardVertex>> {
        let Self { strands, center, body, settings, segments, .. } = *self;
        let segments = segments.count();
        let points = strands.points;
        if lock.len() < settings.min_strands.count() {
            return None;
        }
        let at = |s: u32, t: f32| {
            let x = t * (points - 1) as f32;
            let i = (x as usize).min(points - 2);
            let strand = strands.strand(s as usize);
            strand[i].lerp(strand[i + 1], x - i as f32)
        };
        let mut spine: Vec<Vec3> = (0..=segments)
            .map(|j| {
                let t = j as f32 / segments as f32;
                lock.iter()
                    .map(|&s| at(s, t))
                    .fold(Vec3::default(), |a, b| a + b)
                    * (1.0 / lock.len() as f32)
            })
            .collect();
        // Back from the middle root over the roots behind it, on the
        // scalp.
        let ahead = safe_normalize(spine[1] - spine[0], Vec3::down());
        let mut behind: Vec<f32> = lock
            .iter()
            .map(|&s| (spine[0] - strands.root(s as usize)).dot(ahead).max(0.0))
            .collect();
        behind.sort_by(|a, b| a.total_cmp(b));
        let back = behind[(behind.len() * 9 / 10).min(behind.len() - 1)].min(settings.size);
        spine[0] -= ahead * back;
        if let Some(body) = body {
            spine[0] = push_out(body, spine[0], 0.0005);
            keep_clear(body, &mut spine, 0.0015, 4);
        }
        let radius = lock
            .iter()
            .map(|&s| strands.radius[s as usize])
            .fold(0.0, f32::max);
        let mut frames = Vec::with_capacity(segments + 1);
        let mut widths = Vec::with_capacity(segments + 1);
        for j in 0..=segments {
            let t = j as f32 / segments as f32;
            let tangent = safe_normalize(
                spine[(j + 1).min(segments)] - spine[j.saturating_sub(1)],
                Vec3::down(),
            );
            let out = safe_normalize(spine[j] - center, Vec3::up());
            let normal = safe_normalize(out - tangent * out.dot(tangent), Vec3::forward());
            let side = tangent.cross(normal);
            let mut spread: Vec<f32> = lock
                .iter()
                .map(|&s| (at(s, t) - spine[j]).dot(side).abs())
                .collect();
            spread.sort_by(|a, b| a.total_cmp(b));
            let p85 = spread[(spread.len() * 85 / 100).min(spread.len() - 1)];
            widths.push((2.0 * p85 + 4.0 * radius).clamp(0.0015, settings.size * 3.0));
            frames.push(CardFrame { tangent, normal, side });
        }
        // Smooth the widths along the card.
        let raw = widths.clone();
        for j in 1..segments {
            widths[j] = (raw[j - 1] + 2.0 * raw[j] + raw[j + 1]) * 0.25;
        }
        Some(self.emit(g, CardRows { spine, frames, widths }))
    }

    fn emit(&self, g: u32, rows: CardRows) -> Vec<CardVertex> {
        let strands = self.strands;
        let variants = self.variants;
        let settings = self.settings;
        let occlusion = self.occlusion;
        let segments = self.segments.count();
        let CardRows { spine, frames, widths } = rows;
        let id = strands.id[g as usize];
        let variant = (rng::hash((id).word(), 0x7A) % variants.count()) as f32;
        let random = rng::unit((id).word(), 0x7B);
        let mut vertices = Vec::with_capacity((segments + 1) * 2);
        for j in 0..=segments {
            let CardFrame { tangent, normal, side } = frames[j];
            let half = widths[j] * 0.5 * settings.width_scale;
            // As light reaches the lock's outer strands, which are
            // what is seen: its middle is deeper in the hair.
            let occlusion = occlusion.map_or(1.0, |o| o.at(spine[j] + normal * 0.002));
            for (u, sign) in [(0.0, -1.0), (1.0, 1.0)] {
                vertices.push(CardVertex {
                    position: (spine[j] + side * (half * sign)).to_array(),
                    u,
                    tangent: tangent.to_array(),
                    v: j as f32 / segments as f32,
                    normal: normal.to_array(),
                    variant,
                    random,
                    occlusion,
                    pad: [0.0; 2],
                });
            }
        }
        vertices
    }
}
