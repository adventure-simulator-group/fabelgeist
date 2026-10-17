//! Retain terrain bindings and share matching plant assets across scenes.
use super::*;
use std::collections::VecDeque;

// Active scene plus two previous scenes bounds retained geometry and atlases.
const PREVIOUS_TREE_SCENES: usize = 2;

#[derive(Resource, Default)]
pub(in crate::presentation) struct RetainedTreePresentations {
    scenes: VecDeque<RetainedScene>,
}

struct RetainedScene {
    cache: TreePresentationCache,
    diagnostics: TreeAssetResidencyDiagnostics,
}

impl TreePresentationCache {
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
    pub(super) fn activate(
        &mut self,
        environment: &SceneEnvironment,
        terrain: &SceneTerrain,
        cache: &mut TreePresentationCache,
        diagnostics: &mut TreeAssetResidencyDiagnostics,
        images: &mut Assets<Image>,
    ) {
        if cache.terrain_scene_digest.as_deref() == Some(&environment.scene_digest) {
            return;
        }
        let restored = self
            .scenes
            .iter()
            .position(|scene| {
                scene.cache.terrain_scene_digest.as_deref() == Some(&environment.scene_digest)
            })
            .and_then(|index| self.scenes.remove(index));
        if cache.terrain_scene_digest.is_some() {
            self.scenes.push_back(RetainedScene {
                cache: std::mem::take(cache),
                diagnostics: std::mem::take(diagnostics),
            });
        }
        while self.scenes.len() > PREVIOUS_TREE_SCENES {
            self.scenes.pop_front();
        }
        if let Some(restored) = restored {
            *cache = restored.cache;
            *diagnostics = restored.diagnostics;
        } else {
            let (heightmap, height_range) =
                crate::presentation::terrain::terrain_contact_heightmap_image(terrain);
            cache.terrain_heightmap = Some(images.add(heightmap));
            cache.terrain_height_range = height_range;
            cache.terrain_scene_digest = Some(environment.scene_digest.clone());
        }
        self.update_diagnostics(cache, diagnostics);
    }

    pub(super) fn variant(&self, key: Seed) -> Option<CachedTreePresentation> {
        self.scenes
            .iter()
            .rev()
            .find_map(|scene| scene.cache.variants.get(&key).cloned())
    }

    pub(super) fn update_diagnostics(
        &self,
        active: &TreePresentationCache,
        diagnostics: &mut TreeAssetResidencyDiagnostics,
    ) {
        diagnostics.retained_scenes = self.scenes.len();
        let mut variants = active
            .variants
            .keys()
            .copied()
            .collect::<std::collections::HashSet<_>>();
        let mut atlases = active
            .variants
            .values()
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
            for (key, variant) in &scene.cache.variants {
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
