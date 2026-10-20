//! Required capture terrain rejects through the capture artifact and app exit.
use super::*;
use std::fmt::Display;

impl LightingObservationParams<'_, '_> {
    pub(super) fn required_capture_terrain(
        &self,
        output: &Path,
        commands: &mut Commands,
    ) -> Option<&SceneTerrain> {
        match self.terrain.single() {
            Ok(terrain) => Some(terrain),
            Err(error) => {
                reject(output, error, commands);
                None
            }
        }
    }

    pub(super) fn boundary_camera(
        &self,
        projection: &Projection,
        player: ScenePlanPoint,
        yaw_degrees: f32,
        output: &Path,
        commands: &mut Commands,
    ) -> Option<(Transform, Vec3, Option<CameraObstructionObservation>)> {
        let terrain = self.required_capture_terrain(output, commands)?;
        match camera_obstruction::animation_play_boundary_camera(
            projection, terrain, &self.spatial, player, yaw_degrees,
        ) {
            Ok(camera) => Some(camera),
            Err(error) => {
                reject(output, error, commands);
                None
            }
        }
    }
}

pub(super) fn reject(output: &Path, failure: impl Display, commands: &mut Commands) {
    error!("capture admission failed: {failure}");
    if let Err(error) = fs::write(output.join("failure.txt"), format!("Capture admission failed: {failure}\n")) {
        error!("failed to write capture rejection: {error}");
    }
    commands.write_message(AppExit::error());
}
