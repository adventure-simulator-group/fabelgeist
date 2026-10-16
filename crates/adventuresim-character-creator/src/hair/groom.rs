//! A hairstyle grown on the bust, and the same hair placed on a character,
//! both on the GPU.

use anyhow::Result;
use fabelgeist_gpu::prelude::Buffer;
use fabelgeist_hair::gpu::{GpuGroom, GpuLods, head_shape};
use fabelgeist_hair::lod::Form;
use fabelgeist_hair::{Head, LodSettings, MeshCollider, Strands, Surface};
use fabelgeist_math::Vec3;
use std::sync::Arc;

use super::gpu::{BodyField, Draping};
use super::scalp_map::ScalpMap;
use super::{Groom, HairGpu, HeadPlacement, ScalpCap, ScalpTexture};

/// Edge of the surface-nets cells the scalp is cut from, metres: the size the
/// hair crate's viewer and dataset grow on.
const SCALP_CELL: f32 = 0.004;
/// How far the scalp cap's collider answers, metres: further from the skin
/// than this is outside and needs no exact distance.
const BODY_REACH: f32 = 0.1;
/// How far the body field reaches past the hair as carried, metres: the
/// drape and the shrinkwrap move hair a few centimetres, and the field must
/// cover where it goes.
const FIELD_MARGIN: f32 = 0.08;

/// A hairstyle grown on the mannequin bust, in the bust's frame: metres, up
/// along +Y, the face towards +Z. The strands stay on the GPU they grew on.
pub struct GroomedHair {
    pub groom: Groom,
    on_gpu: GpuGroom,
    /// The box the strands are in.
    bounds: [Vec3; 2],
    /// The longest strand, metres.
    longest: f32,
    /// Whether the hair hangs free, and so is draped on a character.
    drapes: bool,
    head: Head,
    scalp: Surface,
    /// Where on the scalp's texture each point of the scalp lies.
    scalp_map: ScalpMap,
    /// The hair lying on the scalp, baked into its texture.
    scalp_texture: Arc<ScalpTexture>,
}

/// The skin hair must stay out of, in model space.
pub struct BodySurface<'a> {
    pub positions: &'a [[f32; 3]],
    pub faces: &'a [[u32; 3]],
}

/// Hair on a character's head, ready to draw at every level of detail, on
/// the GPU it was placed on.
pub struct PlacedHair {
    pub groom: Groom,
    pub(super) lods: GpuLods,
    /// What the hair is kept clear of, as the hair crate's kernels read it:
    /// the character's body as a field, or the bust's shapes.
    pub(super) body: Buffer,
    /// The longest strand, metres.
    pub(super) longest: f32,
    placement: HeadPlacement,
    cap: ScalpCap,
    scalp_texture: Arc<ScalpTexture>,
}

impl GroomedHair {
    /// Grow `groom` on the mannequin bust on `gpu`, and bake the hair lying
    /// on its scalp.
    pub fn grow(groom: Groom, gpu: &HairGpu) -> Result<Self> {
        let look = groom.look();
        let head = look.head(&Head::mannequin());
        let (scalp, strands, on_gpu, bounds) = {
            let mut groomer = gpu.groomer();
            let scalp = groomer.cut_scalp(&head, SCALP_CELL)?;
            let grown = groomer.grow_look(&look, &head, &scalp)?;
            let bounds = groomer.measure_bounds(&grown)?;
            (
                scalp.read(&groomer)?,
                grown.read(groomer.context())?,
                grown,
                bounds,
            )
        };
        let scalp_map = ScalpMap::fit(&head, &scalp);
        let scalp_texture = Arc::new(ScalpTexture::bake(&strands, &head, &scalp_map));
        Ok(Self {
            groom,
            on_gpu,
            bounds,
            longest: longest(&strands),
            drapes: look.drape,
            head,
            scalp,
            scalp_map,
            scalp_texture,
        })
    }

    /// The bust this hair grew on.
    pub fn head(&self) -> &Head {
        &self.head
    }

    /// Carry the strands onto a character, drape them on its `body`, and
    /// prepare every level of detail there, clear of it ([`Draping`]).
    pub fn place(
        &self,
        gpu: &HairGpu,
        placement: &HeadPlacement,
        body: &BodySurface<'_>,
    ) -> Result<PlacedHair> {
        let field = BodyField::sample(gpu, body, self.carried_bounds(placement))?;
        let draping = Draping {
            bust: &self.head,
            placement,
            drapes: self.drapes,
        };
        let placed = draping.place(gpu, &self.on_gpu, &field)?;
        let collider = MeshCollider::new(
            body.positions
                .iter()
                .copied()
                .map(Vec3::from_array)
                .collect(),
            body.faces.to_vec(),
            BODY_REACH,
        );
        self.prepare(gpu, &placed, placement, field.buffer, &collider)
    }

