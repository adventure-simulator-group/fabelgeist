//! Presentation for replicated server-authoritative window casements.

use bevy::prelude::*;
use bevy_mod_outline::{OutlineMode, OutlineVolume};

use super::{GrabTargetOutline, SceneWindow, TacticalBuildingMaterials, building_closures};

#[derive(Component)]
pub(crate) struct PresentedWindowCasement;

pub(crate) struct WindowPresentationPlugin;

impl Plugin for WindowPresentationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<super::closure_meshes::ClosureMeshes>()
            .add_observer(on_scene_window_added);
    }
}

fn on_scene_window_added(
    event: On<Add, SceneWindow>,
    mut commands: Commands,
    windows: Query<&SceneWindow>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut cache: ResMut<super::closure_meshes::ClosureMeshes>,
    materials: Res<TacticalBuildingMaterials>,
) -> Result {
    let window = windows.get(event.entity)?;
    let batches = cache.window(window.size_metres, window.leaf, &mut meshes);
    let body = batches
        .iter()
        .find(|batch| batch.material == window.leaf.material())
        .expect("window leaf has its primary material");
    commands.entity(event.entity).insert((
        PresentedWindowCasement,
        Mesh3d(body.mesh.clone()),
        MeshMaterial3d(
            materials
                .for_building(window.building_id)
                .get(window.leaf.material()),
        ),
        Visibility::default(),
        super::building_lod_visibility(super::BuildingRenderLevel::Lod0),
        building_closures::PresentedBuildingClosureMesh::new(window.building_id, window.opening_id),
        GrabTargetOutline(event.entity),
        OutlineVolume {
            visible: false,
            colour: Color::WHITE,
            width: 4.0,
        },
        OutlineMode::FloodFlat,
    ));
    commands.entity(event.entity).with_children(|parent| {
        for batch in batches
            .iter()
            .filter(|batch| batch.material != window.leaf.material())
        {
            parent.spawn((
                Mesh3d(batch.mesh.clone()),
                MeshMaterial3d(
                    materials
                        .for_building(window.building_id)
                        .get(batch.material),
                ),
                Transform::IDENTITY,
                super::building_lod_visibility(super::BuildingRenderLevel::Lod0),
                building_closures::PresentedBuildingClosureMesh::new(
                    window.building_id,
                    window.opening_id,
                ),
            ));
        }
    });
    Ok(())
}
