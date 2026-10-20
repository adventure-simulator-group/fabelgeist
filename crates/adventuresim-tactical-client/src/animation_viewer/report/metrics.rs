//! Animation review metric policy and calculations.

use super::*;
use adventuresim_tactical_core::animation::STATIONARY_ANIMATION_SPEED_LIMIT;
pub(super) use catastrophic_contacts::catastrophic_foot_displacement;
use height_peaks::prominent_height_peaks;
use support_slip::supported_foot_slip;

mod catastrophic_contacts;
mod height_peaks;
mod support_slip;

pub(super) const RAISED_MINIMUM_INTER_FOOT_SEPARATION_METRES: f32 = 0.16;
pub(super) const RAISED_MAXIMUM_PELVIS_VERTICAL_STEP_METRES: f32 = 0.05;
// The parent guard-entry fixture already moves 33.13 mm in one sampled frame.
// Allow less than 4 mm of additional height-system overhead without weakening
// the broader raised-guard continuity bound.
pub(super) const LOCOMOTION_STATE_MAXIMUM_PELVIS_VERTICAL_STEP_METRES: f32 = 0.04;
pub(super) const ORDINARY_VERTICAL_RANGE_LIMIT_METRES: f32 = 0.20;
pub(super) const RAISED_GUARD_VERTICAL_RANGE_LIMIT_METRES: f32 = 0.30;

pub(super) const MINIMUM_GUARD_SWING_TRAVEL_METRES: f32 = 0.05;
pub(super) const MINIMUM_GUARD_SWING_CLEARANCE_GAIN_METRES: f32 = 0.03;

#[derive(Debug, Default)]
pub(super) struct GuardStepLivenessMetrics {
    required: bool,
    completed_half_steps: usize,
    visible_half_steps: usize,
    minimum_swing_travel_metres: f32,
    minimum_swing_clearance_gain_metres: f32,
}

/// Native report measurements collected at complete captured gait seams.
#[derive(Default)]
struct LoopSeamMetrics {
    position_metres: f32,
    rotation_degrees: f32,
}

impl GuardStepLivenessMetrics {
    fn is_required(frames: &[&FrameSample]) -> bool {
        frames.first().is_some_and(|frame| {
            let metadata = frame.scenario.clone();
            metadata.kind() == ScenarioKind::RaisedGuard
                && metadata.procedural_solver()
                && metadata.repeatable()
                && frames.iter().all(|frame| {
                    frame.speed_metres_per_second
                        > SignedSpeed::from(STATIONARY_ANIMATION_SPEED_LIMIT)
                })
        })
    }
}

impl LoopSeamMetrics {
    fn measure(metadata: &ScenarioMetadata<'_>, frames: &[&FrameSample]) -> Option<Self> {
        if !expects_loop_seam(metadata) {
            return None;
        }
        let seams = frames
            .windows(2)
            .filter(|pair| (pair[1].gait_phase.cycles() - pair[0].gait_phase.cycles()).abs() > 0.5)
            .map(|pair| loop_seam(pair[0], pair[1]))
            .collect::<Vec<_>>();
        if seams.is_empty() {
            return None;
        }
        Some(Self {
            position_metres: seams.iter().map(|seam| seam.0).fold(0.0, f32::max),
            rotation_degrees: seams.iter().map(|seam| seam.1).fold(0.0, f32::max),
        })
    }
}

pub(super) fn both_feet_behind_hips(frames: &[FrameSample]) -> bool {
    let mut duration = 0.0_f32;
    for pair in frames.windows(2) {
        let [before, after] = pair else {
            continue;
        };
        if before.scenario != after.scenario
            || after.speed_metres_per_second <= SignedSpeed::from(STATIONARY_ANIMATION_SPEED_LIMIT)
        {
            duration = 0.0;
            continue;
        }
        let Some(direction) = Vec3::from_array(after.world_travel_direction).try_normalize() else {
            duration = 0.0;
            continue;
        };
        let behind = [
            (BoneRole::ThighLeft, BoneRole::FootLeft),
            (BoneRole::ThighRight, BoneRole::FootRight),
        ]
        .into_iter()
        .all(|(hip, foot)| {
            let Some(hip) = after.bones.get(hip) else {
                return false;
            };
            let Some(foot) = after.bones.get(foot) else {
                return false;
            };
            (Vec3::from_array(foot.position) - Vec3::from_array(hip.position)).dot(direction)
                < -0.02
        });
        if behind {
            duration += (after.time_seconds.seconds() - before.time_seconds.seconds()).max(0.0);
            if duration >= 0.3 {
                return true;
            }
        } else {
            duration = 0.0;
        }
    }
    false
}

pub(super) fn scenario_metrics(
    frames: &[FrameSample],
) -> AnimationValueResult<Vec<ScenarioMetrics>> {
    let mut grouped = BTreeMap::<&ScenarioMetadata<'static>, Vec<&FrameSample>>::new();
    for frame in frames {
        grouped.entry(&frame.scenario).or_default().push(frame);
    }
    grouped
        .into_iter()
        .map(|(scenario, frames)| {
            let metadata = scenario;
            let metric_frames = metric_frames_after_warmup(metadata, &frames);
            let procedural_frames = metric_frames
                .iter()
                .copied()
                .filter(|frame| metadata.kind() == ScenarioKind::Terrain || frame.guard_action)
                .collect::<Vec<_>>();
            let mut maximum_step = 0.0_f32;
            let mut maximum_leg_step = 0.0_f32;
            let mut maximum_foot_step = 0.0_f32;
            let mut maximum_knee_step = 0.0_f32;
            let mut maximum_rotation = 0.0_f32;
            let mut maximum_foot_rotation = 0.0_f32;
            let mut maximum_slip = 0.0_f32;
            let mut worst_displacement = None;
            let mut worst_rotation = None;
            // Frame zero establishes retained procedural state from a fresh
            // component. The scenario pre-roll begins at frame one; continuity
            // gates therefore start with the first real pre-roll transition
            // (1→2) without exempting any movement frame.
            for pair in metric_frames.windows(2).skip(1) {
                let (before, after) = (pair[0], pair[1]);
                for (name, before_bone) in &before.bones {
                    let Some(after_bone) = after.bones.get(*name) else {
                        continue;
                    };
                    let displacement = body_local(after, *name)
                        .zip(body_local(before, *name))
                        .map_or(f32::INFINITY, |(after, before)| after.distance(before));
                    if displacement > maximum_step {
                        maximum_step = displacement;
                        worst_displacement = Some(ContinuityLocation {
                            bone: *name,
                            from_frame: before.scenario_frame,
                            to_frame: after.scenario_frame,
                            value: displacement,
                        });
                    }
                    if is_leg_bone(name) {
                        maximum_leg_step = maximum_leg_step.max(displacement);
                    }
                    if is_foot_bone(name) {
                        maximum_foot_step = maximum_foot_step.max(displacement);
                    }
                    if is_knee_bone(name) {
                        maximum_knee_step = maximum_knee_step.max(displacement);
                    }
                    let rotation = body_local_rotation(after, after_bone)
                        .angle_between(body_local_rotation(before, before_bone))
                        .to_degrees();
                    if is_foot_bone(name) {
                        maximum_foot_rotation = maximum_foot_rotation.max(rotation);
                    }
                    if rotation > maximum_rotation {
                        maximum_rotation = rotation;
                        worst_rotation = Some(ContinuityLocation {
                            bone: *name,
                            from_frame: before.scenario_frame,
                            to_frame: after.scenario_frame,
                            value: rotation,
                        });
                    }
                }
                maximum_slip =
                    maximum_slip.max(supported_foot_slip(metadata, before, after)?.metres());
            }
            let maximum_plant_drift = maximum_planted_foot_drift(metadata, &metric_frames);
            let seam = LoopSeamMetrics::measure(metadata, &frames);
            let (visual_height_peak_count, visual_height_peaks_in_passing_windows) =
                visual_height_peaks(&metric_frames);
            let guard_liveness = guard_step_liveness_metrics(&metric_frames);
            let contact_sole_range = contact_sole_clearance_range(&metric_frames)?;
            Ok(ScenarioMetrics {
                scenario: (*scenario).clone(),
                frame_count: frames.len(),
                maximum_root_relative_step_metres: maximum_step,
                maximum_leg_root_relative_step_metres: maximum_leg_step,
                maximum_foot_root_relative_step_metres: maximum_foot_step,
                maximum_knee_root_relative_step_metres: maximum_knee_step,
                worst_displacement,
                maximum_bone_rotation_step_degrees: maximum_rotation,
                maximum_foot_rotation_step_degrees: maximum_foot_rotation,
                worst_rotation,
                loop_seam_position_metres: seam.as_ref().map(|seam| seam.position_metres),
                loop_seam_rotation_degrees: seam.as_ref().map(|seam| seam.rotation_degrees),
                pelvis_vertical_range_metres: root_relative_vertical_range(
                    &metric_frames,
                    BoneRole::Pelvis,
                ),
                maximum_pelvis_vertical_step_metres: root_relative_vertical_step(
                    &metric_frames,
                    BoneRole::Pelvis,
                ),
                controller_vertical_range_metres: controller_vertical_range(&metric_frames),
                phase_height_range_metres: phase_height_range(&metric_frames),
                contact_to_passing_height_gain_metres: contact_to_passing_height_gain(
                    &metric_frames,
                ),
                visual_height_peak_count,
                visual_height_peaks_in_passing_windows,
                maximum_no_support_seconds: maximum_no_support_seconds(&metric_frames),
                minimum_flight_sole_clearance_metres: minimum_flight_sole_clearance(&metric_frames),
                minimum_contact_sole_clearance_metres: contact_sole_range.0,
                maximum_contact_sole_clearance_metres: contact_sole_range.1,
                minimum_flight_toe_clearance_metres: minimum_toe_clearance(&metric_frames, false)?,
                minimum_contact_toe_clearance_metres: minimum_toe_clearance(&metric_frames, true)?,
                head_vertical_range_metres: root_relative_vertical_range(
                    &metric_frames,
                    BoneRole::Head,
                ),
                foot_terrain_relief_metres: foot_terrain_relief(&metric_frames),
                minimum_knee_forward_bend_metres: minimum_knee_bend(&metric_frames),
                minimum_signed_foot_track_metres: minimum_signed_foot_track(&metric_frames),
                minimum_inter_foot_separation_metres: minimum_inter_foot_separation(&metric_frames),
                minimum_knee_flexion_degrees: minimum_knee_flexion(&procedural_frames),
                minimum_knee_hemisphere_dot: minimum_knee_hemisphere(&procedural_frames),
                maximum_knee_foot_yaw_offset_degrees: maximum_knee_foot_yaw_offset(
                    &procedural_frames,
                ),
                maximum_facing_motion_error_degrees: maximum_facing_error(&metric_frames),
                maximum_facing_tracking_excess_degrees: maximum_facing_tracking_excess(
                    &metric_frames,
                ),
                maximum_guard_facing_error_degrees: maximum_guard_facing_error(&metric_frames),
                final_facing_motion_error_degrees: final_facing_error(&metric_frames),
                maximum_dive_axis_motion_error_degrees: maximum_dive_axis_motion_error(
                    &metric_frames,
                ),
                maximum_supported_foot_slip_metres_per_frame: maximum_slip,
                maximum_planted_foot_drift_metres: maximum_plant_drift,
                guard_step_liveness_required: guard_liveness.required,
                completed_guard_half_step_count: guard_liveness.completed_half_steps,
                visible_guard_half_step_count: guard_liveness.visible_half_steps,
                minimum_guard_swing_travel_metres: guard_liveness.minimum_swing_travel_metres,
                minimum_guard_swing_clearance_gain_metres: guard_liveness
                    .minimum_swing_clearance_gain_metres,
                minimum_foot_clearance_metres: minimum_foot_clearance(&metric_frames),
            })
        })
        .collect()
}

