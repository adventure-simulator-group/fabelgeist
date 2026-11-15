//! A camera over the shoulder of this player's fighter, turned by the look.

use bevy::prelude::*;
use bevy_netahoy::OwnVisual;

use super::input::Look;

/// Where the camera sits from the fighter's centre, before the look turns it (m).
const OFFSET: Vec3 = Vec3::new(0.55, 1.0, 3.4);
/// The point it looks at, above the fighter's centre (m), and how far ahead of it.
const AIM_HEIGHT: f32 = 0.5;
const AIM_AHEAD: f32 = 2.0;
/// The look's pitch tilts the camera by this share of itself, so looking down still frames the
/// fighter.
const PITCH_SHARE: f32 = 0.5;

/// Puts the camera behind this player's fighter.
pub(super) fn follow(
    look: Res<Look>,
    fighter: Single<&Transform, (With<OwnVisual>, Without<Camera3d>)>,
    mut camera: Single<&mut Transform, With<Camera3d>>,
) {
    let turn = Quat::from_euler(EulerRot::YXZ, look.yaw, look.pitch * PITCH_SHARE, 0.0);
    let centre = fighter.translation;
    camera.translation = centre + turn * OFFSET;
    let aim = centre + Vec3::Y * AIM_HEIGHT + turn * Vec3::NEG_Z * AIM_AHEAD;
    camera.look_at(aim, Vec3::Y);
}
