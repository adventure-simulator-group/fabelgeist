//! Distance and frustum culling of character pose evaluation.
//!
//! A rig that leaves the cull distance or the camera frustum stops being
//! advanced, and `apply_pose_buffers` stops writing it. The procedural passes
//! downstream share that decision through [`PoseCulled`] so an off-screen
//! character is filtered out of their queries rather than being solved for a
//! pose nothing can see.

use bevy::prelude::*;

use super::{PoseBufferMetrics, PoseBufferRig};

/// Marks a character whose pose evaluation was culled.
#[derive(Component)]
pub(in crate::animation) struct PoseCulled;

/// Applies this frame's cull decision to `rig`, maintaining [`PoseCulled`] and
/// resetting the interpolation state a rig carries back into view. Returns
/// `culled`.
pub(super) fn cull_pose_rig(
    rig: &mut PoseBufferRig,
    owner: Entity,
    culled: bool,
    commands: &mut Commands<'_, '_>,
    metrics: &mut PoseBufferMetrics,
) -> bool {
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