pub(super) fn quat(bone: &BoneSample) -> Quat {
    Quat::from_array(bone.rotation_xyzw).normalize()
}

pub(super) fn body_local(frame: &FrameSample, bone: BoneRole) -> Option<Vec3> {
    let world = Vec3::from_array(frame.bones.get(bone)?.position)
        - Vec3::from_array(frame.root_position_metres);
    Some(Quat::from_array(frame.body_rotation_xyzw).inverse() * world)
}

pub(super) fn guard_step_liveness_metrics(frames: &[&FrameSample]) -> GuardStepLivenessMetrics {
    let required = GuardStepLivenessMetrics::is_required(frames);
    if !required {
        return GuardStepLivenessMetrics::default();
    }

    let mut interval_start = 0;
    let mut completed_half_steps = 0;
    let mut visible_half_steps = 0;
    let mut minimum_swing_travel = f32::INFINITY;
    let mut minimum_swing_clearance_gain = f32::INFINITY;

    for interval_end in 1..frames.len() {
        if frames[interval_end].contact_sequence == frames[interval_end - 1].contact_sequence {
            continue;
        }

        let start = frames[interval_start];
        let end = frames[interval_end];
        let left_gain = end.left_support_weight.fraction() - start.left_support_weight.fraction();
        let right_gain =
            end.right_support_weight.fraction() - start.right_support_weight.fraction();
        let (
            swing_foot,
            start_swing_support,
            end_swing_support,
            start_other_support,
            end_other_support,
        ) = if left_gain >= right_gain {
            (
                BoneRole::FootLeft,
                start.left_support_weight.fraction(),
                end.left_support_weight.fraction(),
                start.right_support_weight.fraction(),
                end.right_support_weight.fraction(),
            )
        } else {
            (
                BoneRole::FootRight,
                start.right_support_weight.fraction(),
                end.right_support_weight.fraction(),
                start.left_support_weight.fraction(),
                end.left_support_weight.fraction(),
            )
        };

        completed_half_steps += 1;
        let support_swap_valid = start_swing_support <= 0.25
            && end_swing_support >= 0.75
            && start_other_support >= 0.75
            && end_other_support <= 0.25;
        let travel = start
            .bones
            .get(swing_foot)
            .zip(end.bones.get(swing_foot))
            .map_or(0.0, |(start, end)| {
                (Vec3::from_array(end.position) - Vec3::from_array(start.position))
                    .xz()
                    .length()
            });
        let interval = &frames[interval_start..=interval_end];
        let start_clearance = start
            .bones
            .get(swing_foot)
            .and_then(|bone| bone.terrain_clearance_metres);
        let end_clearance = end
            .bones
            .get(swing_foot)
            .and_then(|bone| bone.terrain_clearance_metres);
        let maximum_clearance = interval
            .iter()
            .filter_map(|frame| {
                frame
                    .bones
                    .get(swing_foot)
                    .and_then(|bone| bone.terrain_clearance_metres)
            })
            .fold(f32::NEG_INFINITY, f32::max);
        let clearance_gain = start_clearance
            .zip(end_clearance)
            .filter(|_| maximum_clearance.is_finite())
            .map_or(0.0, |(start, end)| maximum_clearance - start.max(end));

        minimum_swing_travel = minimum_swing_travel.min(travel);
        minimum_swing_clearance_gain = minimum_swing_clearance_gain.min(clearance_gain);
        if support_swap_valid
            && travel >= MINIMUM_GUARD_SWING_TRAVEL_METRES
            && clearance_gain >= MINIMUM_GUARD_SWING_CLEARANCE_GAIN_METRES
        {
            visible_half_steps += 1;
        }
        interval_start = interval_end;
    }

    GuardStepLivenessMetrics {
        required,
        completed_half_steps,
        visible_half_steps,
        minimum_swing_travel_metres: if minimum_swing_travel.is_finite() {
            minimum_swing_travel
        } else {
            0.0
        },
        minimum_swing_clearance_gain_metres: if minimum_swing_clearance_gain.is_finite() {
            minimum_swing_clearance_gain
        } else {
            0.0
        },
    }
}

pub(super) fn is_leg_bone(name: &BoneRole) -> bool {
    matches!(
        name,
        BoneRole::ThighLeft
            | BoneRole::ThighRight
            | BoneRole::ShinLeft
            | BoneRole::ShinRight
            | BoneRole::FootLeft
            | BoneRole::FootRight
    )
}

pub(super) fn is_foot_bone(name: &BoneRole) -> bool {
    matches!(name, BoneRole::FootLeft | BoneRole::FootRight)
}

pub(super) fn is_knee_bone(name: &BoneRole) -> bool {
    matches!(name, BoneRole::ShinLeft | BoneRole::ShinRight)
}

pub(super) fn target_body_local(frame: &FrameSample, world: Vec3) -> Vec3 {
    Quat::from_array(frame.body_rotation_xyzw).inverse()
        * (world - Vec3::from_array(frame.root_position_metres))
}

pub(super) fn body_local_rotation(frame: &FrameSample, bone: &BoneSample) -> Quat {
    Quat::from_array(frame.body_rotation_xyzw).inverse() * quat(bone)
}

pub(super) fn minimum_signed_foot_track(frames: &[&FrameSample]) -> f32 {
    frames
        .iter()
        .flat_map(|frame| {
            [
                (BoneRole::ThighLeft, BoneRole::FootLeft),
                (BoneRole::ThighRight, BoneRole::FootRight),
            ]
            .map(|(hip, foot)| {
                let side = body_local(frame, hip).map_or(1.0, |value| value.x.signum());
                body_local(frame, foot).map_or(f32::INFINITY, |value| value.x * side)
            })
        })
        .fold(f32::INFINITY, f32::min)
}

pub(super) fn minimum_inter_foot_separation(frames: &[&FrameSample]) -> f32 {
    frames
        .iter()
        .filter_map(|frame| {
            Some(
                body_local(frame, BoneRole::FootLeft)?
                    .xz()
                    .distance(body_local(frame, BoneRole::FootRight)?.xz()),
            )
        })
        .fold(f32::INFINITY, f32::min)
}

pub(super) fn minimum_knee_flexion(frames: &[&FrameSample]) -> f32 {
    frames
        .iter()
        .map(|frame| frame_minimum_knee_flexion(frame))
        .fold(f32::INFINITY, f32::min)
}

pub(super) fn frame_minimum_knee_flexion(frame: &FrameSample) -> f32 {
    [
        (BoneRole::ThighLeft, BoneRole::ShinLeft, BoneRole::FootLeft),
        (
            BoneRole::ThighRight,
            BoneRole::ShinRight,
            BoneRole::FootRight,
        ),
    ]
    .into_iter()
    .map(|(hip, knee, foot)| {
        let (Some(hip), Some(knee), Some(foot)) = (
            body_local(frame, hip),
            body_local(frame, knee),
            body_local(frame, foot),
        ) else {
            return f32::INFINITY;
        };
        180.0 - (hip - knee).angle_between(foot - knee).to_degrees()
    })
    .fold(f32::INFINITY, f32::min)
}

pub(super) fn minimum_knee_hemisphere(frames: &[&FrameSample]) -> f32 {
    frames
        .iter()
        .flat_map(|frame| {
            [
                (BoneRole::ThighLeft, BoneRole::ShinLeft, BoneRole::FootLeft),
                (
                    BoneRole::ThighRight,
                    BoneRole::ShinRight,
                    BoneRole::FootRight,
                ),
            ]
            .map(|(hip, knee, foot)| {
                let (Some(hip), Some(knee), Some(foot)) = (
                    body_local(frame, hip),
                    body_local(frame, knee),
                    body_local(frame, foot),
                ) else {
                    return f32::INFINITY;
                };
                let Some(axis) = (foot - hip).try_normalize() else {
                    return f32::INFINITY;
                };
                let Some(bend) = (knee - hip).reject_from_normalized(axis).try_normalize() else {
                    return -1.0;
                };
                let side = hip.x.signum();
                bend.dot((Vec3::Z + Vec3::X * side * 0.18).normalize())
            })
        })
        .fold(f32::INFINITY, f32::min)
}

