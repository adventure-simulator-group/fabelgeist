//! Hair's levels of detail as plain triangles.
//!
//! Cards are strips already; they only take their atlas coordinates. Strands
//! become ribbons: each point widened across the strand, square to the
//! direction out from the head's centre, as the hair crate's cards lie. A
//! level drawn with a share of its strands draws them wider by the inverse of
//! that share, so the head's coverage holds (the hair crate's strand
//! culling); no ribbon is drawn narrower than a renderer without the crate's
//! sub-pixel coverage can show.

use fabelgeist_hair::cards::{CardMesh, CardVertex};
use fabelgeist_hair::lod::{Form, StrandLevel};
use fabelgeist_hair::material::Taper;
use fabelgeist_math::Vec3;

use super::wrap::{Strip, StripPoint};
use super::{HeadPlacement, PlacedHair};

/// The narrowest a strand ribbon is drawn, metres.
const MIN_RIBBON_WIDTH: f32 = 0.0004;
/// The most ribbon vertices drawn at once: about a hundred megabytes of
/// vertex data in the preview.
const RIBBON_VERTEX_BUDGET: usize = 1_500_000;

/// Triangles to draw hair with, in model space.
pub struct HairGeometry {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub texcoords: Vec<[f32; 2]>,
    /// Baked sky visibility, 0 to 1, from the groom's own self-shadowing.
    pub occlusion: Vec<f32>,
    /// Each card's own random value, 0 to 1: which of a pixel's samples it
    /// covers, so stacked cards do not all cover the same ones. 0 for
    /// strands.
    pub randoms: Vec<f32>,
    /// Along the strands, root to tip.
    pub tangents: Vec<[f32; 3]>,
    pub faces: Vec<[u32; 3]>,
}

/// One mip level of the card atlas: coverage as alpha, the painted strand
/// shade as grey.
pub struct AtlasLevel {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

impl PlacedHair {
    /// The triangles of `form`.
    pub fn geometry(&self, form: Form) -> HairGeometry {
        match form {
            Form::Cards { level } => self.cards(level),
            Form::Strands { level, share } => self.ribbons(level, share),
        }
    }

    /// How many strands `form` draws: its share of the level's strands,
    /// within the ribbon budget. Cards draw none.
    pub fn strands_drawn(&self, form: Form) -> usize {
        let Form::Strands { level, share } = form else {
            return 0;
        };
        let Some(level) = self.lods.levels.get(level) else {
            return 0;
        };
        let wanted = (level.len() as f32 * share.clamp(0.0, 1.0)).round() as usize;
        let affordable = RIBBON_VERTEX_BUDGET / (2 * level.points).max(1);
        wanted.clamp(1, level.len().max(1)).min(affordable)
    }

    /// The card atlas's mip chain, largest first.
    pub fn atlas_levels(&self) -> Vec<AtlasLevel> {
        let atlas = &self.lods.atlas;
        // The atlas's width already spans every variant.
        let mut width = atlas.width;
        let mut height = atlas.height;
        atlas
            .levels
            .iter()
            .map(|texels| {
                let level = AtlasLevel {
                    width,
                    height,
                    rgba: shade_and_coverage(texels),
                };
                width = (width / 2).max(1);
                height = (height / 2).max(1);
                level
            })
            .collect()
    }

    fn cards(&self, level: usize) -> HairGeometry {
        let Some(cards) = self
            .lods
            .cards
            .get(level)
            .or_else(|| self.lods.cards.last())
        else {
            return HairGeometry::empty();
        };
        let variants = self.lods.atlas.variants.max(1) as f32;
        HairGeometry::from_strips(card_strips(cards, variants), &self.body)
    }

