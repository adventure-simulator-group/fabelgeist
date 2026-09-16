//! Clear exhibit scenery and scene-specific caches while retaining the renderer.

use super::*;

pub(crate) fn clear_demo_scene(world: &mut World) {
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
        if let Ok(entity) = world.get_entity_mut(entity) {
            entity.despawn();
        }
    }
    world.insert_resource(ActiveVistaSurface::default());
    world.insert_resource(buildings::TacticalBuildingMeshCache::default());
    if let Some(mut assets) = world.get_resource_mut::<buildings::PreparedCityAssets>() {
        assets.release_details();
    }
    world.insert_resource(obstacles::tree::TreePresentationCache::default());
    world.insert_resource(obstacles::tree::VistaTreePresentationCache::default());
    world.flush();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clearing_exhibit_removes_detached_scatter_and_descendants_but_keeps_camera() {
        let mut world = World::new();
        world.insert_resource(PendingCityBuildings::new(&[]));
        let camera = world.spawn(TacticalGameplayCamera).id();
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
        clear_demo_scene(&mut world);
        assert!(world.get_entity(camera).is_ok());
    }
}
