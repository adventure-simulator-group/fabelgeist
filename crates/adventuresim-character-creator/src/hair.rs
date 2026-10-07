//! The hair a character wears, and the cards it is drawn with.
//!
//! `fabelgeist-hair` grows a hairstyle as strands on its mannequin bust and
//! cuts them into hair cards: strips along the groom's locks, each showing a
//! painted bundle of strands from a shared atlas. A [`HeadPlacement`] carries
//! the cards from the bust onto the character's own head, stretched to its
//! width, depth and crown, facing the way its eyes do.

mod placement;

pub use placement::{HeadPlacement, HeadPlacementError, HeadSkin};

use fabelgeist_hair::cards::{CardAtlas, CardMesh, CardSettings};
use fabelgeist_hair::occlusion::Occlusion;
use fabelgeist_hair::{HairMaterial, Head, LodSettings, Look, Preset};
use serde::{Deserialize, Serialize};

/// Hair's perceptual roughness when drawn as cards: a rough dielectric whose
/// sheen comes from the strands painted in the atlas.
pub const HAIR_ROUGHNESS: f32 = 0.55;

/// Edge of the surface-nets cells the scalp is cut from, metres: the size the
/// hair crate's viewer and dataset grow on.
const SCALP_CELL: f32 = 0.004;

/// Hair pigment as Blender's Principled Hair takes it. Both shares run from 0
/// to 1: melanin from white through blond and brown to black, redness from
/// ashen to red.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, try_from = "PigmentShares")]
pub struct HairPigment {
    melanin: f32,
    redness: f32,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PigmentShares {
    melanin: f32,
    redness: f32,
}

/// A pigment share outside 0 to 1.
#[derive(Debug, thiserror::Error)]
#[error("hair pigment shares must lie between 0 and 1")]
pub struct PigmentRangeError;

impl TryFrom<PigmentShares> for HairPigment {
    type Error = PigmentRangeError;

    fn try_from(shares: PigmentShares) -> Result<Self, Self::Error> {
        let share = 0.0..=1.0;
        if share.contains(&shares.melanin) && share.contains(&shares.redness) {
            Ok(Self {
                melanin: shares.melanin,
                redness: shares.redness,
            })
        } else {
            Err(PigmentRangeError)
        }
    }
}

impl Default for HairPigment {
    /// A mid brown.
    fn default() -> Self {
        let material = HairMaterial::default();
        Self {
            melanin: material.melanin,
            redness: material.redness,
        }
    }
}

impl HairPigment {
    /// A pigment with both shares clamped to 0 to 1.
    pub fn new(melanin: f32, redness: f32) -> Self {
        Self {
            melanin: melanin.clamp(0.0, 1.0),
            redness: redness.clamp(0.0, 1.0),
        }
    }

    pub fn melanin(self) -> f32 {
        self.melanin
    }

    pub fn redness(self) -> f32 {
        self.redness
    }

    /// The hair's real-time colour, linear RGB.
    pub fn color(self) -> [f32; 3] {
        HairMaterial {
            melanin: self.melanin,
            redness: self.redness,
            ..HairMaterial::default()
        }
        .base_color()
    }
}

/// A hairstyle and its colour.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HairDesign {
    pub preset: Preset,
    pub pigment: HairPigment,
}

/// A hairstyle's name as the interface shows it: `SideSwept` as "Side swept".
pub fn preset_label(preset: Preset) -> String {
    let name = format!("{preset:?}");
    let mut label = String::with_capacity(name.len() + 2);
    for (index, letter) in name.chars().enumerate() {
        if index > 0 && letter.is_uppercase() {
            label.push(' ');
            label.extend(letter.to_lowercase());
        } else {
            label.push(letter);
        }
    }
    label
}

/// A hairstyle grown on the mannequin bust and cut into cards, in the bust's
/// frame: metres, up along +Y, the face towards +Z.
pub struct GroomedHair {
    pub preset: Preset,
    cards: CardMesh,
    atlas: CardAtlas,
    head: Head,
}

/// Cards placed on a character, ready to draw.
pub struct HairCards {
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

impl GroomedHair {
    /// Grow `preset` on the mannequin bust. This takes a second or two of
    /// every core.
    pub fn grow(preset: Preset) -> Self {
        let look = Look::new(preset);
        let head = look.head(&Head::mannequin());
        let scalp = head.scalp(SCALP_CELL);
        let strands = look.grow(&head, &scalp);
        let settings = LodSettings::default();
        let occlusion = Occlusion::bake(&strands, Some(&head), &settings.occlusion);
        let cards = CardMesh::build(
            &strands,
            head.cranium.center,
            Some(&occlusion),
            Some(&head),
            &CardSettings::default(),
            settings.atlas_variants,
        );
        let [width, height] = settings.atlas_size;
        let atlas = CardAtlas::paint(settings.atlas_variants, width, height, look.seed as u64);
        Self {
            preset,
            cards,
            atlas,
            head,
        }
    }

    /// The bust this hair grew on.
    pub fn head(&self) -> &Head {
        &self.head
    }

    /// The cards carried onto a character's head.
    pub fn place(&self, placement: &HeadPlacement) -> HairCards {
        let variants = self.atlas.variants.max(1) as f32;
        let vertices = &self.cards.vertices;
        HairCards {
            positions: vertices
                .iter()
                .map(|vertex| placement.point(vertex.position))
                .collect(),
            normals: vertices
                .iter()
                .map(|vertex| placement.normal(vertex.normal))
                .collect(),
            texcoords: vertices
                .iter()
                .map(|vertex| [(vertex.variant + vertex.u) / variants, vertex.v])
                .collect(),
            occlusion: vertices.iter().map(|vertex| vertex.occlusion).collect(),
            faces: self.cards.indices.as_chunks::<3>().0.to_vec(),
        }
    }

    /// The atlas's mip chain, largest first.
    pub fn atlas_levels(&self) -> Vec<AtlasLevel> {
        // The atlas's width already spans every variant.
        let mut width = self.atlas.width;
        let mut height = self.atlas.height;
        self.atlas
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
