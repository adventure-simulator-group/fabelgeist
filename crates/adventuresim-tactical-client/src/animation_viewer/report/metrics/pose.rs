//! Anatomical observations distinguish valid extremes from missing geometry.

use super::*;
use adventuresim_tactical_core::animation::{AngularMagnitudeDegrees, NonnegativeLength};
use fabelgeist_geometry::SignedLength;

const LEGS: [LegBones; 2] = [
    LegBones { hip: BoneRole::ThighLeft, knee: BoneRole::ShinLeft, foot: BoneRole::FootLeft },
    LegBones { hip: BoneRole::ThighRight, knee: BoneRole::ShinRight, foot: BoneRole::FootRight },
];

struct LegBones {
    hip: BoneRole,
    knee: BoneRole,
    foot: BoneRole,
}

impl ScenarioPoseMetrics {
    pub(in crate::animation_viewer::report) fn measure(
        metadata: &ScenarioMetadata<'_>,
        frames: &[&FrameSample],
        procedural_frames: &[&FrameSample],
    ) -> ReportResult<Self> {
        let procedural = procedural_leg_solver_gates_apply(metadata);
        Ok(Self {
            minimum_knee_forward_bend_metres: minimum_knee_bend(frames)?,
            minimum_signed_foot_track_metres: minimum_signed_foot_track(frames)?,
            minimum_inter_foot_separation_metres: minimum_inter_foot_separation(frames)?,
            minimum_knee_flexion_degrees: if procedural { minimum_knee_flexion(procedural_frames)? } else { Observation::NotApplicable },
            minimum_knee_hemisphere_dot: if procedural { minimum_knee_hemisphere(procedural_frames)? } else { Observation::NotApplicable },
            maximum_knee_foot_yaw_offset_degrees: if procedural { maximum_knee_foot_yaw_offset(procedural_frames)? } else { Observation::NotApplicable },
        })
    }
}

pub(in crate::animation_viewer::report) fn minimum_signed_foot_track(
    frames: &[&FrameSample],
) -> ReportResult<Observation<SignedLength>> {
    let mut minimum: Option<SignedLength> = None;
    for frame in frames {
        for LegBones { hip: hip_role, foot: foot_role, .. } in LEGS {
            let Some(hip) = body_local(frame, hip_role) else {
                return Ok(Observation::Unavailable(UnavailableReason::MissingBone { bone: hip_role }));
            };
            let Some(foot) = body_local(frame, foot_role) else {
                return Ok(Observation::Unavailable(UnavailableReason::MissingBone { bone: foot_role }));
            };
            let value = SignedLength::from_metres(foot.x * hip.x.signum())?;
            minimum = Some(minimum.map_or(value, |minimum| minimum.min(value)));
        }
    }
    Ok(minimum.map_or(Observation::Unavailable(UnavailableReason::NoFrames), Observation::Measured))
}

pub(in crate::animation_viewer::report) fn minimum_inter_foot_separation(
    frames: &[&FrameSample],
) -> AnimationValueResult<Observation<NonnegativeLength>> {
    let mut minimum: Option<NonnegativeLength> = None;
    for frame in frames {
        let Some(left) = body_local(frame, BoneRole::FootLeft) else {
            return Ok(Observation::Unavailable(UnavailableReason::MissingBone { bone: BoneRole::FootLeft }));
        };
        let Some(right) = body_local(frame, BoneRole::FootRight) else {
            return Ok(Observation::Unavailable(UnavailableReason::MissingBone { bone: BoneRole::FootRight }));
        };
        let value = NonnegativeLength::from_metres(left.xz().distance(right.xz()))?;
        minimum = Some(minimum.map_or(value, |minimum| minimum.min(value)));
    }
    let Some(minimum) = minimum else {
        return Ok(Observation::Unavailable(UnavailableReason::NoFrames));
    };
    Ok(Observation::Measured(minimum))
}

pub(in crate::animation_viewer::report) fn minimum_knee_flexion(
    frames: &[&FrameSample],
) -> AnimationValueResult<Observation<AngularMagnitudeDegrees>> {
    let mut minimum: Option<AngularMagnitudeDegrees> = None;
    for frame in frames {
        match frame_minimum_knee_flexion(frame)? {
            Observation::Measured(value) => {
                minimum = Some(minimum.map_or(value, |minimum| minimum.min(value)));
            }
            Observation::Unavailable(reason) => return Ok(Observation::Unavailable(reason)),
            Observation::NotApplicable => return Ok(Observation::NotApplicable),
        }
    }
    let Some(minimum) = minimum else {
        return Ok(Observation::Unavailable(UnavailableReason::NoFrames));
    };
    Ok(Observation::Measured(minimum))
}

