use adventuresim_tactical_core::animation::AnimationElapsed;
use collision::{CameraFrame, CameraVolume};
pub use config::{CameraProfile, CameraRigConfig, CameraShapingTime};
use rig_history::CameraRigState;
use smoothing::{camera_view_metrics, critical_damp_scalar, damp_focus, sweet_spot_target};

use adventuresim_tactical_core::prelude::{
    BodyState, CharacterControllerCameraOf, CharacterControllerState, PlayerEquipment,
    SkeletonState, SpatialQuery, SpatialQueryFilter, TacticalPlayerViewer, WeaponGuardState,
};
use adventuresim_tactical_netcode::client::WeaponGuardInputState;
use bevy::prelude::*;

pub(crate) mod collision;
mod config;
mod rig_history;
mod smoothing;

pub struct TacticalCameraPlugin;

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum TacticalCameraSet {
    Offset,
    Aim,
}

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

#[derive(Resource, Debug, Clone, Copy)]
pub(crate) struct CameraMode {
    pub(crate) third_person: bool,
}

impl Default for CameraMode {
    fn default() -> Self {
        Self { third_person: true }
    }
}

/// A collider tagged as soft can obscure the subject but never retracts the
/// camera. Foliage, particles, and thin decorative props should use this.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct CameraSoftOccluder;

#[derive(Resource, Debug, Clone, Copy, Default)]
pub(crate) struct CameraRigDebugState {
    pub(crate) active: bool,
    pub(crate) raised_blend: f32,
    pub(crate) subject: Vec3,
    pub(crate) focus: Vec3,
    pub(crate) collision_origin: Vec3,
    pub(crate) collision_volume: Transform,
    pub(crate) desired_endpoint: Vec3,
    pub(crate) final_endpoint: Vec3,
    pub(crate) collision_normal: Vec3,
    pub(crate) collision_entity: Option<Entity>,
    pub(crate) soft_occluder: Option<Entity>,
    pub(crate) soft_occluder_point: Vec3,
    pub(crate) desired_distance: f32,
    pub(crate) limited_distance: f32,
    pub(crate) focus_velocity: Vec3,
    pub(crate) boom_velocity: f32,
    pub(crate) screen_error: Vec2,
    pub(crate) sweet_spot: Vec2,
}

#[derive(Resource, Debug, Clone, Copy, Default)]
pub(crate) struct CameraAimState {
    pub(crate) active: bool,
    pub(crate) camera_origin: Vec3,
    pub(crate) camera_target: Vec3,
    pub(crate) camera_hit: Option<Entity>,
    pub(crate) muzzle_origin: Vec3,
    pub(crate) actual_target: Vec3,
    pub(crate) actual_hit: Option<Entity>,
    pub(crate) blocked: bool,
}

#[derive(Resource, Debug, Clone, Copy, Default)]
pub(crate) struct CameraDebugEnabled(pub(crate) bool);

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

