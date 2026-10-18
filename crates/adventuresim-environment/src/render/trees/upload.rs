//! Admit a prepared bank once, then upload compact placement records.

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
    TreeFeed,
    batches::{self, TreeChunk},
    materials::{TreeBranchMaterial, TreeLeafMaterial},
};
use crate::prepared::{AddressMesh, TreeLibrary, TreePose};

/// Two aligned vec4 words in the shader's explicit native boundary.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct PlacementRecord {
    origin_horizontal: [f32; 4],
    vertical_yaw: [f32; 4],
}

pub(super) struct TreeLevel {
    pub branch_mesh: Option<Handle<Mesh>>,
    pub leaf_mesh: Option<Handle<Mesh>>,
    pub branches: Handle<TreeBranchMaterial>,
    pub leaves: Handle<TreeLeafMaterial>,
}

struct TreeMaterials {
    branches: Handle<TreeBranchMaterial>,
    leaves: Handle<TreeLeafMaterial>,
}

#[derive(Resource)]
pub(super) struct TreeAssets {
    pub models: Vec<[TreeLevel; 3]>,
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

impl From<TreePose> for PlacementRecord {
    fn from(pose: TreePose) -> Self {
        let [x, y, z] = pose.origin().map(Metres::metres);
        let (sin, cos) = pose.yaw().radians().sin_cos();
        Self {
            origin_horizontal: [x, y, z, pose.horizontal().ratio()],
            vertical_yaw: [pose.vertical().ratio(), sin, cos, 0.0],
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
) {
    let Some(forest) = feed.pending.take() else {
        return;
    };
    for entity in &shown {
        commands.entity(entity).despawn();
    }
    let assets = {
        let mut stores = stores.p0();
        let library = &forest.library;
        let records = Records::from_parts(
            &library.species,
            &library.branches,
            &library.plants,
            &library.leaves,
        );
        let mut stored = ForestBuffers::new(&mut stores.buffers);
        stored.write(records, &mut stores.buffers, &mut stores.writes);
        let placements: Vec<PlacementRecord> = forest
            .population
            .placements
            .iter()
            .map(|p| p.pose.into())
            .collect();
        let mut placed = Stored::new(&mut stores.buffers);
        placed.write(
            Bytes::new(&placements),
            &mut stores.buffers,
            &mut stores.writes,
        );
        let models = upload_models(library, &stored, &placed, &mut stores, probe.as_deref());
        TreeAssets {
            models,
            _records: stored,
            _placements: placed,
        }
    };
    batches::show(
        &mut commands,
        &forest,
        &assets,
        &view,
        &detail,
        probe.as_deref(),
    );
    stores
        .p1()
        .show(&mut commands, &Ground::prepared(&forest.population.frame));
    commands.insert_resource(assets);
    feed.drawn = true;
}

fn upload_models(
    library: &TreeLibrary,
    records: &ForestBuffers,
    placements: &Stored,
    stores: &mut TreeStores,
    probe: Option<&ForestBenchmark>,
) -> Vec<[TreeLevel; 3]> {
    let materials: [_; 3] = std::array::from_fn(|index| {
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
    });
    library
        .models
        .iter()
        .map(|model| {
            std::array::from_fn(|index| {
                let meshes = &model.meshes[index];
                let branch_mesh =
                    (model.branches.count > 0).then(|| stores.meshes.add(mesh(&meshes.branches)));
                let leaf_mesh =
                    (model.leaves.count > 0).then(|| stores.meshes.add(mesh(&meshes.leaves)));
                if let Some(probe) = probe {
                    if let Some(branch_mesh) = &branch_mesh {
                        probe.expect_mesh(
                            branch_mesh.id(),
                            meshes.branches.vertices.len(),
                            meshes.branches.indices.len(),
                        );
                    }
                    if let Some(leaf_mesh) = &leaf_mesh {
                        probe.expect_mesh(
                            leaf_mesh.id(),
                            meshes.leaves.vertices.len(),
                            meshes.leaves.indices.len(),
                        );
                    }
                }
                TreeLevel {
                    branch_mesh,
                    leaf_mesh,
                    branches: materials[index].branches.clone(),
                    leaves: materials[index].leaves.clone(),
                }
            })
        })
        .collect()
}

fn mesh(addresses: &AddressMesh) -> Mesh {
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, addresses.vertices.clone())
    .with_inserted_indices(Indices::U32(addresses.indices.clone()))
}
