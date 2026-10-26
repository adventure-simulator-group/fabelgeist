//! Captured continuity, upper-body loop seams and world-plant drift.

use super::*;
use adventuresim_tactical_core::animation::{AngularMagnitudeDegrees, NonnegativeLength};

struct ContinuityBoneSample {
    bone: BoneRole,
    from_frame: usize,
    to_frame: usize,
    displacement: NonnegativeLength,
    rotation: AngularMagnitudeDegrees,
}

impl ScenarioContinuityMetrics {
    pub(in crate::animation_viewer::report) fn measure(
        metadata: &ScenarioMetadata<'_>,
        frames: &[&FrameSample],
    ) -> AnimationValueResult<Observation<Self>> {
        if frames.len() < 3 {
            return Ok(Observation::Unavailable(UnavailableReason::NoAdjacentFrames));
        }
        let mut metrics = Self {
            maximum_root_relative_step_metres: NonnegativeLength::ZERO,
            maximum_leg_root_relative_step_metres: NonnegativeLength::ZERO,
            maximum_foot_root_relative_step_metres: NonnegativeLength::ZERO,
            maximum_knee_root_relative_step_metres: NonnegativeLength::ZERO,
            worst_displacement: None,
            maximum_bone_rotation_step_degrees: AngularMagnitudeDegrees::ZERO,
            maximum_foot_rotation_step_degrees: AngularMagnitudeDegrees::ZERO,
            worst_rotation: None,
            maximum_supported_foot_slip_metres_per_frame: Observation::NotApplicable,
        };
        // Frame zero establishes retained procedural state. Preserve the first
        // real pre-roll transition (1→2) as the beginning of continuity gates.
        for pair in frames.windows(2).skip(1) {
            let (before, after) = (pair[0], pair[1]);
            for (bone, _) in REPORT_BONES {
                if !before.bones.contains_key(bone) || !after.bones.contains_key(bone) {
                    return Ok(Observation::Unavailable(UnavailableReason::MissingBone { bone }));
                }
            }
            for name in before.bones.keys() {
                match ContinuityBoneSample::measure(before, after, *name)? {
                    Observation::Measured(sample) => metrics.include(sample),
                    Observation::Unavailable(reason) => return Ok(Observation::Unavailable(reason)),
                    Observation::NotApplicable => return Ok(Observation::NotApplicable),
                }
            }
            match supported_foot_slip(metadata, before, after)? {
                Observation::Measured(slip) => {
                    if !matches!(metrics.maximum_supported_foot_slip_metres_per_frame, Observation::Unavailable(_)) {
                        let maximum = metrics.maximum_supported_foot_slip_metres_per_frame.measured().copied().map_or(slip, |maximum| maximum.max(slip));
                        metrics.maximum_supported_foot_slip_metres_per_frame = Observation::Measured(maximum);
                    }
                }
                Observation::Unavailable(reason) => {
                    metrics.maximum_supported_foot_slip_metres_per_frame = Observation::Unavailable(reason);
                }
                Observation::NotApplicable => {}
            }
        }
        Ok(Observation::Measured(metrics))
    }
    fn include(&mut self, sample: ContinuityBoneSample) {
        if sample.displacement > self.maximum_root_relative_step_metres {
            self.maximum_root_relative_step_metres = sample.displacement;
            self.worst_displacement = Some(ContinuityLocation {
                bone: sample.bone, from_frame: sample.from_frame, to_frame: sample.to_frame, value: sample.displacement,
            });
        }
        if is_leg_bone(&sample.bone) { self.maximum_leg_root_relative_step_metres = self.maximum_leg_root_relative_step_metres.max(sample.displacement); }
        if is_foot_bone(&sample.bone) { self.maximum_foot_root_relative_step_metres = self.maximum_foot_root_relative_step_metres.max(sample.displacement); }
        if is_knee_bone(&sample.bone) { self.maximum_knee_root_relative_step_metres = self.maximum_knee_root_relative_step_metres.max(sample.displacement); }
        if is_foot_bone(&sample.bone) { self.maximum_foot_rotation_step_degrees = self.maximum_foot_rotation_step_degrees.max(sample.rotation); }
        if sample.rotation > self.maximum_bone_rotation_step_degrees {
            self.maximum_bone_rotation_step_degrees = sample.rotation;
            self.worst_rotation = Some(ContinuityLocation {
                bone: sample.bone, from_frame: sample.from_frame, to_frame: sample.to_frame, value: sample.rotation,
            });
        }
    }

}

