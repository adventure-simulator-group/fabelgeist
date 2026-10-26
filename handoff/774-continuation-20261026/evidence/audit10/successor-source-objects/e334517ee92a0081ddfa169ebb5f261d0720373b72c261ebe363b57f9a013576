//! Height measurements in the sampled controller and body frames.

use super::*;
use fabelgeist_geometry::SignedLength;

impl ScenarioHeightMetrics {
    pub(in crate::animation_viewer::report) fn measure(
        metadata: &ScenarioMetadata<'_>,
        frames: &[&FrameSample],
    ) -> ReportResult<Self> {
        let phase_height_expected = ExpectedVisualHeight::for_scenario(metadata)?.is_some();
        Ok(Self {
            pelvis_vertical_range_metres: root_relative_vertical_range(frames, BoneRole::Pelvis)?,
            maximum_pelvis_vertical_step_metres: root_relative_vertical_step(
                frames,
                BoneRole::Pelvis,
            )?,
            controller_vertical_range_metres: controller_vertical_range(frames)?,
            phase_height_range_metres: if phase_height_expected {
                phase_height_range(frames)?
            } else {
                Observation::NotApplicable
            },
            contact_to_passing_height_gain_metres: if phase_height_expected {
                contact_to_passing_height_gain(frames)?
            } else {
                Observation::NotApplicable
            },
            visual_height_peaks: if phase_height_expected {
                visual_height_peaks(frames)?
            } else {
                Observation::NotApplicable
            },
            maximum_no_support_seconds: maximum_no_support_seconds(frames)?,
            head_vertical_range_metres: root_relative_vertical_range(frames, BoneRole::Head)?,
        })
    }
}

pub(in crate::animation_viewer::report) fn root_relative_vertical_range(
    frames: &[&FrameSample],
    bone: BoneRole,
) -> ReportResult<Observation<HeightDistance>> {
    let mut bounds: Option<SignedLengthRange> = None;
    for frame in frames {
        let Some(sample) = frame.bones.get(bone) else {
            return Ok(Observation::Unavailable(UnavailableReason::MissingBone {
                bone,
            }));
        };
        let value = SignedLength::from_metres(sample.position[1] - frame.root_position_metres[1])?;
        bounds = Some(bounds.map_or(SignedLengthRange::from_sample(value), |bounds| bounds.include(value)));
    }
    let Some(bounds) = bounds else {
        return Ok(Observation::Unavailable(UnavailableReason::NoFrames));
    };
    Ok(Observation::Measured(HeightDistance::from_metres(
        bounds.maximum().metres() - bounds.minimum().metres(),
    )?))
}

pub(in crate::animation_viewer::report) fn root_relative_vertical_step(
    frames: &[&FrameSample],
    bone: BoneRole,
) -> ReportResult<Observation<HeightDistance>> {
    if frames.len() < 2 {
        return Ok(Observation::Unavailable(
            UnavailableReason::NoAdjacentFrames,
        ));
    }
    let mut maximum = HeightDistance::ZERO;
    for pair in frames.windows(2) {
        let Some(previous) = pair[0].bones.get(bone) else {
            return Ok(Observation::Unavailable(UnavailableReason::MissingBone {
                bone,
            }));
        };
        let Some(current) = pair[1].bones.get(bone) else {
            return Ok(Observation::Unavailable(UnavailableReason::MissingBone {
                bone,
            }));
        };
        let previous = previous.position[1] - pair[0].root_position_metres[1];
        let current = current.position[1] - pair[1].root_position_metres[1];
        maximum = maximum.max(HeightDistance::from_metres((current - previous).abs())?);
    }
    Ok(Observation::Measured(maximum))
}

pub(in crate::animation_viewer::report) fn hard_stop_pelvis_vertical_step(
    frames: &[FrameSample],
) -> ReportResult<Observation<HeightDistance>> {
    let hard_stop = frames
        .iter()
        .filter(|frame| frame.scenario.is(ScenarioId::HardStop))
        .collect::<Vec<_>>();
    if hard_stop.is_empty() {
        return Ok(Observation::NotApplicable);
    }
    let Some(first_stopped) = hard_stop.iter().position(|frame| {
        frame.speed_metres_per_second <= SignedSpeed::from(STATIONARY_ANIMATION_SPEED_LIMIT)
    }) else {
        return Ok(Observation::Unavailable(UnavailableReason::NoStoppedFrame));
    };
    let transition_start = first_stopped.saturating_sub(1);
    root_relative_vertical_step(&hard_stop[transition_start..], BoneRole::Pelvis)
}

pub(in crate::animation_viewer::report) fn controller_vertical_range(
    frames: &[&FrameSample],
) -> ReportResult<Observation<HeightDistance>> {
    let mut bounds: Option<SignedLengthRange> = None;
    for frame in frames {
        let value = SignedLength::from_metres(frame.root_position_metres[1])?;
        bounds = Some(bounds.map_or(SignedLengthRange::from_sample(value), |bounds| bounds.include(value)));
    }
    let Some(bounds) = bounds else { return Ok(Observation::Unavailable(UnavailableReason::NoFrames)); };
    Ok(Observation::Measured(HeightDistance::from_metres(bounds.maximum().metres() - bounds.minimum().metres())?))
}

