//! Contact and flight toe clearance measurements.
use super::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum ToeContactSelection {
    Contact,
    Flight,
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
        for foot in [LeadFoot::Left, LeadFoot::Right] {
            let bone = match foot {
                LeadFoot::Left => BoneRole::FootLeft,
                LeadFoot::Right => BoneRole::FootRight,
            };
            let is_contact = if let Some((left_support, right_support)) = support {
                (match foot {
                    LeadFoot::Left => left_support,
                    LeadFoot::Right => right_support,
                }).fraction() >= 0.95
            } else {
                let phase = frame.gait_phase.cycles().rem_euclid(1.0);
                let distance = match foot {
                    LeadFoot::Left => phase.min(1.0 - phase),
                    LeadFoot::Right => (phase - 0.5).abs(),
                };
                distance <= 0.035
            };
            if is_contact
                && let Some(ankle) = frame
                    .bones
                    .get(bone)
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
    selection: ToeContactSelection,
) -> AnimationValueResult<f32> {
    let mut minimum = f32::INFINITY;
    for frame in frames {
        if selection == ToeContactSelection::Flight
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
            if matches!(
                (selection, support.fraction() > 0.05),
                (ToeContactSelection::Contact, true) | (ToeContactSelection::Flight, false)
            )
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

pub(in crate::animation_viewer::report) fn strict_transition_flight_toe_clearance_is_valid(clearance_metres: f32) -> bool {
    clearance_metres >= 0.01
}