pub(super) fn maximum_knee_foot_yaw_offset(frames: &[&FrameSample]) -> f32 {
    frames
        .iter()
        .flat_map(|frame| {
            [
                frame.ik_left_knee_foot_yaw_offset_degrees,
                frame.ik_right_knee_foot_yaw_offset_degrees,
            ]
        })
        .fold(0.0, f32::max)
}

pub(super) fn maximum_facing_error(frames: &[&FrameSample]) -> f32 {
    frames
        .iter()
        .filter(|frame| {
            frame.speed_metres_per_second > SignedSpeed::from(STATIONARY_ANIMATION_SPEED_LIMIT)
        })
        .map(|frame| {
            Vec3::from_array(frame.body_forward_direction)
                .angle_between(Vec3::from_array(frame.desired_body_forward_direction))
                .to_degrees()
        })
        .fold(0.0, f32::max)
}

pub(super) fn maximum_facing_tracking_excess(frames: &[&FrameSample]) -> f32 {
    frames
        .windows(2)
        .filter(|pair| {
            pair[1].speed_metres_per_second > SignedSpeed::from(STATIONARY_ANIMATION_SPEED_LIMIT)
        })
        .map(|pair| {
            let before = Vec3::from_array(pair[0].body_forward_direction);
            let actual = Vec3::from_array(pair[1].body_forward_direction);
            let desired = Vec3::from_array(pair[1].desired_body_forward_direction);
            let elapsed = pair[1].time_seconds.saturating_sub(pair[0].time_seconds);
            let permitted_residual = body_facing_turn().tracking_residual(before, desired, elapsed);
            (actual.angle_between(desired) - permitted_residual)
                .abs()
                .to_degrees()
        })
        .fold(0.0, f32::max)
}

pub(super) fn maximum_guard_facing_error(frames: &[&FrameSample]) -> f32 {
    frames
        .iter()
        .filter(|frame| {
            frame.guard_action
                && frame.speed_metres_per_second
                    > SignedSpeed::from(STATIONARY_ANIMATION_SPEED_LIMIT)
        })
        .map(|frame| {
            Vec3::from_array(frame.body_forward_direction)
                .angle_between(Vec3::from_array(frame.desired_body_forward_direction))
                .to_degrees()
        })
        .fold(0.0, f32::max)
}

pub(super) fn final_facing_error(frames: &[&FrameSample]) -> f32 {
    frames
        .iter()
        .rev()
        .find(|frame| {
            frame.speed_metres_per_second > SignedSpeed::from(STATIONARY_ANIMATION_SPEED_LIMIT)
        })
        .map_or(0.0, |frame| {
            Vec3::from_array(frame.body_forward_direction)
                .angle_between(Vec3::from_array(frame.desired_body_forward_direction))
                .to_degrees()
        })
}

pub(super) fn maximum_dive_axis_motion_error(frames: &[&FrameSample]) -> f32 {
    frames
        .iter()
        // Preserve the launch vector as the directional contract through the
        // complete terrain-contact recovery. Physical speed may already be
        // zero while the authored body is still resolving its landing pose.
        .filter(|frame| frame.scenario.has(ScenarioFeature::Dive))
        .filter_map(|frame| {
            let head = Vec3::from_array(frame.bones.get(BoneRole::Head)?.position);
            let pelvis = Vec3::from_array(frame.bones.get(BoneRole::Pelvis)?.position);
            let axis = Vec3::new(head.x - pelvis.x, 0.0, head.z - pelvis.z);
            // While the character is still nearly upright, the horizontal
            // projection of its long axis is too short to define a stable
            // travel heading. Begin judging once the dive has visibly tipped.
            if axis.length() < 0.25 {
                return None;
            }
            let axis = axis.normalize();
            let travel = Vec3::from_array(frame.world_travel_direction).try_normalize()?;
            Some(axis.angle_between(travel).to_degrees())
        })
        .fold(0.0, f32::max)
}

pub(super) fn loop_seam(first: &FrameSample, last: &FrameSample) -> (f32, f32) {
    first
        .bones
        .iter()
        .fold((0.0_f32, 0.0_f32), |metrics, (name, a)| {
            // Stateful foot locking intentionally changes the lower-body
            // contact solution with world position and fixed-step aliasing.
            // Its continuity is gated by per-frame plant slip/drift instead;
            // this seam metric checks the repeatable authored upper body.
            if is_leg_bone(name) {
                return metrics;
            }
            let Some(b) = last.bones.get(*name) else {
                return metrics;
            };
            (
                metrics.0.max(
                    body_local(last, *name)
                        .zip(body_local(first, *name))
                        .map_or(f32::INFINITY, |(last, first)| last.distance(first)),
                ),
                metrics.1.max(
                    body_local_rotation(first, a)
                        .angle_between(body_local_rotation(last, b))
                        .to_degrees(),
                ),
            )
        })
}

pub(super) fn root_relative_vertical_range(frames: &[&FrameSample], bone: BoneRole) -> f32 {
    let (minimum, maximum) = frames
        .iter()
        .filter_map(|frame| {
            frame
                .bones
                .get(bone)
                .map(|bone| bone.position[1] - frame.root_position_metres[1])
        })
        .fold(
            (f32::INFINITY, f32::NEG_INFINITY),
            |(minimum, maximum), value| (minimum.min(value), maximum.max(value)),
        );
    if minimum.is_finite() && maximum.is_finite() {
        maximum - minimum
    } else {
        0.0
    }
}

pub(super) fn root_relative_vertical_step(frames: &[&FrameSample], bone: BoneRole) -> f32 {
    frames
        .windows(2)
        .filter_map(|pair| {
            let previous = pair[0].bones.get(bone)?.position[1] - pair[0].root_position_metres[1];
            let current = pair[1].bones.get(bone)?.position[1] - pair[1].root_position_metres[1];
            Some((current - previous).abs())
        })
        .fold(0.0, f32::max)
}

pub(super) fn hard_stop_pelvis_vertical_step(frames: &[FrameSample]) -> Option<f32> {
    let hard_stop = frames
        .iter()
        .filter(|frame| frame.scenario.is(ScenarioId::HardStop))
        .collect::<Vec<_>>();
    if hard_stop.is_empty() {
        return None;
    }
    let Some(first_stopped) = hard_stop.iter().position(|frame| {
        frame.speed_metres_per_second <= SignedSpeed::from(STATIONARY_ANIMATION_SPEED_LIMIT)
    }) else {
        return Some(f32::INFINITY);
    };
    let transition_start = first_stopped.saturating_sub(1);
    Some(root_relative_vertical_step(
        &hard_stop[transition_start..],
        BoneRole::Pelvis,
    ))
}

pub(super) fn controller_vertical_range(frames: &[&FrameSample]) -> f32 {
    let (minimum, maximum) = frames.iter().fold(
        (f32::INFINITY, f32::NEG_INFINITY),
        |(minimum, maximum), frame| {
            let value = frame.root_position_metres[1];
            (minimum.min(value), maximum.max(value))
        },
    );
    if minimum.is_finite() && maximum.is_finite() {
        maximum - minimum
    } else {
        0.0
    }
}

pub(super) fn contact_to_passing_height_gain(frames: &[&FrameSample]) -> f32 {
    let frames = phase_validation_frames(frames);
    let phase_distance = |phase: f32, target: f32| {
        let distance = (phase - target).abs();
        distance.min(1.0 - distance)
    };
    let contact = frames
        .iter()
        .filter(|frame| {
            phase_distance(frame.gait_phase.cycles(), 0.0) <= 0.04
                || phase_distance(frame.gait_phase.cycles(), 0.5) <= 0.04
        })
        .filter_map(|frame| body_local(frame, BoneRole::Pelvis).map(|pelvis| pelvis.y))
        .fold(f32::NEG_INFINITY, f32::max);
    let passing = frames
        .iter()
        .filter(|frame| {
            phase_distance(frame.gait_phase.cycles(), 0.25) <= 0.04
                || phase_distance(frame.gait_phase.cycles(), 0.75) <= 0.04
        })
        .filter_map(|frame| body_local(frame, BoneRole::Pelvis).map(|pelvis| pelvis.y))
        .fold(f32::INFINITY, f32::min);
    if contact.is_finite() && passing.is_finite() {
        passing - contact
    } else {
        0.0
    }
}

pub(super) fn visual_height_peaks(frames: &[&FrameSample]) -> (usize, bool) {
    let warmed = phase_validation_frames(frames);
    let mut cycle_start = 0;
    let mut measurements = Vec::new();
    for cycle_end in warmed
        .windows(2)
        .enumerate()
        .filter_map(|(index, pair)| {
            (pair[1].gait_phase.cycles() < pair[0].gait_phase.cycles()).then_some(index + 1)
        })
        .chain(std::iter::once(warmed.len()))
    {
        let cycle = &warmed[cycle_start..cycle_end];
        let phase_span = cycle
            .first()
            .zip(cycle.last())
            .map_or(0.0, |(first, last)| {
                last.gait_phase.cycles() - first.gait_phase.cycles()
            });
        // Ignore a trailing partial cycle created when capture ends just after
        // a wrap; every complete post-warmup cycle remains independently gated.
        if phase_span >= 0.8 {
            let samples = cycle
                .iter()
                .filter_map(|frame| {
                    body_local(frame, BoneRole::Pelvis)
                        .map(|pelvis| (frame.gait_phase.cycles().rem_euclid(1.0), pelvis.y))
                })
                .collect::<Vec<_>>();
            measurements.push(prominent_height_peaks(&samples));
        }
        cycle_start = cycle_end;
    }
    if measurements.is_empty() {
        return (0, false);
    }
    (
        measurements
            .iter()
            .map(|(count, _)| *count)
            .max()
            .unwrap_or(0),
        measurements
            .iter()
            .all(|(count, in_windows)| *count == 2 && *in_windows),
    )
}

