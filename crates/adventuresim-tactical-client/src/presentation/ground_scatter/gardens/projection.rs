//! A vista may arrive before its terrain; project roots once both are present.
use super::*;
use adventuresim_tactical_core::city_layout::CityGarden;
use adventuresim_tactical_netcode::message::SceneVistaBundle;

#[derive(Resource, Default)]
pub(in crate::presentation) struct PendingDistantGardens(Option<GardenProjection>);

struct GardenProjection {
    scene_digest: String,
    gardens: Vec<CityGarden>,
}

pub(in crate::presentation) fn on_vista(
    bundle: On<SceneVistaBundle>,
    mut commands: Commands,
    existing: Query<Entity, With<DistantGardenPresentation>>,
    mut pending: ResMut<PendingDistantGardens>,
) {
    for entity in &existing {
        commands.entity(entity).despawn();
    }
    let gardens = bundle
        .gardens
        .iter()
        .filter(|garden| {
            bundle
                .distant_buildings
                .iter()
                .any(|building| building.id == garden.front_building_id)
        })
        .cloned()
        .collect();
    pending.0 = Some(GardenProjection {
        scene_digest: bundle.scene_digest.clone(),
        gardens,
    });
}

pub(in crate::presentation) fn project_pending(
    mut commands: Commands,
    terrains: Query<(&SceneTerrain, &SceneEnvironment)>,
    mut pending: ResMut<PendingDistantGardens>,
) -> Result {
    let Some(projection) = &pending.0 else {
        return Ok(());
    };
    let Some((terrain, _)) = terrains
        .iter()
        .find(|(_, environment)| environment.scene_digest == projection.scene_digest)
    else {
        return Ok(());
    };
    let projection = pending
        .0
        .take()
        .expect("exact pending projection was found");
    // Publish all roots together. Missing terrain never means a zero elevation,
    // and a failed plant aborts the complete projection with its exact identity.
    let gardens = projection
        .gardens
        .into_iter()
        .map(|garden| SceneGarden::project(garden, terrain))
        .collect::<std::result::Result<Vec<_>, _>>()?;
    for garden in gardens {
        commands.spawn((DistantGardenPresentation, garden, Transform::default()));
    }
    Ok(())
}

#[cfg(test)]
mod tests;
