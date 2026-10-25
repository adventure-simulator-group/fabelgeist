//! Retain terrain bindings and share matching plant assets across scenes.
use super::*;
use crate::presentation::terrain::{TerrainContactHeightmap, TerrainHeightmapResult};
use adventuresim_tactical_core::scene::SceneElevationRange;
use std::collections::VecDeque;

// Active scene plus two previous scenes bounds retained geometry and atlases.
const PREVIOUS_TREE_SCENES: usize = 2;

#[derive(Resource, Default)]
pub(in crate::presentation) struct RetainedTreePresentations {
    scenes: VecDeque<RetainedScene>,
}

#[derive(Clone)]
pub(super) struct CachedTerrainContactHeightmap {
    scene_digest: String,
    image: Handle<Image>,
    range: SceneElevationRange,
}

pub(super) struct TerrainBarkBinding<'a> {
    pub(super) procedural_assets: &'a ProceduralTextureAssets,
    pub(super) contact: &'a CachedTerrainContactHeightmap,
    pub(super) terrain: &'a SceneTerrain,
    pub(super) materials: &'a mut Assets<TacticalTreeBarkMaterial>,
}

struct RetainedScene {
    cache: TreePresentationCache,
    diagnostics: TreeAssetResidencyDiagnostics,
}

impl TreePresentationCache {
    pub(super) fn bark_for_terrain(
        &mut self,
        species: TreePresentationSpecies,
        binding: &mut TerrainBarkBinding<'_>,
    ) -> Handle<TacticalTreeBarkMaterial> {
        match species {
            TreePresentationSpecies::EnglishOak => {
                self.oak_bark_material.clone().unwrap_or_else(|| {
                    let material = binding.materials.add(oak_bark_material(
                        binding.procedural_assets,
                        binding.contact.image.clone(),
                        binding.contact.range,
                        binding.terrain,
                    ));
                    self.oak_bark_material = Some(material.clone());
                    material
                })
            }
            TreePresentationSpecies::CommonBeech => {
                self.beech_bark_material.clone().unwrap_or_else(|| {
                    let material = binding.materials.add(beech_bark_material(
                        binding.procedural_assets,
                        binding.contact.image.clone(),
                        binding.contact.range,
                        binding.terrain,
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
    ) -> TerrainHeightmapResult<()> {
        if cache
            .terrain_contact
            .as_ref()
            .is_some_and(|contact| contact.scene_digest == environment.scene_digest)
        {
            return Ok(());
        }
        let restored = self
            .scenes
            .iter()
            .position(|scene| {
                scene
                    .cache
                    .terrain_contact
                    .as_ref()
                    .is_some_and(|contact| contact.scene_digest == environment.scene_digest)
            })
            .and_then(|index| self.scenes.remove(index));
        let next = match restored {
            Some(restored) => restored,
            None => {
                let contact = TerrainContactHeightmap::from_terrain(terrain)?;
                let range = contact.range();
                RetainedScene {
                    cache: TreePresentationCache {
                        terrain_contact: Some(CachedTerrainContactHeightmap {
                            scene_digest: environment.scene_digest.clone(),
                            image: images.add(contact.into_image()),
                            range,
                        }),
                        ..default()
                    },
                    diagnostics: default(),
                }
            }
        };
        if cache.terrain_contact.is_some() {
            self.scenes.push_back(RetainedScene {
                cache: std::mem::take(cache),
                diagnostics: std::mem::take(diagnostics),
            });
        }
        while self.scenes.len() > PREVIOUS_TREE_SCENES {
            self.scenes.pop_front();
        }
        *cache = next.cache;
        *diagnostics = next.diagnostics;
        self.update_diagnostics(cache, diagnostics);
        Ok(())
    }

    pub(super) fn restore_variant(
        &self,
        key: Seed,
        cache: &mut TreePresentationCache,
        diagnostics: &mut TreeAssetResidencyDiagnostics,
        species: TreePresentationSpecies,
        binding: &mut TerrainBarkBinding<'_>,
    ) {
        if cache.variants.contains_key(&key) {
            return;
        }
        let Some(mut cached) = self.variant(key) else {
            return;
        };
        // Plant geometry is site-local; bind only bark to destination terrain.
        cached.bark_material = cache.bark_for_terrain(species, binding);
        diagnostics.add_resident_assets(&cached.resident_assets);
        cache.variants.insert(key, cached);
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
