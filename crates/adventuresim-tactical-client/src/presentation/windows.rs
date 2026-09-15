//! Presentation for replicated server-authoritative window casements.

use adventuresim_building_generator::compile_window_leaf;
use bevy::prelude::*;
use bevy_mod_outline::{OutlineMode, OutlineVolume};

use super::{
    BuildingRenderLevel, GrabTargetOutline, SceneWindow, TacticalBuildingMaterials,
    building_lod_visibility,
};

#[derive(Component)]
pub(crate) struct PresentedWindowCasement;

pub(crate) struct WindowPresentationPlugin;

impl Plugin for WindowPresentationPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_scene_window_added);
    }
}

fn on_scene_window_added(
    event: On<Add, SceneWindow>,
    mut commands: Commands,
    windows: Query<&SceneWindow>,
    mut meshes: ResMut<Assets<Mesh>>,
    materials: Res<TacticalBuildingMaterials>,
) -> Result {
    let window = windows.get(event.entity)?;
    let batches = compile_window_leaf(
        window.size_metres,
        window.leaf,
        adventuresim_building_generator::ClosureState::Operable,
    );
    let body = batches
        .iter()
        .find(|batch| batch.material == window.leaf.material())
        .expect("window leaf has its primary material");
    commands.entity(event.entity).insert((
        PresentedWindowCasement,
        Mesh3d(meshes.add(super::recipe_mesh::recipe_mesh(body, Vec3::ZERO))),
        MeshMaterial3d(materials.get_for_building(window.building_id, window.leaf.material())),
        Visibility::default(),
        building_lod_visibility(BuildingRenderLevel::Lod0),
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
                Mesh3d(meshes.add(super::recipe_mesh::recipe_mesh(batch, Vec3::ZERO))),
                MeshMaterial3d(materials.get_for_building(window.building_id, batch.material)),
                Transform::IDENTITY,
                building_lod_visibility(BuildingRenderLevel::Lod0),
            ));
        }
    });
    Ok(())
}