pub(super) fn phase_height_range(frames: &[&FrameSample]) -> f32 {
    let frames = phase_validation_frames(frames);
    root_relative_vertical_range(&frames, BoneRole::Pelvis)
}

pub(super) fn phase_validation_frames<'a>(frames: &[&'a FrameSample]) -> Vec<&'a FrameSample> {
    let after_first_wrap = frames
        .windows(2)
        .position(|pair| pair[1].gait_phase.cycles() < pair[0].gait_phase.cycles())
        .map_or(0, |index| index + 1);
    frames[after_first_wrap..].to_vec()
}

pub(super) fn maximum_no_support_seconds(frames: &[&FrameSample]) -> f32 {
    let mut maximum = 0.0_f32;
    let mut began = None;
    for frame in frames {
        if frame.left_support_weight.fraction() <= 0.001
            && frame.right_support_weight.fraction() <= 0.001
        {
            let began = began.get_or_insert(frame.time_seconds.seconds());
            maximum = maximum.max(frame.time_seconds.seconds() - *began);
        } else {
            began = None;
        }
    }
    maximum
}

pub(super) fn ordinary_swing_frame_is_owned(frame: &FrameSample) -> bool {
    if frame.scenario.kind() != ScenarioKind::Terrain
        || frame.speed_metres_per_second <= SignedSpeed::from(STATIONARY_ANIMATION_SPEED_LIMIT)
        || frame.ik_settle_progress.is_some()
    {
        return true;
    }
    [
        (
            "left",
            frame.ik_left_support_weight,
            frame.ik_left_authored_target,
            frame.ik_left_solve_target,
            frame.ik_left_planned_contact,
            frame.ik_left_release_active,
        ),
        (
            "right",
            frame.ik_right_support_weight,
            frame.ik_right_authored_target,
            frame.ik_right_solve_target,
            frame.ik_right_planned_contact,
            frame.ik_right_release_active,
        ),
    ]
    .into_iter()
    .all(
        |(_side, support, authored, solved, planned, release_active)| {
            support.is_some_and(|support| {
                support.fraction() > 0.05
                    || authored.zip(solved).is_some_and(|(authored, solved)| {
                        let authored = Vec3::from_array(authored);
                        let solved = Vec3::from_array(solved);
                        planned.is_some() || release_active || solved.distance(authored) <= 0.03
                    })
            })
        },
    )
}

pub(super) fn ordinary_unplanned_release_transition_is_valid(
    before: &FrameSample,
    after: &FrameSample,
    before_solve: Option<[f32; 3]>,
    after_solve: Option<[f32; 3]>,
    before_release_target: Option<[f32; 3]>,
    after_release_target: Option<[f32; 3]>,
) -> bool {
    before_solve
        .zip(after_solve)
        .is_some_and(|(before_solve, after_solve)| {
            let before_solve_world = Vec3::from_array(before_solve);
            let after_solve_world = Vec3::from_array(after_solve);
            let before_solve = target_body_local(before, before_solve_world);
            let after_solve = target_body_local(after, after_solve_world);
            let frozen_goal_converges = before_release_target.zip(after_release_target).is_none_or(
                |(before_goal, after_goal)| {
                    let before_goal = Vec3::from_array(before_goal);
                    let after_goal = Vec3::from_array(after_goal);
                    target_body_local(before, before_goal)
                        .distance(target_body_local(after, after_goal))
                        > 0.002
                        || after_solve_world.distance(after_goal)
                            <= before_solve_world.distance(before_goal) + 0.001
                },
            );
            let step_limit = if frame_uses_run_motion_budget(after) {
                // Run's owner-space target advances up to 8.75 cm per sample;
                // its terrain scenario independently rejects rendered foot
                // motion above 9.5 cm. Walk/settle retains the 5.5 cm budget.
                0.095
            } else {
                0.055
            };
            after_solve.distance(before_solve) <= step_limit && frozen_goal_converges
        })
}

pub(super) fn frame_uses_run_motion_budget(frame: &FrameSample) -> bool {
    let run_speed_threshold = (walk_locomotion_profile()
        .reference_speed
        .metres_per_second()
        + run_locomotion_profile().reference_speed.metres_per_second())
        * 0.5;
    frame.speed_metres_per_second.metres_per_second() >= run_speed_threshold
}

#[expect(
    clippy::too_many_arguments,
    reason = "this domain boundary names each independent input explicitly"
)]
pub(super) fn ordinary_planned_transition_is_valid(
    before: &FrameSample,
    after: &FrameSample,
    foot: BoneRole,
    before_plan: Option<[f32; 3]>,
    after_plan: Option<[f32; 3]>,
    before_solve: Option<[f32; 3]>,
    after_solve: Option<[f32; 3]>,
    after_support: UnitInterval,
    release_active: bool,
) -> bool {
    before_plan
        .zip(after_plan)
        .zip(before_solve.zip(after_solve))
        .is_none_or(|((before_plan, after_plan), (before_solve, after_solve))| {
            let before_plan = Vec3::from_array(before_plan);
            let after_plan = Vec3::from_array(after_plan);
            let before_solve_world = Vec3::from_array(before_solve);
            let after_solve_world = Vec3::from_array(after_solve);
            if frame_uses_run_motion_budget(after) {
                let plan_is_frozen = before_plan.distance(after_plan) <= 0.002;
                let atomic_acquisition_retarget = after_support.fraction() > 0.05
                    && !release_active
                    && after_solve_world.distance(after_plan) <= 0.02;
                let owned_airborne_replan = after_support.fraction() <= 0.5 && release_active;
                let rendered_step = before.bones.get(foot).zip(after.bones.get(foot)).map(
                    |(before_foot, after_foot)| {
                        target_body_local(after, Vec3::from_array(after_foot.position)).distance(
                            target_body_local(before, Vec3::from_array(before_foot.position)),
                        )
                    },
                );
                (plan_is_frozen || atomic_acquisition_retarget || owned_airborne_replan)
                    && rendered_step.is_some_and(|step| step <= 0.095)
            } else {
                before_plan.distance(after_plan) <= 0.002
                    && after_solve_world.distance(after_plan)
                        <= before_solve_world.distance(before_plan) + 0.005
            }
        })
}

pub(super) fn minimum_flight_sole_clearance(frames: &[&FrameSample]) -> f32 {
    let minimum = frames
        .iter()
        .filter(|frame| {
            frame.left_support_weight.fraction() <= 0.001
                && frame.right_support_weight.fraction() <= 0.001
        })
        .flat_map(|frame| [BoneRole::FootLeft, BoneRole::FootRight].map(|foot| (frame, foot)))
        .filter_map(|(frame, foot)| {
            frame
                .bones
                .get(foot)?
                .terrain_clearance_metres
                .map(|ankle| ankle - 0.085)
        })
        .fold(f32::INFINITY, f32::min);
    if minimum.is_finite() { minimum } else { 0.0 }
}

pub(super) fn contact_sole_clearance_range(
    frames: &[&FrameSample],
) -> AnimationValueResult<(f32, f32)> {
    let mut minimum = f32::INFINITY;
    let mut maximum = f32::NEG_INFINITY;
    for frame in frames {
        let procedural_solve_active =
            frame.ik_left_solve_target.is_some() || frame.ik_right_solve_target.is_some();
        let support = if procedural_solve_active {
            Some(frame.required_ik_support()?)
        } else {
            None
        };
        for (foot, left) in [(BoneRole::FootLeft, true), (BoneRole::FootRight, false)] {
            let is_contact = if let Some((left_support, right_support)) = support {
                (if left { left_support } else { right_support }).fraction() >= 0.95
            } else {
                let phase = frame.gait_phase.cycles().rem_euclid(1.0);
                let distance = if left {
                    phase.min(1.0 - phase)
                } else {
                    (phase - 0.5).abs()
                };
                distance <= 0.035
            };
            if is_contact
                && let Some(ankle) = frame
                    .bones
                    .get(foot)
                    .and_then(|bone| bone.terrain_clearance_metres)
            {
                let clearance = ankle - measured_ankle_sole_offset_metres().metres();
                minimum = minimum.min(clearance);
                maximum = maximum.max(clearance);
            }
        }
    }
    Ok(if minimum.is_finite() && maximum.is_finite() {
        (minimum, maximum)
    } else {
        (0.0, 0.0)
    })
}

pub(super) fn minimum_toe_clearance(
    frames: &[&FrameSample],
    contact: bool,
) -> AnimationValueResult<f32> {
    let mut minimum = f32::INFINITY;
    for frame in frames {
        if !contact
            && !transition_flight_toe_sample_is_active(
                &frame.scenario,
                frame.speed_metres_per_second,
                frame.ik_settle_progress,
            )
        {
            continue;
        }
        let (left_support, right_support) = if frame.scenario.procedural_solver() {
            frame.required_ik_support()?
        } else {
            (frame.left_support_weight, frame.right_support_weight)
        };
        for (toe, support) in [
            (BoneRole::ToeLeft, left_support),
            (BoneRole::ToeRight, right_support),
        ] {
            if (support.fraction() > 0.05) == contact
                && let Some(clearance) = frame
                    .bones
                    .get(toe)
                    .and_then(|bone| bone.terrain_clearance_metres)
            {
                minimum = minimum.min(clearance);
            }
        }
    }
    Ok(if minimum.is_finite() { minimum } else { 0.0 })
}

pub(super) fn transition_flight_toe_sample_is_active(
    scenario: &ScenarioMetadata<'_>,
    speed_metres_per_second: SignedSpeed,
    settle_progress: Option<f32>,
) -> bool {
    !matches!(
        scenario.identity(),
        Some(ScenarioId::TerrainRunFlightStop) | Some(ScenarioId::TerrainTapRestartCrossfade)
    ) || speed_metres_per_second > SignedSpeed::from(STATIONARY_ANIMATION_SPEED_LIMIT)
        || settle_progress.is_some()
}

