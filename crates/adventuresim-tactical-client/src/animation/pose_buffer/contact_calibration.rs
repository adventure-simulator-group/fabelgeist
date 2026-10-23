//! Infer cadence from rig-local foot contacts and retain checked metre measurements.

use super::phase_calibration::{
    AuthoredContactSample, ContactOrientation, ContactPhaseFit, FootTrajectorySample,
    PhysicalContactSample, StanceDirection, low_height_contacts,
};
use super::*;

/// Infer travel from the low portion of each authored foot trajectory. This
/// discovers contact timing from clip geometry, then fits the single virtual
/// root velocity that best holds every stance segment still. The residual is
/// retained because some clips cannot represent a requested speed with a
/// constant playback rate alone.
#[derive(Debug, Clone)]
pub(super) struct AuthoredLocomotionCalibration {
    pub(super) stride: AuthoredStrideMeasurement,
    pub(super) phase_curve: Option<AuthoredPhaseCurve>,
}

pub(super) fn measure_authored_contact_step_distance(
    definition: &RigDefinition,
    clip: &BakedClip,
    travel_axis: CalibrationAxis,
    expected_stance_direction: StanceDirection,
) -> AnimationValueResult<Option<AuthoredLocomotionCalibration>> {
    if clip.duration.seconds() <= 0.0 || clip.frame_count().samples() < 4 {
        return Ok(None);
    }
    let Some(segments) = contact_segments(definition, clip, travel_axis)? else {
        return Ok(None);
    };
    let Some(orientation) = ContactOrientation::fit(&segments, expected_stance_direction)? else {
        return Ok(None);
    };
    let Some(fitted) = ContactPhaseFit::fit(&segments, orientation, expected_stance_direction)?
    else {
        return Ok(None);
    };
    fit_stride(fitted.trajectories, fitted.curve, expected_stance_direction)
}

pub(super) fn measure_authored_foot_range(
    definition: &RigDefinition,
    clip: &BakedClip,
    travel_axis: CalibrationAxis,
) -> AnimationValueResult<Option<NonnegativeLength>> {
    let feet =
        [RigJointName::L_FOOT, RigJointName::R_FOOT].map(|name| definition.joint_ordinal(&name));
    let [Some(left), Some(right)] = feet else {
        return Ok(None);
    };
    let mut ranges = Vec::with_capacity(2);
    for foot in [left, right] {
        // Neutral extrema belong only to this nonempty native sampled-position fold.
        let mut minimum = f32::INFINITY;
        let mut maximum = f32::NEG_INFINITY;
        for frame in 0..clip.frame_count().samples() {
            let mut globals = RigJointTable::from(vec![None; definition.joints.len()]);
            let position = travel_axis.component(
                sampled_global_transform(
                    definition,
                    clip,
                    foot,
                    clip.frame_time(frame)?,
                    &mut globals,
                )
                .translation,
            )?;
            minimum = minimum.min(position.metres());
            maximum = maximum.max(position.metres());
        }
        ranges.push(NonnegativeLength::from_arithmetic(maximum - minimum)?);
    }
    let stride =
        NonnegativeLength::from_arithmetic((ranges[0].metres() + ranges[1].metres()) * 0.5)?;
    Ok((pose_tuning().minimum_authored_foot_range_metres.metres()
        ..=pose_tuning().maximum_authored_foot_range_metres.metres())
        .contains(&stride.metres())
        .then_some(stride))
}

