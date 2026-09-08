//! Typed artistic recipe controls shared by native bakers and the browser studio.
use serde::{Deserialize, Serialize};
mod inspector;
mod validation;
pub use inspector::ControlPath;
#[cfg(test)]
mod tests;
pub use validation::{ControlBounds, ParameterError};

macro_rules! parameter_block {
    (pub struct $name:ident { $($field:ident: $ty:ty = $default:expr;)* }) => {
        #[derive(Clone, serde::Serialize, serde::Deserialize)]
        #[serde(deny_unknown_fields)]
        pub struct $name { $(pub $field: $ty,)* }
        impl Default for $name {
            fn default() -> Self { Self { $($field: $default,)* } }
        }
    };
}
pub(crate) use parameter_block;

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TextureParameters {
    pub seed: u64,
    pub dry_white_oak_leaf: crate::dry_white_oak_leaf::Parameters,
    pub leaf_colors: LeafColorParameters,
    pub resolution: BakeResolution,
    pub beech_leaf: crate::beech_leaf::Parameters,
    pub blackthorn_leaf: crate::blackthorn_leaf::Parameters,
    pub clay_roof_tile: crate::clay_roof_tile::Parameters,
    pub crenellation_mask: crate::crenellation_mask::Parameters,
    pub dressed_stone: crate::dressed_stone::Parameters,
    pub foliage: crate::foliage::Parameters,
    pub ground: crate::ground::Parameters,
    pub handmade_brick: crate::handmade_brick::Parameters,
    pub hawthorn_leaf: crate::hawthorn_leaf::Parameters,
    pub hazel_leaf: crate::hazel_leaf::Parameters,
    pub hewn_oak: crate::hewn_oak::Parameters,
    pub ironwork: crate::ironwork::Parameters,
    pub lead_sheet: crate::lead_sheet::Parameters,
    pub common: crate::Parameters,
    pub lime_plaster: crate::lime_plaster::Parameters,
    pub plank_floor: crate::plank_floor::Parameters,
    pub rock: crate::rock::Parameters,
    pub rubble_masonry: crate::rubble_masonry::Parameters,
    pub slate_roof: crate::slate_roof::Parameters,
    pub surface: crate::surface::Parameters,
    pub timber_shingle: crate::timber_shingle::Parameters,
    pub wattle_and_daub: crate::wattle_and_daub::Parameters,
    pub window_glass: crate::window_glass::Parameters,
    pub dressed_stone_weathering: crate::dressed_stone::weathering::Parameters,
    pub hewn_oak_grain: crate::hewn_oak::grain::Parameters,
    pub dressed_stone_weathering_mortar: crate::dressed_stone::weathering::mortar::Parameters,
}

/// A cap changes sampling density, never the physical size or authored feature count.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum BakeResolution {
    Draft,
    Medium,
    #[default]
    Full,
}
impl TextureParameters {
    pub(crate) fn size(&self, native: u32) -> u32 {
        match self.resolution {
            BakeResolution::Draft => native.min(128),
            BakeResolution::Medium => native.min(256),
            BakeResolution::Full => native,
        }
    }
}
pub(crate) fn seeded_hash(params: &TextureParameters, value: u64) -> u64 {
    fabelgeist_determinism::splitmix64(value ^ params.seed)
}

#[derive(Clone, Copy, serde::Serialize, serde::Deserialize)]
pub enum LeafSpecies {
    WhiteOak,
    DryWhiteOak,
    Hazel,
    Blackthorn,
    Hawthorn,
    Beech,
}

parameter_block! {
    pub struct LeafColorParameters {
        white_oak: crate::LeafRecipe = crate::LeafRecipe::WHITE_OAK;
        dry_white_oak: crate::LeafRecipe = crate::LeafRecipe::DRY_WHITE_OAK;
        hazel: crate::LeafRecipe = crate::LeafRecipe::HAZEL;
        blackthorn: crate::LeafRecipe = crate::LeafRecipe::BLACKTHORN;
        hawthorn: crate::LeafRecipe = crate::LeafRecipe::HAWTHORN;
        beech: crate::LeafRecipe = crate::LeafRecipe::BEECH;
    }
}