impl ContinuityBoneSample {
    fn measure(before: &FrameSample, after: &FrameSample, bone: BoneRole) -> AnimationValueResult<Observation<Self>> {
        let Some(before_bone) = before.bones.get(bone) else {
            return Ok(Observation::Unavailable(UnavailableReason::MissingBone { bone }));
        };
        let Some(after_bone) = after.bones.get(bone) else {
            return Ok(Observation::Unavailable(UnavailableReason::MissingBone { bone }));
        };
        // The native body-frame projection preserves root subtraction before rotation.
        let after_position = target_body_local(after, Vec3::from_array(after_bone.position));
        let before_position = target_body_local(before, Vec3::from_array(before_bone.position));
        Ok(Observation::Measured(Self {
            bone,
            from_frame: before.scenario_frame,
            to_frame: after.scenario_frame,
            displacement: NonnegativeLength::from_metres(after_position.distance(before_position))?,
            rotation: AngularMagnitudeDegrees::from_degrees(body_local_rotation(after, after_bone).angle_between(body_local_rotation(before, before_bone)).to_degrees())?,
        }))
    }
}

impl LoopSeamMetrics {
    pub(in crate::animation_viewer::report) fn measure(
        metadata: &ScenarioMetadata<'_>,
        frames: &[&FrameSample],
    ) -> AnimationValueResult<Observation<Self>> {
        if !expects_loop_seam(metadata) {
            return Ok(Observation::NotApplicable);
        }
        let mut maximum: Option<Self> = None;
        for pair in frames.windows(2).filter(|pair| (pair[1].gait_phase.cycles() - pair[0].gait_phase.cycles()).abs() > 0.5) {
            match Self::between(pair[0], pair[1])? {
                Observation::Measured(seam) => {
                    maximum = Some(maximum.map_or(seam, |maximum| Self {
                        position_metres: maximum.position_metres.max(seam.position_metres),
                        rotation_degrees: maximum.rotation_degrees.max(seam.rotation_degrees),
                    }));
                }
                Observation::Unavailable(reason) => return Ok(Observation::Unavailable(reason)),
                Observation::NotApplicable => return Ok(Observation::NotApplicable),
            }
        }
        Ok(maximum.map_or(Observation::Unavailable(UnavailableReason::NoCompletedCycle), Observation::Measured))
    }
}

impl LoopSeamMetrics {
pub(in crate::animation_viewer::report) fn between(
    first: &FrameSample,
    last: &FrameSample,
) -> AnimationValueResult<Observation<LoopSeamMetrics>> {
    for (bone, _) in REPORT_BONES.into_iter().filter(|(bone, _)| !is_leg_bone(bone)) {
        if !first.bones.contains_key(bone) || !last.bones.contains_key(bone) {
            return Ok(Observation::Unavailable(UnavailableReason::MissingBone { bone }));
        }
    }
    let mut metrics = LoopSeamMetrics { position_metres: NonnegativeLength::ZERO, rotation_degrees: AngularMagnitudeDegrees::ZERO };
    for (name, a) in &first.bones {
        // Stateful foot locking owns lower-body continuity through plant slip
        // and drift; the loop seam measures repeatable authored upper body.
        if is_leg_bone(name) { continue; }
        let Some(b) = last.bones.get(*name) else {
            return Ok(Observation::Unavailable(UnavailableReason::MissingBone { bone: *name }));
        };
        let Some((last_position, first_position)) = body_local(last, *name).zip(body_local(first, *name)) else {
            return Ok(Observation::Unavailable(UnavailableReason::MissingBone { bone: *name }));
        };
        metrics.position_metres = metrics.position_metres.max(NonnegativeLength::from_metres(last_position.distance(first_position))?);
        metrics.rotation_degrees = metrics.rotation_degrees.max(AngularMagnitudeDegrees::from_degrees(body_local_rotation(first, a).angle_between(body_local_rotation(last, b)).to_degrees())?);
    }
    Ok(Observation::Measured(metrics))
}

}

pub(in crate::animation_viewer::report) fn maximum_planted_foot_drift(
    metadata: &ScenarioMetadata<'_>,
    frames: &[&FrameSample],
) -> AnimationValueResult<Observation<NonnegativeLength>> {
    if frames.is_empty() {
        return Ok(Observation::Unavailable(UnavailableReason::NoFrames));
    }
    let mut maximum: Option<NonnegativeLength> = None;
    for (foot, left) in [(BoneRole::FootLeft, true), (BoneRole::FootRight, false)] {
        let mut anchor = None;
        for frame in frames {
            if metadata.kind() != ScenarioKind::Terrain && !frame.guard_action { anchor = None; continue; }
            let support = if left { frame.left_support_weight.fraction() } else { frame.right_support_weight.fraction() };
            // Full plants remain separate from the blended per-frame slip gate.
            if support >= FULL_PLANT_SUPPORT_WEIGHT {
                let Some(bone) = frame.bones.get(foot) else {
                    return Ok(Observation::Unavailable(UnavailableReason::MissingBone { bone: foot }));
                };
                let position = Vec3::from_array(bone.position).xz();
                let origin = anchor.get_or_insert(position);
                let drift = NonnegativeLength::from_metres(position.distance(*origin))?;
                maximum = Some(maximum.map_or(drift, |maximum| maximum.max(drift)));
            } else { anchor = None; }
        }
    }
    Ok(maximum.map_or(Observation::NotApplicable, Observation::Measured))
}
