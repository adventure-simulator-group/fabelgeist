//! Continuity gates consume measured quantities and scenario-admitted bounds.

use super::*;

pub(super) fn continuity_within_review_bounds(scenarios: &[ScenarioMetrics]) -> bool {
    scenarios.iter().all(|metrics| {
        let Some(continuity) = metrics.continuity.measured() else { return false; };
        if metrics.scenario.is(ScenarioId::FullRagdoll) {
            // Unconstrained limbs and the root handoff retain the rotation gate.
            return continuity.maximum_bone_rotation_step_degrees.degrees() <= 60.0;
        }
        continuity.maximum_root_relative_step_metres.metres()
            <= if metrics.scenario.has(ScenarioFeature::AttackLive) { 0.30 }
            else if is_quickstep_scenario(&metrics.scenario) { 0.25 }
            else { 0.20 }
            && continuity.maximum_foot_root_relative_step_metres <= metrics.expected.foot_step
            && continuity.maximum_knee_root_relative_step_metres <= metrics.expected.knee_step
            && continuity.maximum_bone_rotation_step_degrees.degrees() <= 60.0
            && (!metrics.scenario.has(ScenarioFeature::Run)
                || continuity.maximum_foot_rotation_step_degrees.degrees()
                    <= if metrics.scenario.kind() == ScenarioKind::Terrain { 50.01 } else { 15.01 })
            && (!metrics.scenario.has(ScenarioFeature::RaisedGuard)
                || metrics.height.maximum_pelvis_vertical_step_metres.measured().is_some_and(|step| step.metres() <= RAISED_MAXIMUM_PELVIS_VERTICAL_STEP_METRES))
            && (!metrics.scenario.is(ScenarioId::TerrainSteadyRun55)
                || metrics.height.maximum_pelvis_vertical_step_metres.measured().is_some_and(|step| step.metres() <= 0.02))
    })
}
