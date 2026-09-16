//! Keep recently used studio exhibits resident in a bounded navigation cache.
//! Scenery shares prepared facades, but releases terrain and inspection detail.

use std::collections::{HashMap, VecDeque};

use super::*;

// Render-world removals and asset handle drops settle across frames. Keep the
// release window for large scenery allocations without delaying cached models.
const SCENERY_RELEASE_FRAMES: u8 = 4;
const STUDIO_CACHE_CAPACITY: usize = 6;

#[derive(Resource)]
struct SceneryRetirement(u8);

#[derive(Resource)]
pub(super) struct PendingScenery(ExhibitId);

#[derive(Resource)]
pub(super) struct ExhibitCache {
    studio: HashMap<ExhibitId, Result<Option<Handle<WorldAsset>>, String>>,
    views: HashMap<ExhibitId, OrbitView>,
    recency: VecDeque<ExhibitId>,
}

impl Default for ExhibitCache {
    fn default() -> Self {
        Self {
            studio: HashMap::new(),
            views: HashMap::new(),
            recency: VecDeque::new(),
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
    *world.resource_mut::<OrbitView>() = exhibit.view();
    let result = if exhibit.is_studio() {
        exhibits::studio(world);
        load_studio(world, &exhibit)
    } else if world.contains_resource::<SceneryRetirement>() {
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

fn load_studio(world: &mut World, exhibit: &Exhibit) -> Result<Option<Handle<WorldAsset>>, String> {
    if let Some(result) = world
        .resource::<ExhibitCache>()
        .studio
        .get(&exhibit.id())
        .cloned()
    {
        touch(world, exhibit.id());
        return result;
    }
    evict_for(world, exhibit.id());
    let result = exhibit.spawn(world);
    let mut cache = world.resource_mut::<ExhibitCache>();
    cache.studio.insert(exhibit.id(), result.clone());
    cache.recency.push_back(exhibit.id());
    result
}

fn touch(world: &mut World, id: ExhibitId) {
    let mut cache = world.resource_mut::<ExhibitCache>();
    cache.recency.retain(|cached| *cached != id);
    cache.recency.push_back(id);
}

fn evict_for(world: &mut World, incoming: ExhibitId) {
    if world.resource::<ExhibitCache>().studio.len() < STUDIO_CACHE_CAPACITY {
        return;
    }
    let active = world
        .get_resource::<CurrentExhibit>()
        .map(|current| current.id);
    let evicted = {
        let mut cache = world.resource_mut::<ExhibitCache>();
        let position = cache
            .recency
            .iter()
            .position(|id| Some(*id) != active && *id != incoming);
        position.and_then(|position| cache.recency.remove(position))
    };
    let Some(evicted) = evicted else {
        return;
    };
    {
        let mut cache = world.resource_mut::<ExhibitCache>();
        cache.studio.remove(&evicted);
        cache.views.remove(&evicted);
    }
    let entities = world
        .query::<(Entity, &DemoEntity)>()
        .iter(world)
        .filter_map(|(entity, exhibit)| (exhibit.0 == evicted).then_some(entity))
        .collect::<Vec<_>>();
    for entity in entities {
        world.entity_mut(entity).despawn();
    }
}

/// Prepare one explicitly requested studio exhibit without changing selection.
pub(super) fn prefetch(world: &mut World, id: ExhibitId) {
    let exhibit = Exhibit::get(id);
    if !exhibit.is_studio()
        || world
            .get_resource::<CurrentExhibit>()
            .is_some_and(|current| current.id == id)
    {
        return;
    }
    let active = world
        .get_resource::<CurrentExhibit>()
        .map(|current| current.id);
    let _ = load_studio(world, &exhibit);
    world.flush();
    for (entity, mut visibility) in world
        .query::<(&DemoEntity, &mut Visibility)>()
        .iter_mut(world)
    {
        if Some(entity.0) != active {
            *visibility = Visibility::Hidden;
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

    #[test]
    fn studio_cache_evicts_the_least_recent_inactive_exhibit() {
        let mut world = World::new();
        world.init_resource::<ExhibitCache>();
        world.init_resource::<OrbitView>();
        world.init_resource::<Assets<Mesh>>();
        world.init_resource::<Assets<StandardMaterial>>();
        for id in [
            ExhibitId::Longsword,
            ExhibitId::ArmingSword,
            ExhibitId::Rapier,
            ExhibitId::Spear,
            ExhibitId::Halberd,
            ExhibitId::Dagger,
            ExhibitId::Henry,
        ] {
            show(&mut world, id);
        }
        let cache = world.resource::<ExhibitCache>();
        assert_eq!(cache.studio.len(), STUDIO_CACHE_CAPACITY);
        assert!(!cache.studio.contains_key(&ExhibitId::Longsword));
        assert!(cache.studio.contains_key(&ExhibitId::Henry));
    }
}