fn contact_segments(
    definition: &RigDefinition,
    clip: &BakedClip,
    travel_axis: CalibrationAxis,
) -> AnimationValueResult<Option<Vec<Vec<AuthoredContactSample>>>> {
    let sample_count = clip.frame_count().intervals();
    let mut segments = Vec::new();
    for name in [RigJointName::L_FOOT, RigJointName::R_FOOT] {
        let Some(foot) = definition.joint_ordinal(&name) else {
            return Ok(None);
        };
        let samples = (0..sample_count)
            .map(|frame| {
                let time = clip.frame_time(frame)?;
                let mut globals = RigJointTable::from(vec![None; definition.joints.len()]);
                let position = sampled_global_transform(definition, clip, foot, time, &mut globals)
                    .translation;
                if !position.is_finite() {
                    return Err(AnimationValueError::NonFinite {
                        quantity: AnimationQuantity::CalibrationPosition,
                    });
                }
                Ok(FootTrajectorySample {
                    authored: CyclePhase::from_arithmetic(
                        time.seconds() / clip.duration.seconds(),
                    )?,
                    position,
                })
            })
            .collect::<AnimationValueResult<Vec<_>>>()?;
        let Some(supported) = low_height_contacts(&samples)? else {
            return Ok(None);
        };
        if supported.iter().all(|supported| *supported) {
            return Ok(None);
        }
        // Start after a non-contact sample: a seam-crossing stance is one unwrapped segment.
        let Some(start) = supported.iter().position(|supported| !*supported) else {
            return Ok(None);
        };
        let mut current = Vec::new();
        let mut foot_segments = Vec::new();
        for offset in 1..=sample_count {
            let index = (start + offset) % sample_count;
            if supported[index] {
                let wraps = (start + offset) / sample_count;
                current.push(AuthoredContactSample {
                    authored: CyclePhase::from_arithmetic(
                        samples[index].authored.cycles() + wraps as f32,
                    )?,
                    position: travel_axis.component(samples[index].position)?,
                });
            } else if !current.is_empty() {
                if current.len() >= 3 {
                    foot_segments.push(std::mem::take(&mut current));
                } else {
                    current.clear();
                }
            }
        }
        if current.len() >= 3 {
            foot_segments.push(current);
        }
        // A low swing-through can enter the height band; retain the longest circular interval.
        let Some(segment) = foot_segments.into_iter().max_by_key(Vec::len) else {
            return Ok(None);
        };
        segments.push(segment);
    }
    Ok(Some(segments))
}

fn fit_stride(
    warped_segments: Vec<Vec<PhysicalContactSample>>,
    curve: AuthoredPhaseCurve,
    expected_stance_direction: StanceDirection,
) -> AnimationValueResult<Option<AuthoredLocomotionCalibration>> {
    // Each stance gets its own intercept; only the common virtual-root slope
    // is shared between feet and contact lobes. Measure the residual after the
    // phase warp, because that is the curve actually sampled at runtime.
    let mut covariance = 0.0;
    let mut phase_variance = 0.0;
    for samples in &warped_segments {
        let count = samples.len() as f32;
        let mean_phase = samples
            .iter()
            .map(|sample| sample.physical.cycles())
            .sum::<f32>()
            / count;
        let mean_position = samples
            .iter()
            .map(|sample| sample.position.metres())
            .sum::<f32>()
            / count;
        for sample in samples {
            let centered_phase = sample.physical.cycles() - mean_phase;
            covariance += centered_phase * (sample.position.metres() - mean_position);
            phase_variance += centered_phase * centered_phase;
        }
    }
    if phase_variance <= 0.0 {
        return Ok(None);
    }
    let fitted_slope = covariance / phase_variance;
    let cycle_distance = fitted_slope * expected_stance_direction.sign();
    let step_distance = cycle_distance * 0.5;
    let maximum_stance_slip = warped_segments
        .iter()
        .map(|samples| {
            let residuals = samples
                .iter()
                .map(|sample| sample.position.metres() - fitted_slope * sample.physical.cycles());
            let (minimum, maximum) = residuals.fold(
                (f32::INFINITY, f32::NEG_INFINITY),
                |(minimum, maximum), residual| (minimum.min(residual), maximum.max(residual)),
            );
            maximum - minimum
        })
        .fold(0.0_f32, f32::max);
    if !step_distance.is_finite() || !maximum_stance_slip.is_finite() {
        return Err(AnimationValueError::Overflow {
            quantity: AnimationQuantity::CalibrationPosition,
        });
    }
    if cycle_distance <= 0.0 {
        return Ok(None);
    }
    let step_distance = NonnegativeLength::from_arithmetic(step_distance)?;
    let maximum_stance_slip = NonnegativeLength::from_arithmetic(maximum_stance_slip)?;
    if !(pose_tuning().minimum_authored_foot_range_metres.metres()
        ..=pose_tuning().maximum_authored_foot_range_metres.metres())
        .contains(&step_distance.metres())
    {
        return Ok(None);
    }
    Ok(Some(AuthoredLocomotionCalibration {
        stride: AuthoredStrideMeasurement {
            step_distance,
            maximum_stance_slip,
        },
        phase_curve: Some(curve),
    }))
}