#[expect(
    clippy::too_many_arguments,
    reason = "this domain boundary names each independent input explicitly"
)]
fn update_camera_rig(
    time: Res<Time>,
    mode: Res<CameraMode>,
    guard: Res<WeaponGuardInputState>,
    config: Res<CameraRigConfig>,
    spatial: SpatialQuery,
    soft_occluders: Query<(), With<CameraSoftOccluder>>,
    controllers: Query<
        (
            &Transform,
            &CharacterControllerState,
            Option<&SkeletonState>,
        ),
        Without<CharacterControllerCameraOf>,
    >,
    mut cameras: Query<(&mut Transform, &CharacterControllerCameraOf, &Projection)>,
    mut state: ResMut<CameraRigState>,
    mut debug: ResMut<CameraRigDebugState>,
) {
    if !mode.third_person {
        state.reset_observation();
        debug.active = false;
        return;
    }

    let elapsed = match AnimationElapsed::from_seconds(time.delta_secs().min(0.1)) {
        Ok(elapsed) => elapsed,
        Err(error) => {
            warn!(%error, "cannot admit camera frame interval");
            return;
        }
    };
    for (mut camera, camera_of, projection) in &mut cameras {
        let Ok((controller, controller_state, skeleton)) =
            controllers.get(camera_of.character_controller)
        else {
            continue;
        };
        let profile = match state.advance_profile(guard.desired, &config, elapsed) {
            Ok(profile) => profile,
            Err(error) => {
                warn!(%error, "cannot blend camera profile");
                continue;
            }
        };
        let crouch_adjustment = match skeleton.map(SkeletonState::body) {
            Some(BodyState::Prone) => config.prone_focus_adjustment,
            Some(BodyState::Supine) => config.supine_focus_adjustment,
            _ if controller_state.crouching => config.crouch_focus_adjustment,
            _ => 0.0,
        };
        let anchor = controller.translation + Vec3::Y * (profile.focus_height + crouch_adjustment);

        let raised_blend = state.blend.fraction();
        let history = state.observe(
            camera_of.character_controller,
            anchor,
            profile,
            config.teleport_distance,
        );

        let rotation = camera.rotation;
        let (aspect, tan_half_fov) = camera_view_metrics(projection);
        let (focus_target, screen_error) = sweet_spot_target(
            anchor,
            history.focus,
            rotation,
            history.boom.distance,
            profile.sweet_spot,
            aspect,
            tan_half_fov,
        );
        history.focus = damp_focus(
            history.focus,
            focus_target,
            &mut history.focus_velocity,
            profile,
            elapsed,
        );
        let follow_error = anchor - history.focus;
        if follow_error.length() > profile.maximum_follow_error {
            history.focus = anchor - follow_error.normalize() * profile.maximum_follow_error;
            history.focus_velocity = history
                .focus_velocity
                .reject_from_normalized(follow_error.normalize());
        }

        history.shoulder_offset = critical_damp_scalar(
            history.shoulder_offset,
            profile.shoulder_offset,
            &mut history.shoulder_velocity,
            config.transition_time,
            elapsed,
        );
        let frame = CameraFrame {
            origin: controller.translation,
            anchor,
            focus: history.focus,
            rotation,
            shoulder_offset: history.shoulder_offset,
            distance: profile.distance,
        };
        let volume = CameraVolume::new(projection, config.collision_margin);
        let placement = history.boom.place(
            &frame,
            &volume,
            &spatial,
            &SpatialQueryFilter::from_excluded_entities([camera_of.character_controller]),
            &|entity| !soft_occluders.contains(entity),
            &config,
            elapsed,
        );
        camera.translation = placement.position;
        let subject_delta = anchor - camera.translation;
        let subject_distance = subject_delta.length();
        let subject_direction = Dir3::new(subject_delta).unwrap_or(Dir3::NEG_Z);
        let soft_occlusion = spatial.cast_ray_predicate(
            camera.translation,
            subject_direction,
            subject_distance,
            true,
            &SpatialQueryFilter::from_excluded_entities([camera_of.character_controller]),
            &|entity| soft_occluders.contains(entity),
        );

        *debug = CameraRigDebugState {
            active: true,
            raised_blend,
            subject: anchor,
            focus: history.focus,
            collision_origin: frame.origin,
            collision_volume: Transform {
                translation: camera.translation + rotation * volume.local_center,
                rotation,
                scale: volume.size,
            },
            desired_endpoint: placement.desired,
            final_endpoint: camera.translation,
            collision_normal: placement.hit.map_or(Vec3::ZERO, |hit| hit.normal1),
            collision_entity: placement.hit.map(|hit| hit.entity),
            soft_occluder: soft_occlusion.map(|hit| hit.entity),
            soft_occluder_point: camera.translation
                + *subject_direction * soft_occlusion.map_or(0.0, |hit| hit.distance),
            desired_distance: profile.distance,
            limited_distance: placement.limited_distance,
            focus_velocity: history.focus_velocity,
            boom_velocity: history.boom.velocity,
            screen_error,
            sweet_spot: profile.sweet_spot,
        };
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "Bevy injects the camera, controller, input, and spatial state as independent system parameters"
)]
fn update_camera_aim(
    mode: Res<CameraMode>,
    guard: Res<WeaponGuardInputState>,
    config: Res<CameraRigConfig>,
    spatial: SpatialQuery,
    cameras: Query<(&Transform, &CharacterControllerCameraOf)>,
    controllers: Query<&Transform, Without<CharacterControllerCameraOf>>,
    viewer: TacticalPlayerViewer,
    mut aim: ResMut<CameraAimState>,
) {
    let raised = mode.third_person && guard.desired == WeaponGuardState::Raised;
    if !raised {
        aim.active = false;
        aim.blocked = false;
        aim.camera_hit = None;
        aim.actual_hit = None;
        return;
    }
    aim.active = false;

    for (camera, camera_of) in &cameras {
        aim.active = viewer
            .get(camera_of.character_controller)
            .is_ok_and(|player| player.weapon_is_ranged());
        if !aim.active {
            aim.blocked = false;
            aim.camera_hit = None;
            aim.actual_hit = None;
            continue;
        }
        let Ok(controller) = controllers.get(camera_of.character_controller) else {
            continue;
        };
        let filter = SpatialQueryFilter::from_excluded_entities([camera_of.character_controller]);
        let camera_origin = camera.translation;
        let camera_direction = camera.forward();
        let camera_hit = spatial.cast_ray(
            camera_origin,
            camera_direction,
            config.aim_distance,
            true,
            &filter,
        );
        let camera_target = camera_origin
            + *camera_direction * camera_hit.map_or(config.aim_distance, |hit| hit.distance);

        let planar_forward = Vec3::new(camera_direction.x, 0.0, camera_direction.z)
            .try_normalize()
            .unwrap_or(Vec3::NEG_Z);
        let right = planar_forward.cross(Vec3::Y).normalize_or_zero();
        let muzzle_origin = controller.translation
            + right * config.muzzle_offset.x
            + Vec3::Y * config.muzzle_offset.y
            + planar_forward * config.muzzle_offset.z;
        let muzzle_delta = camera_target - muzzle_origin;
        let muzzle_distance = muzzle_delta.length();
        let muzzle_direction = Dir3::new(muzzle_delta).unwrap_or(camera_direction);
        let actual_hit = spatial.cast_ray(
            muzzle_origin,
            muzzle_direction,
            muzzle_distance,
            true,
            &filter,
        );
        let actual_target = muzzle_origin
            + *muzzle_direction * actual_hit.map_or(muzzle_distance, |hit| hit.distance);
        let camera_entity = camera_hit.map(|hit| hit.entity);
        let actual_entity = actual_hit.map(|hit| hit.entity);
        *aim = CameraAimState {
            active: true,
            camera_origin,
            camera_target,
            camera_hit: camera_entity,
            muzzle_origin,
            actual_target,
            actual_hit: actual_entity,
            blocked: muzzle_path_is_blocked(
                camera_entity,
                actual_entity,
                actual_target,
                camera_target,
            ),
        };
    }
}

fn muzzle_path_is_blocked(
    camera_hit: Option<Entity>,
    muzzle_hit: Option<Entity>,
    _actual_target: Vec3,
    _camera_target: Vec3,
) -> bool {
    muzzle_hit.is_some() && muzzle_hit != camera_hit
}

/// Reproduces the lowered gameplay-camera boom for deterministic animation
/// captures without advancing the stateful rig.
pub(crate) fn animation_capture_camera_offset(rotation: Quat) -> Vec3 {
    rotation * Vec3::Z * CameraRigConfig::default().lowered.distance
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
        assert!(muzzle_path_is_blocked(
            Some(target),
            Some(cover),
            Vec3::Z,
            Vec3::Z * 10.0,
        ));
        assert!(!muzzle_path_is_blocked(
            Some(target),
            Some(target),
            Vec3::Z * 10.0,
            Vec3::Z * 10.0,
        ));
    }
}