    /// This hair left on its own bust, as the hair crate grew it, with every
    /// level of detail prepared against the bust: what drawing it on a
    /// character is measured against. The bust's surface comes beside it.
    pub fn on_bust(&self, gpu: &HairGpu) -> Result<(PlacedHair, Surface)> {
        let bust = self.head.mesh(SCALP_CELL);
        let collider =
            MeshCollider::new(bust.positions.clone(), bust.triangles.clone(), BODY_REACH);
        let shape = super::gpu::upload(gpu.context(), &head_shape(&self.head), "hair bust shape")?;
        let placement = HeadPlacement::identity(0);
        let placed = self.prepare(gpu, &self.on_gpu, &placement, shape, &collider)?;
        Ok((placed, bust))
    }

    /// The box the strands are in once `placement` carries them, and as far
    /// again as they may move.
    fn carried_bounds(&self, placement: &HeadPlacement) -> [Vec3; 2] {
        let [lo, hi] = self.bounds;
        let corners = (0..8u32).map(|corner| {
            let pick =
                |bit: u32, low: f32, high: f32| if corner >> bit & 1 == 0 { low } else { high };
            Vec3::from_array(placement.point([
                pick(0, lo.x, hi.x),
                pick(1, lo.y, hi.y),
                pick(2, lo.z, hi.z),
            ]))
        });
        let (low, high) = corners.fold(
            (Vec3::splat(f32::INFINITY), Vec3::splat(f32::NEG_INFINITY)),
            |(low, high), corner| (low.min(corner), high.max(corner)),
        );
        let margin = Vec3::splat(FIELD_MARGIN);
        [low - margin, high + margin]
    }

    /// Every level of detail of `strands`, placed by `placement` and kept
    /// clear of `body`, and the scalp cap under them, laid on `skin`.
    fn prepare(
        &self,
        gpu: &HairGpu,
        strands: &GpuGroom,
        placement: &HeadPlacement,
        body: Buffer,
        skin: &MeshCollider,
    ) -> Result<PlacedHair> {
        let center = Vec3::from_array(placement.point(self.head.cranium.center.to_array()));
        let lods =
            gpu.groomer()
                .lods_against(strands, center, &LodSettings::default(), Some(&body))?;
        let cap = ScalpCap::fit(&self.scalp, &self.head, &self.scalp_map, placement, skin);
        Ok(PlacedHair {
            groom: self.groom,
            lods,
            body,
            longest: self.longest,
            placement: *placement,
            cap,
            scalp_texture: self.scalp_texture.clone(),
        })
    }
}

/// The longest of `strands`, metres.
fn longest(strands: &Strands) -> f32 {
    strands.lengths().into_iter().fold(0.0, f32::max)
}

impl PlacedHair {
    pub fn placement(&self) -> &HeadPlacement {
        &self.placement
    }

    /// The scalp under the hair, fitted to the character's skull.
    pub fn cap(&self) -> &ScalpCap {
        &self.cap
    }

    /// The hair lying on the scalp, baked into textures over the cap's
    /// texture coordinates.
    pub fn scalp_texture(&self) -> &ScalpTexture {
        &self.scalp_texture
    }

    /// Every form this hair can be drawn in, finest first: each strand
    /// level with all its strands, then each cut of cards.
    pub fn forms(&self) -> Vec<Form> {
        let strands = (0..self.lods.levels.len()).map(|level| Form::Strands { level, share: 1.0 });
        let cards = (0..self.lods.cards.len()).map(|level| Form::Cards { level });
        strands.chain(cards).collect()
    }

    /// Points along each strand at strand `level`.
    pub fn strand_points(&self, level: usize) -> usize {
        self.lods
            .levels
            .get(level)
            .map_or(0, |level| level.points as usize)
    }

    /// Strands in the groom.
    pub fn strand_count(&self) -> usize {
        self.lods.strand_count()
    }

    /// The cards in cut `level`.
    pub fn card_count(&self, level: usize) -> usize {
        self.lods
            .cards
            .get(level)
            .map_or(0, |cut| cut.cards as usize)
    }
}
