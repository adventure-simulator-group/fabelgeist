//! Canonical settlement exteriors share geometry and GPU packing with actor cities.
use super::*;
use adventuresim_tactical_core::{
    regional_city::RegionalCityInput, scene_input::TacticalBuildingPlacement,
};
use adventuresim_world_schema::coordinates::Wgs84CoordinateMicrodegrees;
use bevy::ecs::system::SystemState;

pub(in crate::presentation) fn queue_focused_city(
    world: &mut World,
    city: &RegionalCityInput,
    root: Entity,
    window_origin: Wgs84CoordinateMicrodegrees,
) -> Result {
    let mut state = SystemState::<(
        ResMut<TacticalBuildingMeshCache>,
        ResMut<Assets<Mesh>>,
        ResMut<gpu::PendingGpuCities>,
    )>::new(world);
    let (mut cache, mut meshes, mut cities) = state.get_mut(world);
    let owner = PresentationOwner::RegionalMap;
    let mut queued = gpu::PendingGpuBuildings::default();
    queued.set_publication(
        root,
        gpu::CityFrame::from_geographic_city(city, window_origin),
    );
    let mut append = |placement: &TacticalBuildingPlacement, appearance| -> Result {
        let compiled = cached_building_levels(
            owner,
            &mut cache,
            &placement.program,
            BuildingDetail::Facade,
            &mut meshes,
            None,
        )?;
        let rotation = Quat::from_rotation_y(placement.orientation.yaw_radians());
        let centre = Vec3::new(
            placement.centre_metres.metres().x,
            placement.base_elevation_metres.metres(),
            placement.centre_metres.metres().y,
        );
        // Facade vertices are rebased around their compiled origin. Restore
        // all three components after rotation, preserving asymmetric recipes.
        let transform =
            Transform::from_translation(centre + rotation * compiled.local_origin.metres())
                .with_rotation(rotation);
        queued.push(root, &transform, appearance, &compiled);
        Ok(())
    };
    for placement in &city.input().buildings {
        append(placement, gpu::PlacementAppearance::Primary(placement.id))?;
    }
    for placement in &city.input().distant_buildings {
        append(
            &(*placement).into(),
            gpu::PlacementAppearance::Distant(*placement),
        )?;
    }
    // Publish only the complete queue. A failed facade leaves the current GPU
    // city and the actor owner's unpublished placements unchanged.
    *cities.owners.get_mut(owner) = queued;
    Ok(())
}
