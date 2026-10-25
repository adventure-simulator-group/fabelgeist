//! Acceptance of phase-owned height requires measured range, gain and peaks.

use super::*;

pub(super) fn phase_owned_height_valid(
    scenarios: &[ScenarioMetrics],
) -> AnimationValueResult<bool> {
    for metrics in scenarios {
        if matches!(
            metrics.scenario.identity(),
            Some(ScenarioId::StartStopTransition) | Some(ScenarioId::RaisedGuardTransition)
        ) && !metrics.height.maximum_pelvis_vertical_step_metres.measured().is_some_and(|step| {
            step.metres() <= LOCOMOTION_STATE_MAXIMUM_PELVIS_VERTICAL_STEP_METRES
        }) {
            return Ok(false);
        }
        let Some(expected) = ExpectedVisualHeight::for_scenario(&metrics.scenario)? else {
            continue;
        };
        let range_valid = metrics.height.phase_height_range_metres.measured().is_some_and(|range| {
            range.metres() >= expected.minimum_range().metres()
                && range.metres() <= expected.maximum_range().metres()
        });
        let gain_valid = metrics.height.contact_to_passing_height_gain_metres.measured().is_some_and(|gain| {
            gain.metres() >= expected.minimum_range().metres() * 0.75
        });
        let peaks_valid = metrics.height.visual_height_peaks.measured().is_some_and(|peaks| {
            peaks.maximum_count() == expected.peak_count() && peaks.each_cycle_has_two_passing_peaks()
        });
        if !(range_valid && gain_valid && peaks_valid) {
            return Ok(false);
        }
    }
    Ok(true)
}
