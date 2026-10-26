//! Scenario-specific anatomical gates retain explicit required evidence.

use super::*;

pub(super) fn biomechanics_within_review_bounds(scenarios: &[ScenarioMetrics]) -> bool {
    scenarios.iter().all(|metrics| {
        if metrics.scenario.has(ScenarioFeature::Dive) {
            // Lateral dives are judged by their pelvis-to-head long axis.
            return metrics.facing.maximum_dive_axis_motion_error_degrees.measured().is_some_and(|angle| angle.degrees() <= 20.0);
        }
        if metrics.scenario.is(ScenarioId::FullRagdoll)
            || metrics.scenario.has(ScenarioFeature::Downed)
            || metrics.scenario.has(ScenarioFeature::GetUp)
            || metrics.scenario.is(ScenarioId::JumpChargeAnticipation)
            || metrics.scenario.is(ScenarioId::OrdinaryCameraPitch)
        {
            // Posture and stationary pitch fixtures retain finite output,
            // continuity, penetration and visual review, without gait gates.
            return true;
        }
        let Some(vertical_limit) = metrics.expected.vertical_range.measured() else { return false; };
        let vertical_valid = metrics.height.pelvis_vertical_range_metres.measured().is_some_and(|range| range.metres() <= vertical_limit.metres())
            && metrics.height.head_vertical_range_metres.measured().is_some_and(|range| range.metres() <= vertical_limit.metres());
        let separation_valid = metrics.pose.minimum_inter_foot_separation_metres.measured().is_some_and(|value| *value >= metrics.expected.inter_foot_separation);
        let final_facing_valid = metrics.facing.final_facing_motion_error_degrees.when_applicable(|angle| angle.degrees() <= 3.0);
        if metrics.scenario.has(ScenarioFeature::RaisedTapStopAny) {
            return separation_valid && final_facing_valid && vertical_valid;
        }
        let kind = metrics.scenario.kind();
        let attack = kind == ScenarioKind::Attack;
        let procedural = procedural_leg_solver_gates_apply(&metrics.scenario);
        let world_plants = matches!(kind, ScenarioKind::RaisedGuard | ScenarioKind::Attack)
            && !uses_authored_combat_locomotion(&metrics.scenario)
            && !is_guard_stop_transition(&metrics.scenario);
        let plants_valid = !world_plants || attack || (
            metrics.continuity.measured().is_some_and(|continuity| continuity.maximum_supported_foot_slip_metres_per_frame.when_applicable(|slip| *slip <= metrics.expected.supported_slip))
                && metrics.maximum_planted_foot_drift_metres.when_applicable(|drift| *drift <= metrics.expected.planted_drift)
        );
        let seam_valid = if expects_loop_seam(&metrics.scenario) {
            metrics.loop_seam.measured().is_some_and(|seam| {
                seam.position_metres <= metrics.expected.seam_position
                    && seam.rotation_degrees.degrees() <= if uses_authored_combat_locomotion(&metrics.scenario) { 5.5 } else { 5.0 }
            })
        } else { matches!(metrics.loop_seam, Observation::NotApplicable) };
        plants_valid
            && (is_quickstep_scenario(&metrics.scenario)
                || metrics.scenario.is(ScenarioId::RaisedGuardStationaryTurn)
                || !procedural
                || metrics.pose.minimum_signed_foot_track_metres.measured().is_some_and(|value| value.metres() >= -0.01))
            && separation_valid
            && (!procedural
                || metrics.scenario.has(ScenarioFeature::StationaryAttack)
                || (metrics.pose.minimum_knee_flexion_degrees.measured().is_some_and(|angle| angle.degrees() >= 3.9)
                    && metrics.pose.minimum_knee_hemisphere_dot.measured().is_some_and(|dot| dot.value() >= 0.0)))
            && (!procedural || metrics.pose.maximum_knee_foot_yaw_offset_degrees.measured().is_some_and(|angle| angle.degrees() <= 22.6))
            && metrics.facing.maximum_facing_tracking_excess_degrees.when_applicable(|angle| angle.degrees() <= 0.2)
            && final_facing_valid
            && (attack || !procedural || metrics.clearance.contact_sole_clearance_metres.measured().is_some_and(|range| {
                range.maximum().metres() <= if metrics.scenario.is(ScenarioId::TerrainSteadyRun55) { 0.01 } else { 0.04 }
            }))
            && vertical_valid
            && seam_valid
    })
}
