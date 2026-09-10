//! Stateless obstruction captures use the gameplay placement and projection.

use super::*;
use crate::camera::collision::{BoomRecovery, CameraFrame, CameraVolume};

pub(super) fn animation_play_obstruction_camera(
    state: &SceneCaptureState,
    spatial: &SpatialQuery,
    projection: &Projection,
    yaw_degrees: f32,
) -> (Transform, Vec3, Option<CameraObstructionObservation>) {
    let config = CameraRigConfig::default();
    let Some(tree) = state.tree_focus else {
        let (transform, target) =
            camera_for_view(CapturePose::AnimationPlay { yaw_degrees }, state);
        return (
            transform,
            target,
            Some(CameraObstructionObservation {
                desired_metres: config.lowered.distance,
                resolved_metres: config.lowered.distance,
                hit: false,
            }),
        );
    };
    let yaw = Quat::from_rotation_y(yaw_degrees.to_radians());
    let outward = yaw * Vec3::Z;
    let tree_root_y = tree.y - TREE_TRUNK_HEIGHT_METRES * 0.5;
    let focus = Vec3::new(tree.x, tree_root_y + 1.35, tree.z) + outward * 0.95;
    let rotation = Transform::IDENTITY.looking_to(outward, Vec3::Y).rotation;
    resolve(spatial, projection, focus, rotation)
}

pub(super) fn animation_play_boundary_camera(
    projection: &Projection,
    terrain: &SceneTerrain,
    spatial: &SpatialQuery,
    player_x: f32,
    player_z: f32,
    yaw_degrees: f32,
) -> (Transform, Vec3, Option<CameraObstructionObservation>) {
    let focus = Vec3::new(
        player_x,
        terrain
            .height_at(Vec2::new(player_x, player_z))
            .unwrap_or_default()
            + 1.48,
        player_z,
    );
    resolve(
        spatial,
        projection,
        focus,
        Quat::from_rotation_y(yaw_degrees.to_radians()),
    )
}

fn resolve(
    spatial: &SpatialQuery,
    projection: &Projection,
    focus: Vec3,
    rotation: Quat,
) -> (Transform, Vec3, Option<CameraObstructionObservation>) {
    let config = CameraRigConfig::default();
    let frame = CameraFrame {
        origin: focus - Vec3::Y * config.lowered.focus_height,
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
        &CameraVolume::new(projection, config.collision_margin),
        spatial,
        &SpatialQueryFilter::default(),
        &|_| true,
        &config,
        0.0,
    );
    (
        Transform::from_translation(placement.position).with_rotation(rotation),
        focus,
        Some(CameraObstructionObservation {
            desired_metres: frame.distance,
            resolved_metres: boom.distance,
            hit: placement.hit.is_some(),
        }),
    )
}
