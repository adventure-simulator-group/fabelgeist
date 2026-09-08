use super::{LeafShape, relief::LeafRelief};
use crate::LeafSpecies;
use serde::{Deserialize, Serialize};
#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LeafParameters {
    pub shape: LeafShape,
    pub relief: LeafRelief,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LeafPresets {
    pub white_oak: LeafParameters,
    pub dry_white_oak: LeafParameters,
    pub hazel: LeafParameters,
    pub blackthorn: LeafParameters,
    pub hawthorn: LeafParameters,
    pub beech: LeafParameters,
}
impl Default for LeafPresets {
    fn default() -> Self {
        let preset = |name| LeafParameters {
            shape: LeafShape::preset(name).expect("species preset"),
            relief: LeafRelief::default(),
        };
        let mut dry = preset("white-oak");
        dry.relief.curl = 0.12;
        dry.relief.corrugation = 0.025;
        Self {
            white_oak: preset("white-oak"),
            dry_white_oak: dry,
            hazel: preset("hazel"),
            blackthorn: preset("blackthorn"),
            hawthorn: preset("hawthorn"),
            beech: preset("beech"),
        }
    }
}
impl LeafPresets {
    pub fn get(&self, species: LeafSpecies) -> &LeafParameters {
        match species {
            LeafSpecies::WhiteOak => &self.white_oak,
            LeafSpecies::DryWhiteOak => &self.dry_white_oak,
            LeafSpecies::Hazel => &self.hazel,
            LeafSpecies::Blackthorn => &self.blackthorn,
            LeafSpecies::Hawthorn => &self.hawthorn,
            LeafSpecies::Beech => &self.beech,
        }
    }
}
impl crate::TextureRecipeId {
    pub fn leaf_species(self) -> Option<LeafSpecies> {
        use crate::TextureRecipeId::*;
        match self {
            WhiteOakLeaf => Some(LeafSpecies::WhiteOak),
            DryWhiteOakLeaf => Some(LeafSpecies::DryWhiteOak),
            HazelLeaf => Some(LeafSpecies::Hazel),
            BlackthornLeaf => Some(LeafSpecies::Blackthorn),
            HawthornLeaf => Some(LeafSpecies::Hawthorn),
            BeechLeaf => Some(LeafSpecies::Beech),
            _ => None,
        }
    }
}
impl LeafSpecies {
    pub fn parameter_key(self) -> &'static str {
        match self {
            Self::WhiteOak => "white_oak",
            Self::DryWhiteOak => "dry_white_oak",
            Self::Hazel => "hazel",
            Self::Blackthorn => "blackthorn",
            Self::Hawthorn => "hawthorn",
            Self::Beech => "beech",
        }
    }
}
