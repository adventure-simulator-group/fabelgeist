//! Upload shared capacity grids once, with each placement's actual record spans.

use std::{collections::BTreeMap, sync::Arc};

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy::render::storage::ShaderBuffer;
use fabelgeist_math::units::Metres;

use super::super::{
    ForestDetail,
    benchmark::ForestBenchmark,
    ground::{Ground, GroundAssets},
    levels::{ForestLodView, Level},
    records::{Bytes, ForestBuffers, PendingWrites, Stored},
};
use super::{
    TreeBatching, TreeFeed, TreeLayout, TreeMeshStats,
    batches::{self, TreeChunk},
    grids::GridKey,
    materials::{TreeBranchMaterial, TreeLeafMaterial},
    resident::{ResidentBank, TreeBankAssets, TreeResidencyPolicy},
};
use crate::prepared::{SharedForest, TreeLibrary, TreeModel, TreePose};

/// GPU ABI: two aligned pose vec4s, then branch start/count and leaf start/count.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct PlacementRecord {
    origin_horizontal: [f32; 4],
    vertical_yaw: [f32; 4],
    records: [u32; 4],
}

pub(super) struct TreeMaterials {
    pub branches: Handle<TreeBranchMaterial>,
    pub leaves: Handle<TreeLeafMaterial>,
}

#[derive(Resource)]
pub(super) struct TreeAssets {
    pub grids: BTreeMap<GridKey, [Handle<Mesh>; 3]>,
    pub stats: TreeMeshStats,
    pub materials: [TreeMaterials; 3],
    _bank: Arc<TreeBankAssets>,
    _placements: Stored,
}

#[derive(SystemParam)]
pub(super) struct TreeStores<'w> {
    pub(super) meshes: ResMut<'w, Assets<Mesh>>,
    branches: ResMut<'w, Assets<TreeBranchMaterial>>,
    leaves: ResMut<'w, Assets<TreeLeafMaterial>>,
    pub(super) buffers: ResMut<'w, Assets<ShaderBuffer>>,
    pub(super) writes: ResMut<'w, PendingWrites>,
    pub(super) resident: ResMut<'w, ResidentBank>,
    pub(super) residency_policy: Res<'w, TreeResidencyPolicy>,
}

#[derive(SystemParam)]
pub(super) struct TreeSight<'w> {
    detail: Res<'w, ForestDetail>,
    view: Res<'w, ForestLodView>,
    probe: Option<Res<'w, ForestBenchmark>>,
    layout: Res<'w, TreeLayout>,
    batching: Res<'w, TreeBatching>,
}

impl PlacementRecord {
    fn new(pose: TreePose, model: &TreeModel) -> Self {
        let [x, y, z] = pose.origin().map(Metres::metres);
        let (sin, cos) = pose.yaw().radians().sin_cos();
        Self {
            origin_horizontal: [x, y, z, pose.horizontal().ratio()],
            vertical_yaw: [pose.vertical().ratio(), sin, cos, 0.0],
            records: [
                model.branches.start,
                model.branches.count,
                model.leaves.start,
                model.leaves.count,
            ],
        }
    }
}

impl TreeStores<'_> {
    fn upload(
        &mut self,
        forest: &SharedForest,
        layout: TreeLayout,
        probe: Option<&ForestBenchmark>,
    ) -> crate::prepared::Result<TreeAssets> {
        let library = forest.bank().library();
        let selected = self.bank(Arc::clone(forest.bank()), layout, probe)?;
        let placements: Vec<_> = forest
            .population()
            .placements
            .iter()
            .map(|placement| {
                PlacementRecord::new(placement.pose, &library.models[placement.variant.index()])
            })
            .collect();
        let mut placed = Stored::new(&mut self.buffers);
        placed.write(Bytes::new(&placements), &mut self.buffers, &mut self.writes);
        let materials = materials(library, &selected.bank.records, &placed, self);
        let mut stats = selected.bank.grids.stats.clone();
        stats.residency = Some(selected.evidence(
            forest,
            &placed,
            std::mem::size_of_val(placements.as_slice()),
        ));
        Ok(TreeAssets {
            grids: selected.bank.grids.handles.clone(),
            stats,
            materials,
            _bank: selected.bank,
            _placements: placed,
        })
    }
}

pub(super) fn draw(
    mut commands: Commands,
    mut feed: ResMut<TreeFeed>,
    mut stores: ParamSet<(TreeStores, GroundAssets)>,
    shown: Query<Entity, With<TreeChunk>>,
    sight: TreeSight,
) -> Result {
    let TreeSight {
        detail,
        view,
        probe,
        layout,
        batching,
    } = sight;
    let Some(forest) = feed.pending.take() else {
        return Ok(());
    };
    if *layout != TreeLayout::CapacityBatches
        && matches!(detail.level, super::super::LevelSelection::Adaptive)
    {
        return Err(crate::prepared::PreparedError::Invalid(
            "model-layout diagnostics require a fixed LOD",
        )
        .into());
    }
    for entity in &shown {
        commands.entity(entity).despawn();
    }
    let mut assets = stores.p0().upload(&forest, *layout, probe.as_deref())?;
    let spawned = batches::show(
        &mut commands,
        &forest,
        &assets,
        &view,
        &detail,
        probe.as_deref(),
        *batching,
    )?;
    assets.stats.batch_digest = spawned.digest;
    assets.stats.occupancy = spawned.occupancy;
    commands.insert_resource(assets.stats.clone());
    stores
        .p1()
        .show(&mut commands, &Ground::prepared(&forest.population().frame));
    commands.insert_resource(assets);
    feed.drawn = true;
    Ok(())
}

fn materials(
    library: &TreeLibrary,
    records: &ForestBuffers,
    placements: &Stored,
    stores: &mut TreeStores,
) -> [TreeMaterials; 3] {
    std::array::from_fn(|index| {
        let shape = Level::ALL[index].shape(&library.detail);
        let branches = stores.branches.add(TreeBranchMaterial {
            lod: UVec4::new(shape.sides, shape.spans, 0, 0),
            branches: records.branches(),
            species: records.species(),
            plants: records.plants(),
            placements: placements.handle.clone(),
        });
        let leaves = stores.leaves.add(TreeLeafMaterial {
            lod: UVec4::new(shape.blades, 0, 0, 0),
            leaves: records.leaves(),
            species: records.species(),
            placements: placements.handle.clone(),
        });
        TreeMaterials { branches, leaves }
    })
}
