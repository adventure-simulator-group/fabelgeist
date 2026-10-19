//! Upload shared capacity grids once, with each placement's actual record spans.

use std::collections::BTreeMap;

use bevy::asset::RenderAssetUsages;
use bevy::ecs::system::SystemParam;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use bevy::render::storage::ShaderBuffer;
use fabelgeist_math::units::Metres;

use super::super::{
    ForestDetail,
    benchmark::ForestBenchmark,
    ground::{Ground, GroundAssets},
    levels::{Level, LevelView},
    records::{Bytes, ForestBuffers, PendingWrites, Records, Stored},
};
use super::{
    SharedForest, TreeFeed, VERTEX_ADDRESS,
    batches::{self, TreeChunk},
    materials::{TreeBranchMaterial, TreeLeafMaterial},
};
use crate::prepared::{AddressMesh, MeshClass, TreeLibrary, TreeModel, TreePose};

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
    pub grids: BTreeMap<MeshClass, [Handle<Mesh>; 3]>,
    pub materials: [TreeMaterials; 3],
    _records: ForestBuffers,
    _placements: Stored,
}

#[derive(SystemParam)]
pub(super) struct TreeStores<'w> {
    meshes: ResMut<'w, Assets<Mesh>>,
    branches: ResMut<'w, Assets<TreeBranchMaterial>>,
    leaves: ResMut<'w, Assets<TreeLeafMaterial>>,
    buffers: ResMut<'w, Assets<ShaderBuffer>>,
    writes: ResMut<'w, PendingWrites>,
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
    fn upload(&mut self, forest: &SharedForest, probe: Option<&ForestBenchmark>) -> TreeAssets {
        let library = &forest.library;
        let records = Records::from_parts(
            &library.species,
            &library.branches,
            &library.plants,
            &library.leaves,
        );
        let mut stored = ForestBuffers::new(&mut self.buffers);
        stored.write(records, &mut self.buffers, &mut self.writes);
        let placements: Vec<_> = forest
            .population
            .placements
            .iter()
            .map(|placement| {
                PlacementRecord::new(placement.pose, &library.models[placement.variant.index()])
            })
            .collect();
        let mut placed = Stored::new(&mut self.buffers);
        placed.write(Bytes::new(&placements), &mut self.buffers, &mut self.writes);
        let materials = materials(library, &stored, &placed, self);
        let grids = upload_grids(library, self, probe);
        TreeAssets {
            grids,
            materials,
            _records: stored,
            _placements: placed,
        }
    }
}

pub(super) fn draw(
    mut commands: Commands,
    mut feed: ResMut<TreeFeed>,
    mut stores: ParamSet<(TreeStores, GroundAssets)>,
    shown: Query<Entity, With<TreeChunk>>,
    detail: Res<ForestDetail>,
    view: Res<LevelView>,
    probe: Option<Res<ForestBenchmark>>,
) -> Result {
    let Some(forest) = feed.pending.take() else {
        return Ok(());
    };
    for entity in &shown {
        commands.entity(entity).despawn();
    }
    let assets = stores.p0().upload(&forest, probe.as_deref());
    batches::show(
        &mut commands,
        &forest,
        &assets,
        &view,
        &detail,
        probe.as_deref(),
    )?;
    stores
        .p1()
        .show(&mut commands, &Ground::prepared(&forest.population.frame));
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

fn upload_grids(
    library: &TreeLibrary,
    stores: &mut TreeStores,
    probe: Option<&ForestBenchmark>,
) -> BTreeMap<MeshClass, [Handle<Mesh>; 3]> {
    library
        .grids
        .iter()
        .map(|grid| {
            let levels = std::array::from_fn(|index| {
                let addresses = &grid.levels[index];
                let mesh = stores.meshes.add(mesh(addresses));
                if let Some(probe) = probe {
                    probe.expect_mesh(mesh.id(), addresses.vertices.len(), addresses.indices.len());
                }
                mesh
            });
            (grid.class, levels)
        })
        .collect()
}

fn mesh(addresses: &AddressMesh) -> Mesh {
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    )
    .with_inserted_attribute(VERTEX_ADDRESS, addresses.vertices.clone())
    .with_inserted_indices(Indices::U32(addresses.indices.clone()))
}
