//! Signed terrain clearance and the measured terrain allowance.

use super::*;
use fabelgeist_geometry::SignedLength;

impl ScenarioClearanceMetrics {
    pub(in crate::animation_viewer::report) fn measure(
        frames: &[&FrameSample],
    ) -> ReportResult<Self> {
        Ok(Self {
            minimum_flight_sole_clearance_metres: minimum_flight_sole_clearance(frames)?,
            contact_sole_clearance_metres: contact_sole_clearance_range(frames)?,
            minimum_flight_toe_clearance_metres: minimum_toe_clearance(frames, ContactSample::Flight)?,
            minimum_contact_toe_clearance_metres: minimum_toe_clearance(frames, ContactSample::Contact)?,
            foot_terrain_relief_metres: foot_terrain_relief(frames)?,
            minimum_foot_clearance_metres: minimum_foot_clearance(frames),
        })
    }
}

pub(in crate::animation_viewer::report) fn minimum_flight_sole_clearance(
    frames: &[&FrameSample],
) -> ReportResult<Observation<SignedLength>> {
    if frames.is_empty() {
        return Ok(Observation::Unavailable(UnavailableReason::NoFrames));
    }
    let mut minimum: Option<SignedLength> = None;
    for frame in frames.iter().filter(|frame| {
        frame.left_support_weight.fraction() <= 0.001
            && frame.right_support_weight.fraction() <= 0.001
    }) {
        for foot in [BoneRole::FootLeft, BoneRole::FootRight] {
            let Some(bone) = frame.bones.get(foot) else {
                return Ok(Observation::Unavailable(UnavailableReason::MissingBone { bone: foot }));
            };
            let Some(ankle) = bone.terrain_clearance_metres else {
                return Ok(Observation::Unavailable(UnavailableReason::MissingTerrainClearance { bone: foot }));
            };
            // Preserve the capture's existing flight-sole numerical offset.
            let clearance = SignedLength::from_metres(ankle.metres() - 0.085)?;
            minimum = Some(minimum.map_or(clearance, |value| value.min(clearance)));
        }
    }
    Ok(minimum.map_or(Observation::NotApplicable, Observation::Measured))
}

pub(in crate::animation_viewer::report) fn contact_sole_clearance_range(
    frames: &[&FrameSample],
) -> ReportResult<Observation<SignedLengthRange>> {
    let mut bounds: Option<SignedLengthRange> = None;
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
            if !is_contact {
                continue;
            }
            let Some(bone) = frame.bones.get(foot) else {
                return Ok(Observation::Unavailable(UnavailableReason::MissingBone { bone: foot }));
            };
            let Some(ankle) = bone.terrain_clearance_metres else {
                return Ok(Observation::Unavailable(UnavailableReason::MissingTerrainClearance { bone: foot }));
            };
            let clearance = SignedLength::from_metres(
                ankle.metres() - measured_ankle_sole_offset_metres().metres(),
            )?;
            bounds = Some(bounds.map_or(SignedLengthRange::from_sample(clearance), |range| range.include(clearance)));
        }
    }
    Ok(bounds.map_or(
        Observation::Unavailable(UnavailableReason::NoContactSamples),
        Observation::Measured,
    ))
}

pub(in crate::animation_viewer::report) fn minimum_toe_clearance(
    frames: &[&FrameSample],
    sample: ContactSample,
) -> ReportResult<Observation<SignedLength>> {
    if frames.is_empty() {
        return Ok(Observation::Unavailable(UnavailableReason::NoFrames));
    }
    let mut minimum: Option<SignedLength> = None;
    for frame in frames {
        if sample == ContactSample::Flight
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
        for (toe, support) in [(BoneRole::ToeLeft, left_support), (BoneRole::ToeRight, right_support)] {
            if (support.fraction() > 0.05) != (sample == ContactSample::Contact) {
                continue;
            }
            let Some(bone) = frame.bones.get(toe) else {
                return Ok(Observation::Unavailable(UnavailableReason::MissingBone { bone: toe }));
            };
            let Some(clearance) = bone.terrain_clearance_metres else {
                return Ok(Observation::Unavailable(UnavailableReason::MissingTerrainClearance { bone: toe }));
            };
            minimum = Some(minimum.map_or(clearance, |value| value.min(clearance)));
        }
    }
    Ok(minimum.map_or_else(
        || if sample == ContactSample::Flight {
            Observation::NotApplicable
        } else {
            Observation::Unavailable(UnavailableReason::NoContactSamples)
        },
        Observation::Measured,
    ))
}

/// Terrain variation relative to the controller ground retains the existing
/// subtract-clearance, subtract-root, extrema-then-range numerical order.
pub(in crate::animation_viewer::report) fn foot_terrain_relief(
    frames: &[&FrameSample],
) -> ReportResult<Observation<HeightDistance>> {
    let mut bounds: Option<SignedLengthRange> = None;
    for frame in frames {
        for foot in [BoneRole::FootLeft, BoneRole::FootRight] {
            let Some(bone) = frame.bones.get(foot) else {
                return Ok(Observation::Unavailable(UnavailableReason::MissingBone { bone: foot }));
            };
            let Some(clearance) = bone.terrain_clearance_metres else {
                return Ok(Observation::Unavailable(UnavailableReason::MissingTerrainClearance { bone: foot }));
            };
            let terrain_height = bone.position[1] - clearance.metres();
            let root_ground = frame.root_position_metres[1] - CAPTURE_ROOT_GROUND_OFFSET_METRES;
            let value = SignedLength::from_metres(terrain_height - root_ground)?;
            bounds = Some(bounds.map_or(SignedLengthRange::from_sample(value), |range| range.include(value)));
        }
    }
    let Some(bounds) = bounds else {
        return Ok(Observation::Unavailable(UnavailableReason::NoFrames));
    };
    Ok(Observation::Measured(HeightDistance::from_metres(
        bounds.maximum().metres() - bounds.minimum().metres(),
    )?))
}

pub(in crate::animation_viewer::report) fn minimum_foot_clearance(
    frames: &[&FrameSample],
) -> Observation<SignedLength> {
    let mut minimum: Option<SignedLength> = None;
    for frame in frames {
        for foot in [BoneRole::FootLeft, BoneRole::FootRight] {
            let Some(bone) = frame.bones.get(foot) else {
                return Observation::Unavailable(UnavailableReason::MissingBone { bone: foot });
            };
            let Some(clearance) = bone.terrain_clearance_metres else {
                return Observation::Unavailable(UnavailableReason::MissingTerrainClearance { bone: foot });
            };
            minimum = Some(minimum.map_or(clearance, |value| value.min(clearance)));
        }
    }
    minimum.map_or(Observation::Unavailable(UnavailableReason::NoFrames), Observation::Measured)
}
