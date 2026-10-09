//! Client-generated static city geometry, selected and compacted on the GPU.
mod assembly;
mod compute;
mod draw;
mod geometry;
mod material;
mod scratch;
pub(super) use assembly::PendingGpuBuildings;

use bevy::{
    prelude::*,
    render::{
        extract_resource::{ExtractResource, ExtractResourcePlugin},
        storage::ShaderBuffer,
    },
};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

static READY: AtomicBool = AtomicBool::new(false);

pub(super) fn is_ready() -> bool {
    READY.load(Ordering::Relaxed)
}

pub(super) fn reset(world: &mut World) {
    READY.store(false, Ordering::Relaxed);
    world.insert_resource(CityGpuScene::default());
    world.insert_resource(PendingGpuBuildings::default());
}

pub(super) const TRIANGLES_PER_CLUSTER: usize = 64;
pub(super) const VERTICES_PER_CLUSTER: u32 = (TRIANGLES_PER_CLUSTER * 3) as u32;
// Initial header captures and their shadow views may coexist during warm-up.
pub(super) const MAX_CITY_VIEWS: usize = 64;

#[derive(Clone)]
struct GpuBatch {
    pub owners: Handle<ShaderBuffer>,
    pub material: Handle<material::CityMaterial>,
    pub source: Handle<ShaderBuffer>,
    pub vertices: Handle<ShaderBuffer>,
    pub indices: Handle<ShaderBuffer>,
    pub ranges: u32,
    pub capacity: u32,
}

#[derive(Resource, Clone, Default, ExtractResource)]
struct CityGpuScene {
    pub buildings: Handle<ShaderBuffer>,
    pub selection: Handle<ShaderBuffer>,
    pub count: u32,
    pub batches: Arc<Vec<GpuBatch>>,
}

pub(super) struct CityGpuPlugin;

impl Plugin for CityGpuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CityGpuScene>()
            .init_resource::<PendingGpuBuildings>()
            .add_plugins((
                MaterialPlugin::<material::CityMaterial>::default(),
                ExtractResourcePlugin::<CityGpuScene>::default(),
            ))
            // PostUpdate material conversion queues entity updates. Publish
            // after those commands are applied, before render extraction.
            .add_systems(Last, assembly::assemble);
        compute::install(app);
        draw::install(app);
    }
}
