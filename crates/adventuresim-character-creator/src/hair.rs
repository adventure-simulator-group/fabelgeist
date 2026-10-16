//! The hair a character wears, and the forms it is drawn in.
//!
//! `fabelgeist-hair` grows a hairstyle as strands on its mannequin bust
//! ([`GroomedHair`]). A [`HeadPlacement`] carries the strands onto the
//! character's own head, stretched to its width, depth and crown and facing
//! the way its eyes do; there the hair's levels of detail are prepared
//! against the character's body ([`PlacedHair`]), so no strand or card sinks
//! into a scalp the bust did not have. Each level is drawn as plain triangles
//! ([`HairGeometry`]): strands as ribbons, cards as textured strips.

mod cap;
mod geometry;
mod gpu;
mod groom;
mod placement;
mod scalp_map;
mod shape;

pub use cap::ScalpCap;
pub use geometry::{AtlasLevel, HairGeometry};
pub use gpu::HairGpu;
pub use gpu::ScalpTexture;
pub use groom::{BodySurface, GroomedHair, PlacedHair};
pub use placement::{HeadPlacement, HeadPlacementError, HeadSkin};
pub use shape::{
    CLUMPING, CURL, FRIZZ, Groom, HairShape, LENGTH, PART, ShapeKnobs, ShapeRangeError, VOLUME,
    WAVE,
};

use fabelgeist_hair::{HairMaterial, Preset};
use serde::{Deserialize, Serialize};

/// Hair's perceptual roughness when drawn as cards: a rough dielectric whose
/// sheen comes from the strands painted in the atlas.
pub const HAIR_ROUGHNESS: f32 = 0.55;

/// Hair pigment as Blender's Principled Hair takes it. Both shares run from 0
/// to 1: melanin from white through blond and brown to black, redness from
/// ashen to red. A dye tints the strands on top, linear RGB, each channel 0
/// to 1 of the light it lets through: white is no dye. A dye shows best on
/// light hair, as on bleached hair.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, try_from = "PigmentShares")]
pub struct HairPigment {
    melanin: f32,
    redness: f32,
    dye: [f32; 3],
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PigmentShares {
    melanin: f32,
    redness: f32,
    dye: [f32; 3],
}

/// A pigment share or a dye channel outside 0 to 1.
#[derive(Debug, thiserror::Error)]
#[error("hair pigment shares and dye channels must lie between 0 and 1")]
pub struct PigmentRangeError;

impl TryFrom<PigmentShares> for HairPigment {
    type Error = PigmentRangeError;

    fn try_from(shares: PigmentShares) -> Result<Self, Self::Error> {
        let share = 0.0..=1.0;
        let within = share.contains(&shares.melanin)
            && share.contains(&shares.redness)
            && shares.dye.iter().all(|channel| share.contains(channel));
        if !within {
            return Err(PigmentRangeError);
        }
        Ok(Self {
            melanin: shares.melanin,
            redness: shares.redness,
            dye: shares.dye,
        })
    }
}

impl Default for HairPigment {
    /// A mid brown, undyed.
    fn default() -> Self {
        let material = HairMaterial::default();
        Self {
            melanin: material.melanin,
            redness: material.redness,
            dye: NO_DYE,
        }
    }
}

/// The dye that leaves hair its own colour.
pub const NO_DYE: [f32; 3] = [1.0; 3];

impl HairPigment {
    /// An undyed pigment with both shares clamped to 0 to 1.
    pub fn new(melanin: f32, redness: f32) -> Self {
        Self {
            melanin: melanin.clamp(0.0, 1.0),
            redness: redness.clamp(0.0, 1.0),
            dye: NO_DYE,
        }
    }

    /// The hair crate's named natural colour, as "auburn" or "platinum".
    pub fn named(name: &str) -> Option<Self> {
        HairMaterial::named(name).map(|material| Self::new(material.melanin, material.redness))
    }

    /// This pigment dyed `dye`, each channel clamped to 0 to 1.
    pub fn dyed(self, dye: [f32; 3]) -> Self {
        Self {
            dye: dye.map(|channel| channel.clamp(0.0, 1.0)),
            ..self
        }
    }

    pub fn melanin(self) -> f32 {
        self.melanin
    }

    pub fn redness(self) -> f32 {
        self.redness
    }

    pub fn dye(self) -> [f32; 3] {
        self.dye
    }

    /// The hair's real-time colour, linear RGB.
    pub fn color(self) -> [f32; 3] {
        HairMaterial {
            melanin: self.melanin,
            redness: self.redness,
            tint: self.dye,
            ..HairMaterial::default()
        }
        .base_color()
    }
}

/// A hairstyle, how it is varied from its preset, and its colour.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HairDesign {
    pub preset: Preset,
    pub shape: HairShape,
    pub pigment: HairPigment,
}

impl HairDesign {
    /// What grows: the colour aside.
    pub fn groom(&self) -> Groom {
        Groom {
            preset: self.preset,
            shape: self.shape,
        }
    }
}

/// A hairstyle's, a cut's, an arrangement's or a growth pattern's name as
/// the interface shows it: `SideSwept` as "Side swept".
pub fn variant_label(variant: impl std::fmt::Debug) -> String {
    let name = format!("{variant:?}");
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
