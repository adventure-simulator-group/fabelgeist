//! Coordinate fixed simulation time with multi-view ragdoll capture.

use super::*;
use bevy::time::TimeUpdateStrategy;

#[derive(Default, Resource)]
pub(super) struct CaptureTimeControl {
    previous: Option<TimeUpdateStrategy>,
    sample_index: usize,
    step_requested: bool,
}

pub(super) fn finish_ragdoll_step(
    sequence: Res<CaptureSequence>,
    mut control: ResMut<CaptureTimeControl>,
    mut strategy: ResMut<TimeUpdateStrategy>,
) {
    if control.previous.is_none() {
        return;
    }
    if !sequence.admission.is_open()
        || sequence
            .plan
            .get(sequence.index)
            .is_none_or(|frame| !frame.scenario.is(ScenarioId::FullRagdoll))
    {
        if let Some(previous) = control.previous.take() {
            *strategy = previous;
        }
        control.step_requested = false;
    } else if control.step_requested {
        // The time system has already supplied this frame's one fixed tick.
        *strategy = TimeUpdateStrategy::FixedTimesteps(0);
        control.step_requested = false;
    }
}

pub(super) fn schedule_ragdoll_step(
    sequence: Res<CaptureSequence>,
    mut control: ResMut<CaptureTimeControl>,
    mut strategy: ResMut<TimeUpdateStrategy>,
) {
    if control.previous.is_some() && sequence.applied && control.sample_index != sequence.index {
        control.sample_index = sequence.index;
        control.step_requested = true;
        *strategy = TimeUpdateStrategy::FixedTimesteps(1);
    }
}

pub(super) fn arm_ragdoll_capture(
    mut sequence: ResMut<CaptureSequence>,
    mut control: ResMut<CaptureTimeControl>,
    mut strategy: ResMut<TimeUpdateStrategy>,
) {
    let first_view = sequence.applied
        && sequence.view_index == 0
        && !sequence.capture_in_flight
        && sequence
            .plan
            .get(sequence.index)
            .is_some_and(|frame| frame.scenario.is(ScenarioId::FullRagdoll));
    if !first_view
        || control.previous.is_some()
        || sequence.settled.saturating_add(1) < required_settle_frames(&sequence)
    {
        return;
    }
    control.previous = Some(std::mem::replace(
        &mut *strategy,
        TimeUpdateStrategy::FixedTimesteps(0),
    ));
    control.sample_index = sequence.index;
    // Let the frozen state settle before recording the first view.
    sequence.settled = 0;
}

pub(super) fn capture_time_ready(control: Res<CaptureTimeControl>) -> bool {
    !control.step_requested
}

pub(super) fn required_settle_frames(sequence: &CaptureSequence) -> u32 {
    if sequence.index == 0 {
        sequence.settle_frames.max(60)
    } else {
        sequence.settle_frames.max(2)
    }
}
