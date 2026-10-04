//! Distant walls wait for the exact scene support rather than using a house floor.
use super::*;
use adventuresim_tactical_core::{city_layout::CityCompound, prelude::GeneratedBoundary};

#[derive(Component)]
pub(super) struct DistantBoundaryPresentation;

#[derive(Resource, Default)]
pub(super) struct PendingDistantBoundaries(Option<BoundaryProjection>);

struct BoundaryProjection {
    scene_digest: String,
    compounds: Vec<CityCompound>,
}

pub(super) fn on_vista(
    bundle: On<SceneVistaBundle>,
    mut commands: Commands,
    existing: Query<Entity, With<DistantBoundaryPresentation>>,
    mut pending: ResMut<PendingDistantBoundaries>,
) {
    for entity in &existing {
        commands.entity(entity).despawn();
    }
    let compounds = bundle
        .compounds
        .iter()
        .filter(|compound| {
            bundle
                .distant_buildings
                .iter()
                .any(|building| building.id == compound.front_building_id)
        })
        .cloned()
        .collect();
    pending.0 = Some(BoundaryProjection {
        scene_digest: bundle.scene_digest.clone(),
        compounds,
    });
}

impl PendingDistantBoundaries {
    fn take_for_scene(
        &mut self,
        terrain: &SceneTerrain,
        environment: &SceneEnvironment,
    ) -> std::result::Result<
        Option<Vec<GeneratedBoundary>>,
        adventuresim_tactical_core::scene_input::SceneInputError,
    > {
        let Some(projection) = &self.0 else {
            return Ok(None);
        };
        if projection.scene_digest != environment.scene_digest {
            return Ok(None);
        }
        let projection = self.0.take().expect("exact scene support is present");
        projection
            .compounds
            .iter()
            .map(|compound| GeneratedBoundary::project(compound, terrain))
            .collect::<std::result::Result<Vec<_>, _>>()
            .map(Some)
    }
}

pub(super) fn project_pending(
    mut commands: Commands,
    terrains: Query<(&SceneTerrain, &SceneEnvironment)>,
    mut pending: ResMut<PendingDistantBoundaries>,
    mut meshes: ResMut<Assets<Mesh>>,
    materials: Res<TacticalBuildingMaterials>,
) -> Result {
    let Some(projection) = &pending.0 else {
        return Ok(());
    };
    let Some((terrain, environment)) = terrains
        .iter()
        .find(|(_, environment)| environment.scene_digest == projection.scene_digest)
    else {
        return Ok(());
    };
    let boundaries = pending
        .take_for_scene(terrain, environment)?
        .expect("the exact pending scene matched");
    let mut batches = BoundaryBatches::default();
    for boundary in boundaries {
        fixed(
            &mut batches,
            &boundary.scene,
            boundary.elevation_metres,
            &materials,
        );
        let door = boundary
            .scene
            .boundary
            .gate
            .door(boundary.scene.property_id);
        commands
            .spawn((
                DistantCityBuildingPresentation,
                DistantBoundaryPresentation,
                Visibility::default(),
                Transform::from_xyz(0.0, boundary.elevation_metres, 0.0),
            ))
            .with_children(|parent| {
                parent.spawn((
                    Mesh3d(meshes.add(crate::presentation::recipe_mesh::metric_cuboid(
                        door.size_metres,
                    ))),
                    MeshMaterial3d(
                        materials
                            .for_building(boundary.scene.front_building_id)
                            .get(BuildingLodMaterial::Timber),
                    ),
                    Transform::from_translation(door.closed_centre)
                        .with_rotation(Quat::from_rotation_y(door.closed_yaw_radians)),
                ));
            });
    }
    let members = batches.members;
    let batches = batches.finish();
    info!(
        members,
        batches = batches.len(),
        "City grounded boundary batches"
    );
    for batch in batches {
        commands.spawn((
            Name::new("City grounded boundary batch"),
            DistantCityBuildingPresentation,
            DistantBoundaryPresentation,
            Mesh3d(meshes.add(batch.mesh)),
            MeshMaterial3d(batch.material),
            Transform::from_translation(batch.origin),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests;