pub(super) fn strict_transition_flight_toe_clearance_is_valid(clearance_metres: f32) -> bool {
    clearance_metres >= 0.01
}

pub(super) fn reported_support_contacts_are_valid(frames: &[FrameSample]) -> bool {
    let tolerance = sole_contact_tolerance_metres();
    frames.iter().all(|frame| {
        if is_quickstep_scenario(&frame.scenario)
            || frame.scenario.kind() == ScenarioKind::Attack
            || frame.action != SkeletonAction::None
            || (!scenario_uses_terrain_ik(&frame.scenario)
                && frame.weapon_guard == WeaponGuardState::Lowered)
        {
            // In an FK-only comparison the semantic weights describe authored
            // loading, not a claim that the procedural solver owns contact.
            // Attack captures exercise the same raised-guard locomotion and
            // terrain IK ownership used outside attacks.
            return true;
        }
        [
            (BoneRole::FootLeft, frame.ik_left_support_weight),
            (BoneRole::FootRight, frame.ik_right_support_weight),
        ]
        .into_iter()
        .all(|(foot, support)| {
            let quickstep_toe_contact = if is_quickstep_scenario(&frame.scenario) {
                let toe = if foot == BoneRole::FootLeft {
                    BoneRole::ToeLeft
                } else {
                    BoneRole::ToeRight
                };
                frame
                    .bones
                    .get(toe)
                    .and_then(|bone| bone.terrain_clearance_metres)
                    .is_some_and(|clearance| clearance.abs() <= tolerance.metres())
            } else {
                false
            };
            support.is_some_and(|support| {
                support.fraction() < 0.95
                    || quickstep_toe_contact
                    || frame
                        .bones
                        .get(foot)
                        .and_then(|bone| bone.terrain_clearance_metres)
                        .is_some_and(|ankle_clearance| {
                            (ankle_clearance - measured_ankle_sole_offset_metres().metres()).abs()
                                <= if frame.scenario.is(ScenarioId::RaisedGuardStationaryTurn) {
                                    // The combat stance is deliberately
                                    // non-flat-footed; its planted pole target
                                    // has a measured 1.11 cm ankle residual.
                                    0.012
                                } else {
                                    tolerance.metres()
                                }
                        })
            })
        })
    })
}

pub(super) fn terrain_run_contacts_are_valid(frames: &[FrameSample]) -> bool {
    let run = frames
        .iter()
        .filter(|frame| frame.scenario.is(ScenarioId::TerrainSteadyRun55))
        .collect::<Vec<_>>();
    if run.is_empty() {
        return true;
    }
    let warmed = phase_validation_frames(&run);
    let mut acquisitions = Vec::new();
    let mut last = None;
    for frame in &warmed {
        let Ok((left, right)) = frame.required_ik_support() else {
            return false;
        };
        let support = if left.fraction().max(right.fraction()) < 0.5 {
            None
        } else {
            Some(left.fraction() >= right.fraction())
        };
        if support != last {
            if let Some(left) = support {
                acquisitions.push(left);
            }
            last = support;
        }
    }
    acquisitions.len() >= 4
        && acquisitions.contains(&true)
        && acquisitions.contains(&false)
        && acquisitions.windows(2).all(|pair| pair[0] != pair[1])
        && (0.08..=0.20).contains(&maximum_no_support_seconds(&warmed))
}

/// Terrain height variation under the two sampled feet relative to the
/// controller root's ground point. Subtracting this measured relief from the
/// torso range preserves the flat-ground gait envelope while allowing the
/// bounded pelvis correction required to keep both legs reachable on a slope.
pub(super) fn foot_terrain_relief(frames: &[&FrameSample]) -> f32 {
    let (minimum, maximum) = frames
        .iter()
        .flat_map(|frame| {
            [BoneRole::FootLeft, BoneRole::FootRight]
                .into_iter()
                .filter_map(|foot| {
                    let bone = frame.bones.get(foot)?;
                    let clearance = bone.terrain_clearance_metres?;
                    let terrain_height = bone.position[1] - clearance;
                    let root_ground =
                        frame.root_position_metres[1] - CAPTURE_ROOT_GROUND_OFFSET_METRES;
                    Some(terrain_height - root_ground)
                })
        })
        .fold(
            (f32::INFINITY, f32::NEG_INFINITY),
            |(minimum, maximum), value| (minimum.min(value), maximum.max(value)),
        );
    if minimum.is_finite() && maximum.is_finite() {
        maximum - minimum
    } else {
        0.0
    }
}

fn maximum_planted_foot_drift(metadata: &ScenarioMetadata<'_>, frames: &[&FrameSample]) -> f32 {
    let mut maximum = 0.0_f32;
    for (foot, left) in [(BoneRole::FootLeft, true), (BoneRole::FootRight, false)] {
        let mut anchor = None;
        for frame in frames {
            if metadata.kind() != ScenarioKind::Terrain && !frame.guard_action {
                anchor = None;
                continue;
            }
            let support = if left {
                frame.left_support_weight.fraction()
            } else {
                frame.right_support_weight.fraction()
            };
            // The IK target deliberately releases continuously below full
            // support. Measure cumulative plant drift only while the solver is
            // effectively pinned; the separate >=0.9 per-frame slip metric
            // continues to gate the blended release interval.
            if support >= FULL_PLANT_SUPPORT_WEIGHT {
                let Some(position) = frame
                    .bones
                    .get(foot)
                    .map(|bone| Vec3::from_array(bone.position).xz())
                else {
                    continue;
                };
                let origin = anchor.get_or_insert(position);
                maximum = maximum.max(position.distance(*origin));
            } else {
                anchor = None;
            }
        }
    }
    maximum
}

pub(super) fn minimum_knee_bend(frames: &[&FrameSample]) -> f32 {
    let mut minimum = f32::INFINITY;
    for frame in frames {
        for (hip_role, knee_role, foot_role) in [
            (BoneRole::ThighLeft, BoneRole::ShinLeft, BoneRole::FootLeft),
            (
                BoneRole::ThighRight,
                BoneRole::ShinRight,
                BoneRole::FootRight,
            ),
        ] {
            let (Some(hip), Some(knee), Some(foot)) = (
                frame.bones.get(hip_role),
                frame.bones.get(knee_role),
                frame.bones.get(foot_role),
            ) else {
                continue;
            };
            let (hip, knee, foot) = (
                Vec3::from_array(hip.position),
                Vec3::from_array(knee.position),
                Vec3::from_array(foot.position),
            );
            let axis = foot - hip;
            if axis.length_squared() > 0.0001 {
                let closest = hip + axis * (knee - hip).dot(axis) / axis.length_squared();
                let forward = Vec3::from_array(frame.world_travel_direction)
                    .try_normalize()
                    .unwrap_or(Vec3::Z);
                minimum = minimum.min((knee - closest).dot(forward));
            }
        }
    }
    if minimum.is_finite() { minimum } else { 0.0 }
}

pub(super) fn minimum_foot_clearance(frames: &[&FrameSample]) -> f32 {
    let minimum = frames
        .iter()
        .flat_map(|frame| {
            [
                frame.bones.get(BoneRole::FootLeft),
                frame.bones.get(BoneRole::FootRight),
            ]
        })
        .flatten()
        .filter_map(|foot| foot.terrain_clearance_metres)
        .fold(f32::INFINITY, f32::min);
    if minimum.is_finite() { minimum } else { -1.0 }
}