pub(in crate::animation_viewer::report) fn contact_to_passing_height_gain(
    frames: &[&FrameSample],
) -> ReportResult<Observation<SignedLength>> {
    let frames = phase_validation_frames(frames);
    let phase_distance = |phase: f32, target: f32| {
        let distance = (phase - target).abs();
        distance.min(1.0 - distance)
    };
    let mut contact: Option<SignedLength> = None;
    for frame in frames.iter().filter(|frame| {
        phase_distance(frame.gait_phase.cycles(), 0.0) <= 0.04
            || phase_distance(frame.gait_phase.cycles(), 0.5) <= 0.04
    }) {
        let Some(pelvis) = body_local(frame, BoneRole::Pelvis) else {
            return Ok(Observation::Unavailable(UnavailableReason::MissingBone {
                bone: BoneRole::Pelvis,
            }));
        };
        let height = SignedLength::from_metres(pelvis.y)?;
        contact = Some(contact.map_or(height, |maximum| maximum.max(height)));
    }
    let mut passing: Option<SignedLength> = None;
    for frame in frames.iter().filter(|frame| {
        phase_distance(frame.gait_phase.cycles(), 0.25) <= 0.04
            || phase_distance(frame.gait_phase.cycles(), 0.75) <= 0.04
    }) {
        let Some(pelvis) = body_local(frame, BoneRole::Pelvis) else {
            return Ok(Observation::Unavailable(UnavailableReason::MissingBone {
                bone: BoneRole::Pelvis,
            }));
        };
        let height = SignedLength::from_metres(pelvis.y)?;
        passing = Some(passing.map_or(height, |minimum| minimum.min(height)));
    }
    let Some(contact) = contact else {
        return Ok(Observation::Unavailable(
            UnavailableReason::NoContactHeightSamples,
        ));
    };
    let Some(passing) = passing else {
        return Ok(Observation::Unavailable(
            UnavailableReason::NoPassingHeightSamples,
        ));
    };
    Ok(Observation::Measured(SignedLength::from_metres(
        passing.metres() - contact.metres(),
    )?))
}

pub(in crate::animation_viewer::report) fn visual_height_peaks(
    frames: &[&FrameSample],
) -> ReportResult<Observation<VisualHeightPeaks>> {
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
            let mut samples = Vec::new();
            for frame in cycle {
                let Some(pelvis) = body_local(frame, BoneRole::Pelvis) else {
                    return Ok(Observation::Unavailable(UnavailableReason::MissingBone {
                        bone: BoneRole::Pelvis,
                    }));
                };
                samples.push(HeightSample::new(frame.gait_phase, SignedLength::from_metres(pelvis.y)?));
            }
            match prominent_height_peaks(&samples)? {
                Observation::Measured(measurement) => measurements.push(measurement),
                Observation::Unavailable(reason) => return Ok(Observation::Unavailable(reason)),
                Observation::NotApplicable => return Ok(Observation::NotApplicable),
            }
        }
        cycle_start = cycle_end;
    }
    let Some(peaks) = VisualHeightPeaks::from_cycles(&measurements) else {
        return Ok(Observation::Unavailable(UnavailableReason::NoCompletedCycle));
    };
    Ok(Observation::Measured(peaks))
}

pub(in crate::animation_viewer::report) fn phase_height_range(
    frames: &[&FrameSample],
) -> ReportResult<Observation<HeightDistance>> {
    let frames = phase_validation_frames(frames);
    root_relative_vertical_range(&frames, BoneRole::Pelvis)
}

pub(in crate::animation_viewer::report) fn phase_validation_frames<'a>(
    frames: &[&'a FrameSample],
) -> Vec<&'a FrameSample> {
    let after_first_wrap = frames
        .windows(2)
        .position(|pair| pair[1].gait_phase.cycles() < pair[0].gait_phase.cycles())
        .map_or(0, |index| index + 1);
    frames[after_first_wrap..].to_vec()
}

pub(in crate::animation_viewer::report) fn maximum_no_support_seconds(
    frames: &[&FrameSample],
) -> AnimationValueResult<Observation<AnimationElapsed>> {
    if frames.is_empty() {
        return Ok(Observation::Unavailable(UnavailableReason::NoFrames));
    }
    let mut maximum = 0.0_f32;
    let mut began = None;
    for frame in frames {
        if frame.left_support_weight.fraction() <= 0.001
            && frame.right_support_weight.fraction() <= 0.001
        {
            let began = began.get_or_insert(frame.time_seconds);
            maximum = maximum.max(frame.time_seconds.seconds() - began.seconds());
        } else {
            began = None;
        }
    }
    let elapsed = AnimationElapsed::from_seconds(maximum).map_err(AnimationValueError::from)?;
    Ok(Observation::Measured(elapsed))
}
