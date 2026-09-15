//! Matched-camera diagnostics select existing production meshes and materials.
use super::*;
use crate::presentation::{BuildingRenderLevel, PresentedBuildingMesh};
use crate::tactical_scene_viewer::capture_state::{CapturePhase, SceneCaptureState};
use bevy::camera::visibility::VisibilityRange;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(in crate::tactical_scene_viewer) enum ReviewLod {
    Detail,
    Facade,
    Shell,
}

impl From<BuildingRenderLevel> for ReviewLod {
    fn from(level: BuildingRenderLevel) -> Self {
        match level {
            BuildingRenderLevel::Lod0 => Self::Detail,
            BuildingRenderLevel::Lod1 => Self::Facade,
            BuildingRenderLevel::Lod2 => Self::Shell,
        }
    }
}

#[derive(Component)]
pub(super) struct OriginalRange(VisibilityRange);

type ReviewGeometry = (
    Entity,
    &'static mut VisibilityRange,
    Option<&'static PresentedBuildingMesh>,
    Option<&'static adventuresim_tactical_core::prelude::SceneDoor>,
    Option<&'static adventuresim_tactical_core::prelude::SceneWindow>,
    Option<&'static OriginalRange>,
);

pub(super) fn select(
    mut commands: Commands,
    state: Option<Res<SceneCaptureState>>,
    requirements: Option<Res<ReviewRequirements>>,
    mut recorded: Local<std::collections::BTreeSet<usize>>,
    mut geometry: Query<ReviewGeometry>,
) {
    let (Some(state), Some(requirements)) = (state, requirements) else {
        return;
    };
    let view = state.views[state.view];
    let mut selected_batches = 0;
    for (entity, mut range, mesh, door, window, original) in &mut geometry {
        if mesh.is_none() && door.is_none() && window.is_none() {
            continue;
        }
        if let Some(lod) = view.building_lod_override {
            if original.is_none() {
                commands.entity(entity).insert(OriginalRange(range.clone()));
            }
            let level = mesh.map_or(ReviewLod::Detail, |mesh| mesh.level.into());
            let visible = level == lod;
            selected_batches += usize::from(visible && mesh.is_some());
            *range = VisibilityRange {
                start_margin: 0.0..0.0,
                end_margin: if visible {
                    f32::MAX..f32::MAX
                } else {
                    0.0..0.0
                },
                use_aabb: false,
            };
        } else if let Some(original) = original {
            *range = original.0.clone();
            commands.entity(entity).remove::<OriginalRange>();
        }
    }
    if view.building_lod_override.is_some()
        && selected_batches > 0
        && state.phase != CapturePhase::Configure
        && recorded.insert(state.view)
    {
        std::fs::write(
            requirements
                .output
                .join(format!("{}.building-lod.json", view.slug)),
            serde_json::to_vec_pretty(&serde_json::json!({
                "selected_production_lod": view.building_lod_override,
                "selected_production_batches": selected_batches,
                "meshes_and_materials_unchanged": true,
            }))
            .unwrap(),
        )
        .expect("write matched building LOD evidence");
    }
}