#[cfg(test)]
mod tests {
    use super::super::baking::{BakedFrameCount, BoneTrack};
    use super::*;
    #[test]
    fn authored_foot_range_is_the_average_foot_travel_along_the_motion_axis() {
        let joint = |name: &str, parent: Option<usize>| {
            let identity = AuthoredRigJoint::new(name.into());
            RigJoint {
                target: AnimationTargetId::from_name(&Name::new(name.to_owned())),
                bind: LocalPose::from_transform(Transform::from_translation(Vec3::ZERO)),
                parent: parent.map(RigJointOrdinal::from),
                name: Some(identity.name().clone()),
                role: identity.role(),
                body_region: AnimationBodyRegion::Lower,
            }
        };
        let definition = RigDefinition {
            family: "test".into(),
            joints: vec![
                joint("root", None),
                joint("l_foot", Some(0)),
                joint("r_foot", Some(0)),
            ]
            .into(),
            name_ordinals: Default::default(),
        };
        let track = |translations| BoneTrack {
            translations,
            rotations: vec![Quat::IDENTITY; 3],
            scales: vec![Vec3::ONE; 3],
            source: BoneTrackSource::Authored,
        };
        let clip = BakedClip::new(
            ClipDuration::from_seconds(2.0 / 30.0).unwrap(),
            adventuresim_tactical_core::animation::PositiveAnimationDuration::try_from(1.0 / 30.0)
                .unwrap(),
            BakedFrameCount::try_from(3).unwrap(),
            vec![
                track(vec![Vec3::ZERO; 3]),
                track(vec![Vec3::ZERO, Vec3::Z, Vec3::Z * 2.0]),
                track(vec![Vec3::ZERO, Vec3::Z * -1.5, Vec3::Z * -3.0]),
            ]
            .into(),
        )
        .unwrap();

        assert_eq!(
            measure_authored_foot_range(&definition, &clip, CalibrationAxis::Longitudinal).unwrap(),
            Some(NonnegativeLength::from_metres(2.5).unwrap())
        );
    }

