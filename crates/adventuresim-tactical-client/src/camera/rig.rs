//! Follow and collision observation for the gameplay camera.
use super::*;
use adventuresim_tactical_core::combat_config::TacticalCombatConfig;
use adventuresim_tactical_core::{
    animation::AnimationElapsed,
    prelude::{
        BodyState, CharacterControllerCameraOf, CharacterControllerState, SkeletonState,
        SpatialQuery, SpatialQueryFilter,
    },
};
use adventuresim_tactical_netcode::client::WeaponGuardInputState;
use observation::{CameraRigInput, CameraSubjectObservation};

mod observation;

#[expect(
    clippy::too_many_arguments,
    reason = "this domain boundary names each independent input explicitly"
)]
pub(super) fn update_camera_rig(
    local: Res<adventuresim_tactical_netcode::control::LocalControl>,
    combat: Res<TacticalCombatConfig>,
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
        let Ok((transform, controller, skeleton)) = controllers.get(camera_of.character_controller)
        else {
            continue;
        };
        let mut profile = match state.advance_profile(guard.desired, &config, elapsed) {
            Ok(profile) => profile,
            Err(error) => {
                warn!(%error, "cannot blend camera profile");
                continue;
            }
        };
        if local.acknowledged.mode == adventuresim_tactical_core::control::ControlMode::Indirect
            && let Ok(distance) = fabelgeist_geometry::SignedLength::from_metres(
                combat.ai.control.orbit_distance_metres.metres(),
            )
        {
            profile.distance = distance;
        }
        let input = match CameraRigInput::observe(
            CameraSubjectObservation {
                entity: camera_of.character_controller,
                transform,
                controller,
                skeleton,
            },
            &camera,
            profile,
            state.blend,
            &config,
            elapsed,
        ) {
            Ok(input) => input,
            Err(error) => {
                warn!(%error, "cannot admit camera frame");
                continue;
            }
        };
        let mut next = state.observe(
            camera_of.character_controller,
            input.anchor,
            profile,
            config.teleport_distance,
        );
        match input.prepare(&mut next, projection, &spatial, &soft_occluders, &config) {
            Ok(observation) => {
                state.commit_observation(next);
                camera.translation = observation.final_endpoint.metres();
                debug.0 = Some(observation);
            }
            Err(error) => warn!(%error, "cannot prepare camera observation"),
        }
    }
}
