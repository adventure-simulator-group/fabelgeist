//! Hair's levels of detail as plain triangles: cards as textured strips,
//! strands as ribbons, built on the GPU the hair was placed on
//! ([`super::gpu`]'s strips) and read back.

use anyhow::Result;
use fabelgeist_hair::lod::Form;
use fabelgeist_hair::parameters::StrandCount;
use fabelgeist_math::units::Ratio;

use super::gpu::{GpuGeometry, StripSource};
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
    /// The triangles of `form`, built on `gpu` and read back.
    pub fn geometry(&self, gpu: &HairGpu, form: Form) -> Result<HairGeometry> {
        match self.gpu_geometry(gpu, form)? {
            Some(geometry) => geometry.read(gpu),
            None => Ok(HairGeometry::empty()),
        }
    }

    /// The triangles of `form`, built on `gpu` and left there, to draw from
    /// where they lie; none for a form this hair has not.
    pub fn gpu_geometry(&self, gpu: &HairGpu, form: Form) -> Result<Option<GpuGeometry>> {
        let source = self.strip_source();
        let built = match form {
            Form::Cards { level } => {
                let Some(cards) = self
                    .lods
                    .cards()
                    .get(level.index())
                    .or_else(|| self.lods.cards().last())
                else {
                    return Ok(None);
                };
                source.cards(gpu, cards, self.lods.atlas().variants())?
            }
            Form::Strands { level, .. } => {
                let Some(strands) = self.lods.levels().get(level.index()) else {
                    return Ok(None);
                };
                let drawn = self.strands_drawn(form)?;
                let widen = Ratio::from_ratio(
                    strands.layout().strands().count() as f32 / drawn.count().max(1) as f32,
                );
                source.ribbons(gpu, strands, drawn, widen, self.lods.center())?
            }
        };
        Ok(Some(built))
    }

    /// Where strips of this hair are built from.
    fn strip_source(&self) -> StripSource<'_> {
        StripSource {
            body: &self.body,
            device: self.lods.device(),
            longest: self.longest,
        }
    }

    /// How many strands `form` draws: its share of the level's strands,
    /// within the ribbon budget. Cards draw none.
    pub fn strands_drawn(&self, form: Form) -> Result<StrandCount> {
        let Form::Strands { level, share } = form else {
            return Ok(StrandCount::default());
        };
        let Some(level) = self.lods.levels().get(level.index()) else {
            return Ok(StrandCount::default());
        };
        let strands = level.layout().strands().count();
        if strands == 0 {
            return Ok(StrandCount::default());
        }
        // The preview budget is counted in native two-edge row vertices.
        let wanted = (strands as f32 * share.ratio().clamp(0.0, 1.0)).round() as usize;
        let rows = self.strip_source().ribbon_rows(level)?;
        let affordable = RIBBON_VERTEX_BUDGET as u64 / (u64::from(rows.native_rows()) * 2);
        Ok(StrandCount::from_count(wanted.clamp(1, strands).min(affordable as usize)))
    }

    /// The card atlas's mip chain, largest first.
    pub fn atlas_levels(&self) -> Vec<AtlasLevel> {
        self.lods.atlas().levels().map(|mip| AtlasLevel {
            width: mip.width_texels(),
            height: mip.height_texels(),
            rgba: shade_and_coverage(mip.pixel_bytes()),
        }).collect()
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
