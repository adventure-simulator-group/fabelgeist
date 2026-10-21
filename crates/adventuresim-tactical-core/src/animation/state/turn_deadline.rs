//! Deadline-based planar body facing.
use bevy::math::{Quat, Vec3, Vec3Swizzles};

/// Angular speed required to reach a planar facing by a deadline without an
/// instantaneous turn. Re-evaluating this as the target moves still lands on
/// the live heading at canonical contact.
pub fn body_turn_speed_for_deadline(
    current: Quat,
    desired_forward: Vec3,
    remaining_seconds: f32,
    delta_seconds: f32,
) -> f32 {
    let Some(desired) = desired_forward.xz().try_normalize() else {
        return 0.0;
    };
    let current = (current * Vec3::Z).xz().normalize_or_zero();
    let duration = remaining_seconds.max(delta_seconds);
    if duration <= 0.0 {
        return 0.0;
    }
    current.angle_to(desired).abs() / duration
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn paused_turns_do_not_advance_and_small_deadlines_remain_meaningful() {
        assert_eq!(
            body_turn_speed_for_deadline(Quat::IDENTITY, Vec3::X, 0.0, 0.0),
            0.0
        );
        let rate = body_turn_speed_for_deadline(Quat::IDENTITY, Vec3::X, 1.0e-10, 0.0);
        assert!(rate.is_finite() && rate > 1.0e10);
        assert_eq!(
            body_turn_speed_for_deadline(Quat::IDENTITY, Vec3::ZERO, 1.0, 1.0),
            0.0
        );
    }
}
