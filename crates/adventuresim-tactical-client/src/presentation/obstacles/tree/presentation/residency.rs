//! Retain terrain bindings and share matching plant assets across scenes.
use super::*;
use std::collections::VecDeque;

// Active scene plus two previous scenes bounds retained geometry and atlases.
const PREVIOUS_TREE_SCENES: usize = 2;

#[derive(Resource, Default)]
pub(in crate::presentation) struct RetainedTreePresentations {
    pub(super) scenes: VecDeque<RetainedScene>,
}

pub(super) struct RetainedScene {
    pub(super) cache: ActiveTreeCache,
    pub(super) diagnostics: RecipeTreeAssetResidencyDiagnostics,
}

impl RecipeTreeCache {
    pub(super) fn bark_for_terrain(
        &mut self,
        species: TreePresentationSpecies,
        procedural_assets: &ProceduralTextureAssets,
        terrain_heightmap: Handle<Image>,
        terrain: &SceneTerrain,
        bark_materials: &mut Assets<TacticalTreeBarkMaterial>,
    ) -> Handle<TacticalTreeBarkMaterial> {
        match species {
            TreePresentationSpecies::EnglishOak => {
                self.oak_bark_material.clone().unwrap_or_else(|| {
                    let material = bark_materials.add(oak_bark_material(
                        procedural_assets,
                        terrain_heightmap.clone(),
                        self.terrain_height_range,
                        terrain,
                    ));
                    self.oak_bark_material = Some(material.clone());
                    material
                })
            }
            TreePresentationSpecies::CommonBeech => {
                self.beech_bark_material.clone().unwrap_or_else(|| {
                    let material = bark_materials.add(beech_bark_material(
                        procedural_assets,
                        terrain_heightmap.clone(),
                        self.terrain_height_range,
                        terrain,
                    ));
                    self.beech_bark_material = Some(material.clone());
                    material
                })
            }
        }
    }
}

impl RetainedTreePresentations {
    pub(super) fn replace_active(
        &mut self,
        cache: &mut TreePresentationCache,
        incoming: RetainedScene,
        diagnostics: &mut RecipeTreeAssetResidencyDiagnostics,
    ) {
        if let Some(active) = cache.active.take() {
            self.scenes.push_back(RetainedScene {
                cache: active,
                diagnostics: std::mem::take(diagnostics),
            });
        }
        while self.scenes.len() > PREVIOUS_TREE_SCENES {
            self.scenes.pop_front();
        }
        cache.active = Some(incoming.cache);
        *diagnostics = incoming.diagnostics;
    }

    pub(super) fn activate_recipe(
        &mut self,
        scene: Entity,
        environment: &SceneEnvironment,
        terrain: &SceneTerrain,
        cache: &mut TreePresentationCache,
        diagnostics: &mut RecipeTreeAssetResidencyDiagnostics,
        images: &mut Assets<Image>,
    ) -> anyhow::Result<()> {
        if cache.active.as_ref().is_some_and(|active| {
            active.binding.scene == scene && active.binding.environment == *environment
        }) {
            cache.recipe_for_scene(scene, environment)?;
            return Ok(());
        }
        let restored = self
            .scenes
            .iter()
            .position(|retained| {
                retained.cache.binding.environment == *environment
                    && matches!(retained.cache.contents, TreeCacheContents::Recipe(_))
            })
            .and_then(|index| self.scenes.remove(index));
        let incoming = if let Some(mut restored) = restored {
            restored.cache.binding.scene = scene;
            restored
        } else {
            let (heightmap, height_range) =
                crate::presentation::terrain::terrain_contact_heightmap_image(terrain);
            RetainedScene {
                cache: ActiveTreeCache {
                    binding: TreeSceneBinding {
                        scene,
                        environment: environment.clone(),
                    },
                    contents: TreeCacheContents::Recipe(RecipeTreeCache {
                        variants: Default::default(),
                        oak_bark_material: None,
                        beech_bark_material: None,
                        oak_aggregate_bark_material: None,
                        beech_aggregate_bark_material: None,
                        terrain_heightmap: images.add(heightmap),
                        terrain_height_range: height_range,
                    }),
                },
                diagnostics: RecipeTreeAssetResidencyDiagnostics::default(),
            }
        };
        self.replace_active(cache, incoming, diagnostics);
        cache.recipe_for_scene(scene, environment)?;
        self.update_diagnostics(cache, diagnostics);
        Ok(())
    }

    pub(super) fn recipe_variant(&self, key: Seed) -> Option<CachedTreePresentation> {
        self.scenes.iter().rev().find_map(|scene| {
            let recipe = scene.cache.contents.as_recipe()?;
            recipe
                .variants
                .get(&key)
                .filter(|variant| matches!(variant.origin, TreeAssetOrigin::Generated))
                .cloned()
        })
    }

    pub(super) fn update_diagnostics(
        &self,
        active: &TreePresentationCache,
        diagnostics: &mut RecipeTreeAssetResidencyDiagnostics,
    ) {
        diagnostics.retained_scenes = self.scenes.len();
        let active_recipe = active
            .active
            .as_ref()
            .and_then(|cache| cache.contents.as_recipe());
        let mut variants = active_recipe
            .into_iter()
            .flat_map(|recipe| recipe.variants.keys().copied())
            .collect::<std::collections::HashSet<_>>();
        let mut atlases = active_recipe
            .into_iter()
            .flat_map(|recipe| recipe.variants.values())
            .flat_map(|variant| {
                variant
                    .lod_cards
                    .iter()
                    .flatten()
                    .map(|card| card.material.id())
            })
            .collect::<std::collections::HashSet<_>>();
        diagnostics.retained_variants = 0;
        diagnostics.retained_impostor_texture_bytes = 0;
        for scene in &self.scenes {
            let Some(recipe) = scene.cache.contents.as_recipe() else {
                continue;
            };
            for (key, variant) in &recipe.variants {
                diagnostics.retained_variants += usize::from(variants.insert(*key));
                for card in variant.lod_cards.iter().flatten() {
                    if atlases.insert(card.material.id()) {
                        diagnostics.retained_impostor_texture_bytes += card.provenance.atlas_width
                            as usize
                            * card.provenance.atlas_height as usize
                            * 4;
                    }
                }
            }
        }
    }
}
