pub(crate) mod collision;
mod smoothing;

use collision::{BoomRecovery, CameraFrame, CameraVolume, CloseCameraProfile};
use smoothing::{
    blend_profile, camera_view_metrics, critical_damp_scalar, damp_focus, sweet_spot_target,
};

use adventuresim_tactical_core::prelude::{
    BodyState, CharacterControllerCameraOf, CharacterControllerState, PlayerEquipment,
    SkeletonState, SpatialQuery, SpatialQueryFilter, TacticalPlayerViewer, WeaponGuardState,
};
use adventuresim_tactical_netcode::client::WeaponGuardInputState;
use bevy::prelude::*;

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

#[derive(Debug, Clone, Copy)]
pub struct CameraProfile {
    pub distance: f32,
    pub shoulder_offset: f32,
    /// Height above the controller transform, which is centered in its capsule.
    pub focus_height: f32,
    pub horizontal_follow_time: f32,
    pub vertical_follow_time: f32,
    pub maximum_follow_error: f32,
    /// Half-extents in normalized screen coordinates.
    pub sweet_spot: Vec2,
}

#[derive(Resource, Debug, Clone, Copy)]
pub struct CameraRigConfig {
    pub lowered: CameraProfile,
    pub raised: CameraProfile,
    pub transition_time: f32,
    pub close: CloseCameraProfile,
    pub collision_margin: f32,
    pub collision_recovery_time: f32,
    pub collision_hysteresis: f32,
    /// Seconds to hold a retracted view before recovering through a doorway.
    pub collision_hold_time: f32,
    pub teleport_distance: f32,
    pub crouch_focus_adjustment: f32,
    pub prone_focus_adjustment: f32,
    pub supine_focus_adjustment: f32,
    pub aim_distance: f32,
    pub muzzle_offset: Vec3,
}

impl Default for CameraRigConfig {
    fn default() -> Self {
        Self {
            lowered: CameraProfile {
                distance: 3.75,
                shoulder_offset: 0.0,
                focus_height: 0.48,
                horizontal_follow_time: 0.18,
                vertical_follow_time: 0.34,
                maximum_follow_error: 0.85,
                sweet_spot: Vec2::new(0.09, 0.065),
            },
            raised: CameraProfile {
                distance: 2.75,
                shoulder_offset: 0.5,
                focus_height: 0.62,
                horizontal_follow_time: 0.055,
                vertical_follow_time: 0.08,
                maximum_follow_error: 0.22,
                sweet_spot: Vec2::splat(0.01),
            },
            transition_time: 0.18,
            close: CloseCameraProfile::default(),
            collision_margin: 0.04,
            collision_recovery_time: 0.32,
            collision_hysteresis: 0.08,
            collision_hold_time: 0.12,
            teleport_distance: 2.0,
            crouch_focus_adjustment: -0.45,
            prone_focus_adjustment: -0.82,
            supine_focus_adjustment: -0.76,
            aim_distance: 100.0,
            // Local right, up, and forward from the controller center.
            muzzle_offset: Vec3::new(0.28, 0.55, 0.42),
        }
    }
}

#[derive(Resource, Debug, Clone)]
pub(crate) struct CameraRigState {
    initialized: bool,
    focus: Vec3,
    focus_velocity: Vec3,
    blend: f32,
    blend_velocity: f32,
    boom: BoomRecovery,
    shoulder_offset: f32,
    shoulder_velocity: f32,
    last_anchor: Vec3,
    subject: Option<Entity>,
}

impl Default for CameraRigState {
    fn default() -> Self {
        Self {
            initialized: false,
            focus: Vec3::ZERO,
            focus_velocity: Vec3::ZERO,
            blend: 0.0,
            blend_velocity: 0.0,
            boom: BoomRecovery::default(),
            shoulder_offset: 0.0,
            shoulder_velocity: 0.0,
            last_anchor: Vec3::ZERO,
            subject: None,
        }
    }
}

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
        state.initialized = false;
        debug.active = false;
        return;
    }

    let dt = time.delta_secs().min(0.1);
    for (mut camera, camera_of, projection) in &mut cameras {
        let Ok((controller, controller_state, skeleton)) =
            controllers.get(camera_of.character_controller)
        else {
            continue;
        };
        let target_blend = f32::from(guard.desired == WeaponGuardState::Raised);
        state.blend = critical_damp_scalar(
            state.blend,
            target_blend,
            &mut state.blend_velocity,
            config.transition_time,
            dt,
        )
        .clamp(0.0, 1.0);
        let profile = blend_profile(config.lowered, config.raised, state.blend);
        let crouch_adjustment = match skeleton.map(SkeletonState::body) {
            Some(BodyState::Prone) => config.prone_focus_adjustment,
            Some(BodyState::Supine) => config.supine_focus_adjustment,
            _ if controller_state.crouching => config.crouch_focus_adjustment,
            _ => 0.0,
        };
        let anchor = controller.translation + Vec3::Y * (profile.focus_height + crouch_adjustment);

        let discontinuity = state.initialized
            && (state.subject != Some(camera_of.character_controller)
                || anchor.distance(state.last_anchor) > config.teleport_distance);
        if !state.initialized || discontinuity {
            state.initialized = true;
            state.focus = anchor;
            state.focus_velocity = Vec3::ZERO;
            state.boom.reset(profile.distance);
            state.shoulder_offset = profile.shoulder_offset;
            state.shoulder_velocity = 0.0;
        }
        state.last_anchor = anchor;
        state.subject = Some(camera_of.character_controller);

        let rotation = camera.rotation;
        let (aspect, tan_half_fov) = camera_view_metrics(projection);
        let (focus_target, screen_error) = sweet_spot_target(
            anchor,
            state.focus,
            rotation,
            state.boom.distance,
            profile.sweet_spot,
            aspect,
            tan_half_fov,
        );
        state.focus = damp_focus(
            state.focus,
            focus_target,
            &mut state.focus_velocity,
            profile,
            dt,
        );
        let follow_error = anchor - state.focus;
        if follow_error.length() > profile.maximum_follow_error {
            state.focus = anchor - follow_error.normalize() * profile.maximum_follow_error;
            state.focus_velocity = state
                .focus_velocity
                .reject_from_normalized(follow_error.normalize());
        }

        state.shoulder_offset = critical_damp_scalar(
            state.shoulder_offset,
            profile.shoulder_offset,
            &mut state.shoulder_velocity,
            config.transition_time,
            dt,
        );
        let frame = CameraFrame {
            origin: controller.translation,
            anchor,
            focus: state.focus,
            rotation,
            shoulder_offset: state.shoulder_offset,
            distance: profile.distance,
        };
        let volume = CameraVolume::new(projection, config.collision_margin);
        let placement = state.boom.place(
            &frame,
            &volume,
            &spatial,
            &SpatialQueryFilter::from_excluded_entities([camera_of.character_controller]),
            &|entity| !soft_occluders.contains(entity),
            &config,
            dt,
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
            raised_blend: state.blend,
            subject: anchor,
            focus: state.focus,
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
            focus_velocity: state.focus_velocity,
            boom_velocity: state.boom.velocity,
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
    fn reversing_a_mode_blend_continues_from_the_displayed_state() {
        let mut velocity = 0.0;
        let raised = critical_damp_scalar(0.0, 1.0, &mut velocity, 0.18, 0.08);
        let reversed = critical_damp_scalar(raised, 0.0, &mut velocity, 0.18, 0.01);
        assert!((reversed - raised).abs() < 0.1);
        assert!(reversed > 0.0 && reversed < 1.0);
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
