//! Data model for the standalone MHR character creator.

use adventuresim_core::character_morph::IDENTITY_MORPH_COUNT;
pub use adventuresim_core::item_catalog_schema;
pub use armor_gpu::{FittingSlot, armor_gpu, armor_gpu_async, fitting_slot};
pub use clothing_material::pbr as equipment_pbr;
pub use metal_gpu::metal_gpu;
use serde::{Deserialize, Serialize};

pub mod armor_design_input;
pub mod armor_design_output;
pub mod armor_frames;
mod armor_gpu;
pub mod armor_layer;
pub mod armor_metal;
pub mod armor_recipes;
mod bodice_cut;
mod body_cloth_plan;
pub mod bracer;
mod cloth_closure_fit;
mod cloth_fastener_mesh;
mod cloth_opening;
mod cloth_patch_plan;
mod cloth_ribbon_plan;
mod cloth_tie_plan;
mod cloth_torso_frame;
pub mod clothing;
mod clothing_material;
mod collar_closures;
mod collar_oval;
mod collar_rows;
pub mod decoration;
pub mod design_input;
pub mod device_body;
mod device_body_cloth;
mod device_boot_layers;
pub mod device_bracer;
mod device_chausses;
pub mod device_clearance;
pub mod device_close_helmet;
mod device_cloth_collar;
mod device_cloth_patch;
mod device_cloth_ribbon;
mod device_cloth_ties;
pub mod device_coif;
mod device_collar_seating;
pub mod device_extremities;
pub mod device_fit;
pub mod device_foot_frame;
mod device_foot_sections;
mod device_footwear_fit;
pub mod device_frames;
pub mod device_garment;
mod device_garment_kernel;
mod device_garment_skirt;
mod device_garment_tassets;
mod device_garment_torso;
mod device_garment_tube;
mod device_gorget;
mod device_gorget_bib;
mod device_gorget_cage;
mod device_gorget_collar;
mod device_gorget_projection;
mod device_gorget_sections;
mod device_gown_enclosure;
pub mod device_head_frame;
pub mod device_helmet;
pub mod device_limb;
mod device_partlet;
mod device_pauldron;
pub mod device_piece;
mod device_piece_components;
mod device_piece_skin;
mod device_puff;
mod device_recipe_plan;
mod device_sabaton;
mod device_sewn_sleeves;
mod device_skirt;
mod device_skirt_join;
mod device_textile_materials;
mod device_textile_relief;
mod device_textile_support;
pub mod device_torso;
mod device_torso_wgsl;
pub mod device_underlayer;
mod device_wrapped_tasset;
pub mod equipment_catalog;
pub mod equipment_layers;
#[cfg(feature = "offline-creator")]
pub mod export;
pub mod garment;
pub mod garment_material;
mod gown_bindings;
mod gown_charts;
mod gown_cuffs;
mod gown_ease;
mod gown_plan;
pub mod hair;
pub mod inventory;
pub mod item_design;
pub mod library_name;
#[cfg(feature = "offline-creator")]
pub mod lod;
mod metal_gpu;
pub mod nearest_vertex;
mod partlet_collar;
mod partlet_cut;
mod partlet_plan;
pub mod profiling;
#[cfg(feature = "offline-creator")]
pub mod proportions;
mod rig_parameters;
pub mod runtime_equipment;
mod sewn_sleeve_opening;
mod sewn_sleeve_pattern;
mod sewn_sleeves;
mod sewn_torso_boundaries;
pub mod skin_rules;
pub mod skin_transfer;
mod skirt_clearance;
mod skirt_join;
pub mod studio_environment;
pub mod surface_cut;
pub mod textile_surface;
pub mod torso_domain;
pub mod underlayer;
pub mod underlayer_armor;
#[cfg(feature = "offline-creator")]
pub mod underlayer_material;
pub mod wardrobe;

pub const EXPRESSION_COUNT: usize = 72;

/// Character recipes use this schema version; older recipes are not read.
pub const RECIPE_VERSION: u8 = 14;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct CharacterRecipe {
    pub version: u8,
    pub name: String,
    pub proportions: adventuresim_core::character_proportions::CharacterProportions,
    pub identity: Vec<f32>,
    pub expression: Vec<f32>,
    /// The hairstyle grown on the head; `None` is bald.
    pub hair: Option<hair::HairDesign>,
    pub inventory: inventory::Inventory,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IdentityGroup {
    Body,
    Head,
    Hands,
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
            hair: None,
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
    fn plate_construction_round_trips_in_recipe() {
        let mut recipe = CharacterRecipe::default();
        let mut tiling = fabelgeist_armor::Tiling::scale();
        tiling.plate.roundness = 0.9;
        let mut article = inventory::CatalogArticle::new("vambrace", "left");
        article.construction = fabelgeist_armor::Construction::Scale(tiling);
        recipe.inventory.add(inventory::Article::Catalog(article));
        let parsed: CharacterRecipe =
            serde_json::from_slice(&serde_json::to_vec(&recipe).unwrap()).unwrap();
        assert_eq!(recipe, parsed);
        assert!(parsed.validate().is_ok());
    }
}
