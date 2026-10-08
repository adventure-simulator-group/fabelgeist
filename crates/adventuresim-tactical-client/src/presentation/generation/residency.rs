//! Bounded immutable scene products survive preparation and view installation.
//! These are generated assets, never Bevy entities or tactical tick state.
use super::*;

/// CPU scene-product availability; GPU installation has separate readiness.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum SceneReadiness {
    Missing,
    Prepared,
}

impl PreparedProducts {
    pub(super) fn begin_generation(&mut self) {
        self.facades.clear();
        self.placements.clear();
    }

    pub(super) fn scene_readiness(
        &mut self,
        input: &TacticalSceneInput,
    ) -> PreparationResult<SceneReadiness> {
        if self.scenes.is_empty() {
            return Ok(SceneReadiness::Missing);
        }
        let digest = input.digest()?;
        let Some(index) = self.scenes.iter().position(|scene| scene.digest == digest) else {
            return Ok(SceneReadiness::Missing);
        };
        let scene = self.scenes.remove(index);
        self.scenes.push(scene);
        Ok(SceneReadiness::Prepared)
    }

    pub(super) fn retain_scene(&mut self, scene: GeneratedTacticalScene) {
        self.scenes
            .retain(|resident| resident.digest != scene.digest);
        self.scenes.push(scene);
        if self.scenes.len() > RETAINED_SCENE_PRODUCTS {
            self.scenes.remove(0);
        }
    }

    pub(super) fn scene_for_installation(
        &self,
        input: &TacticalSceneInput,
    ) -> PreparationResult<GeneratedTacticalScene> {
        let digest = input.digest()?;
        self.scenes
            .iter()
            .find(|scene| scene.digest == digest)
            .cloned()
            .ok_or(PreparationError::NotPrepared {
                product: ProductKind::Scene,
            })
    }
}