pub(in crate::animation_viewer::report) fn frame_minimum_knee_flexion(
    frame: &FrameSample,
) -> AnimationValueResult<Observation<AngularMagnitudeDegrees>> {
    let mut minimum: Option<AngularMagnitudeDegrees> = None;
    for LegBones { hip: hip_role, knee: knee_role, foot: foot_role } in LEGS {
        let mut positions = [Vec3::ZERO; 3];
        for (index, bone) in [hip_role, knee_role, foot_role].into_iter().enumerate() {
            let Some(position) = body_local(frame, bone) else {
                return Ok(Observation::Unavailable(UnavailableReason::MissingBone { bone }));
            };
            positions[index] = position;
        }
        let [hip, knee, foot] = positions;
        let value = AngularMagnitudeDegrees::from_degrees(180.0 - (hip - knee).angle_between(foot - knee).to_degrees())?;
        minimum = Some(minimum.map_or(value, |minimum| minimum.min(value)));
    }
    let Some(minimum) = minimum else {
        return Ok(Observation::Unavailable(UnavailableReason::NoFrames));
    };
    Ok(Observation::Measured(minimum))
}

pub(in crate::animation_viewer::report) fn minimum_knee_hemisphere(
    frames: &[&FrameSample],
) -> ReportResult<Observation<KneeHemisphereDot>> {
    let mut minimum: Option<KneeHemisphereDot> = None;
    for frame in frames {
        for LegBones { hip: hip_role, knee: knee_role, foot: foot_role } in LEGS {
            let mut positions = [Vec3::ZERO; 3];
            for (index, bone) in [hip_role, knee_role, foot_role].into_iter().enumerate() {
                let Some(position) = body_local(frame, bone) else {
                    return Ok(Observation::Unavailable(UnavailableReason::MissingBone { bone }));
                };
                positions[index] = position;
            }
            let [hip, knee, foot] = positions;
            let Some(axis) = (foot - hip).try_normalize() else {
                return Ok(Observation::Unavailable(UnavailableReason::DegenerateLegAxis));
            };
            let Some(bend) = (knee - hip).reject_from_normalized(axis).try_normalize() else {
                return Ok(Observation::Unavailable(UnavailableReason::UndefinedKneeBend));
            };
            let side = hip.x.signum();
            let value = KneeHemisphereDot::from_native(bend.dot((Vec3::Z + Vec3::X * side * 0.18).normalize()))?;
            minimum = Some(minimum.map_or(value, |minimum| minimum.min(value)));
        }
    }
    Ok(minimum.map_or(Observation::Unavailable(UnavailableReason::NoFrames), Observation::Measured))
}

pub(in crate::animation_viewer::report) fn maximum_knee_foot_yaw_offset(
    frames: &[&FrameSample],
) -> AnimationValueResult<Observation<AngularMagnitudeDegrees>> {
    if frames.is_empty() {
        return Ok(Observation::Unavailable(UnavailableReason::NoFrames));
    }
    let mut maximum = AngularMagnitudeDegrees::ZERO;
    for frame in frames {
        for value in [frame.ik_left_knee_foot_yaw_offset_degrees, frame.ik_right_knee_foot_yaw_offset_degrees] {
            let Some(value) = value else {
                return Ok(Observation::Unavailable(UnavailableReason::MissingIkDiagnostics));
            };
            maximum = maximum.max(value);
        }
    }
    Ok(Observation::Measured(maximum))
}

pub(in crate::animation_viewer::report) fn minimum_knee_bend(
    frames: &[&FrameSample],
) -> ReportResult<Observation<SignedLength>> {
    let mut minimum: Option<SignedLength> = None;
    for frame in frames {
        for LegBones { hip: hip_role, knee: knee_role, foot: foot_role } in LEGS {
            let mut positions = [Vec3::ZERO; 3];
            for (index, bone) in [hip_role, knee_role, foot_role].into_iter().enumerate() {
                let Some(sample) = frame.bones.get(bone) else {
                    return Ok(Observation::Unavailable(UnavailableReason::MissingBone { bone }));
                };
                positions[index] = Vec3::from_array(sample.position);
            }
            let [hip, knee, foot] = positions;
            let axis = foot - hip;
            if axis.length_squared() > 0.0001 {
                let closest = hip + axis * (knee - hip).dot(axis) / axis.length_squared();
                let Some(forward) = Vec3::from_array(frame.world_travel_direction).try_normalize() else {
                    return Ok(Observation::Unavailable(UnavailableReason::NoMotionSamples));
                };
                let value = SignedLength::from_metres((knee - closest).dot(forward))?;
                minimum = Some(minimum.map_or(value, |minimum| minimum.min(value)));
            }
        }
    }
    Ok(minimum.map_or(Observation::Unavailable(if frames.is_empty() { UnavailableReason::NoFrames } else { UnavailableReason::DegenerateLegAxis }), Observation::Measured))
}
