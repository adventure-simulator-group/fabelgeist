//! Retain complete terrain-bound tree assets across ordinary scene replacement.
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
        diagnostics.retained_scenes = self.scenes.len();
        diagnostics.retained_variants = self.scenes.iter().map(|s| s.diagnostics.variants).sum();
        diagnostics.retained_impostor_texture_bytes = self
            .scenes
            .iter()
            .map(|s| s.diagnostics.impostor_texture_bytes)
            .sum();
    }
}
