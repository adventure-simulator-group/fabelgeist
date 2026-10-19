//! Authored, run-flight and stop-settle foot-height geometry kernels.

use super::*;

/// Conform only upward from the authored pose, then apply the existing solve weight.
pub(in crate::animation::procedural) fn ordinary_terrain_target(
    foot_position: Vec3,
    height: f32,
    weight: f32,
) -> Vec3 {
    // Pose-buffer terrain conformity owns the upcoming sample. This
    // compatibility solve may lift the rendered sample further when
    // terrain rises, but must never lower the authored flat-ground
    // clearance toward a fixed ankle offset.
    let terrain_target = foot_position.with_y(foot_position.y.max(
        height + measured_ankle_sole_offset_metres().metres()
            - ik_tuning().sole_contact_margin_metres.metres(),
    ));
    foot_position.lerp(terrain_target, weight.clamp(0.0, 1.0))
}

pub(in crate::animation::procedural) fn run_swing_clearance(
    phase_to_contact: CycleDelta,
    planned_progress: Option<f32>,
) -> f32 {
    if let Some(progress) = planned_progress {
        let progress = progress.clamp(0.0, 1.0);
        ik_tuning().run_swing_minimum_sole_clearance_metres.metres() * (1.0 - progress)
            + (std::f32::consts::PI * progress).sin()
                * ik_tuning().run_swing_sole_clearance_metres.metres()
    } else {
        let progress = (1.0 - phase_to_contact.cycles()).clamp(0.0, 1.0);
        ik_tuning().run_swing_minimum_sole_clearance_metres.metres()
            + (std::f32::consts::PI * progress).sin()
                * ik_tuning().run_swing_sole_clearance_metres.metres()
    }
}

pub(in crate::animation::procedural) fn run_airborne_clearance(
    phase_to_contact: CycleDelta,
    planned_progress: Option<f32>,
    support_eligible_for_descent: bool,
) -> f32 {
    let clearance = run_swing_clearance(phase_to_contact, planned_progress);
    if support_eligible_for_descent {
        clearance
    } else {
        clearance.max(ik_tuning().run_swing_minimum_sole_clearance_metres.metres())
    }
}

pub(in crate::animation::procedural) fn run_airborne_clearance_for_sample(
    just_released: bool,
    phase_to_contact: CycleDelta,
    planned_progress: Option<f32>,
    support_eligible_for_descent: bool,
) -> f32 {
    if just_released {
        // Toe-off spends its first sample establishing the semantic flight
        // floor. Adding the phase swing arc here requested ~9.6 cm of vertical
        // clearance on the captured uphill edge, leaving no terrain-valid
        // point inside the visible foot budget. Later samples build the arc.
        ik_tuning().run_swing_minimum_sole_clearance_metres.metres()
    } else {
        run_airborne_clearance(
            phase_to_contact,
            planned_progress,
            support_eligible_for_descent,
        )
    }
}

pub(in crate::animation::procedural) fn run_clearance_target_height(
    current_target_y: f32,
    required_height: f32,
    support_eligible_for_descent: bool,
) -> f32 {
    if support_eligible_for_descent {
        // The target may already be resting on the semantic 5 cm flight
        // floor. Once contact becomes eligible, that old raised target is not
        // a lower bound: explicitly request the contact-height descent and let
        // the owner-local follower bound the resulting step.
        required_height
    } else {
        current_target_y.max(required_height)
    }
}

pub(in crate::animation::procedural) fn settle_swing_target(
    start: Vec3,
    landing: Vec3,
    progress: f32,
) -> Vec3 {
    let progress = progress.clamp(0.0, 1.0);
    let horizontal = smoothstep(0.0, 1.0, progress);
    let mut target = start.lerp(landing, horizontal);
    target.y +=
        (std::f32::consts::PI * progress).sin() * ik_tuning().settle_step_clearance_metres.metres();
    target
}

pub(in crate::animation::procedural) fn toe_aware_minimum_ankle_y(
    rendered_ankle: Vec3,
    rendered_toe: Vec3,
    desired_ankle_xz: Vec2,
    minimum_toe_clearance: f32,
    terrain_height_at: impl Fn(Vec2) -> Option<f32>,
) -> Option<f32> {
    let ankle_clearance = rendered_ankle.y - terrain_height_at(rendered_ankle.xz())?;
    let toe_clearance = rendered_toe.y - terrain_height_at(rendered_toe.xz())?;
    let ankle_above_toe = ankle_clearance - toe_clearance;
    let desired_height = terrain_height_at(desired_ankle_xz)?;
    Some(desired_height + ankle_above_toe + minimum_toe_clearance)
        .filter(|height| height.is_finite())
}

pub(in crate::animation::procedural) fn transition_toe_clearance_with_rotation_margin(
    rendered_ankle: Vec3,
    rendered_toe: Vec3,
    delta_seconds: AnimationElapsed,
) -> f32 {
    // The cached foot chain may rotate by up to nine degrees after the ankle
    // target is selected. Reserve the maximum vertical motion of the visible
    // ankle-to-toe lever so a target that was toe-safe before finalization is
    // still toe-safe in the propagated pose.
    let angular_step = (ik_tuning().airborne_foot_rotation_speed_degrees_per_second
        * delta_seconds.seconds())
    .min(90.0)
    .to_radians();
    ik_tuning()
        .terrain_transition_flight_toe_clearance_metres
        .metres()
        + rendered_ankle.distance(rendered_toe) * angular_step.sin()
}

#[cfg(test)]
mod tests {
    use super::settle_swing_target;
    use bevy::prelude::Vec3;

    #[test]
    fn settle_swing_leaves_and_returns_to_ground_only_at_contact() {
        let start = Vec3::new(-0.12, 0.085, 0.25);
        let landing = Vec3::new(-0.12, 0.085, -0.35);
        assert_eq!(settle_swing_target(start, landing, 0.0), start);
        assert!(settle_swing_target(start, landing, 0.5).y > start.y + 0.09);
        assert!(settle_swing_target(start, landing, 0.75).z < start.z);
        assert!(settle_swing_target(start, landing, 1.0).abs_diff_eq(landing, 0.0001));
    }
}
