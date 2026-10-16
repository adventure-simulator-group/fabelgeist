//! Client-generated static city geometry, selected and compacted on the GPU.
use crate::presentation::ownership::{PresentationOwner, PresentationOwners};
pub(in crate::presentation) use assembly::{CityAssemblyFailure, CityAssemblyPublished};
pub(super) use assembly::{PendingGpuBuildings, PendingGpuCities, PlacementAppearance};
use bevy::{
    prelude::*,
    render::{
        extract_resource::{ExtractResource, ExtractResourcePlugin},
        storage::ShaderBuffer,
    },
};
pub(in crate::presentation) use frame::CityFrame;
pub(in crate::presentation) use retained::RetainedCityGeometry;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

mod assembly;
mod compute;
mod draw;
mod frame;
mod geometry;
mod material;
mod retained;
mod scratch;

static READY: PresentationOwners<AtomicBool> =
    PresentationOwners::new(AtomicBool::new(false), AtomicBool::new(false));
pub(super) const TRIANGLES_PER_CLUSTER: usize = 64;
pub(super) const VERTICES_PER_CLUSTER: u32 = (TRIANGLES_PER_CLUSTER * 3) as u32;
// Initial header captures and their shadow views may coexist during warm-up.
pub(super) const MAX_CITY_VIEWS: usize = 64;

#[derive(Clone)]
struct GpuBatch {
    pub anchor: Entity,
    pub owners: Handle<ShaderBuffer>,
    pub material: Handle<material::CityMaterial>,
    pub source: Handle<ShaderBuffer>,
    pub vertices: Handle<ShaderBuffer>,
    pub indices: Handle<ShaderBuffer>,
    pub ranges: u32,
    pub capacity: u32,
}

#[derive(Resource, Clone, Default, ExtractResource)]
struct CityGpuScenes {
    owners: PresentationOwners<CityGpuScene>,
}

#[derive(Clone, Default)]
struct CityGpuScene {
    pub publication_root: Option<Entity>,
    pub frame: CityFrame,
    pub frame_buffer: Handle<ShaderBuffer>,
    pub buildings: Handle<ShaderBuffer>,
    pub selection: Handle<ShaderBuffer>,
    pub count: u32,
    pub batches: Arc<Vec<GpuBatch>>,
}

#[derive(Component)]
struct CityBatchAnchor;

#[derive(Component)]
struct RetainedCityAnchor;

pub(super) struct CityGpuPlugin;

impl Plugin for CityGpuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CityGpuScenes>()
            .init_resource::<PendingGpuCities>()
            .add_plugins((
                MaterialPlugin::<material::CityMaterial>::default(),
                ExtractResourcePlugin::<CityGpuScenes>::default(),
            ))
            // PostUpdate material conversion queues entity updates. Publish
            // after those commands are applied, before render extraction.
            .add_systems(Last, assembly::assemble);
        compute::install(app);
        draw::install(app);
    }
}

pub(super) fn is_ready(owner: PresentationOwner) -> bool {
    READY.get(owner).load(Ordering::Relaxed)
}

/// Ground coverage controls map anchors independently of the actor owner.
pub(in crate::presentation) fn set_visible(
    world: &mut World,
    owner: PresentationOwner,
    visible: bool,
) {
    let Some(scenes) = world.get_resource::<CityGpuScenes>() else {
        return;
    };
    let anchors = scenes
        .owners
        .get(owner)
        .batches
        .iter()
        .map(|batch| batch.anchor)
        .collect::<Vec<_>>();
    for anchor in anchors {
        if let Some(mut visibility) = world.get_mut::<Visibility>(anchor) {
            *visibility = if visible {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            };
        }
    }
}

/// Drop the active city and unretained anchors; cached cities own their lifetime.
pub(super) fn reset(world: &mut World, owner: PresentationOwner) {
    READY.get(owner).store(false, Ordering::Relaxed);
    world.init_resource::<CityGpuScenes>();
    world.init_resource::<PendingGpuCities>();
    let previous = std::mem::take(world.resource_mut::<CityGpuScenes>().owners.get_mut(owner));
    world
        .resource_mut::<PendingGpuCities>()
        .owners
        .get_mut(owner)
        .clear();
    previous.discard(world);
    let anchors = world
        .query_filtered::<(Entity, &PresentationOwner), (With<CityBatchAnchor>, Without<RetainedCityAnchor>)>()
        .iter(world)
        .filter_map(|(entity, candidate)| (*candidate == owner).then_some(entity))
        .collect::<Vec<_>>();
    for anchor in anchors {
        if let Ok(entity) = world.get_entity_mut(anchor) {
            entity.despawn();
        }
    }
}
