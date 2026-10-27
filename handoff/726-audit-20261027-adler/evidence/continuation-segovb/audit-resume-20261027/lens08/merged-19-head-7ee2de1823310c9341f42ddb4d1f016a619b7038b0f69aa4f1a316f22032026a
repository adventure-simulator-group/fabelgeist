use adventuresim_building_generator::spatial_geometry::{Displacement, Position, RigidRotation};
use adventuresim_core::linear_speed::SignedSpeed;
use adventuresim_tactical_core::{
    animation::UnitInterval,
    scene_coordinates::{Scene, SceneVelocity},
};
use bevy::prelude::*;
use fabelgeist_geometry::SignedLength;

pub use adventuresim_tactical_core::animation::ResponseShapingTime;
use aim::update_camera_aim;
use collision::{CameraHit, CameraVolumeDebug};
pub use config::{CameraProfile, CameraRigConfig};
use rig::update_camera_rig;
use rig_history::CameraRigState;
use values::ScreenError;
pub use values::{CameraError, CameraResult, ScreenWindow};

mod aim;
pub(crate) mod collision;
pub(crate) mod config;
mod rig;
mod rig_history;
mod smoothing;
mod values;

pub struct TacticalCameraPlugin;

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum TacticalCameraSet {
    Offset,
    Aim,
}

#[derive(Resource, Debug, Clone, Copy)]
pub(crate) struct CameraMode {
    pub(crate) third_person: bool,
}

/// A collider tagged as soft can obscure the subject but never retracts the
/// camera. Foliage, particles, and thin decorative props should use this.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct CameraSoftOccluder;

#[derive(Resource, Debug, Clone, Copy, Default)]
pub(crate) struct CameraRigDebugState(pub(crate) Option<CameraRigObservation>);

#[derive(Debug, Clone, Copy)]
pub(crate) struct CameraRigObservation {
    pub(crate) raised_blend: UnitInterval,
    pub(crate) subject: Position<Scene>,
    pub(crate) focus: Position<Scene>,
    pub(crate) collision_origin: Position<Scene>,
    pub(crate) collision_volume: CameraVolumeDebug,
    pub(crate) desired_endpoint: Position<Scene>,
    pub(crate) final_endpoint: Position<Scene>,
    pub(crate) collision: Option<CameraHit>,
    pub(crate) soft_occluder: Option<SoftOcclusion>,
    pub(crate) desired_distance: SignedLength,
    pub(crate) limited_distance: SignedLength,
    pub(crate) focus_velocity: SceneVelocity,
    pub(crate) boom_velocity: SignedSpeed,
    pub(crate) screen_error: ScreenError,
    pub(crate) sweet_spot: ScreenWindow,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct SoftOcclusion {
    pub(crate) entity: Entity,
    pub(crate) point: Position<Scene>,
}

#[derive(Resource, Debug, Clone, Copy, Default)]
pub(crate) struct CameraAimState(pub(crate) Option<CameraAimObservation>);

#[derive(Debug, Clone, Copy)]
pub(crate) struct CameraAimObservation {
    pub(crate) camera_origin: Position<Scene>,
    pub(crate) camera_target: Position<Scene>,
    pub(crate) camera_hit: Option<Entity>,
    pub(crate) muzzle_origin: Position<Scene>,
    pub(crate) actual_target: Position<Scene>,
    pub(crate) actual_hit: Option<Entity>,
}

#[derive(Resource, Debug, Clone, Copy, Default)]
pub(crate) struct CameraDebugEnabled(pub(crate) bool);

impl Plugin for TacticalCameraPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CameraMode>()
            .init_resource::<CameraRigConfig>()
            .init_resource::<CameraRigState>()
            .init_resource::<CameraRigDebugState>()
            .init_resource::<CameraAimState>()
            .init_resource::<CameraDebugEnabled>()
            .add_systems(Update, toggle_camera_mode)
            .add_systems(
                PostUpdate,
                update_camera_rig
                    .in_set(TacticalCameraSet::Offset)
                    .before(TransformSystems::Propagate),
            )
            .add_systems(
                PostUpdate,
                update_camera_aim
                    .in_set(TacticalCameraSet::Aim)
                    .after(TacticalCameraSet::Offset)
                    .before(TransformSystems::Propagate),
            );
        #[cfg(feature = "debug")]
        app.add_systems(Update, toggle_camera_debug);
    }
}

impl Default for CameraMode {
    fn default() -> Self {
        Self { third_person: true }
    }
}

impl CameraAimObservation {
    pub(crate) fn blocked(self) -> bool {
        muzzle_path_is_blocked(self.camera_hit, self.actual_hit)
    }
}

fn toggle_camera_mode(keyboard: Res<ButtonInput<KeyCode>>, mut mode: ResMut<CameraMode>) {
    if keyboard.just_pressed(KeyCode::F9) {
        mode.third_person = !mode.third_person;
        info!(
            third_person = mode.third_person,
            "Changed tactical camera mode"
        );
    }
}

#[cfg(feature = "debug")]
fn toggle_camera_debug(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut enabled: ResMut<CameraDebugEnabled>,
) {
    if keyboard.just_pressed(KeyCode::F6) {
        enabled.0 = !enabled.0;
    }
}

fn muzzle_path_is_blocked(camera_hit: Option<Entity>, muzzle_hit: Option<Entity>) -> bool {
    muzzle_hit.is_some() && muzzle_hit != camera_hit
}

/// Reproduces the lowered gameplay-camera boom for deterministic animation
/// captures without advancing the stateful rig.
pub(crate) fn animation_capture_camera_offset(
    rotation: RigidRotation,
) -> CameraResult<Displacement<Scene>> {
    Ok(Displacement::from_metres(
        rotation.quaternion() * Vec3::Z * CameraRigConfig::default().lowered.distance.metres(),
    )?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn f9_toggles_third_person_mode() {
        let mut world = World::new();
        let mut keyboard = ButtonInput::default();
        keyboard.press(KeyCode::F9);
        world.insert_resource(keyboard);
        world.insert_resource(CameraMode::default());
        world.run_system_cached(toggle_camera_mode).unwrap();
        assert!(!world.resource::<CameraMode>().third_person);
    }

    #[test]
    fn a_different_muzzle_hit_reports_cover_blockage() {
        let target = Entity::from_bits(1);
        let cover = Entity::from_bits(2);
        assert!(muzzle_path_is_blocked(Some(target), Some(cover),));
        assert!(!muzzle_path_is_blocked(Some(target), Some(target),));
    }
}