    fn ribbons(&self, level: usize, share: f32) -> HairGeometry {
        let form = Form::Strands { level, share };
        let Some(strands) = self.lods.levels.get(level) else {
            return HairGeometry::empty();
        };
        let drawn = self.strands_drawn(form);
        let widen = strands.len() as f32 / drawn.max(1) as f32;
        let ribbons = ribbon_strips(strands, drawn, widen, self.lods.center);
        HairGeometry::from_strips(ribbons, &self.body)
    }
}

impl HairGeometry {
    fn empty() -> Self {
        Self {
            positions: Vec::new(),
            normals: Vec::new(),
            texcoords: Vec::new(),
            occlusion: Vec::new(),
            randoms: Vec::new(),
            tangents: Vec::new(),
            faces: Vec::new(),
        }
    }

    /// This geometry, placed by `from`, carried to where `to` places the
    /// same hair: how hair follows a head reshaped by a morph channel.
    pub fn carried(&self, from: &HeadPlacement, to: &HeadPlacement) -> Self {
        Self {
            positions: self
                .positions
                .iter()
                .map(|&point| to.point(from.bust_point(point)))
                .collect(),
            normals: self
                .normals
                .iter()
                .map(|&normal| to.normal(from.bust_normal(normal)))
                .collect(),
            texcoords: self.texcoords.clone(),
            occlusion: self.occlusion.clone(),
            randoms: self.randoms.clone(),
            tangents: self
                .tangents
                .iter()
                .map(|&tangent| to.direction(from.bust_direction(tangent)))
                .collect(),
            faces: self.faces.clone(),
        }
    }
}

/// Each card as a strip: the hair crate cuts every card into as many rows,
/// two points to a row.
fn card_strips(cards: &CardMesh, variants: f32) -> Vec<Strip> {
    let rows = cards.vertices.len() / (2 * cards.cards.max(1));
    let point = |vertex: &CardVertex| StripPoint {
        position: Vec3::from_array(vertex.position),
        normal: Vec3::from_array(vertex.normal),
        tangent: Vec3::from_array(vertex.tangent),
        texcoord: [(vertex.variant + vertex.u) / variants, vertex.v],
        occlusion: vertex.occlusion,
        random: vertex.random,
    };
    cards
        .vertices
        .chunks_exact(2 * rows.max(1))
        .map(|card| {
            Strip(
                card.as_chunks::<2>()
                    .0
                    .iter()
                    .map(|[left, right]| [point(left), point(right)])
                    .collect(),
            )
        })
        .collect()
}

/// The first `drawn` strands of `level` as ribbons, `widen` times as wide as
/// the strands they draw.
fn ribbon_strips(level: &StrandLevel, drawn: usize, widen: f32, center: Vec3) -> Vec<Strip> {
    let points = level.points;
    let taper = Taper::default();
    level
        .strands
        .iter()
        .take(drawn)
        .enumerate()
        .map(|(strand, info)| {
            let samples = &level.positions[strand * points..][..points];
            let width = (2.0 * info.radius * widen).max(MIN_RIBBON_WIDTH);
            let at = |i: usize| {
                let [x, y, z, _] = samples[i];
                Vec3::new(x, y, z)
            };
            let rows = (0..points)
                .map(|index| {
                    let point = at(index);
                    let tangent =
                        (at((index + 1).min(points - 1)) - at(index.saturating_sub(1))).normalize();
                    let side = tangent.cross(point - center).normalize();
                    let normal = side.cross(tangent).normalize();
                    let along = index as f32 / (points - 1).max(1) as f32;
                    let half = 0.5 * width * taper.at(along);
                    let visible = samples[index][3];
                    [(0.0, -half), (1.0, half)].map(|(across, offset)| StripPoint {
                        position: point + side * offset,
                        normal,
                        tangent,
                        texcoord: [across, along],
                        occlusion: visible,
                        random: 0.0,
                    })
                })
                .collect();
            Strip(rows)
        })
        .collect()
}

/// The atlas stores coverage, shade, depth and coverage; drawing wants the
/// shade as colour and the coverage as alpha.
fn shade_and_coverage(texels: &[u8]) -> Vec<u8> {
    texels
        .as_chunks::<4>()
        .0
        .iter()
        .flat_map(|&[coverage, shade, _, _]| [shade, shade, shade, coverage])
        .collect()
}
