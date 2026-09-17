//! Distance and frustum culling of character pose evaluation.
//!
//! A rig that leaves the cull distance or the camera frustum stops being
//! advanced, and `apply_pose_buffers` stops writing it. The procedural passes
//! downstream share that decision through [`PoseCulled`] so an off-screen
//! character is filtered out of their queries rather than being solved for a
//! pose nothing can see.

use bevy::camera::primitives::{Frustum, Sphere};
use bevy::prelude::*;

use super::{PoseBufferMetrics, PoseBufferRig, pose_tuning};

/// Marks a character whose pose evaluation was culled.
#[derive(Component)]
pub(in crate::animation) struct PoseCulled;

/// Returns whether `rig` is culled this frame, maintaining [`PoseCulled`] and
/// resetting the interpolation state a rig carries back into view.
pub(super) fn cull_pose_rig(
    rig: &mut PoseBufferRig,
    owner: Entity,
    position: Vec3,
    camera: Option<(&GlobalTransform, &Frustum)>,
    commands: &mut Commands<'_, '_>,
    metrics: &mut PoseBufferMetrics,
) -> bool {
    let culled = camera.is_some_and(|(camera_transform, frustum)| {
        position.distance_squared(camera_transform.translation())
            > pose_tuning().cull_distance_metres * pose_tuning().cull_distance_metres
            || !frustum.intersects_sphere(
                &Sphere {
                    center: position.into(),
                    radius: pose_tuning().cull_radius_metres,
                },
                false,
            )
    });
    if culled {
        rig.frozen = true;
        commands.entity(owner).insert(PoseCulled);
        metrics.culled_character_count += 1;
        return true;
    }
    if rig.frozen {
        commands.entity(owner).remove::<PoseCulled>();
        rig.sample_accumulator = 0.0;
        rig.interpolation_alpha = 0.0;
        rig.target_linear_velocities.fill(Vec3::ZERO);
        rig.target_angular_velocities.fill(Vec3::ZERO);
    }
    rig.frozen = false;
    false
}
