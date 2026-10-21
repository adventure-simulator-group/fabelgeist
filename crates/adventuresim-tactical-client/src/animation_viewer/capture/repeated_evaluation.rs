//! Repeated logical-tick comparison across capture views.

use super::*;
use adventuresim_tactical_core::animation::{
    AngularMagnitudeDegrees, AnimationValueError, NonnegativeLength,
};

pub(super) struct RepeatedEvaluationSnapshot {
    scenario: ScenarioMetadata<'static>,
    scenario_frame: usize,
    bones: CapturedBones,
    contact_sequence: ContactSequence,
    landing_sequence: adventuresim_tactical_core::animation::LandingSequence,
    event_count: usize,
    leg_ik: Option<LegIkDiagnostics>,
}

#[derive(Debug)]
pub(super) enum RepeatedBoneMismatch {
    MissingBone { bone: BoneRole },
    UnexpectedBone { bone: BoneRole },
    Transform {
        bone: BoneRole,
        position_delta: NonnegativeLength,
        rotation_delta: AngularMagnitudeDegrees,
    },
    InvalidObservation { bone: BoneRole, cause: AnimationValueError },
}

impl RepeatedBoneMismatch {
    fn bone(&self) -> BoneRole {
        match self {
            Self::MissingBone { bone } | Self::UnexpectedBone { bone }
            | Self::Transform { bone, .. } | Self::InvalidObservation { bone, .. } => *bone,
        }
    }
}

impl CaptureSequence {
    pub(super) fn record_repeated_evaluation(
        &mut self,
        frame: &PlannedFrame,
        bones: &CapturedBones,
        skeleton: &SkeletonState,
        leg_ik: Option<LegIkDiagnostics>,
    ) {
        let event_count = self.presentation_events.len();
        if self.view_index == 0 {
            self.repeated_evaluation_baseline = Some(RepeatedEvaluationSnapshot {
                scenario: frame.scenario.clone(),
                scenario_frame: frame.scenario_frame,
                bones: bones.clone(),
                contact_sequence: skeleton.contact_sequence,
                landing_sequence: skeleton.landing_sequence,
                event_count,
                leg_ik,
            });
        } else if let Some(baseline) = &self.repeated_evaluation_baseline {
            let repeated_evaluation_matches =
                baseline.matches(frame, bones, skeleton, event_count, leg_ik);
            self.repeated_evaluation_valid &= repeated_evaluation_matches;
        } else {
            self.repeated_evaluation_valid = false;
            warn!(scenario = frame.scenario.label(), scenario_frame = frame.scenario_frame,
                "repeated animation evaluation has no baseline observation");
        }
    }
}

impl RepeatedEvaluationSnapshot {
    fn matches(
        &self,
        frame: &PlannedFrame,
        bones: &CapturedBones,
        skeleton: &SkeletonState,
        event_count: usize,
        leg_ik: Option<LegIkDiagnostics>,
    ) -> bool {
        let bone_mismatch = repeated_bone_mismatch(&self.bones, bones);
        let bones_match = bone_mismatch.is_none();
        let repeated_evaluation_matches = self.scenario == frame.scenario
            && self.scenario_frame == frame.scenario_frame
            && bones_match
            && self.contact_sequence == skeleton.contact_sequence
            && self.landing_sequence == skeleton.landing_sequence
            && self.event_count == event_count
            && repeated_leg_ik_matches(self.leg_ik, leg_ik);
        if !repeated_evaluation_matches
            && let Some(mismatch) = &bone_mismatch
        {
            let bone = mismatch.bone();
            let (position_delta, rotation_delta) = match mismatch {
                RepeatedBoneMismatch::Transform { position_delta, rotation_delta, .. } => {
                    (Some(position_delta.metres()), Some(rotation_delta.degrees()))
                }
                _ => (None, None),
            };
            let invalid_cause = match mismatch {
                RepeatedBoneMismatch::InvalidObservation { cause, .. } => Some(cause),
                _ => None,
            };
            warn!(
                scenario = frame.scenario.label(),
                scenario_frame = frame.scenario_frame,
                bone = report_bone_label(bone)
                    .map(str::to_owned)
                    .unwrap_or_else(|| format!("{bone:?}")),
                ?position_delta,
                ?rotation_delta,
                ?invalid_cause,
                ?mismatch,
                "repeated animation evaluation changed a captured bone"
            );
        }
        if !repeated_evaluation_matches && bone_mismatch.is_none() {
            warn!(
                scenario = frame.scenario.label(),
                scenario_frame = frame.scenario_frame,
                baseline_contact = self.contact_sequence.encoded(),
                contact = skeleton.contact_sequence.encoded(),
                baseline_landing = self.landing_sequence.encoded(),
                landing = skeleton.landing_sequence.encoded(),
                baseline_events = self.event_count,
                events = event_count,
                "repeated animation evaluation changed non-bone state"
            );
        }
        repeated_evaluation_matches
    }
}

