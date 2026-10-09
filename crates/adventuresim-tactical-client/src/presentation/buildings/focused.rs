//! Canonical settlement exteriors share geometry and GPU packing with actor cities.
use super::*;
use adventuresim_tactical_core::{
    regional_city::RegionalCityInput, scene_input::TacticalBuildingPlacement,
};
use bevy::ecs::system::SystemState;

pub(in crate::presentation) fn queue_focused_city(
    world: &mut World,
    city: &RegionalCityInput,
    root: Entity,
) -> Result {
    let mut state = SystemState::<(
        ResMut<TacticalBuildingMeshCache>,
        ResMut<Assets<Mesh>>,
        ResMut<gpu::PendingGpuCities>,
    )>::new(world);
    let (mut cache, mut meshes, mut cities) = state.get_mut(world);
    let owner = PresentationOwner::RegionalMap;
    let mut queued = gpu::PendingGpuBuildings::default();
    let mut append = |placement: &TacticalBuildingPlacement, appearance| -> Result {
        let compiled = cached_building_levels(
            owner,
            &mut cache,
            &placement.program,
            BuildingDetail::Facade,
            &mut meshes,
            None,
        )?;
        let transform = Transform::from_xyz(
            placement.centre_metres.metres().x,
            placement.base_elevation_metres.metres() + compiled.local_origin.metres().y,
            placement.centre_metres.metres().y,
        )
        .with_rotation(Quat::from_rotation_y(placement.orientation.yaw_radians()));
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
