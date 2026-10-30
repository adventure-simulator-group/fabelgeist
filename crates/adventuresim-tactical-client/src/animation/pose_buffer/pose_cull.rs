//! Distance and frustum culling of character pose evaluation.
//!
//! A rig that leaves the cull distance or the camera frustum stops being
//! advanced, and `apply_pose_buffers` stops writing it. The procedural passes
//! downstream share that decision through [`PoseCulled`] so an off-screen
//! character is filtered out of their queries rather than being solved for a
//! pose nothing can see.

use bevy::prelude::*;

use super::{PoseBufferMetrics, PoseBufferRig, PoseVisibility};

/// Marks a character whose pose evaluation was culled.
#[derive(Component)]
pub(in crate::animation) struct PoseCulled;

/// Applies this frame's cull decision to `rig`, maintaining [`PoseCulled`]
/// alongside the rig's own visibility. Returns `culled`.
pub(super) fn cull_pose_rig(
    rig: &mut PoseBufferRig,
    owner: Entity,
    culled: bool,
    commands: &mut Commands<'_, '_>,
    metrics: &mut PoseBufferMetrics,
) -> bool {
    if culled {
        rig.visibility = PoseVisibility::Frozen;
        commands.entity(owner).insert(PoseCulled);
        metrics.culled_character_count += 1;
        return true;
    }
    if rig.visibility == PoseVisibility::Frozen {
        commands.entity(owner).remove::<PoseCulled>();
    }
    rig.resume_visible_sampling();
    false
}
