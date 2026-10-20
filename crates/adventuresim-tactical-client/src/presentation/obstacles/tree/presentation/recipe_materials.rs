//! Recipe surface construction and its independently borrowed Bevy asset stores.

use super::*;
use bevy::ecs::system::SystemParam;

#[derive(SystemParam)]
pub(in crate::presentation) struct RecipeMaterials<'w> {
    pub(super) bark: ResMut<'w, Assets<TacticalTreeBarkMaterial>>,
    aggregate: ResMut<'w, Assets<TacticalTreeAggregateBarkMaterial>>,
    leaves: ResMut<'w, Assets<TacticalTreeLeafCardMaterial>>,
    buds: ResMut<'w, Assets<StandardMaterial>>,
    pub(super) textures: Res<'w, ProceduralTextureAssets>,
}

impl RecipeMaterials<'_> {
    pub(super) fn bind(
        &mut self,
        cache: &mut RecipeTreeCache,
        species: TreePresentationSpecies,
        terrain: &SceneTerrain,
    ) -> assets::TreeMaterialBindings {
        let bark_material = cache.bark_for_terrain(
            species,
            &self.textures,
            cache.terrain_heightmap.clone(),
            terrain,
            &mut self.bark,
        );
        let aggregate_bark_material = match species {
            TreePresentationSpecies::EnglishOak => cache
                .oak_aggregate_bark_material
                .clone()
                .unwrap_or_else(|| {
                    let material = self.aggregate.add(oak_aggregate_bark_material());
                    cache.oak_aggregate_bark_material = Some(material.clone());
                    material
                }),
            TreePresentationSpecies::CommonBeech => cache
                .beech_aggregate_bark_material
                .clone()
                .unwrap_or_else(|| {
                    let material = self.aggregate.add(beech_aggregate_bark_material());
                    cache.beech_aggregate_bark_material = Some(material.clone());
                    material
                }),
        };
        let leaf_material = self.leaves.add(match species {
            TreePresentationSpecies::EnglishOak => oak_leaf_material(&self.textures),
            TreePresentationSpecies::CommonBeech => beech_leaf_material(&self.textures),
        });
        let bud_material = self
            .buds
            .add(super::super::materials::tree_bud_material(species));
        assets::TreeMaterialBindings {
            bark: bark_material,
            aggregate_bark: aggregate_bark_material,
            leaf: leaf_material,
            bud: bud_material,
        }
    }
}