fn metric_frames_after_warmup<'a>(
    metadata: &ScenarioMetadata<'_>,
    frames: &[&'a FrameSample],
) -> Vec<&'a FrameSample> {
    // Terrain conformity calibrates retained pelvis/plant state during its first cycle.
    if metadata.kind() == ScenarioKind::Terrain {
        phase_validation_frames(frames)
    } else {
        frames.to_vec()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn foot_metric_frame(
        scenario_frame: usize,
        left_x: f32,
        left_support_weight: f32,
        left_terrain_height: f32,
        right_terrain_height: f32,
    ) -> FrameSample {
        let foot = |x: f32, terrain_height: f32| BoneSample {
            position: Vec3::new(x, terrain_height, 0.0).to_array(),
            rotation_xyzw: Quat::IDENTITY.to_array(),
            terrain_clearance_metres: Some(0.0),
        };
        FrameSample {
            scenario: "metric-test".into(),
            scenario_frame,
            time_seconds: AnimationElapsed::from_seconds(
                scenario_frame as f32 / locomotion_sample_rate().hertz(),
            )
            .unwrap(),
            speed_metres_per_second: SignedSpeed::from_metres_per_second(2.0).unwrap(),
            gait_phase: CyclePhase::from_cycles(0.0).unwrap(),
            locomotion_sample_tick: AnimationSampleTick::from(scenario_frame as u64),
            body_acceleration: Vec3::ZERO.to_array(),
            world_acceleration: Vec3::ZERO.to_array(),
            bouncy_active_springs: 0,
            bouncy_maximum_deflection_radians: 0.0,
            contact_sequence: ContactSequence::ZERO,
            contact_foot: LeadFoot::Left,
            landing_sequence: adventuresim_tactical_core::animation::LandingSequence::ZERO,
            landing_impact_speed: LandingImpactSpeed::ZERO,
            body_lean_pitch_degrees: BodyAngle::ZERO,
            body_lean_roll_degrees: BodyAngle::ZERO,
            landing_compression_metres: HeightDistance::ZERO,
            root_distance_metres: 0.0,
            root_position_metres: Vec3::new(0.0, CAPTURE_ROOT_GROUND_OFFSET_METRES, 0.0).to_array(),
            world_travel_direction: Vec3::NEG_Z.to_array(),
            desired_body_forward_direction: Vec3::NEG_Z.to_array(),
            body_forward_direction: Vec3::NEG_Z.to_array(),
            body_rotation_xyzw: Quat::IDENTITY.to_array(),
            weapon_guard: WeaponGuardState::Lowered,
            lead_foot: LeadFoot::Left,
            action: SkeletonAction::None,
            action_phase: adventuresim_tactical_core::animation::UnitInterval::ZERO,
            attack_animation: None,
            strike_family: StrikeFamily::Thrust,
            guard_action: false,
            left_support_weight: UnitInterval::from_fraction(left_support_weight).unwrap(),
            right_support_weight: UnitInterval::from_fraction(0.0).unwrap(),
            desired_left_foot_target: None,
            desired_right_foot_target: None,
            ik_left_authored_target: None,
            ik_right_authored_target: None,
            ik_left_planned_contact: None,
            ik_right_planned_contact: None,
            ik_settle_capture_point: None,
            ik_left_solve_target: None,
            ik_right_solve_target: None,
            ik_left_support_weight: Some(UnitInterval::from_fraction(0.0).unwrap()),
            ik_right_support_weight: Some(UnitInterval::from_fraction(0.0).unwrap()),
            ik_left_release_active: false,
            ik_right_release_active: false,
            ik_left_release_target: None,
            ik_right_release_target: None,
            ik_settle_progress: None,
            ik_left_knee_foot_yaw_offset_degrees: 0.0,
            ik_right_knee_foot_yaw_offset_degrees: 0.0,
            semantic_route_requested_path: SemanticRoutePath::GeneralPose,
            semantic_route_selected_path: SemanticRoutePath::GeneralPose,
            semantic_route_runtime_evaluated: false,
            screenshots: BTreeMap::new(),
            bones: CapturedBones::from_iter([
                (BoneRole::FootLeft, foot(left_x, left_terrain_height)),
                (BoneRole::FootRight, foot(0.2, right_terrain_height)),
            ]),
        }
    }

    #[test]
    fn complete_run_cycles_use_the_run_foot_continuity_budget_only() {
        assert!(
            0.149
                <= foot_continuity_limit(&ScenarioMetadata::from_label("terrain-run-flight-stop"))
        );
        assert!(
            0.151 > foot_continuity_limit(&ScenarioMetadata::from_label("terrain-run-flight-stop"))
        );
        assert!(
            0.149
                <= foot_continuity_limit(&ScenarioMetadata::from_label(
                    "terrain-tap-restart-crossfade"
                ))
        );
        assert!(
            0.151
                > foot_continuity_limit(&ScenarioMetadata::from_label(
                    "terrain-tap-restart-crossfade"
                ))
        );
        assert!(
            0.056
                > foot_continuity_limit(&ScenarioMetadata::from_label("terrain-tap-stop-forward"))
        );
        assert_eq!(
            foot_continuity_limit(&ScenarioMetadata::from_label("terrain-steady-run-5.5")),
            0.15
        );
        assert_eq!(
            knee_continuity_limit(&ScenarioMetadata::from_label("terrain-steady-run-5.5")),
            0.16
        );
        assert_eq!(
            knee_continuity_limit(&ScenarioMetadata::from_label("steady-run-5.5")),
            0.13
        );
        assert_eq!(
            knee_continuity_limit(&ScenarioMetadata::from_label("steady-walk")),
            0.10
        );
        assert_eq!(
            loop_seam_position_limit(&ScenarioMetadata::from_label("steady-run-5.5")),
            0.03
        );
        assert_eq!(
            loop_seam_position_limit(&ScenarioMetadata::from_label("steady-walk")),
            0.015
        );
    }

    #[test]
    fn terrain_run_stop_and_restart_require_the_strict_toe_clearance_contract() {
        assert!(scenario_requires_strict_terrain_toe_clearance(
            &ScenarioMetadata::authored(ScenarioId::TerrainSteadyRun55)
        ));
        assert!(scenario_requires_strict_terrain_toe_clearance(
            &ScenarioMetadata::authored(ScenarioId::TerrainRunFlightStop)
        ));
        assert!(scenario_requires_strict_terrain_toe_clearance(
            &ScenarioMetadata::authored(ScenarioId::TerrainTapRestartCrossfade)
        ));
        assert!(!scenario_requires_strict_terrain_toe_clearance(
            &ScenarioMetadata::authored(ScenarioId::TerrainTapStopForward)
        ));
    }

    #[test]
    fn transient_toe_gate_excludes_idle_preroll_but_keeps_motion_and_settle() {
        for scenario in ["terrain-run-flight-stop", "terrain-tap-restart-crossfade"]
            .map(ScenarioMetadata::from_label)
        {
            assert!(!transition_flight_toe_sample_is_active(
                &scenario, 0.0, None
            ));
            assert!(transition_flight_toe_sample_is_active(&scenario, 5.5, None));
            assert!(transition_flight_toe_sample_is_active(
                &scenario,
                0.0,
                Some(0.25)
            ));
        }
        assert!(transition_flight_toe_sample_is_active(
            &ScenarioMetadata::from_label("terrain-steady-run-5.5"),
            0.0,
            None
        ));
        assert!(strict_transition_flight_toe_clearance_is_valid(0.01));
        assert!(!strict_transition_flight_toe_clearance_is_valid(-0.001));
    }

    #[test]
    fn manifest_metrics_detect_crossed_feet_and_facing_mismatch() {
        let bone = |position: Vec3| BoneSample {
            position: position.to_array(),
            rotation_xyzw: Quat::IDENTITY.to_array(),
            terrain_clearance_metres: Some(0.0),
        };
        let bones = CapturedBones::from_iter([
            (BoneRole::ThighLeft, bone(Vec3::new(-0.15, 1.0, 0.0))),
            (BoneRole::ShinLeft, bone(Vec3::new(-0.1, 0.5, 0.2))),
            (BoneRole::FootLeft, bone(Vec3::new(0.12, 0.0, 0.0))),
            (BoneRole::ThighRight, bone(Vec3::new(0.15, 1.0, 0.0))),
            (BoneRole::ShinRight, bone(Vec3::new(0.1, 0.5, 0.2))),
            (BoneRole::FootRight, bone(Vec3::new(-0.12, 0.0, 0.0))),
        ]);
        let mut frame = FrameSample {
            scenario: "metric-test".into(),
            scenario_frame: 0,
            time_seconds: AnimationElapsed::from_seconds(0.0).unwrap(),
            speed_metres_per_second: SignedSpeed::from_metres_per_second(2.0).unwrap(),
            gait_phase: CyclePhase::from_cycles(0.0).unwrap(),
            locomotion_sample_tick: AnimationSampleTick::from(0u64),
            body_acceleration: Vec3::ZERO.to_array(),
            world_acceleration: Vec3::ZERO.to_array(),
            bouncy_active_springs: 0,
            bouncy_maximum_deflection_radians: 0.0,
            contact_sequence: ContactSequence::ZERO,
            contact_foot: LeadFoot::Left,
            landing_sequence: adventuresim_tactical_core::animation::LandingSequence::ZERO,
            landing_impact_speed: LandingImpactSpeed::ZERO,
            body_lean_pitch_degrees: BodyAngle::ZERO,
            body_lean_roll_degrees: BodyAngle::ZERO,
            landing_compression_metres: HeightDistance::ZERO,
            root_distance_metres: 0.0,
            root_position_metres: Vec3::ZERO.to_array(),
            world_travel_direction: Vec3::NEG_Z.to_array(),
            desired_body_forward_direction: Vec3::NEG_Z.to_array(),
            body_forward_direction: Vec3::Z.to_array(),
            body_rotation_xyzw: Quat::IDENTITY.to_array(),
            weapon_guard: WeaponGuardState::Lowered,
            lead_foot: LeadFoot::Left,
            action: SkeletonAction::None,
            action_phase: adventuresim_tactical_core::animation::UnitInterval::ZERO,
            attack_animation: None,
            strike_family: StrikeFamily::Thrust,
            guard_action: false,
            left_support_weight: UnitInterval::from_fraction(1.0).unwrap(),
            right_support_weight: UnitInterval::from_fraction(1.0).unwrap(),
            desired_left_foot_target: None,
            desired_right_foot_target: None,
            ik_left_authored_target: None,
            ik_right_authored_target: None,
            ik_left_planned_contact: None,
            ik_right_planned_contact: None,
            ik_settle_capture_point: None,
            ik_left_solve_target: None,
            ik_right_solve_target: None,
            ik_left_support_weight: Some(UnitInterval::from_fraction(0.0).unwrap()),
            ik_right_support_weight: Some(UnitInterval::from_fraction(0.0).unwrap()),
            ik_left_release_active: false,
            ik_right_release_active: false,
            ik_left_release_target: None,
            ik_right_release_target: None,
            ik_settle_progress: None,
            ik_left_knee_foot_yaw_offset_degrees: 0.0,
            ik_right_knee_foot_yaw_offset_degrees: 0.0,
            semantic_route_requested_path: SemanticRoutePath::GeneralPose,
            semantic_route_selected_path: SemanticRoutePath::GeneralPose,
            semantic_route_runtime_evaluated: false,
            screenshots: BTreeMap::new(),
            bones,
        };
        let frames = [&frame];
        assert!(minimum_signed_foot_track(&frames) < 0.0);
        assert!(maximum_facing_error(&frames) > 179.0);

        let mut previous = frame.clone();
        previous.desired_body_forward_direction = Vec3::Z.to_array();
        frame.scenario_frame = 1;
        frame.time_seconds = locomotion_sample_rate().interval().elapsed();
        assert!(maximum_facing_tracking_excess(&[&previous, &frame]) > 8.0);

        frame.guard_action = true;
        assert!(maximum_guard_facing_error(&[&frame]) > 179.0);

        frame.ik_left_knee_foot_yaw_offset_degrees = 90.0;
        assert!(maximum_knee_foot_yaw_offset(&[&frame]) > 45.0);
    }

    #[test]
    fn hard_stop_height_gate_measures_only_the_transition_and_settle_window() {
        let frame = |scenario_frame: usize, speed: f32, pelvis_y: f32| {
            let mut frame = foot_metric_frame(scenario_frame, -0.1, 1.0, 0.0, 0.0);
            frame.scenario = "hard-stop".into();
            frame.speed_metres_per_second = SignedSpeed::from_metres_per_second(speed).unwrap();
            frame.bones.insert(
                BoneRole::Pelvis,
                BoneSample {
                    position: (Vec3::Y * pelvis_y).to_array(),
                    rotation_xyzw: Quat::IDENTITY.to_array(),
                    terrain_clearance_metres: None,
                },
            );
            frame
        };
        let frames = vec![
            frame(0, 5.5, 0.0),
            frame(1, 5.5, 0.03),
            frame(2, 5.5, 0.04),
            frame(3, 0.0, 0.055),
            frame(4, 0.0, 0.065),
        ];
        assert!(
            (root_relative_vertical_step(&frames.iter().collect::<Vec<_>>(), BoneRole::Pelvis)
                - 0.03)
                .abs()
                < 0.0001
        );
        assert!((hard_stop_pelvis_vertical_step(&frames).unwrap() - 0.015).abs() < 0.0001);
        assert_eq!(hard_stop_pelvis_vertical_step(&[]), None);
    }

    #[test]
    fn cold_start_manifest_frame_requires_truthful_release_ownership() {
        let mut frame = foot_metric_frame(0, -0.1, 0.0, 0.0, 0.0);
        frame.scenario = "terrain-steady-run-5.5".into();
        frame.speed_metres_per_second = SignedSpeed::from_metres_per_second(5.5).unwrap();
        frame.ik_left_authored_target = Some(Vec3::ZERO.to_array());
        frame.ik_left_solve_target = Some((Vec3::Y * 0.095).to_array());
        frame.ik_right_authored_target = Some(Vec3::ZERO.to_array());
        frame.ik_right_solve_target = Some(Vec3::ZERO.to_array());
        assert!(!ordinary_swing_frame_is_owned(&frame));

        frame.ik_left_release_active = true;
        assert!(ordinary_swing_frame_is_owned(&frame));
    }

    #[test]
    fn unplanned_release_transition_uses_run_specific_motion_budget() {
        let mut before = foot_metric_frame(0, -0.1, 0.0, 0.0, 0.0);
        let mut after = foot_metric_frame(1, -0.1, 0.0, 0.0, 0.0);
        before.scenario = "terrain-steady-run-5.5".into();
        after.scenario = before.scenario.clone();
        before.speed_metres_per_second = SignedSpeed::from_metres_per_second(5.5).unwrap();
        after.speed_metres_per_second = SignedSpeed::from_metres_per_second(5.5).unwrap();
        let origin = Some(Vec3::ZERO.to_array());
        let run_step = Some((Vec3::X * 0.0875).to_array());
        assert!(ordinary_unplanned_release_transition_is_valid(
            &before, &after, origin, run_step, None, None,
        ));

        let over_run_budget = Some((Vec3::X * 0.096).to_array());
        assert!(!ordinary_unplanned_release_transition_is_valid(
            &before,
            &after,
            origin,
            over_run_budget,
            None,
            None,
        ));

        before.speed_metres_per_second = SignedSpeed::from_metres_per_second(2.0).unwrap();
        after.speed_metres_per_second = SignedSpeed::from_metres_per_second(2.0).unwrap();
        assert!(!ordinary_unplanned_release_transition_is_valid(
            &before, &after, origin, run_step, None, None,
        ));
    }

    #[test]
    fn planned_run_transition_validates_metadata_and_body_local_motion() {
        let mut before = foot_metric_frame(0, -0.1, 0.0, 0.0, 0.0);
        let mut after = foot_metric_frame(1, -0.1, 0.0, 0.0, 0.0);
        before.scenario = "terrain-steady-run-5.5".into();
        after.scenario = before.scenario.clone();
        before.speed_metres_per_second = SignedSpeed::from_metres_per_second(5.5).unwrap();
        after.speed_metres_per_second = SignedSpeed::from_metres_per_second(5.5).unwrap();
        let plan = Some(Vec3::ZERO.to_array());
        let before_solve = Some((Vec3::X * 0.05).to_array());
        let bounded_run_solve = Some((Vec3::X * 0.1375).to_array());
        after.bones.get_mut(BoneRole::FootLeft).unwrap().position =
            Vec3::new(-0.0125, 0.0, 0.0).to_array();
        assert!(ordinary_planned_transition_is_valid(
            &before,
            &after,
            BoneRole::FootLeft,
            plan,
            plan,
            before_solve,
            bounded_run_solve,
            UnitInterval::from_fraction(0.0).unwrap(),
            false,
        ));

        let over_budget_solve = Some((Vec3::X * 0.146).to_array());
        after.bones.get_mut(BoneRole::FootLeft).unwrap().position =
            Vec3::new(-0.004, 0.0, 0.0).to_array();
        assert!(!ordinary_planned_transition_is_valid(
            &before,
            &after,
            BoneRole::FootLeft,
            plan,
            plan,
            before_solve,
            over_budget_solve,
            UnitInterval::from_fraction(0.0).unwrap(),
            false,
        ));

        before.speed_metres_per_second = SignedSpeed::from_metres_per_second(2.0).unwrap();
        after.speed_metres_per_second = SignedSpeed::from_metres_per_second(2.0).unwrap();
        assert!(!ordinary_planned_transition_is_valid(
            &before,
            &after,
            BoneRole::FootLeft,
            plan,
            plan,
            before_solve,
            bounded_run_solve,
            UnitInterval::from_fraction(0.0).unwrap(),
            false,
        ));

        before.speed_metres_per_second = SignedSpeed::from_metres_per_second(5.5).unwrap();
        after.speed_metres_per_second = SignedSpeed::from_metres_per_second(5.5).unwrap();
        after.bones.get_mut(BoneRole::FootLeft).unwrap().position =
            Vec3::new(-0.05, 0.0, 0.0).to_array();
        let retarget = Some((Vec3::X * 0.05).to_array());
        assert!(ordinary_planned_transition_is_valid(
            &before,
            &after,
            BoneRole::FootLeft,
            plan,
            retarget,
            plan,
            retarget,
            UnitInterval::from_fraction(0.205).unwrap(),
            false,
        ));

        let replacement = Some(Vec3::new(0.0, 0.0, -5.0).to_array());
        let hidden_solve_jump = Some((Vec3::X * 0.50).to_array());
        assert!(ordinary_planned_transition_is_valid(
            &before,
            &after,
            BoneRole::FootLeft,
            plan,
            replacement,
            before_solve,
            hidden_solve_jump,
            UnitInterval::from_fraction(0.0).unwrap(),
            true,
        ));
        after.bones.get_mut(BoneRole::FootLeft).unwrap().position =
            Vec3::new(-0.004, 0.0, 0.0).to_array();
        assert!(!ordinary_planned_transition_is_valid(
            &before,
            &after,
            BoneRole::FootLeft,
            plan,
            replacement,
            before_solve,
            hidden_solve_jump,
            UnitInterval::from_fraction(0.0).unwrap(),
            true,
        ));
    }

    #[test]
    fn planted_drift_excludes_the_deliberate_support_release_interval() {
        let frames = [
            foot_metric_frame(0, 0.0, 1.0, 0.0, 0.0),
            foot_metric_frame(1, 0.02, 1.0, 0.0, 0.0),
            foot_metric_frame(2, 0.20, 0.95, 0.0, 0.0),
        ];
        let references = frames.iter().collect::<Vec<_>>();

        assert!(
            (maximum_planted_foot_drift(
                &ScenarioMetadata::from_label("cross-slope-walk"),
                &references
            ) - 0.02)
                .abs()
                < 0.0001
        );
    }

    #[test]
    fn authored_guard_locomotion_does_not_require_procedural_step_liveness() {
        let mut frames = [
            foot_metric_frame(0, 0.0, 0.0, 0.0, 0.0),
            foot_metric_frame(1, 0.0, 0.0, 0.0, 0.0),
            foot_metric_frame(2, 0.0, 1.0, 0.0, 0.0),
        ];
        for frame in &mut frames {
            frame.scenario = "raised-guard-right".into();
            frame.weapon_guard = WeaponGuardState::Raised;
            frame.right_support_weight = if frame.scenario_frame < 2 {
                UnitInterval::ONE
            } else {
                UnitInterval::ZERO
            };
        }
        frames[2].contact_sequence = ContactSequence::from(1u64);
        frames[2].contact_foot = LeadFoot::Right;
        let references = frames.iter().collect::<Vec<_>>();

        let metrics = guard_step_liveness_metrics(&references);

        assert!(!metrics.required);
    }

    #[test]
    fn contact_clearance_uses_effective_ik_support_instead_of_gait_phase() {
        let mut frame = foot_metric_frame(0, 0.0, 1.0, 0.0, 0.0);
        frame.ik_left_solve_target = Some(Vec3::ZERO.to_array());
        frame.ik_right_solve_target = Some(Vec3::ZERO.to_array());
        frame.ik_left_support_weight = Some(UnitInterval::from_fraction(0.0).unwrap());
        frame.ik_right_support_weight = Some(UnitInterval::from_fraction(1.0).unwrap());
        frame
            .bones
            .get_mut(BoneRole::FootLeft)
            .unwrap()
            .terrain_clearance_metres = Some(0.161);
        frame
            .bones
            .get_mut(BoneRole::FootRight)
            .unwrap()
            .terrain_clearance_metres = Some(0.085);

        // Gait phase zero identifies the left foot, but the solver reports it
        // airborne and the right foot supported. Only the supported sole is a
        // contact-clearance sample.
        assert_eq!(contact_sole_clearance_range(&[&frame]).unwrap(), (0.0, 0.0));
    }

    #[test]
    fn contact_clearance_falls_back_to_phase_without_procedural_targets() {
        let mut frame = foot_metric_frame(0, 0.0, 0.0, 0.0, 0.0);
        frame
            .bones
            .get_mut(BoneRole::FootLeft)
            .unwrap()
            .terrain_clearance_metres = Some(0.095);
        frame
            .bones
            .get_mut(BoneRole::FootRight)
            .unwrap()
            .terrain_clearance_metres = Some(0.145);

        let (minimum, maximum) = contact_sole_clearance_range(&[&frame]).unwrap();
        assert!((minimum - 0.01).abs() < 0.0001);
        assert!((maximum - 0.01).abs() < 0.0001);
    }

    #[test]
    fn viewer_rejects_reported_support_beyond_the_shared_contact_tolerance() {
        let mut frame = foot_metric_frame(0, 0.0, 0.0, 0.0, 0.0);
        frame.scenario = "terrain-steady-run-5.5".into();
        frame.ik_left_support_weight = Some(UnitInterval::from_fraction(1.0).unwrap());
        frame
            .bones
            .get_mut(BoneRole::FootLeft)
            .unwrap()
            .terrain_clearance_metres = Some(
            measured_ankle_sole_offset_metres().metres() + sole_contact_tolerance_metres().metres()
                - 0.00001,
        );
        assert!(reported_support_contacts_are_valid(&[frame.clone()]));

        frame
            .bones
            .get_mut(BoneRole::FootLeft)
            .unwrap()
            .terrain_clearance_metres = Some(
            measured_ankle_sole_offset_metres().metres()
                + sole_contact_tolerance_metres().metres()
                + 0.0001,
        );
        assert!(!reported_support_contacts_are_valid(&[frame]));
    }

    #[test]
    fn viewer_uses_action_support_instead_of_stale_ordinary_ik_support() {
        let mut frame = foot_metric_frame(0, 0.0, 0.0, 0.0, 0.0);
        frame.scenario = "attack-live-forward-left-support".into();
        frame.action = SkeletonAction::Attack;
        frame.ik_left_support_weight = Some(UnitInterval::from_fraction(1.0).unwrap());
        frame
            .bones
            .get_mut(BoneRole::FootLeft)
            .unwrap()
            .terrain_clearance_metres = Some(1.0);
        assert!(reported_support_contacts_are_valid(&[frame]));
    }

    #[test]
    fn terrain_run_contact_gate_is_non_vacuous_and_requires_alternation() {
        let phase_step = gait_cycle_phase_delta(
            run_locomotion_profile(),
            run_locomotion_profile().reference_speed,
            locomotion_sample_rate().interval().elapsed(),
        )
        .unwrap()
        .cycles();
        let mut unsupported = (0..160)
            .map(|index| {
                let mut frame = foot_metric_frame(index, 0.0, 0.0, 0.0, 0.0);
                frame.scenario = "terrain-steady-run-5.5".into();
                frame.gait_phase =
                    CyclePhase::from_cycles((index as f32 * phase_step).rem_euclid(1.0)).unwrap();
                frame.time_seconds =
                    AnimationElapsed::from_seconds(index as f32 / locomotion_sample_rate().hertz())
                        .unwrap();
                frame
            })
            .collect::<Vec<_>>();
        assert!(!terrain_run_contacts_are_valid(&unsupported));

        for frame in &mut unsupported {
            let phase = frame.gait_phase.cycles();
            let left_distance = phase.min(1.0 - phase);
            let right_distance = (phase - 0.5).abs();
            frame.ik_left_support_weight = Some(if left_distance <= 0.17 {
                UnitInterval::ONE
            } else {
                UnitInterval::ZERO
            });
            frame.ik_right_support_weight = Some(if right_distance <= 0.17 {
                UnitInterval::ONE
            } else {
                UnitInterval::ZERO
            });
            frame.left_support_weight = frame.ik_left_support_weight.unwrap();
            frame.right_support_weight = frame.ik_right_support_weight.unwrap();
        }
        assert!(terrain_run_contacts_are_valid(&unsupported));
    }

    #[test]
    fn continuity_bone_classes_separate_feet_from_knees() {
        assert!(is_foot_bone(&BoneRole::FootLeft));
        assert!(is_foot_bone(&BoneRole::FootRight));
        assert!(!is_foot_bone(&BoneRole::ShinLeft));
        assert!(is_knee_bone(&BoneRole::ShinLeft));
        assert!(is_knee_bone(&BoneRole::ShinRight));
        assert!(!is_knee_bone(&BoneRole::ThighRight));
    }

    #[test]
    fn terrain_vertical_allowance_is_derived_from_sampled_foot_relief() {
        let frame = foot_metric_frame(0, 0.0, 1.0, -0.04, 0.04);

        assert!((foot_terrain_relief(&[&frame]) - 0.08).abs() < 0.0001);
        assert_eq!(
            vertical_range_limit(&ScenarioMetadata::from_label("steady-walk"), 0.08),
            ORDINARY_VERTICAL_RANGE_LIMIT_METRES
        );
        assert!(
            (vertical_range_limit(&ScenarioMetadata::from_label("cross-slope-walk"), 0.08) - 0.28)
                .abs()
                < 0.0001
        );
    }

    #[test]
    fn pelvis_vertical_step_detects_a_one_frame_guard_height_snap() {
        let mut first = foot_metric_frame(0, 0.0, 1.0, 0.0, 0.0);
        let mut second = foot_metric_frame(1, 0.0, 1.0, 0.0, 0.0);
        let pelvis = |height: f32| BoneSample {
            position: Vec3::new(0.0, height, 0.0).to_array(),
            rotation_xyzw: Quat::IDENTITY.to_array(),
            terrain_clearance_metres: None,
        };
        first.bones.insert(BoneRole::Pelvis, pelvis(1.0));
        second.bones.insert(BoneRole::Pelvis, pelvis(0.86));

        assert!(
            (root_relative_vertical_step(&[&first, &second], BoneRole::Pelvis) - 0.14).abs()
                < 0.0001
        );
    }

    #[test]
    fn prominent_height_gate_requires_only_the_two_passing_peaks() {
        let mut clean = (0..64)
            .map(|sample| {
                let phase = sample as f32 / 64.0;
                let sine = (phase * std::f32::consts::TAU).sin();
                (phase, 0.04 * sine * sine)
            })
            .collect::<Vec<_>>();
        assert_eq!(prominent_height_peaks(&clean), (2, true));

        // A visible contact beat is a third gait-height peak even though the
        // intended passing peaks remain correctly positioned.
        clean[0].1 = 0.01;
        let (count, passing_windows) = prominent_height_peaks(&clean);
        assert!(passing_windows);
        assert_ne!(count, 2);
    }

    #[test]
    fn release_transition_is_not_misclassified_as_a_repeatable_loop() {
        assert!(!expects_loop_seam(&ScenarioMetadata::from_label(
            "raised-guard-release-at-peak"
        )));
        assert!(expects_loop_seam(&ScenarioMetadata::from_label(
            "raised-guard-forward"
        )));
        assert!(expects_loop_seam(&ScenarioMetadata::from_label(
            "steady-walk"
        )));
    }
}

