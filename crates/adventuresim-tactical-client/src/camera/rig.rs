//! Follow and collision observation for the gameplay camera.
use super::*;
use super::{collision::{CameraFrame, CameraVolume}, smoothing::{camera_view_metrics, critical_damp_scalar, damp_focus, sweet_spot_target}};
use adventuresim_tactical_core::{
    animation::AnimationElapsed,
    prelude::{BodyState, CharacterControllerCameraOf, CharacterControllerState, SkeletonState, SpatialQuery, SpatialQueryFilter},
};
use adventuresim_tactical_netcode::client::WeaponGuardInputState;

#[expect(
    clippy::too_many_arguments,
    reason = "this domain boundary names each independent input explicitly"
)]
pub(super) fn update_camera_rig(
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
    debug.0 = None;
    if !mode.third_person {
        state.reset_observation();
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
        debug.0 = None;
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
            _ => SignedLength::ZERO,
        };
        let anchor = match Position::from_metres(controller.translation + Vec3::Y * (profile.focus_height.metres() + crouch_adjustment.metres())) {
            Ok(anchor) => anchor,
            Err(error) => { warn!(%error, "cannot admit camera anchor"); continue; }
        };

        let raised_blend = state.blend;
        let mut next = state.observe(
            camera_of.character_controller,
            anchor,
            profile,
            config.teleport_distance,
        );

        let prepared = (|| -> CameraResult<CameraRigObservation> {
        let rotation = RigidRotation::from_quaternion(camera.rotation)?;
        let view = camera_view_metrics(projection);
        let sweet_spot = sweet_spot_target(
            anchor,
            next.focus,
            rotation,
            next.boom.distance,
            profile.sweet_spot,
            view,
        )?;
        let focus = damp_focus(
            next.focus,
            sweet_spot.target,
            next.focus_velocity,
            profile,
            elapsed,
        )?;
        next.focus = focus.position;
        next.focus_velocity = focus.velocity;
        let follow_error = anchor.metres() - next.focus.metres();
        if follow_error.length() > profile.maximum_follow_error.metres() {
            next.focus = Position::from_metres(anchor.metres() - follow_error.normalize() * profile.maximum_follow_error.metres())?;
            next.focus_velocity = SceneVelocity::from_metres_per_second(next.focus_velocity.metres_per_second().reject_from_normalized(follow_error.normalize()))?;
        }

        let mut shoulder_velocity = next.shoulder_velocity.metres_per_second();
        next.shoulder_offset = SignedLength::from_metres(critical_damp_scalar(
            next.shoulder_offset.metres(),
            profile.shoulder_offset.metres(),
            &mut shoulder_velocity,
            config.transition_time,
            elapsed,
        ))?;
        next.shoulder_velocity = SignedSpeed::from_metres_per_second(shoulder_velocity)?;
        let frame = CameraFrame {
            origin: Position::from_metres(controller.translation)?,
            anchor,
            focus: next.focus,
            rotation,
            shoulder_offset: next.shoulder_offset,
            distance: profile.distance,
        };
        let volume = CameraVolume::new(projection, config.collision_margin)?;
        let placement = next.boom.place(
            &frame,
            &volume,
            &spatial,
            &SpatialQueryFilter::from_excluded_entities([camera_of.character_controller]),
            &|entity| !soft_occluders.contains(entity),
            &config,
            elapsed,
        )?;
        let subject_delta = anchor.metres() - placement.position.metres();
        let subject_distance = subject_delta.length();
        let subject_direction = Dir3::new(subject_delta).unwrap_or(Dir3::NEG_Z);
        let soft_occlusion = spatial.cast_ray_predicate(
            placement.position.metres(),
            subject_direction,
            subject_distance,
            true,
            &SpatialQueryFilter::from_excluded_entities([camera_of.character_controller]),
            &|entity| soft_occluders.contains(entity),
        );

        let soft_occluder = soft_occlusion.map(|hit| Ok::<_, CameraError>(SoftOcclusion {
            entity: hit.entity,
            point: Position::from_metres(placement.position.metres() + *subject_direction * hit.distance)?,
        })).transpose()?;
        Ok(CameraRigObservation {
            raised_blend,
            subject: anchor,
            focus: next.focus,
            collision_origin: frame.origin,
            collision_volume: volume.debug_volume(placement.position, rotation)?,
            desired_endpoint: placement.desired,
            final_endpoint: placement.position,
            collision: placement.hit,
            soft_occluder,
            desired_distance: profile.distance,
            limited_distance: placement.limited_distance,
            focus_velocity: next.focus_velocity,
            boom_velocity: next.boom.velocity,
            screen_error: sweet_spot.error,
            sweet_spot: profile.sweet_spot,
        })
        })();
        match prepared {
            Ok(observation) => {
                state.commit_observation(next);
                camera.translation = observation.final_endpoint.metres();
                debug.0 = Some(observation);
            }
            Err(error) => warn!(%error, "cannot prepare camera observation"),
        }
    }
}

