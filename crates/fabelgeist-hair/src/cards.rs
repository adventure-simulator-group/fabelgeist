//! Hair cards: the groom as a few thousand textured strips.
//!
//! Games far from the camera, and phones at any distance, cannot draw a
//! hundred thousand strands. They draw cards: strips following the groom's
//! locks, each showing a painted bundle of strands with transparency. Here
//! the cards come from the groom itself rather than from an artist -- the
//! strands are clustered into locks by the same guide map that clumps them
//! ([`GuideMap`]), and each lock becomes a strip along its members' mean
//! path, as wide as they spread across it, lying flat against the head
//! (its face turned out from the head's centre), starting behind its middle
//! root far enough to cover the scalp its lock grows from -- a card from
//! the middle would leave the half of the lock behind it bare, as partings
//! show -- and kept off the head where its few segments would cut through
//! the curve of the scalp. The bundle it shows is one
//! of a few variants in an atlas painted procedurally: wavy strands
//! converging towards the tip, each with its own shade and depth in the
//! bundle, mipmapped by coverage.

use crate::identity::StrandId;
use fabelgeist_math::Vec3;
use rayon::prelude::*;

use crate::collider::{Collider, keep_clear, push_out};
use crate::occlusion::Occlusion;
use crate::ops::{GuideMap, smoothstep};
use crate::rng::{self, Rng};
use crate::strands::{Strands, safe_normalize};

#[derive(Clone, Copy, Debug)]
pub struct CardSettings {
    /// Width of a lock at the root, metres: smaller makes more cards.
    pub size: f32,
    /// Quads along each card.
    pub segments: usize,
    /// Scale on each card's measured width.
    pub width_scale: f32,
    /// Locks of fewer strands than this are left out.
    pub min_strands: usize,
}

impl Default for CardSettings {
    fn default() -> Self {
        Self {
            size: 0.009,
            segments: 12,
            width_scale: 1.3,
            min_strands: 3,
        }
    }
}

/// One card vertex, as the card shader reads it.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CardVertex {
    pub position: [f32; 3],
    /// Across the card, 0 to 1.
    pub u: f32,
    /// Along the lock, root to tip.
    pub tangent: [f32; 3],
    /// Along the card, 0 at the root.
    pub v: f32,
    /// The card's face.
    pub normal: [f32; 3],
    /// Which atlas variant.
    pub variant: f32,
    /// Per card, 0 to 1.
    pub random: f32,
    /// Baked sky visibility.
    pub occlusion: f32,
    pub pad: [f32; 2],
}

/// Cards as a triangle list.
#[derive(Clone, Debug, Default)]
pub struct CardMesh {
    pub vertices: Vec<CardVertex>,
    pub indices: Vec<u32>,
    pub cards: usize,
}

