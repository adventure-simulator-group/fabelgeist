//! Data model for the standalone MHR character creator.

mod armor_clearance;
pub mod armor_design_input;
pub mod armor_design_output;
pub mod armor_frames;
pub mod armor_recipes;
pub mod bracer;
pub mod breastplate;
mod close_helmet_fit;
pub mod clothing;
mod clothing_material;
mod coif_fit;
pub mod garment_fit;
mod gorget_fit;
pub mod limb_fit;
pub use clothing_material::pbr as equipment_pbr;
pub mod design_input;
pub mod equipment_catalog;
pub mod export;
pub mod inventory;
pub mod item_design;
pub mod proportions;
pub mod surface_cut;
pub mod underlayer;
pub use adventuresim_core::item_catalog_schema;
pub mod garment;
pub mod garment_material;

use serde::{Deserialize, Serialize};

use adventuresim_core::character_morph::IDENTITY_MORPH_COUNT;
pub const EXPRESSION_COUNT: usize = 72;

/// Character recipes use this schema version; older recipes are not read.
pub const RECIPE_VERSION: u8 = 7;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct CharacterRecipe {
    pub version: u8,
    pub name: String,
    pub proportions: adventuresim_core::character_proportions::CharacterProportions,
    pub identity: Vec<f32>,
    pub expression: Vec<f32>,
    pub inventory: inventory::Inventory,
}

impl Default for CharacterRecipe {
    fn default() -> Self {
        use inventory::{Article, CatalogArticle};
        let mut inventory = inventory::Inventory::default();
        for (item, placement) in [
            ("linen_tunic", "worn"),
            ("linen_breeches", "worn"),
            ("leather_boot", "left"),
            ("leather_boot", "right"),
        ] {
            let id = inventory.add(Article::Catalog(CatalogArticle::new(item, placement)));
            inventory.get_mut(id).expect("just added").worn = true;
        }
        Self {
            version: RECIPE_VERSION,
            proportions: Default::default(),
            name: "New adventurer".into(),
            identity: vec![0.0; IDENTITY_MORPH_COUNT],
            expression: vec![0.0; EXPRESSION_COUNT],
            inventory,
        }
    }
}

impl CharacterRecipe {
    pub fn validate(&self) -> Result<(), String> {
        if self.version != RECIPE_VERSION {
            return Err(format!(
                "unsupported character recipe version {}",
                self.version
            ));
        }
        if self.identity.len() != IDENTITY_MORPH_COUNT || self.expression.len() != EXPRESSION_COUNT
        {
            return Err("recipe has the wrong MHR coefficient counts".into());
        }
        if self.name.trim().is_empty() {
            return Err("character name cannot be empty".into());
        }
        if self
            .identity
            .iter()
            .chain(&self.expression)
            .any(|value| !value.is_finite())
        {
            return Err("recipe contains a non-finite coefficient".into());
        }
        self.inventory.validate()
    }

    pub fn reset_body(&mut self) {
        self.identity.fill(0.0);
        self.proportions = Default::default();
    }
    pub fn reset_face(&mut self) {
        self.expression.fill(0.0);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IdentityGroup {
    Body,
    Head,
    Hands,
}

impl IdentityGroup {
    pub const ALL: [Self; 3] = [Self::Body, Self::Head, Self::Hands];
    pub fn label(self) -> &'static str {
        match self {
            Self::Body => "Body",
            Self::Head => "Head",
            Self::Hands => "Hands",
        }
    }
    pub fn range(self) -> std::ops::Range<usize> {
        match self {
            Self::Body => 0..20,
            Self::Head => 20..40,
            Self::Hands => 40..45,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_recipe_matches_mhr_layout() {
        let recipe = CharacterRecipe::default();
        assert!(recipe.validate().is_ok());
        assert_eq!(
            IdentityGroup::ALL
                .iter()
                .map(|g| g.range().len())
                .sum::<usize>(),
            45
        );
    }

    #[test]
    fn validation_rejects_corrupt_recipe() {
        let mut recipe = CharacterRecipe::default();
        recipe.identity.pop();
        assert!(recipe.validate().is_err());
    }

    #[test]
    fn canonical_mhr_base_has_zero_coefficients_and_carries_nothing() {
        let recipe: CharacterRecipe =
            serde_json::from_str(include_str!("../../../assets_src/characters/mhr_base.json"))
                .unwrap();
        assert!(recipe.validate().is_ok());
        assert!(recipe.identity.iter().all(|value| *value == 0.0));
        assert!(recipe.expression.iter().all(|value| *value == 0.0));
        assert!(recipe.inventory.items().is_empty());
    }

    #[test]
    fn armor_parameters_round_trip_in_recipe() {
        let mut recipe = CharacterRecipe::default();
        let mut armor = fabelgeist_armor::Armor::default();
        armor.construction = fabelgeist_armor::Construction::Scale;
        armor.plate.roundness = 0.9;
        armor.metal.seed = 42;
        recipe.inventory.add(inventory::Article::Plate(armor));
        let parsed: CharacterRecipe =
            serde_json::from_slice(&serde_json::to_vec(&recipe).unwrap()).unwrap();
        assert_eq!(recipe, parsed);
        assert!(parsed.validate().is_ok());
    }
}

pub mod underlayer_material;
