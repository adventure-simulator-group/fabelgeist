//! Sourced pigment/binder choices and explicitly estimated display responses.
pub mod catalog;
pub mod mixing;
use crate::{
    artwork::PaintTone,
    document::{Ratio, Tincture},
};
pub use catalog::{Binder, PaintRecipe, RecipeEvidence};
use serde::{Deserialize, Serialize};
use std::ops::{Index, IndexMut};

/// Unlit sRGB swatches. These are not measured pigment reflectance spectra.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PaintColors {
    pub base: [u8; 3],
    pub shadow: [u8; 3],
    pub highlight: [u8; 3],
}
impl PaintColors {
    pub fn color(self, tone: PaintTone) -> [u8; 3] {
        match tone {
            PaintTone::Base => self.base,
            PaintTone::Shadow => self.shadow,
            PaintTone::Highlight => self.highlight,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PaintAppearance {
    pub colors: PaintColors,
    pub roughness: Ratio,
}
/// Sourced catalog appearance or recipes in a bounded measured paint family.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum Paint {
    Recipe { recipe: PaintRecipe },
    Mixed { paint: mixing::MixedPaint },
}
impl Paint {
    pub fn appearance(self) -> PaintAppearance {
        match self {
            Self::Recipe { recipe } => recipe.definition().appearance,
            Self::Mixed { paint } => paint.appearance(),
        }
    }
    pub fn color(self, tone: PaintTone) -> [u8; 3] {
        self.appearance().colors.color(tone)
    }
}

/// One physical paint choice per heraldic tincture, indexed by tincture.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PaintPalette(pub [Paint; 7]);
impl Default for PaintPalette {
    fn default() -> Self {
        Self(Tincture::ALL.map(|t| Paint::Recipe {
            recipe: PaintRecipe::for_tincture(t),
        }))
    }
}
impl Index<Tincture> for PaintPalette {
    type Output = Paint;
    fn index(&self, t: Tincture) -> &Self::Output {
        &self.0[t.index()]
    }
}
impl IndexMut<Tincture> for PaintPalette {
    fn index_mut(&mut self, t: Tincture) -> &mut Self::Output {
        &mut self.0[t.index()]
    }
}
