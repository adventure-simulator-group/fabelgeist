//! Terrain and flight policy classified once at scenario source admission.

use super::*;

pub(in crate::animation_viewer) fn scenario_uses_terrain_ik(
    scenario: &ScenarioMetadata<'_>,
) -> bool {
    matches!(
        scenario.terrain(),
        ScenarioTerrain::Authored | ScenarioTerrain::RaisedStop { .. } | ScenarioTerrain::Toggle
    )
}

/// Startup selection excludes per-frame raised-stop and toggle-window overrides.
pub(in crate::animation_viewer) fn startup_terrain_ik_mode(
    scenario: Option<&ScenarioMetadata<'_>>,
) -> TerrainIkMode {
    scenario.map_or(TerrainIkMode::Disabled, |scenario| {
        scenario.terrain().startup()
    })
}

pub(in crate::animation_viewer) fn terrain_ik_mode_for_frame(
    frame: &PlannedFrame,
) -> TerrainIkMode {
    match frame.scenario.terrain() {
        ScenarioTerrain::Absent => TerrainIkMode::Disabled,
        ScenarioTerrain::Toggle if !(16..80).contains(&frame.scenario_frame) => {
            TerrainIkMode::Disabled
        }
        _ => TerrainIkMode::Enabled,
    }
}

pub(in crate::animation_viewer) fn raised_scenario_requires_zero_flight(
    scenario: &ScenarioMetadata<'_>,
) -> bool {
    scenario.kind() == ScenarioKind::RaisedGuard
        && !matches!(scenario.terrain(), ScenarioTerrain::RaisedStop { .. })
}
