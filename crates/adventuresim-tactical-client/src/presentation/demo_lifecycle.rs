//! Clear exhibit scenery and scene-specific caches while retaining the renderer.

use super::*;

pub(crate) fn clear_demo_scene(world: &mut World) {
    clear_scene_entities(world);
    buildings::reset_gpu(world, ownership::PresentationOwner::RegionalMap);
    world.insert_resource(buildings::TacticalBuildingMeshCache::default());
    #[cfg(target_family = "wasm")]
    if let Err(error) = generation::clear_residency() {
        warn!(%error, "Could not clear generated product residency");
    }
    world.insert_resource(obstacles::tree::TreePresentationCache::default());
    world.insert_resource(obstacles::tree::VistaTreePresentationCache::default());
}

/// Replace scene entities while retaining prepared geometry for tactical handoff.
pub(crate) fn clear_scene_entities(world: &mut World) {
    if let Some(mut cache) = world.get_resource_mut::<buildings::TacticalBuildingMeshCache>() {
        cache.recipes.clear();
    }
    buildings::reset_gpu(world, ownership::PresentationOwner::Scene);
    world.remove_resource::<StreamCityTraffic>();
    world.remove_resource::<PendingCityBuildings>();
    world.remove_resource::<vista::streets::streaming::CityTrafficResidency>();
    let mut entities = world
        .query_filtered::<Entity, Or<(
            With<SceneTerrain>,
            With<SceneObstacle>,
            With<SceneBuilding>,
            With<SceneFurniture>,
            With<SceneDoor>,
            With<SceneWindow>,
            With<terrain::ScenePresentationOf>,
            With<GroundScatterLayer>,
            With<GroundLitterCaptureAnchors>,
            With<GroundLitterDiagnostics>,
            With<VistaTerrain>,
            With<buildings::DistantCityBuildingPresentation>,
            With<VistaTreePresentation>,
            With<vista::VistaGrassPresentation>,
            With<vista::VistaRockPresentation>,
        )>>()
        .iter(world)
        .collect::<Vec<_>>();
    entities.extend(
        world
            .query_filtered::<Entity, Or<(
                With<vista::streets::CityStreetPresentation>,
                With<vista::streets::CityYardPresentation>,
                With<SceneBoundary>,
                With<SceneGarden>,
                With<ground_scatter::gardens::DistantGardenPresentation>,
            )>>()
            .iter(world),
    );
    for entity in entities {
        if world.get::<ownership::PresentationOwner>(entity)
            == Some(&ownership::PresentationOwner::RegionalMap)
        {
            continue;
        }
        if let Ok(entity) = world.get_entity_mut(entity) {
            entity.despawn();
        }
    }
    world.insert_resource(ActiveVistaSurface::default());
    world.flush();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replacing_a_scene_releases_unconsumed_generation_recipes() {
        let mut world = World::new();
        let mut cache = buildings::TacticalBuildingMeshCache::default();
        let program = adventuresim_building_generator::BuildingProgram::fixture(
            adventuresim_building_generator::BuildingArchetype::TownHouse,
            fabelgeist_determinism::Seed::from_u64(42),
        );
        cache.recipes.get_or_generate(&program).unwrap();
        assert!(!cache.recipes.is_empty());
        world.insert_resource(cache);
        let actor_city = world
            .spawn((
                buildings::DistantCityBuildingPresentation,
                ownership::PresentationOwner::Scene,
            ))
            .id();
        let map_city = world
            .spawn((
                buildings::DistantCityBuildingPresentation,
                ownership::PresentationOwner::RegionalMap,
            ))
            .id();
        clear_scene_entities(&mut world);
        assert!(world.get_entity(actor_city).is_err());
        assert!(world.get_entity(map_city).is_ok());
        assert!(
            world
                .resource::<buildings::TacticalBuildingMeshCache>()
                .recipes
                .is_empty()
        );
    }

    #[test]
    fn clearing_exhibit_removes_detached_scatter_and_descendants_but_keeps_camera() {
        let mut world = World::new();
        world.insert_resource(PendingCityBuildings::new(&[], &[]));
        let camera = world.spawn(TacticalGameplayCamera).id();
        let map_camera = world.spawn(RegionalMapCamera).id();
        let map_root = world.spawn((Transform::default(), Visibility::Hidden)).id();
        let map_mesh = world.spawn((Mesh3d::default(), ChildOf(map_root))).id();
        let oak = world.spawn(SceneObstacle::Tree).id();
        let leaf = world.spawn(ChildOf(oak)).id();
        let grass = world.spawn(GroundScatterLayer::Grass).id();
        let terrain = world.spawn(terrain::ScenePresentationOf(oak)).id();
        clear_demo_scene(&mut world);
        assert!(!world.contains_resource::<PendingCityBuildings>());
        for removed in [oak, leaf, grass, terrain] {
            assert!(world.get_entity(removed).is_err());
        }
        assert!(world.get_entity(camera).is_ok());
        for retained in [map_camera, map_root, map_mesh] {
            assert!(world.get_entity(retained).is_ok());
        }
        clear_demo_scene(&mut world);
        assert!(world.get_entity(camera).is_ok());
        assert!(world.get_entity(map_mesh).is_ok());
    }
}