    #[test]
    fn authored_contact_fit_ignores_airborne_foot_excursions() {
        let joint = |name: &str, parent: Option<usize>| {
            let identity = AuthoredRigJoint::new(name.into());
            RigJoint {
                target: AnimationTargetId::from_name(&Name::new(name.to_owned())),
                bind: LocalPose::from_transform(Transform::from_translation(Vec3::ZERO)),
                parent: parent.map(RigJointOrdinal::from),
                name: Some(identity.name().clone()),
                role: identity.role(),
                body_region: AnimationBodyRegion::Lower,
            }
        };
        let definition = RigDefinition {
            family: "test".into(),
            joints: vec![
                joint("root", None),
                joint("l_foot", Some(0)),
                joint("r_foot", Some(0)),
            ]
            .into(),
            name_ordinals: Default::default(),
        };
        let frames = 21;
        let duration = 1.0;
        let frame_dt = duration / (frames - 1) as f32;
        let foot_track = |contact_phase: f32, airborne_offset: f32| BoneTrack {
            translations: (0..frames)
                .map(|frame| {
                    let phase = frame as f32 / (frames - 1) as f32;
                    let stance_phase = phase - contact_phase;
                    let (position, height) = if (0.0..=0.2).contains(&stance_phase) {
                        // A two-metre virtual root displacement toward -Z per
                        // cycle exactly cancels this +Z foot retraction.
                        (2.0 * stance_phase, 0.0)
                    } else {
                        (airborne_offset + phase * phase * 10.0, 0.5)
                    };
                    Vec3::new(0.0, height, position)
                })
                .collect(),
            rotations: vec![Quat::IDENTITY; frames],
            scales: vec![Vec3::ONE; frames],
            source: BoneTrackSource::Authored,
        };
        let clip = BakedClip::new(
            ClipDuration::from_seconds(duration).unwrap(),
            adventuresim_tactical_core::animation::PositiveAnimationDuration::try_from(frame_dt)
                .unwrap(),
            BakedFrameCount::try_from(frames).unwrap(),
            vec![
                BoneTrack {
                    translations: vec![Vec3::ZERO; frames],
                    rotations: vec![Quat::IDENTITY; frames],
                    scales: vec![Vec3::ONE; frames],
                    source: BoneTrackSource::Authored,
                },
                foot_track(0.1, 50.0),
                foot_track(0.6, -80.0),
            ]
            .into(),
        )
        .unwrap();

        let measurement = measure_authored_contact_step_distance(
            &definition,
            &clip,
            CalibrationAxis::Longitudinal,
            StanceDirection::Increasing,
        )
        .unwrap()
        .expect("the two stance windows should determine travel");
        assert!((measurement.stride.step_distance.metres() - 1.0).abs() < 0.0001);
        assert!(measurement.stride.maximum_stance_slip.metres() < 0.0001);
        assert!(measurement.phase_curve.is_some());

        let mut projected_rotation = clip.clone();
        for (track_index, contact_phase) in [(1usize, 0.1_f32), (2, 0.6_f32)] {
            let translations =
                projected_rotation.translations_mut(RigJointOrdinal::from(track_index));
            for (frame, translation) in translations.iter_mut().enumerate() {
                let phase = frame as f32 / (frames - 1) as f32;
                let stance_phase = phase - contact_phase;
                if (-0.0001..=0.2001).contains(&stance_phase) {
                    // A quadratic projection stands in for the sine-like
                    // horizontal motion produced by a rotating leg.
                    translation.y = 0.0;
                    translation.z = 10.0 * stance_phase * stance_phase;
                }
            }
        }
        let projected = measure_authored_contact_step_distance(
            &definition,
            &projected_rotation,
            CalibrationAxis::Longitudinal,
            StanceDirection::Increasing,
        )
        .unwrap()
        .expect("a monotone articulated projection should be phase-warpable");
        assert!((projected.stride.step_distance.metres() - 1.0).abs() < 0.02);
        assert!(projected.stride.maximum_stance_slip.metres() < 0.08);
        let curve = projected
            .phase_curve
            .expect("ordinary gait derives a curve");
        assert!(
            curve
                .sample(CyclePhase::from_cycles(0.2).unwrap())
                .unwrap()
                .cycles()
                > 0.22,
            "distance inversion must be nonlinear"
        );

        let mut low_wrong_way_approach = clip.clone();
        for (track_index, indices) in [(1usize, [0, 1]), (2, [10, 11])] {
            let translations =
                low_wrong_way_approach.translations_mut(RigJointOrdinal::from(track_index));
            for (offset, index) in indices.into_iter().enumerate() {
                translations[index] = Vec3::new(0.0, 0.005, 0.2 - offset as f32 * 0.1);
            }
        }
        let misleading = measure_authored_contact_step_distance(
            &definition,
            &low_wrong_way_approach,
            CalibrationAxis::Longitudinal,
            StanceDirection::Increasing,
        )
        .unwrap();
        assert!(
            misleading.is_none_or(|measurement| {
                measurement.stride.maximum_stance_slip.metres()
                    > presentation::maximum_authored_stance_slip_metres().metres()
            }),
            "a visibly low wrong-way approach must make the fit unrepresentable"
        );

        let mut wrong_way = clip.clone();
        for joint in definition.joints.ordinals().skip(1) {
            for translation in wrong_way.translations_mut(joint) {
                translation.z = -translation.z;
            }
        }
        let reversed = measure_authored_contact_step_distance(
            &definition,
            &wrong_way,
            CalibrationAxis::Longitudinal,
            StanceDirection::Increasing,
        )
        .unwrap()
        .expect("a cyclic clip authored backward should be sampled in reverse");
        assert!((reversed.stride.step_distance.metres() - 1.0).abs() < 0.0001);
        let curve = reversed.phase_curve.unwrap();
        let authored_delta = (curve
            .sample(CyclePhase::from_cycles(0.25).unwrap())
            .unwrap()
            .cycles()
            - curve
                .sample(CyclePhase::from_cycles(0.2).unwrap())
                .unwrap()
                .cycles()
            + 0.5)
            .rem_euclid(1.0)
            - 0.5;
        assert!(
            authored_delta < 0.0,
            "reverse playback must decrease authored phase while physical phase advances"
        );
    }
}