impl CardMesh {
    pub fn build(
        strands: &Strands,
        center: Vec3,
        occlusion: Option<&Occlusion>,
        body: Option<&dyn Collider>,
        settings: &CardSettings,
        variants: u32,
    ) -> Self {
        let map = GuideMap::new(strands, settings.size, 1.0, 0xCA4D);
        let mut members: Vec<Vec<u32>> = vec![Vec::new(); strands.len()];
        for (s, &g) in map.guide.iter().enumerate() {
            members[g as usize].push(s as u32);
        }
        let segments = settings.segments.max(1);
        let points = strands.points;
        let cards: Vec<(Vec<CardVertex>, StrandId)> = map
            .guides
            .par_iter()
            .filter_map(|&g| {
                let lock = &members[g as usize];
                if lock.len() < settings.min_strands {
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
                    frames.push((tangent, normal, side));
                }
                // Smooth the widths along the card.
                let raw = widths.clone();
                for j in 1..segments {
                    widths[j] = (raw[j - 1] + 2.0 * raw[j] + raw[j + 1]) * 0.25;
                }
                let id = strands.id[g as usize];
                let variant = (rng::hash((id).word(), 0x7A) % variants.max(1)) as f32;
                let random = rng::unit((id).word(), 0x7B);
                let mut vertices = Vec::with_capacity((segments + 1) * 2);
                for j in 0..=segments {
                    let (tangent, normal, side) = frames[j];
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
                Some((vertices, id))
            })
            .collect();

        let mut mesh = CardMesh::default();
        for (vertices, _) in cards {
            let base = mesh.vertices.len() as u32;
            for j in 0..segments as u32 {
                let (a, b, c, d) = (
                    base + 2 * j,
                    base + 2 * j + 1,
                    base + 2 * j + 2,
                    base + 2 * j + 3,
                );
                mesh.indices.extend_from_slice(&[a, c, b, b, c, d]);
            }
            mesh.vertices.extend(vertices);
            mesh.cards += 1;
        }
        mesh
    }
}

/// Painted strand bundles, side by side, with their mip chain.
pub struct CardAtlas {
    pub width: u32,
    pub height: u32,
    pub variants: u32,
    /// RGBA8 per level: coverage, strand shade, depth in the bundle,
    /// coverage again.
    pub levels: Vec<Vec<u8>>,
}

impl CardAtlas {
    /// `variants` bundles, each `width` by `height` texels.
    pub fn paint(variants: u32, width: u32, height: u32, seed: u64) -> Self {
        let atlas_width = width * variants;
        let mut base = vec![0u8; (atlas_width * height * 4) as usize];
        let columns: Vec<Vec<u8>> = (0..variants)
            .into_par_iter()
            .map(|v| paint_bundle(width, height, seed ^ (v as u64 * 0x9E37)))
            .collect();
        for (v, column) in columns.iter().enumerate() {
            for y in 0..height {
                let from = (y * width * 4) as usize;
                let to = ((y * atlas_width + v as u32 * width) * 4) as usize;
                base[to..to + (width * 4) as usize]
                    .copy_from_slice(&column[from..from + (width * 4) as usize]);
            }
        }
        let mut levels = vec![base];
        let (mut w, mut h) = (atlas_width, height);
        while w > 1 && h > 1 && levels.len() < 12 {
            let previous = levels.last().unwrap();
            let (nw, nh) = ((w / 2).max(1), (h / 2).max(1));
            let mut next = vec![0u8; (nw * nh * 4) as usize];
            for y in 0..nh {
                for x in 0..nw {
                    for c in 0..4 {
                        let texel =
                            |xx: u32, yy: u32| previous[((yy * w + xx) * 4 + c) as usize] as u32;
                        let sum = texel(2 * x, 2 * y)
                            + texel((2 * x + 1).min(w - 1), 2 * y)
                            + texel(2 * x, (2 * y + 1).min(h - 1))
                            + texel((2 * x + 1).min(w - 1), (2 * y + 1).min(h - 1));
                        next[((y * nw + x) * 4 + c) as usize] = ((sum + 2) / 4) as u8;
                    }
                }
            }
            levels.push(next);
            w = nw;
            h = nh;
        }
        Self {
            width: atlas_width,
            height,
            variants,
            levels,
        }
    }
}

/// One bundle: strands down a `width` by `height` column, root at the top.
fn paint_bundle(width: u32, height: u32, seed: u64) -> Vec<u8> {
    let mut rng = Rng::new(seed);
    let mut coverage = vec![0.0f32; (width * height) as usize];
    let mut shade = vec![0.0f32; (width * height) as usize];
    let mut depth = vec![0.0f32; (width * height) as usize];
    let strands = 26 + rng.index(20);
    let converge = rng.range(0.2, 0.6);
    for _ in 0..strands {
        let x0 = rng.range(0.1, 0.9);
        let amplitude = rng.range(0.0, 0.035);
        let frequency = rng.range(0.5, 3.0);
        let phase = rng.range(0.0, std::f32::consts::TAU);
        let end = rng.range(0.7, 1.0);
        let start = rng.range(0.0, 0.05);
        let root_width = rng.range(0.9, 1.6) * width as f32 / 64.0;
        let tone = rng.next_f32();
        let layer = rng.next_f32();
        let path = |v: f32| {
            let pull = smoothstep(0.2, 1.0, v) * converge;
            let x = x0
                + (0.5 - x0) * pull
                + amplitude * (frequency * v * std::f32::consts::TAU + phase).sin();
            [x * width as f32, v * height as f32]
        };
        let steps = height as usize * 2;
        for k in 0..steps {
            let v = start + (end - start) * k as f32 / steps as f32;
            let [px, py] = path(v);
            let taper = 1.0 - smoothstep(0.6, 1.0, (v - start) / (end - start)) * 0.7;
            let r = (root_width * 0.5 * taper).max(0.35);
            let (x0i, x1i) = ((px - r - 1.0).floor() as i32, (px + r + 1.0).ceil() as i32);
            let y = py as i32;
            if y < 0 || y >= height as i32 {
                continue;
            }
            for x in x0i..=x1i {
                // The column wraps, so bundles tile across wide cards.
                let xw = x.rem_euclid(width as i32) as u32;
                let d = (x as f32 + 0.5 - px).abs();
                let c = (r + 0.5 - d).clamp(0.0, 1.0);
                let i = (y as u32 * width + xw) as usize;
                if c > coverage[i] || (c > 0.0 && layer < depth[i]) {
                    coverage[i] = coverage[i].max(c);
                    shade[i] = tone;
                    depth[i] = layer;
                }
            }
        }
    }
    let mut out = vec![0u8; (width * height * 4) as usize];
    for i in 0..(width * height) as usize {
        let c = (coverage[i] * 255.0).round() as u8;
        out[i * 4] = c;
        out[i * 4 + 1] = (shade[i] * 255.0) as u8;
        out[i * 4 + 2] = (depth[i] * 255.0) as u8;
        out[i * 4 + 3] = c;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::PartingGroup;
    use crate::strands::Attachment;

    #[test]
    fn a_comb_of_strands_becomes_a_few_cards() {
        let mut strands = Strands::new(16);
        for i in 0..60 {
            let x = (i % 30) as f32 * 0.001 + (i / 30) as f32 * 0.05;
            let positions: Vec<Vec3> = (0..16)
                .map(|j| Vec3::new(x, 0.1 - j as f32 * 0.01, 0.1))
                .collect();
            strands.push(
                &positions,
                Attachment {
                    id: StrandId::from_word(i),
                    group: PartingGroup::Unassigned,
                    radius: 3e-5,
                    ..Default::default()
                },
            );
        }
        let mesh = CardMesh::build(
            &strands,
            Vec3::default(),
            None,
            None,
            &CardSettings {
                size: 0.02,
                ..Default::default()
            },
            4,
        );
        assert!(mesh.cards >= 2 && mesh.cards <= 8, "{}", mesh.cards);
        assert_eq!(mesh.indices.len(), mesh.cards * 12 * 6);
        let atlas = CardAtlas::paint(4, 32, 128, 1);
        assert_eq!(
            atlas.levels[0].len(),
            (atlas.width * atlas.height * 4) as usize
        );
        assert!(atlas.levels[0].chunks(4).any(|t| t[0] > 200));
    }
}
