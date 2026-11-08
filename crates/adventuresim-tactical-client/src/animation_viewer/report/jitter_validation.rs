//! Jitter applicability and acceptance for a viewer capture.

use super::*;

pub(super) fn observe(
    frames: &[FrameSample],
    plan: &[PlannedFrame],
) -> Observation<JitterValidationSummary<BoneRole>> {
    if frames.is_empty() {
        Observation::Unavailable(UnavailableReason::NoFrames)
    } else if !plan.is_empty()
        && plan
            .iter()
            .all(|frame| frame.scenario.is(ScenarioId::FullRagdoll))
    {
        Observation::NotApplicable
    } else {
        Observation::Measured(jitter::validate(&jitter_frames(frames), Default::default()))
    }
}

pub(super) fn passes(observation: &Observation<JitterValidationSummary<BoneRole>>) -> bool {
    observation.when_applicable(|jitter| {
        jitter.diagnostics_complete && jitter.unacceptable_final_incident_count == 0
    })
}
