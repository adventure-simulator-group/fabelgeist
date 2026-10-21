//! Completed raised-guard steps retain both liveness evidence and its absence.

use super::*;
use adventuresim_tactical_core::animation::NonnegativeLength;
use fabelgeist_geometry::SignedLength;

struct GuardHalfStep {
    travel: NonnegativeLength,
    clearance_gain: SignedLength,
    visible: bool,
}

struct SwingSupport {
    foot: BoneRole,
    start: UnitInterval,
    end: UnitInterval,
    other_start: UnitInterval,
    other_end: UnitInterval,
}

impl GuardStepLivenessMetrics {
    pub(in crate::animation_viewer::report) fn measure(
        frames: &[&FrameSample],
    ) -> ReportResult<Observation<Self>> {
        if !Self::is_required(frames) { return Ok(Observation::NotApplicable); }
        let mut interval_start = 0;
        let mut completed_half_steps = 0;
        let mut visible_half_steps = 0;
        let mut minimum_swing_travel: Option<NonnegativeLength> = None;
        let mut minimum_swing_clearance_gain: Option<SignedLength> = None;
        for interval_end in 1..frames.len() {
            if frames[interval_end].contact_sequence == frames[interval_end - 1].contact_sequence { continue; }
            match GuardHalfStep::measure(&frames[interval_start..=interval_end])? {
                Observation::Measured(step) => {
                    completed_half_steps += 1;
                    visible_half_steps += usize::from(step.visible);
                    minimum_swing_travel = Some(minimum_swing_travel.map_or(step.travel, |minimum| minimum.min(step.travel)));
                    minimum_swing_clearance_gain = Some(minimum_swing_clearance_gain.map_or(step.clearance_gain, |minimum| minimum.min(step.clearance_gain)));
                }
                Observation::Unavailable(reason) => return Ok(Observation::Unavailable(reason)),
                Observation::NotApplicable => return Ok(Observation::NotApplicable),
            }
            interval_start = interval_end;
        }
        let Some((minimum_swing_travel_metres, minimum_swing_clearance_gain_metres)) = minimum_swing_travel.zip(minimum_swing_clearance_gain) else {
            return Ok(Observation::Unavailable(UnavailableReason::NoCompletedHalfStep));
        };
        Ok(Observation::Measured(Self { completed_half_steps, visible_half_steps, minimum_swing_travel_metres, minimum_swing_clearance_gain_metres }))
    }

    fn is_required(frames: &[&FrameSample]) -> bool {
        frames.first().is_some_and(|frame| {
            let metadata = frame.scenario.clone();
            metadata.kind() == ScenarioKind::RaisedGuard
                && metadata.procedural_solver()
                && metadata.repeatable()
                && frames.iter().all(|frame| frame.speed_metres_per_second > SignedSpeed::from(STATIONARY_ANIMATION_SPEED_LIMIT))
        })
    }
}

impl GuardHalfStep {
    fn measure(frames: &[&FrameSample]) -> ReportResult<Observation<Self>> {
        let Some((start, end)) = frames.first().zip(frames.last()) else {
            return Ok(Observation::Unavailable(UnavailableReason::NoFrames));
        };
        let support = SwingSupport::between(start, end);
        let Some((start_bone, end_bone)) = start.bones.get(support.foot).zip(end.bones.get(support.foot)) else {
            return Ok(Observation::Unavailable(UnavailableReason::MissingBone { bone: support.foot }));
        };
        let travel = NonnegativeLength::from_metres((Vec3::from_array(end_bone.position) - Vec3::from_array(start_bone.position)).xz().length())?;
        let Some((start_clearance, end_clearance)) = start_bone.terrain_clearance_metres.zip(end_bone.terrain_clearance_metres) else {
            return Ok(Observation::Unavailable(UnavailableReason::MissingTerrainClearance { bone: support.foot }));
        };
        let mut maximum_clearance: Option<SignedLength> = None;
        for frame in frames {
            let Some(bone) = frame.bones.get(support.foot) else {
                return Ok(Observation::Unavailable(UnavailableReason::MissingBone { bone: support.foot }));
            };
            let Some(clearance) = bone.terrain_clearance_metres else {
                return Ok(Observation::Unavailable(UnavailableReason::MissingTerrainClearance { bone: support.foot }));
            };
            maximum_clearance = Some(maximum_clearance.map_or(clearance, |maximum| maximum.max(clearance)));
        }
        let Some(maximum_clearance) = maximum_clearance else { return Ok(Observation::Unavailable(UnavailableReason::NoFrames)); };
        let clearance_gain = SignedLength::from_metres(maximum_clearance.metres() - start_clearance.metres().max(end_clearance.metres()))?;
        let support_swap_valid = support.start.fraction() <= 0.25 && support.end.fraction() >= 0.75
            && support.other_start.fraction() >= 0.75 && support.other_end.fraction() <= 0.25;
        Ok(Observation::Measured(Self {
            travel,
            clearance_gain,
            visible: support_swap_valid && travel.metres() >= MINIMUM_GUARD_SWING_TRAVEL_METRES
                && clearance_gain.metres() >= MINIMUM_GUARD_SWING_CLEARANCE_GAIN_METRES,
        }))
    }
}

impl SwingSupport {
    fn between(start: &FrameSample, end: &FrameSample) -> Self {
        let left_gain = end.left_support_weight.fraction() - start.left_support_weight.fraction();
        let right_gain = end.right_support_weight.fraction() - start.right_support_weight.fraction();
        if left_gain >= right_gain {
            Self { foot: BoneRole::FootLeft, start: start.left_support_weight, end: end.left_support_weight, other_start: start.right_support_weight, other_end: end.right_support_weight }
        } else {
            Self { foot: BoneRole::FootRight, start: start.right_support_weight, end: end.right_support_weight, other_start: start.left_support_weight, other_end: end.left_support_weight }
        }
    }
}
