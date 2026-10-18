//! Terrain ownership and zero-flight policy for authored capture scenarios.

use super::*;

pub(in crate::animation_viewer) fn scenario_uses_terrain_ik(scenario: &str) -> bool {
    scenario_metadata(scenario).kind == ScenarioKind::Terrain
        || scenario.starts_with("raised-guard-tap-stop-")
}

pub(in crate::animation_viewer) fn terrain_ik_mode_for_frame(
    frame: &PlannedFrame,
) -> TerrainIkMode {
    let terrain_required = (scenario_uses_terrain_ik(frame.scenario)
        || frame.scenario.contains("terrain"))
        && (frame.scenario != "terrain-toggle-mid-stride"
            || (16..80).contains(&frame.scenario_frame));
    if terrain_required {
        TerrainIkMode::Enabled
    } else {
        TerrainIkMode::Disabled
    }
}

pub(in crate::animation_viewer) fn raised_scenario_requires_zero_flight(scenario: &str) -> bool {
    scenario_metadata(scenario).kind == ScenarioKind::RaisedGuard
        && !scenario.starts_with("raised-guard-tap-stop-")
}
