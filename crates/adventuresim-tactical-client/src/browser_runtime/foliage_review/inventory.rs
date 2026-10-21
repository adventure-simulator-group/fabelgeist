//! Post-drain logical asset counts; these do not measure peak or physical VRAM.

use super::session::Arrival;
use crate::presentation::interior_lighting::InteriorMaterial;
use adventuresim_environment::render::trees::{TreeBranchMaterial, TreeLeafMaterial};
use anyhow::{Result, ensure};
use bevy::{prelude::*, render::storage::ShaderBuffer};
use serde::Serialize;

#[derive(Clone, Serialize)]
pub(super) struct AssetInventory {
    meshes: usize,
    storage: usize,
    branches: usize,
    leaves: usize,
    standard: usize,
    interior: usize,
    images: usize,
    render_meshes: usize,
    render_storage: usize,
}

impl AssetInventory {
    pub fn snapshot(world: &World, arrival: &Arrival) -> Result<Self> {
        let native = arrival
            .probe
            .as_ref()
            .and_then(|p| p.native_residency())
            .ok_or_else(|| anyhow::anyhow!("missing completed frame inventory"))?
            .asset_counts();
        Ok(Self {
            meshes: world.resource::<Assets<Mesh>>().len(),
            storage: world.resource::<Assets<ShaderBuffer>>().len(),
            branches: world.resource::<Assets<TreeBranchMaterial>>().len(),
            leaves: world.resource::<Assets<TreeLeafMaterial>>().len(),
            standard: world.resource::<Assets<StandardMaterial>>().len(),
            interior: world.resource::<Assets<InteriorMaterial>>().len(),
            images: world.resource::<Assets<Image>>().len(),
            render_meshes: native.meshes,
            render_storage: native.storage,
        })
    }
    pub fn within(&self, baseline: &Self) -> Result<()> {
        ensure!(
            self.meshes <= baseline.meshes
                && self.storage <= baseline.storage
                && self.branches <= baseline.branches
                && self.leaves <= baseline.leaves
                && self.standard <= baseline.standard
                && self.interior <= baseline.interior
                && self.images <= baseline.images
                && self.render_meshes <= baseline.render_meshes
                && self.render_storage <= baseline.render_storage,
            "post-drain assets exceed the first bank checkpoint"
        );
        Ok(())
    }
}