pub(super) fn repeated_bone_mismatch(
    expected: &CapturedBones,
    actual: &CapturedBones,
) -> Option<RepeatedBoneMismatch> {
    if let Some(bone) = expected.keys().find(|bone| !actual.contains_key(**bone)) {
        return Some(RepeatedBoneMismatch::MissingBone { bone: *bone });
    }
    if let Some(bone) = actual.keys().find(|bone| !expected.contains_key(**bone)) {
        return Some(RepeatedBoneMismatch::UnexpectedBone { bone: *bone });
    }
    // Equal key sets retain the same lexical report order in both captures.
    expected.iter().zip(actual.iter()).find_map(|((name, expected), (_, actual))| {
        let position_delta =
            Vec3::from_array(expected.position).distance(Vec3::from_array(actual.position));
        let expected_rotation = Quat::from_array(expected.rotation_xyzw);
        let actual_rotation = Quat::from_array(actual.rotation_xyzw);
        // `acos` quantizes identical f32 quaternions to roughly 0.056-0.079
        // degrees on some frames. Treat a direct dot match as identity, then
        // retain the angular report for genuine changes.
        let rotation_dot = expected_rotation.dot(actual_rotation).abs();
        let rotation_delta = if rotation_dot >= 1.0 - 0.000_001 {
            0.0
        } else {
            expected_rotation
                .angle_between(actual_rotation)
                .to_degrees()
        };
        // Re-evaluating one logical tick must be visually identical. Keep only
        // sub-half-millimetre/sub-twentieth-degree numeric noise.
        let position_delta = match NonnegativeLength::from_metres(position_delta) {
            Ok(delta) => delta,
            Err(cause) => return Some(RepeatedBoneMismatch::InvalidObservation { bone: *name, cause }),
        };
        let rotation_delta = match AngularMagnitudeDegrees::from_degrees(rotation_delta) {
            Ok(delta) => delta,
            Err(cause) => return Some(RepeatedBoneMismatch::InvalidObservation { bone: *name, cause }),
        };
        (position_delta.metres() > 0.0005 || rotation_delta.degrees() > 0.05).then_some(
            RepeatedBoneMismatch::Transform { bone: *name, position_delta, rotation_delta },
        )
    })
}

pub(super) fn repeated_leg_ik_matches(
    expected: Option<LegIkDiagnostics>,
    actual: Option<LegIkDiagnostics>,
) -> bool {
    let (expected, actual) = match (expected, actual) {
        (Some(expected), Some(actual)) => (expected, actual),
        (None, None) => return true,
        _ => return false,
    };
    let option_vec3_matches =
        |expected: Option<Vec3>, actual: Option<Vec3>| match (expected, actual) {
            (Some(expected), Some(actual)) => expected.distance(actual) <= 0.0005,
            (None, None) => true,
            _ => false,
        };
    let option_scalar_matches =
        |expected: Option<f32>, actual: Option<f32>| match (expected, actual) {
            (Some(expected), Some(actual)) => (expected - actual).abs() <= 0.001,
            (None, None) => true,
            _ => false,
        };
    option_vec3_matches(expected.left_authored_target, actual.left_authored_target)
        && option_vec3_matches(expected.right_authored_target, actual.right_authored_target)
        && option_vec3_matches(expected.left_planned_contact, actual.left_planned_contact)
        && option_vec3_matches(expected.right_planned_contact, actual.right_planned_contact)
        && option_vec3_matches(expected.settle_capture_point, actual.settle_capture_point)
        && option_vec3_matches(expected.left_solve_target, actual.left_solve_target)
        && option_vec3_matches(expected.right_solve_target, actual.right_solve_target)
        && option_scalar_matches(
            expected.left_support_weight.map(|weight| weight.fraction()),
            actual.left_support_weight.map(|weight| weight.fraction()),
        )
        && option_scalar_matches(
            expected
                .right_support_weight
                .map(|weight| weight.fraction()),
            actual.right_support_weight.map(|weight| weight.fraction()),
        )
        && expected.left_release_active == actual.left_release_active
        && expected.right_release_active == actual.right_release_active
        && option_vec3_matches(expected.left_release_target, actual.left_release_target)
        && option_vec3_matches(expected.right_release_target, actual.right_release_target)
        && option_scalar_matches(
            expected.settle_progress.map(|progress| progress.fraction()),
            actual.settle_progress.map(|progress| progress.fraction()),
        )
        && (expected.left_knee_foot_yaw_offset_degrees.degrees()
            - actual.left_knee_foot_yaw_offset_degrees.degrees())
        .abs()
            <= 0.05
        && (expected.right_knee_foot_yaw_offset_degrees.degrees()
            - actual.right_knee_foot_yaw_offset_degrees.degrees())
        .abs()
            <= 0.05
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repeated_evaluation_rejects_missing_or_extra_tracked_bones() {
        let sample = BoneSample {
            position: Vec3::ZERO.to_array(),
            rotation_xyzw: Quat::IDENTITY.to_array(),
            terrain_clearance_metres: Some(fabelgeist_geometry::SignedLength::from_metres(0.0).unwrap()),
        };
        let expected = CapturedBones::from_iter([(BoneRole::FootLeft, sample)]);
        let missing = CapturedBones::default();
        assert!(repeated_bone_mismatch(&expected, &missing).is_some());
        assert!(repeated_bone_mismatch(&missing, &expected).is_some());
        assert!(repeated_bone_mismatch(&expected, &expected).is_none());
    }

    #[test]
    fn repeated_evaluation_diagnostics_detect_hidden_ik_state_mutation() {
        let baseline = LegIkDiagnostics::default();
        let changed = LegIkDiagnostics {
            left_support_weight: Some(adventuresim_tactical_core::animation::UnitInterval::ONE),
            left_release_active: true,
            left_planned_contact: Some(Vec3::NEG_Z),
            settle_progress: Some(
                adventuresim_tactical_core::animation::SettleProgress::from_progress(0.5).unwrap(),
            ),
            ..default()
        };
        assert_ne!(baseline, changed);
    }
}
