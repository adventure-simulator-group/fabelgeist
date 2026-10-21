//! Stateless obstruction captures use the gameplay placement and projection.

use super::*;
use crate::camera::collision::{BoomRecovery, CameraFrame, CameraVolume};
use adventuresim_building_generator::spatial_geometry::{Position, RigidRotation};
use adventuresim_tactical_core::{
    animation::{AnimationElapsed, SignedAngleDegrees},
    scene_coordinates::{Scene, ScenePlanPoint},
};
use super::view_camera::{CaptureCameraPlacement, CaptureCameraResult, SceneCameraView};

pub(super) fn animation_play_obstruction_camera(
    state: &SceneCaptureState,
    spatial: &SpatialQuery,
    projection: &Projection,
    yaw_degrees: SignedAngleDegrees,
) -> CaptureCameraResult<CaptureCameraPlacement> {
    let config = CameraRigConfig::default();
    let Some(tree) = state.tree_focus else {
        return Ok(CaptureCameraPlacement {
            view: camera_for_view(CapturePose::AnimationPlay { yaw_degrees }, state)?,
            obstruction: Some(CameraObstructionObservation {
                desired_metres: config.lowered.distance,
                resolved_metres: config.lowered.distance,
                hit: false,
            }),
        });
    };
    let tree = tree.metres();
    let yaw = Quat::from_rotation_y(yaw_degrees.radians());
    let outward = yaw * Vec3::Z;
    let tree_root_y = tree.y - TREE_TRUNK_HEIGHT_METRES * 0.5;
    let focus = Position::from_metres(Vec3::new(tree.x, tree_root_y + 1.35, tree.z) + outward * 0.95)?;
    let rotation = RigidRotation::from_quaternion(Transform::IDENTITY.looking_to(outward, Vec3::Y).rotation)?;
    resolve(spatial, projection, focus, rotation)
}

pub(super) fn animation_play_boundary_camera(
    projection: &Projection,
    terrain: &SceneTerrain,
    spatial: &SpatialQuery,
    player: ScenePlanPoint,
    yaw_degrees: SignedAngleDegrees,
) -> CaptureCameraResult<CaptureCameraPlacement> {
    let elevation = terrain.required_height_at(player)?;
    let focus = Position::from_metres(Vec3::new(player.metres().x, elevation.metres() + 1.48, player.metres().y))?;
    resolve(spatial, projection, focus, RigidRotation::from_quaternion(Quat::from_rotation_y(yaw_degrees.radians()))?)
}

fn resolve(
    spatial: &SpatialQuery,
    projection: &Projection,
    focus: Position<Scene>,
    rotation: RigidRotation,
) -> CaptureCameraResult<CaptureCameraPlacement> {
    let config = CameraRigConfig::default();
    let frame = CameraFrame {
        origin: Position::from_metres(focus.metres() - Vec3::Y * config.lowered.focus_height.metres())?,
        anchor: focus,
        focus,
        rotation,
        shoulder_offset: config.lowered.shoulder_offset,
        distance: config.lowered.distance,
    };
    let mut boom = BoomRecovery::default();
    boom.reset(frame.distance);
    let placement = boom.place(
        &frame,
        &CameraVolume::new(projection, config.collision_margin)?,
        spatial,
        &SpatialQueryFilter::default(),
        &|_| true,
        &config,
        AnimationElapsed::ZERO,
    )?;
    Ok(CaptureCameraPlacement {
        view: SceneCameraView::oriented(placement.position, focus, rotation),
        obstruction: Some(CameraObstructionObservation {
            desired_metres: frame.distance,
            resolved_metres: boom.distance,
            hit: placement.hit.is_some(),
        }),
    })
}
