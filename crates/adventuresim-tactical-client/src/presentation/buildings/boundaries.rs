use super::*;
use adventuresim_tactical_core::prelude::{CityBoundaryMaterial, SceneBoundary};

pub(super) fn on_boundary(
    event: On<Add, SceneBoundary>,
    mut commands: Commands,
    boundaries: Query<&SceneBoundary>,
    mut meshes: ResMut<Assets<Mesh>>,
    materials: Res<TacticalBuildingMaterials>,
) -> Result {
    let boundary = boundaries.get(event.entity)?;
    commands
        .entity(event.entity)
        .insert(Visibility::default())
        .with_children(|parent| {
            fixed(parent, boundary, &mut meshes, &materials);
        });
    Ok(())
}

pub(super) fn on_vista(
    bundle: On<SceneVistaBundle>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    materials: Res<TacticalBuildingMaterials>,
) {
    for compound in &bundle.compounds {
        let Some(front) = bundle
            .distant_buildings
            .iter()
            .find(|b| b.id == compound.front_building_id)
        else {
            continue;
        };
        let boundary = SceneBoundary {
            property_id: compound.id,
            front_building_id: front.id,
            boundary: compound.boundary.clone(),
        };
        commands
            .spawn((
                DistantCityBuildingPresentation,
                Visibility::default(),
                Transform::from_xyz(0.0, front.base_elevation_metres, 0.0),
            ))
            .with_children(|parent| {
                fixed(parent, &boundary, &mut meshes, &materials);
                let door = compound.boundary.gate.door(compound.id);
                parent.spawn((
                    Mesh3d(meshes.add(super::super::recipe_mesh::metric_cuboid(door.size_metres))),
                    MeshMaterial3d(
                        materials.get_for_building(front.id, BuildingLodMaterial::Timber),
                    ),
                    Transform::from_translation(door.closed_centre)
                        .with_rotation(Quat::from_rotation_y(door.closed_yaw_radians)),
                ));
            });
    }
}

fn fixed(
    parent: &mut ChildSpawnerCommands,
    boundary: &SceneBoundary,
    meshes: &mut Assets<Mesh>,
    materials: &TacticalBuildingMaterials,
) {
    for member in boundary.boundary.fixed_members() {
        let material = match member.material {
            CityBoundaryMaterial::Masonry => BuildingLodMaterial::Wall(
                adventuresim_building_generator::WallMaterialClass::RubbleMasonry,
            ),
            CityBoundaryMaterial::Timber => BuildingLodMaterial::Timber,
            CityBoundaryMaterial::Iron => BuildingLodMaterial::Iron,
        };
        parent.spawn((
            Mesh3d(meshes.add(super::super::recipe_mesh::metric_cuboid(member.size_metres))),
            MeshMaterial3d(materials.get_for_building(boundary.front_building_id, material)),
            Transform::from_translation(member.centre_metres)
                .with_rotation(Quat::from_rotation_y(member.yaw_radians)),
        ));
    }
}
