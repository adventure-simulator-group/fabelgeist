//! Conversion of captured poses to local-frame jitter measurements.

use super::*;

pub(in crate::animation_viewer) fn jitter_frames(
    frames: &[FrameSample],
) -> Vec<JitterFrame<BoneRole>> {
    let mut previous: Option<&FrameSample> = None;
    let mut analysis_segment = 0_u64;
    frames
        .iter()
        // A ragdoll is intentionally non-smooth at the animation-to-physics
        // handoff and does not obey authored locomotion jerk thresholds.
        .filter(|frame| !frame.scenario.is(ScenarioId::FullRagdoll))
        .map(|frame| {
            if let Some(previous_frame) = previous {
                let action_transition = previous_frame.action != frame.action;
                let landing = previous_frame.landing_sequence != frame.landing_sequence;
                let foot_contact = previous_frame.contact_sequence != frame.contact_sequence;
                let guard_stop_handoff = is_guard_stop_transition(&frame.scenario)
                    && previous_frame.speed_metres_per_second
                        > SignedSpeed::from(STATIONARY_ANIMATION_SPEED_LIMIT)
                    && frame.speed_metres_per_second
                        <= SignedSpeed::from(STATIONARY_ANIMATION_SPEED_LIMIT);
                let quickstep_takeoff = is_quickstep_scenario(&frame.scenario)
                    && previous_frame.left_support_weight.fraction() >= FULL_PLANT_SUPPORT_WEIGHT
                    && previous_frame.right_support_weight.fraction() >= FULL_PLANT_SUPPORT_WEIGHT
                    && frame.left_support_weight.fraction() <= 0.01
                    && frame.right_support_weight.fraction() <= 0.01;
                if previous_frame.scenario != frame.scenario {
                    analysis_segment = 0;
                } else if action_transition
                    || landing
                    || foot_contact
                    || guard_stop_handoff
                    || quickstep_takeoff
                {
                    analysis_segment = analysis_segment.wrapping_add(1);
                }
            }
            previous = Some(frame);
            let mut missing_parent_bones = Vec::new();
            let bones = frame
                .bones
                .iter()
                .filter_map(|(name, bone)| {
                    let parent = match parent_bone(name) {
                        Some(parent_name) => match frame.bones.get(parent_name) {
                            Some(parent) => Some(parent),
                            None => {
                                missing_parent_bones.push(*name);
                                return None;
                            }
                        },
                        None => None,
                    };
                    let position = if *name == BoneRole::Pelvis {
                        // The capture root is authoritative locomotion, not a joint.
                        Vec3::ZERO
                    } else {
                        Vec3::from_array(bone.position)
                    };
                    let rotation = Quat::from_array(bone.rotation_xyzw);
                    let (position, rotation) = parent.map_or((position, rotation), |parent| {
                        let parent_position = Vec3::from_array(parent.position);
                        let parent_rotation = Quat::from_array(parent.rotation_xyzw);
                        (
                            parent_rotation.inverse() * (position - parent_position),
                            parent_rotation.inverse() * rotation,
                        )
                    });
                    Some((
                        *name,
                        JitterBone {
                            position: position.to_array(),
                            rotation_xyzw: rotation.to_array(),
                        },
                    ))
                })
                .collect();
            JitterFrame {
                scenario: frame.scenario.clone(),
                analysis_segment,
                scenario_frame: frame.scenario_frame,
                time_seconds: frame.time_seconds,
                bones,
                missing_parent_bones,
            }
        })
        .collect()
}
