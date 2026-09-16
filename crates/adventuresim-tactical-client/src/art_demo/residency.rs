//! Keep studio exhibits and their assets resident independently of navigation.
//! Scenery shares prepared facades, but releases terrain and inspection detail.

use std::collections::{HashMap, VecDeque};

use super::*;

// Render-world removals and asset handle drops settle across frames. Keep the
// release window for large scenery allocations without delaying cached models.
const SCENERY_RELEASE_FRAMES: u8 = 4;

#[derive(Resource)]
struct SceneryRetirement(u8);

#[derive(Resource)]
pub(super) struct PendingScenery(ExhibitId);

#[derive(Resource)]
pub(super) struct ExhibitCache {
    studio: HashMap<ExhibitId, Result<Option<Handle<WorldAsset>>, String>>,
    views: HashMap<ExhibitId, OrbitView>,
    pending: VecDeque<Exhibit>,
}

impl Default for ExhibitCache {
    fn default() -> Self {
        Self {
            studio: HashMap::new(),
            views: HashMap::new(),
            pending: Exhibit::catalog()
                .into_iter()
                .filter(Exhibit::is_studio)
                .collect(),
        }
    }
}

pub(super) fn show(world: &mut World, id: ExhibitId) {
    if world
        .get_resource::<CurrentExhibit>()
        .is_some_and(|current| current.id == id)
    {
        return;
    }
    world.remove_resource::<PendingScenery>();
    if let Some(current) = world.remove_resource::<CurrentExhibit>() {
        let view = world.resource::<OrbitView>().clone();
        world
            .resource_mut::<ExhibitCache>()
            .views
            .insert(current.id, view);
        if !Exhibit::get(current.id).is_studio() {
            crate::presentation::clear_demo_scene(world);
            world.insert_resource(SceneryRetirement(SCENERY_RELEASE_FRAMES));
        }
    }
    let studio = world
        .query_filtered::<Entity, With<StudioEntity>>()
        .iter(world)
        .collect::<Vec<_>>();
    for entity in studio {
        world.entity_mut(entity).despawn();
    }

    let exhibit = Exhibit::get(id);
    if !exhibit.texture_recipes().is_empty() && world.contains_resource::<AssetServer>() {
        let server = world.resource::<AssetServer>().clone();
        world
            .resource_mut::<adventuresim_procedural_textures::ProceduralTextureResidency>()
            .request(&server, exhibit.texture_recipes().iter().copied());
    }
    *world.resource_mut::<OrbitView>() = exhibit.view();
    let result = if exhibit.is_studio() {
        exhibits::studio(world);
        load_studio(world, &exhibit)
    } else if world.contains_resource::<SceneryRetirement>() || !textures_ready(world, &exhibit) {
        world.insert_resource(PendingScenery(id));
        Ok(None)
    } else {
        exhibit.spawn(world)
    };
    world.flush();
    for (entity, mut visibility) in world
        .query::<(&DemoEntity, &mut Visibility)>()
        .iter_mut(world)
    {
        *visibility = if entity.0 == id {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
    if let Some(view) = world.resource::<ExhibitCache>().views.get(&id).cloned() {
        *world.resource_mut::<OrbitView>() = view;
    }
    world.insert_resource(CurrentExhibit { id, scene: result });
}

pub(super) fn spawn_pending(world: &mut World) {
    if let Some(mut retirement) = world.get_resource_mut::<SceneryRetirement>() {
        if retirement.0 > 0 {
            retirement.0 -= 1;
            return;
        }
        world.remove_resource::<SceneryRetirement>();
    }
    if world
        .get_resource::<PendingScenery>()
        .is_some_and(|pending| {
            let exhibit = Exhibit::get(pending.0);
            !textures_ready(world, &exhibit)
        })
    {
        return;
    }
    if let Some(pending) = world.remove_resource::<PendingScenery>() {
        let result = Exhibit::get(pending.0).spawn(world);
        if let Some(view) = world
            .resource::<ExhibitCache>()
            .views
            .get(&pending.0)
            .cloned()
        {
            *world.resource_mut::<OrbitView>() = view;
        }
        world.resource_mut::<CurrentExhibit>().scene = result;
    }
}

fn textures_ready(world: &World, exhibit: &Exhibit) -> bool {
    world
        .get_resource::<adventuresim_procedural_textures::ProceduralTextureResidency>()
        .is_none_or(|residency| residency.is_ready(exhibit.texture_recipes().iter().copied()))
}

fn load_studio(world: &mut World, exhibit: &Exhibit) -> Result<Option<Handle<WorldAsset>>, String> {
    if let Some(result) = world.resource::<ExhibitCache>().studio.get(&exhibit.id()) {
        return result.clone();
    }
    let result = exhibit.spawn(world);
    world
        .resource_mut::<ExhibitCache>()
        .studio
        .insert(exhibit.id(), result.clone());
    result
}

/// Prepare at most one small exhibit at a time, after the selected exhibit's
/// file dependencies and city assembly finish. No hidden studio lights run.
pub(super) fn prefetch(world: &mut World) {
    let Some(current) = world.get_resource::<CurrentExhibit>() else {
        return;
    };
    let active = current.id;
    if world.contains_resource::<PendingScenery>() {
        return;
    }
    if world
        .get_resource::<crate::presentation::PendingCityBuildings>()
        .is_some_and(|pending| !pending.finished())
    {
        return;
    }
    let server = world.resource::<AssetServer>();
    if world
        .resource::<ExhibitCache>()
        .studio
        .values()
        .any(|result| {
            let Ok(Some(handle)) = result else {
                return false;
            };
            !server.is_loaded_with_dependencies(handle.id())
                && !matches!(
                    server.load_state(handle.id()),
                    bevy::asset::LoadState::Failed(_)
                )
                && !matches!(
                    server.recursive_dependency_load_state(handle.id()),
                    bevy::asset::RecursiveDependencyLoadState::Failed(_)
                )
        })
    {
        return;
    }
    let next = {
        let mut cache = world.resource_mut::<ExhibitCache>();
        loop {
            let Some(exhibit) = cache.pending.pop_front() else {
                break None;
            };
            if !cache.studio.contains_key(&exhibit.id()) {
                break Some(exhibit);
            }
        }
    };
    if let Some(exhibit) = next {
        let _ = load_studio(world, &exhibit);
        world.flush();
        for (entity, mut visibility) in world
            .query::<(&DemoEntity, &mut Visibility)>()
            .iter_mut(world)
        {
            if entity.0 != active {
                *visibility = Visibility::Hidden;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn navigating_away_cancels_deferred_scenery_without_replacing_the_new_exhibit() {
        let mut world = World::new();
        world.init_resource::<ExhibitCache>();
        world.init_resource::<OrbitView>();
        world.init_resource::<Assets<Mesh>>();
        world.init_resource::<Assets<StandardMaterial>>();
        world.insert_resource(CurrentExhibit {
            id: ExhibitId::City,
            scene: Ok(None),
        });
        show(&mut world, ExhibitId::Oak);
        assert!(world.contains_resource::<PendingScenery>());
        show(&mut world, ExhibitId::Dagger);
        for _ in 0..=SCENERY_RELEASE_FRAMES {
            spawn_pending(&mut world);
        }
        assert!(!world.contains_resource::<PendingScenery>());
        assert_eq!(world.resource::<CurrentExhibit>().id, ExhibitId::Dagger);
        assert_eq!(world.query::<&DemoEntity>().iter(&world).count(), 1);
    }

    #[test]
    fn switching_weapons_reuses_entities_and_geometry_and_restores_camera() {
        let mut world = World::new();
        world.init_resource::<ExhibitCache>();
        world.init_resource::<OrbitView>();
        world.init_resource::<Assets<Mesh>>();
        world.init_resource::<Assets<StandardMaterial>>();
        show(&mut world, ExhibitId::Longsword);
        let sword = world
            .query::<(Entity, &DemoEntity)>()
            .iter(&world)
            .next()
            .unwrap()
            .0;
        world.resource_mut::<OrbitView>().yaw = 1.7;
        show(&mut world, ExhibitId::Dagger);
        assert_eq!(*world.get::<Visibility>(sword).unwrap(), Visibility::Hidden);
        let meshes = world.resource::<Assets<Mesh>>().len();
        show(&mut world, ExhibitId::Longsword);
        assert_eq!(
            *world.get::<Visibility>(sword).unwrap(),
            Visibility::Inherited
        );
        assert_eq!(world.resource::<OrbitView>().yaw, 1.7);
        // Only the freshly constructed studio dome adds geometry.
        assert_eq!(world.resource::<Assets<Mesh>>().len(), meshes + 1);
        assert_eq!(world.query::<&DemoEntity>().iter(&world).count(), 2);
        assert_eq!(world.query::<&DirectionalLight>().iter(&world).count(), 2);
        show(&mut world, ExhibitId::Longsword);
        assert_eq!(world.resource::<Assets<Mesh>>().len(), meshes + 1);
    }
}
