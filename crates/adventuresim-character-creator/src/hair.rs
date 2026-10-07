//! The hair a character wears, and the forms it is drawn in.
//!
//! `fabelgeist-hair` grows a hairstyle as strands on its mannequin bust
//! ([`GroomedHair`]). A [`HeadPlacement`] carries the strands onto the
//! character's own head, stretched to its width, depth and crown and facing
//! the way its eyes do; there the hair's levels of detail are prepared
//! against the character's body ([`PlacedHair`]), so no strand or card sinks
//! into a scalp the bust did not have. Each level is drawn as plain triangles
//! ([`HairGeometry`]): strands as ribbons, cards as textured strips.

mod geometry;
mod groom;
mod placement;

pub use geometry::{AtlasLevel, HairGeometry};
pub use groom::{BodySurface, GroomedHair, PlacedHair, SkinCover};
pub use placement::{HeadPlacement, HeadPlacementError, HeadSkin};

use fabelgeist_hair::{HairMaterial, Preset};
use serde::{Deserialize, Serialize};

/// Hair's perceptual roughness when drawn as cards: a rough dielectric whose
/// sheen comes from the strands painted in the atlas.
pub const HAIR_ROUGHNESS: f32 = 0.55;

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
