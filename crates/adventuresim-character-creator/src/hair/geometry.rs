//! Hair's levels of detail as plain triangles.
//!
//! Cards are strips already; they only take their atlas coordinates. Strands
//! become ribbons: each point widened across the strand, square to the
//! direction out from the head's centre, as the hair crate's cards lie. A
//! level drawn with a share of its strands draws them wider by the inverse of
//! that share, so the head's coverage holds (the hair crate's strand
//! culling); no ribbon is drawn narrower than a renderer without the crate's
//! sub-pixel coverage can show.

use fabelgeist_hair::cards::CardMesh;
use fabelgeist_hair::collider::{Collider, push_out};
use fabelgeist_hair::lod::{Form, StrandLevel};
use fabelgeist_hair::material::Taper;
use fabelgeist_math::Vec3;

use super::{HeadPlacement, PlacedHair};

/// How far off the skin a card's edge is kept, metres. The hair crate keeps
/// each card's spine clear, but a wide flat strip over a curved scalp still
/// dips its edges in.
const CARD_CLEARANCE: f32 = 0.0005;
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
        card_geometry(cards, variants, &self.body)
    }

    fn ribbons(&self, level: usize, share: f32) -> HairGeometry {
        let form = Form::Strands { level, share };
        let Some(strands) = self.lods.levels.get(level) else {
            return HairGeometry::empty();
        };
        let drawn = self.strands_drawn(form);
        let widen = strands.len() as f32 / drawn.max(1) as f32;
        ribbon_geometry(strands, drawn, widen, self.lods.center)
    }
}

impl HairGeometry {
    fn empty() -> Self {
        Self {
            positions: Vec::new(),
            normals: Vec::new(),
            texcoords: Vec::new(),
            occlusion: Vec::new(),
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
            faces: self.faces.clone(),
        }
    }
}

fn card_geometry(cards: &CardMesh, variants: f32, body: &impl Collider) -> HairGeometry {
    let vertices = &cards.vertices;
    HairGeometry {
        positions: vertices
            .iter()
            .map(|vertex| {
                push_out(body, Vec3::from_array(vertex.position), CARD_CLEARANCE).to_array()
            })
            .collect(),
        normals: vertices.iter().map(|vertex| vertex.normal).collect(),
        texcoords: vertices
            .iter()
            .map(|vertex| [(vertex.variant + vertex.u) / variants, vertex.v])
            .collect(),
        occlusion: vertices.iter().map(|vertex| vertex.occlusion).collect(),
        faces: cards.indices.as_chunks::<3>().0.to_vec(),
    }
}

/// The first `drawn` strands of `level` as ribbons, `widen` times as wide as
/// the strands they draw.
fn ribbon_geometry(level: &StrandLevel, drawn: usize, widen: f32, center: Vec3) -> HairGeometry {
    let points = level.points;
    let taper = Taper::default();
    let mut geometry = HairGeometry::empty();
    for (strand, info) in level.strands.iter().take(drawn).enumerate() {
        let samples = &level.positions[strand * points..][..points];
        let base = geometry.positions.len() as u32;
        let width = (2.0 * info.radius * widen).max(MIN_RIBBON_WIDTH);
        for (index, sample) in samples.iter().enumerate() {
            let [x, y, z, visible] = *sample;
            let point = Vec3::new(x, y, z);
            let at = |i: usize| {
                let [x, y, z, _] = samples[i];
                Vec3::new(x, y, z)
            };
            let tangent =
                (at((index + 1).min(points - 1)) - at(index.saturating_sub(1))).normalize();
            let out = point - center;
            let side = tangent.cross(out).normalize();
            let normal = side.cross(tangent).normalize();
            let along = index as f32 / (points - 1).max(1) as f32;
            let half = 0.5 * width * taper.at(along);
            for (across, offset) in [(0.0, -half), (1.0, half)] {
                geometry.positions.push((point + side * offset).to_array());
                geometry.normals.push(normal.to_array());
                geometry.texcoords.push([across, along]);
                geometry.occlusion.push(visible);
            }
        }
        for segment in 0..points as u32 - 1 {
            let [a, b] = [base + 2 * segment, base + 2 * segment + 1];
            let [c, d] = [a + 2, b + 2];
            geometry.faces.extend([[a, c, b], [b, c, d]]);
        }
    }
    geometry
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
