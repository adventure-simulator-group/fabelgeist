//! Whole trees as instances of shared capacity grids. Frustum culling is per
//! capacity/size/ground-cell batch; instance distance culling remains Eidolon's.

use std::sync::Arc;

use bevy::prelude::*;
use bevy_eidolon::prelude::*;

use super::{ForestLodSystems, ForestPlugin};
use crate::prepared::SharedForest;

mod batches;
mod batching;
mod grids;
mod materials;
mod resident;
mod upload;

/// Explicit Bevy vertex ABI: one integer address, decoded by the forest shader.
const VERTEX_ADDRESS: bevy::mesh::MeshVertexAttribute = bevy::mesh::MeshVertexAttribute::new(
    "ForestAddress",
    0x4652_5354_4144_4452,
    bevy::render::render_resource::VertexFormat::Uint32,
);

pub use batching::{TreeBatchOccupancy, TreeBatching};
pub use grids::{TreeLayout, TreeMeshStats};
pub use materials::{TreeBranchMaterial, TreeLeafMaterial};
pub use resident::{TreeResidencyEvidence, TreeResidencyPolicy};

#[derive(Resource, Default)]
pub struct TreeFeed {
    pending: Option<Arc<SharedForest>>,
    drawn: bool,
}

pub struct TreeForestPlugin;

impl TreeFeed {
    pub fn show(&mut self, forest: Arc<SharedForest>) {
        self.pending = Some(forest);
        self.drawn = false;
    }
    pub fn is_drawn(&self) -> bool {
        self.drawn
    }
}

impl Plugin for TreeForestPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<ForestPlugin>() {
            app.add_plugins(ForestPlugin);
        }
        app.add_plugins((
            InstancedMaterialPlugin::<TreeBranchMaterial>::default(),
            GpuCullComputePlugin::<TreeBranchMaterial>::default(),
            InstancedMaterialPlugin::<TreeLeafMaterial>::default(),
            GpuCullComputePlugin::<TreeLeafMaterial>::default(),
        ))
        .init_resource::<TreeFeed>()
        .init_resource::<TreeLayout>()
        .init_resource::<TreeBatching>()
        .init_resource::<TreeResidencyPolicy>()
        .init_resource::<resident::ResidentBank>()
        .add_systems(Update, upload::draw)
        .add_systems(PostUpdate, batches::pick.in_set(ForestLodSystems::Select));
    }
}
