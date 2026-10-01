//! Presentation for replicated server-authoritative door leaves.

use bevy::prelude::*;
use bevy_mod_outline::{OutlineMode, OutlineVolume};

use super::{SceneDoor, TacticalBuildingMaterials, building_closures};
use adventuresim_building_generator::BuildingLodMaterial;

#[derive(Component)]
pub(crate) struct PresentedDoorLeaf;

#[derive(Component)]
pub(crate) struct GrabTargetOutline(pub(crate) Entity);

pub(crate) struct DoorPresentationPlugin;

impl Plugin for DoorPresentationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<super::closure_meshes::ClosureMeshes>()
            .add_observer(on_scene_door_added);
        app.add_systems(
            Update,
            building_closures::sync.in_set(building_closures::BuildingClosureVisibility),
        );
    }
}

pub(in crate::presentation) fn on_scene_door_added(
    event: On<Add, SceneDoor>,
    mut commands: Commands,
    doors: Query<&SceneDoor>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut cache: ResMut<super::closure_meshes::ClosureMeshes>,
    materials: Res<TacticalBuildingMaterials>,
) -> Result {
    let door = doors.get(event.entity)?;
    commands.entity(event.entity).insert((
        PresentedDoorLeaf,
        Mesh3d(cache.door(door.size_metres, &mut meshes)),
        MeshMaterial3d(
            materials
                .for_building(door.building_id)
                .get(BuildingLodMaterial::Timber),
        ),
        Visibility::default(),
        GrabTargetOutline(event.entity),
        OutlineVolume {
            visible: false,
            colour: Color::WHITE,
            width: 4.0,
        },
        OutlineMode::FloodFlat,
    ));
    if !adventuresim_tactical_core::city_layout::CityGate::owns_opening(
        adventuresim_building_generator::OpeningAssemblyId(door.opening_id),
    ) {
        commands.entity(event.entity).insert((
            super::building_lod_visibility(super::BuildingRenderLevel::Lod0),
            building_closures::PresentedBuildingClosureMesh::new(door.building_id, door.opening_id),
        ));
    }
    Ok(())
}
