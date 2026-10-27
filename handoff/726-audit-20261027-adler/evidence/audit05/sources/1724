//! A checked scene camera view remains typed until rendering or serialization.
use super::*;
use adventuresim_building_generator::spatial_geometry::{
    Position, RigidRotation, SpatialDirection,
};
use adventuresim_tactical_core::scene_coordinates::Scene;

#[derive(Clone, Copy, Debug)]
pub(in crate::tactical_scene_viewer) struct SceneCameraView {
    pub(in crate::tactical_scene_viewer) position: Position<Scene>,
    pub(in crate::tactical_scene_viewer) target: Position<Scene>,
    orientation: SceneCameraOrientation,
}

pub(in crate::tactical_scene_viewer) struct CaptureCameraPlacement {
    pub(in crate::tactical_scene_viewer) view: SceneCameraView,
    pub(in crate::tactical_scene_viewer) obstruction: Option<CameraObstructionObservation>,
}

#[derive(Clone, Copy, Debug)]
enum SceneCameraOrientation {
    LookingAt { up: SpatialDirection<Scene> },
    Fixed(RigidRotation),
}

impl SceneCameraView {
    pub(in crate::tactical_scene_viewer) fn looking_at(
        position: Position<Scene>,
        target: Position<Scene>,
        up: SpatialDirection<Scene>,
    ) -> CaptureCameraResult<Self> {
        let forward = (target.metres() - position.metres())
            .try_normalize()
            .ok_or(CaptureCameraError::InvalidView)?;
        forward
            .cross(up.vector())
            .try_normalize()
            .ok_or(CaptureCameraError::InvalidView)?;
        Ok(Self {
            position,
            target,
            orientation: SceneCameraOrientation::LookingAt { up },
        })
    }

    pub(in crate::tactical_scene_viewer) fn oriented(
        position: Position<Scene>,
        target: Position<Scene>,
        rotation: RigidRotation,
    ) -> Self {
        Self {
            position,
            target,
            orientation: SceneCameraOrientation::Fixed(rotation),
        }
    }

    /// Native scene-vector representation for the framework view port.
    pub(in crate::tactical_scene_viewer) fn native_up(self) -> Vec3 {
        match self.orientation {
            SceneCameraOrientation::LookingAt { up } => up.vector(),
            SceneCameraOrientation::Fixed(rotation) => rotation.quaternion() * Vec3::Y,
        }
    }

    pub(in crate::tactical_scene_viewer) fn native_transform(self) -> Transform {
        let transform = Transform::from_translation(self.position.metres());
        match self.orientation {
            // Bevy owns the final look-at kernel. Keep its original arithmetic.
            SceneCameraOrientation::LookingAt { .. } => {
                transform.looking_at(self.target.metres(), self.native_up())
            }
            SceneCameraOrientation::Fixed(rotation) => {
                transform.with_rotation(rotation.quaternion())
            }
        }
    }
}
