//! Terrain and flight policy classified once at scenario source admission.

use super::*;

#[derive(Debug, Clone, Copy)]
pub(super) enum ScenarioTerrain {
    Absent,
    Authored,
    Named,
    RaisedStop { startup: TerrainIkMode },
    Toggle,
}

impl<'a> ScenarioMetadata<'a> {
    pub(in crate::animation_viewer) fn from_label(label: &'a str) -> Self {
        scenario_metadata(label)
    }
    pub(in crate::animation_viewer) fn label(self) -> &'a str {
        self.label
    }
    pub(in crate::animation_viewer) fn kind(self) -> ScenarioKind {
        self.kind
    }
    pub(in crate::animation_viewer) fn repeatable(self) -> bool {
        self.repeatable
    }
    pub(in crate::animation_viewer) fn procedural_solver(self) -> bool {
        self.procedural_solver
    }
}

impl ScenarioTerrain {
    pub(super) fn from_label(name: &str, kind: ScenarioKind) -> Self {
        let toggle = name == "terrain-toggle-mid-stride";
        let raised_stop = name.starts_with("raised-guard-tap-stop-");
        let named = name.contains("terrain");
        if toggle {
            Self::Toggle
        } else if raised_stop {
            Self::RaisedStop {
                startup: if named {
                    TerrainIkMode::Enabled
                } else {
                    TerrainIkMode::Disabled
                },
            }
        } else if kind == ScenarioKind::Terrain {
            Self::Authored
        } else if named {
            Self::Named
        } else {
            Self::Absent
        }
    }

    fn startup(self) -> TerrainIkMode {
        match self {
            Self::Absent => TerrainIkMode::Disabled,
            Self::RaisedStop { startup } => startup,
            Self::Authored | Self::Named | Self::Toggle => TerrainIkMode::Enabled,
        }
    }
}

pub(in crate::animation_viewer) fn scenario_uses_terrain_ik(
    scenario: ScenarioMetadata<'_>,
) -> bool {
    matches!(
        scenario.terrain,
        ScenarioTerrain::Authored | ScenarioTerrain::RaisedStop { .. } | ScenarioTerrain::Toggle
    )
}

/// Startup selection excludes per-frame raised-stop and toggle-window overrides.
pub(in crate::animation_viewer) fn startup_terrain_ik_mode(
    scenario: Option<ScenarioMetadata<'_>>,
) -> TerrainIkMode {
    scenario.map_or(TerrainIkMode::Disabled, |scenario| {
        scenario.terrain.startup()
    })
}

pub(in crate::animation_viewer) fn terrain_ik_mode_for_frame(
    frame: &PlannedFrame,
) -> TerrainIkMode {
    match frame.scenario.terrain {
        ScenarioTerrain::Absent => TerrainIkMode::Disabled,
        ScenarioTerrain::Toggle if !(16..80).contains(&frame.scenario_frame) => {
            TerrainIkMode::Disabled
        }
        _ => TerrainIkMode::Enabled,
    }
}

pub(in crate::animation_viewer) fn raised_scenario_requires_zero_flight(
    scenario: ScenarioMetadata<'_>,
) -> bool {
    scenario.kind == ScenarioKind::RaisedGuard
        && !matches!(scenario.terrain, ScenarioTerrain::RaisedStop { .. })
}
