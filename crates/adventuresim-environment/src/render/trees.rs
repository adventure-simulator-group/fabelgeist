//! Whole trees as instances of stored model meshes. Frustum culling is per
//! variant/size/ground-cell batch; instance distance culling remains Eidolon's.

use std::sync::Arc;

use bevy::prelude::*;
use bevy_eidolon::prelude::*;

use super::ForestPlugin;
use crate::prepared::{TreeLibrary, TreePopulation};

mod batches;
mod materials;
mod upload;

pub use materials::{TreeBranchMaterial, TreeLeafMaterial};

/// Library residency and one regional descriptor set, already validated.
pub struct SharedForest {
    pub library: Arc<TreeLibrary>,
    pub population: Arc<TreePopulation>,
}

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
        .add_systems(
            Update,
            (upload::draw, batches::pick)
                .chain()
                .after(super::levels::follow_camera),
        );
    }
}
