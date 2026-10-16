//! Hair's levels of detail as plain triangles: cards as textured strips,
//! strands as ribbons, built on the GPU the hair was placed on
//! ([`super::gpu`]'s strips) and read back.

use anyhow::Result;
use fabelgeist_hair::lod::Form;

use super::gpu::StripSource;
use super::{HairGpu, HeadPlacement, PlacedHair};

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
    /// The triangles of `form`, built on `gpu`.
    pub fn geometry(&self, gpu: &HairGpu, form: Form) -> Result<HairGeometry> {
        let source = self.strip_source();
        let built = match form {
            Form::Cards { level } => {
                let Some(cards) = self
                    .lods
                    .cards
                    .get(level)
                    .or_else(|| self.lods.cards.last())
                else {
                    return Ok(HairGeometry::empty());
                };
                source.cards(gpu, cards, self.lods.atlas.variants)?
            }
            Form::Strands { level, .. } => {
                let Some(strands) = self.lods.levels.get(level) else {
                    return Ok(HairGeometry::empty());
                };
                let drawn = self.strands_drawn(form);
                let widen = strands.count as f32 / drawn.max(1) as f32;
                source.ribbons(gpu, strands, drawn as u32, widen, self.lods.center)?
            }
        };
        built.read(gpu)
    }

    /// Where strips of this hair are built from.
    fn strip_source(&self) -> StripSource<'_> {
        StripSource {
            body: &self.body,
            longest: self.longest,
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
        let strands = level.count as usize;
        let wanted = (strands as f32 * share.clamp(0.0, 1.0)).round() as usize;
        let rows = self.strip_source().ribbon_rows(level.points) as usize;
        let affordable = RIBBON_VERTEX_BUDGET / (2 * rows).max(1);
        wanted.clamp(1, strands.max(1)).min(affordable)
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
}

impl HairGeometry {
    pub(super) fn empty() -> Self {
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