#[cfg(test)]
mod scenario_policy_tests {
    use super::*;

    #[test]
    fn raised_guard_uses_strict_plant_and_separation_gates() {
        assert_eq!(
            inter_foot_separation_limit(&ScenarioMetadata::from_label("quickstep-right")),
            0.04
        );
        assert_eq!(
            planted_drift_limit(&ScenarioMetadata::from_label("raised-guard-right")),
            0.01
        );
        assert_eq!(
            inter_foot_separation_limit(&ScenarioMetadata::from_label("raised-guard-right")),
            0.08
        );
        assert_eq!(
            inter_foot_separation_limit(&ScenarioMetadata::from_label(
                "raised-guard-stationary-turn"
            )),
            0.16
        );
        assert_eq!(
            planted_drift_limit(&ScenarioMetadata::from_label("steady-walk-2.0")),
            0.035
        );
        assert_eq!(
            inter_foot_separation_limit(&ScenarioMetadata::from_label("steady-walk-2.0")),
            0.08
        );
        assert!(!procedural_leg_solver_gates_apply(
            &ScenarioMetadata::from_label("steady-walk-2.0")
        ));
        assert!(!procedural_leg_solver_gates_apply(
            &ScenarioMetadata::from_label("start-stop-transition")
        ));
        assert!(procedural_leg_solver_gates_apply(
            &ScenarioMetadata::from_label("cross-slope-walk")
        ));
        assert!(!procedural_leg_solver_gates_apply(
            &ScenarioMetadata::from_label("raised-guard-forward")
        ));
        assert!(procedural_leg_solver_gates_apply(
            &ScenarioMetadata::authored(ScenarioId::RaisedGuardStationaryTurn)
        ));
        assert!(!procedural_leg_solver_gates_apply(
            &ScenarioMetadata::authored(ScenarioId::RaisedGuardReleaseAtPeak)
        ));
        assert!(!procedural_leg_solver_gates_apply(
            &ScenarioMetadata::authored(ScenarioId::RaisedGuardTapStopLeft)
        ));
        assert!(ScenarioMetadata::from_label("raised-guard-release-at-peak").procedural_solver());
        assert!(ScenarioMetadata::from_label("raised-guard-tap-stop-left").procedural_solver());
        assert!(!procedural_leg_solver_gates_apply(
            &ScenarioMetadata::authored(ScenarioId::RaisedGuardLeftRightReversal)
        ));
        for transition in [
            "raised-guard-release-at-peak",
            "raised-guard-right-support-release",
            "raised-guard-left-right-reversal",
            "raised-guard-right-support-reversal",
            "raised-guard-accelerate-from-rest",
            "terrain-tap-restart-crossfade",
            "terrain-speed-threshold-chatter",
        ]
        .map(ScenarioMetadata::from_label)
        {
            assert!(!expects_loop_seam(&transition));
        }
    }
}
