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
        let mut oak = preset("white-oak");
        oak.shape.lobe_frequency = 5.0;
        oak.shape.secondary_count = 5.0;
        oak.shape.lobe_roundness = 0.7;
        oak.shape.tip_exponent = 0.55;
        oak.shape.secondary_width = 0.004;
        oak.shape.midrib_width = 0.007;
        oak.shape.lobe_stagger = 0.08;
        let mut dry = oak.clone();
        dry.relief.curl = 0.24;
        dry.relief.corrugation = 0.10;
        dry.relief.corrugation_frequency = 5.0;
        dry.relief.dome = 0.06;
        dry.relief.normal_strength = 0.18;
        dry.relief.vein_height = 0.006;
        dry.relief.back_vein_height = 0.01;
        dry.relief.tissue_height = 0.0001;
        dry.shape.bend = 0.07;
        let mut hazel = preset("hazel");
        hazel.shape.base_width = 0.0;
        hazel.shape.base_notch_depth = 0.08;
        hazel.shape.base_notch_width = 0.055;
        hazel.shape.base_exponent = 0.4;
        hazel.shape.widest_at = 0.35;
        hazel.shape.tip_exponent = 0.95;
        hazel.shape.secondary_width = 0.0035;
        hazel.shape.midrib_width = 0.007;
        let mut blackthorn = preset("blackthorn");
        blackthorn.shape.secondary_width = 0.003;
        blackthorn.shape.midrib_width = 0.006;
        let mut hawthorn = preset("hawthorn");
        hawthorn.shape.lobe_frequency = 3.0;
        hawthorn.shape.secondary_count = 3.0;
        hawthorn.shape.lobe_roundness = 0.8;
        hawthorn.shape.lobe_depth = 0.48;
        hawthorn.shape.secondary_width = 0.004;
        hawthorn.shape.midrib_width = 0.007;
        hawthorn.shape.secondary_first = 0.06;
        hawthorn.shape.secondary_last = 0.75;
        hawthorn.shape.secondary_sweep = 0.1165;
        hawthorn.shape.lobe_progressive_sweep = 0.1739;
        hawthorn.shape.vein_alternation = 0.0;
        hawthorn.shape.vein_branching = 0.0;
        hawthorn.shape.secondary_reach = 1.0;
        hawthorn.shape.lobe_basal_scale = 0.85;
        hawthorn.shape.lobe_apical_scale = 0.4;
        let mut beech = preset("beech");
        beech.shape.secondary_count = 8.0;
        beech.shape.secondary_width = 0.003;
        beech.shape.midrib_width = 0.006;
        Self {
            white_oak: oak,
            dry_white_oak: dry,
            hazel,
            blackthorn,
            hawthorn,
            beech,
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
