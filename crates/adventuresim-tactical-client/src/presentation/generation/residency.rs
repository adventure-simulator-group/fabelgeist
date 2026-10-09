//! Bounded immutable scene products survive preparation and view installation.
//! These are generated assets, never Bevy entities or tactical tick state.
use super::*;

impl PreparedProducts {
    pub(super) fn begin_generation(&mut self) {
        self.facades.clear();
        self.placements.clear();
    }

    pub(super) fn scene_is_prepared(&mut self, input: &TacticalSceneInput) -> Result<bool, String> {
        if self.scenes.is_empty() {
            return Ok(false);
        }
        let digest = input.digest().map_err(|error| error.to_string())?;
        let Some(index) = self.scenes.iter().position(|scene| scene.digest == digest) else {
            return Ok(false);
        };
        let scene = self.scenes.remove(index);
        self.scenes.push(scene);
        Ok(true)
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
    ) -> Result<GeneratedTacticalScene, String> {
        let digest = input.digest().map_err(|error| error.to_string())?;
        self.scenes
            .iter()
            .find(|scene| scene.digest == digest)
            .cloned()
            .ok_or_else(|| "scene generation was not completed before installation".into())
    }
}
