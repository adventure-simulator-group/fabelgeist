//! Phase-aware sagittal run-foot roll over the canonical bilateral contact.

use super::*;

/// Phase-aware sagittal foot roll for running. Negative phase is the approach
/// to this foot's contact and positive phase is its stance/release. The curve
/// arrives with a modest dorsiflexed heel presentation, flattens early in
/// stance, then plantar-flexes into toe-off before returning to neutral during
/// swing. Terrain-normal alignment remains the base orientation.
pub(super) fn run_foot_roll_degrees(skeleton: &SkeletonState, foot_side: LeadFoot) -> f32 {
    if locomotion_profile(skeleton).gait != LocomotionGait::Run
        || skeleton.action_kind() != SkeletonAction::None
        || skeleton.weapon_guard() != WeaponGuardState::Lowered
        || skeleton.animation_speed() <= 0.05
    {
        return 0.0;
    }
    let signed = skeleton
        .gait_phase
        .nearest_contact_offset(foot_side)
        .cycles();
    let radius = locomotion_profile(skeleton).support_phase_radius;
    if signed < -radius {
        // Prepare the heel during the latter half of flight.
        8.0 * smoothstep(-0.25, -radius, signed)
    } else if signed < -0.05 {
        8.0 * (1.0 - smoothstep(-radius, -0.05, signed))
    } else if signed <= 0.06 {
        0.0
    } else if signed <= radius {
        -8.0 * smoothstep(0.06, radius, signed)
    } else {
        // Release the toe smoothly instead of carrying a pointed foot through
        // the whole airborne arc.
        -8.0 * (1.0 - smoothstep(radius, 0.25, signed))
    }
}

#[cfg(test)]
mod tests {
    use super::run_foot_roll_degrees;
    use adventuresim_tactical_core::animation::{CyclePhase, LeadFoot, SkeletonState};
    use bevy::prelude::Vec3;

    #[test]
    fn run_foot_roll_has_heel_flat_and_toe_off_beats() {
        let mut run = SkeletonState::default();
        run.local_velocity = Vec3::new(0.0, 0.0, -5.5);
        run.world_velocity = Vec3::new(0.0, 0.0, -5.5);
        run.gait_phase = CyclePhase::from_cycles(0.84).unwrap();
        assert!(
            run_foot_roll_degrees(&run, LeadFoot::Left) > 0.0,
            "heel prepares"
        );
        run.gait_phase = CyclePhase::from_cycles(0.0).unwrap();
        assert_eq!(
            run_foot_roll_degrees(&run, LeadFoot::Left),
            0.0,
            "flat stance"
        );
        run.gait_phase = CyclePhase::from_cycles(0.15).unwrap();
        assert!(run_foot_roll_degrees(&run, LeadFoot::Left) < 0.0, "toe off");
        run.gait_phase = CyclePhase::from_cycles(0.25).unwrap();
        assert_eq!(
            run_foot_roll_degrees(&run, LeadFoot::Left),
            0.0,
            "neutral swing"
        );
        run.gait_phase = CyclePhase::from_cycles(0.5).unwrap();
        assert_eq!(
            run_foot_roll_degrees(&run, LeadFoot::Right),
            0.0,
            "mirrored contact"
        );
    }
}
