//! Signed body response and straight-run sway retain missing evidence.

use super::*;
use adventuresim_tactical_core::animation::{NonnegativeLength, SignedAngleDegrees};
use fabelgeist_geometry::SignedLength;

pub(super) struct BodyResponseValidation {
    pub(super) response: bool,
    pub(super) straight_run_sway: bool,
}

struct LeanRange {
    minimum: SignedAngleDegrees,
    maximum: SignedAngleDegrees,
}

#[derive(Clone, Copy)]
enum LeanComponent {
    Pitch,
    RollMagnitude,
}

impl BodyResponseValidation {
pub(super) fn measure(
    frames: &[FrameSample],
) -> ReportResult<Self> {
    let ramp_pitch = LeanRange::measure(frames, &ScenarioMetadata::authored(ScenarioId::SpeedRampUpDown), LeanComponent::Pitch)?;
    let hard_stop_pitch = LeanRange::measure(frames, &ScenarioMetadata::authored(ScenarioId::HardStop), LeanComponent::Pitch)?;
    let walk_stop_pitch = LeanRange::measure(frames, &ScenarioMetadata::authored(ScenarioId::FlatGridWalkStop), LeanComponent::Pitch)?;
    let turn_90_roll = LeanRange::measure(frames, &ScenarioMetadata::authored(ScenarioId::DynamicsTurn90), LeanComponent::RollMagnitude)?;
    let turn_180_roll = LeanRange::measure(frames, &ScenarioMetadata::authored(ScenarioId::DynamicsTurn180), LeanComponent::RollMagnitude)?;
    let lean_step_valid = frames.windows(2).all(|pair| {
        if pair[0].scenario != pair[1].scenario { return true; }
        let Some(((before_pitch, before_roll), (after_pitch, after_roll))) = pair[0].body_lean_pitch_degrees.zip(pair[0].body_lean_roll_degrees)
            .zip(pair[1].body_lean_pitch_degrees.zip(pair[1].body_lean_roll_degrees)) else { return false; };
        Vec2::new(after_pitch.degrees() - before_pitch.degrees(), after_roll.degrees() - before_roll.degrees()).length() <= 2.01
    });
    let has_scenario = |name: &ScenarioMetadata<'_>| frames.iter().any(|frame| &frame.scenario == name);
    let mut straight_run_sway = true;
    for scenario in [ScenarioMetadata::authored(ScenarioId::SteadyRun55), ScenarioMetadata::authored(ScenarioId::FlatGridRun55)] {
        straight_run_sway &= body_lateral_range(frames, &scenario, BoneRole::Chest)?.when_applicable(|range| range.metres() <= 0.015)
            && body_lateral_range(frames, &scenario, BoneRole::Head)?.when_applicable(|range| range.metres() <= 0.025);
    }
    let settled = [ScenarioMetadata::authored(ScenarioId::SpeedRampUpDown), ScenarioMetadata::authored(ScenarioId::HardStop), ScenarioMetadata::authored(ScenarioId::FlatGridWalkStop)]
        .iter().filter(|scenario| has_scenario(scenario)).all(|scenario| {
            frames.iter().rev().find(|frame| frame.scenario == *scenario).is_some_and(|frame| {
                frame.body_lean_pitch_degrees.zip(frame.body_lean_roll_degrees).is_some_and(|(pitch, roll)| Vec2::new(pitch.degrees(), roll.degrees()).length() <= 0.5)
            })
        });
    Ok(Self {
        response: ramp_pitch.when_applicable(|range| (-2.5..=0.0).contains(&range.minimum.degrees()) && (15.5..=18.5).contains(&range.maximum.degrees()))
            && hard_stop_pitch.when_applicable(|range| (-0.1..=0.1).contains(&range.minimum.degrees()))
            && walk_stop_pitch.when_applicable(|range| (-2.5..=0.0).contains(&range.minimum.degrees()) && (3.5..=5.0).contains(&range.maximum.degrees()))
            && turn_90_roll.when_applicable(|range| (6.0..=18.0).contains(&range.maximum.degrees()))
            && turn_180_roll.when_applicable(|range| (6.0..=18.0).contains(&range.maximum.degrees()))
            && lean_step_valid && settled,
        straight_run_sway,
    })
}

}

impl LeanRange {
fn measure(
    frames: &[FrameSample],
    scenario: &ScenarioMetadata<'_>,
    component: LeanComponent,
) -> AnimationValueResult<Observation<Self>> {
    let mut bounds: Option<LeanRange> = None;
    for frame in frames.iter().filter(|frame| &frame.scenario == scenario) {
        let value = match component {
            LeanComponent::Pitch => frame.body_lean_pitch_degrees,
            LeanComponent::RollMagnitude => frame.body_lean_roll_degrees
                .map(|angle| SignedAngleDegrees::from_degrees(angle.degrees().abs())).transpose()?,
        };
        let Some(value) = value else {
            return Ok(Observation::Unavailable(UnavailableReason::MissingBodyResponse));
        };
        bounds = Some(bounds.map_or(LeanRange { minimum: value, maximum: value }, |bounds| LeanRange {
            minimum: bounds.minimum.min(value), maximum: bounds.maximum.max(value),
        }));
    }
    Ok(bounds.map_or(Observation::NotApplicable, Observation::Measured))
}

}

fn body_lateral_range(
    frames: &[FrameSample],
    scenario: &ScenarioMetadata<'_>,
    bone: BoneRole,
) -> ReportResult<Observation<NonnegativeLength>> {
    let mut bounds: Option<SignedLengthRange> = None;
    for frame in frames.iter().filter(|frame| &frame.scenario == scenario) {
        let Some(position) = body_local(frame, bone) else {
            return Ok(Observation::Unavailable(UnavailableReason::MissingBone { bone }));
        };
        let value = SignedLength::from_metres(position.x)?;
        bounds = Some(bounds.map_or(SignedLengthRange::from_sample(value), |bounds| bounds.include(value)));
    }
    let Some(bounds) = bounds else { return Ok(Observation::NotApplicable); };
    Ok(Observation::Measured(NonnegativeLength::from_metres(bounds.maximum().metres() - bounds.minimum().metres())?))
}
