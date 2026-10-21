//! The branch and leaf materials, and the snapshot data they read.
//!
//! Both read their instances from storage buffers by the index each
//! instance carries; a new snapshot replaces the buffers.

use bevy::mesh::MeshVertexBufferLayoutRef;
use bevy::prelude::*;
use bevy::render::render_resource::{
    AsBindGroup, RenderPipelineDescriptor, SpecializedMeshPipelineError,
};
use bevy::render::storage::ShaderBuffer;
use bevy::shader::ShaderRef;
use bevy_eidolon::prelude::*;
use fabelgeist_botany::species::Foliage;
use fabelgeist_botany::viewer::sim::SpeciesInfo;

use super::levels::{Level, Shape};
use super::records::{ForestBuffers, PendingWrites, Records};
use super::{ForestDetail, shapes};

const BRANCH_SHADER: &str = "embedded://adventuresim_environment/render/shaders/branch.wgsl";
const LEAF_SHADER: &str = "embedded://adventuresim_environment/render/shaders/leaf.wgsl";

/// Branch segments, built as tubes.
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct BranchMaterial {
    /// x: sides around a tube; y: spans along it.
    #[uniform(0)]
    lod: UVec4,
    #[storage(1, read_only, visibility(vertex, fragment))]
    branches: Handle<ShaderBuffer>,
    #[storage(2, read_only, visibility(vertex, fragment))]
    species: Handle<ShaderBuffer>,
    #[storage(3, read_only, visibility(vertex, fragment))]
    plants: Handle<ShaderBuffer>,
}

/// Leaf clusters, built as blades.
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct LeafMaterial {
    /// x: blades a cluster.
    #[uniform(0)]
    lod: UVec4,
    #[storage(1, read_only, visibility(vertex, fragment))]
    leaves: Handle<ShaderBuffer>,
    #[storage(2, read_only, visibility(vertex, fragment))]
    species: Handle<ShaderBuffer>,
}

impl InstancedMaterial for BranchMaterial {
    fn vertex_shader() -> ShaderRef {
        BRANCH_SHADER.into()
    }

    fn fragment_shader() -> ShaderRef {
        BRANCH_SHADER.into()
    }

    /// The default prepass would draw the base grid unbuilt.
    fn disable_prepass(&self) -> bool {
        true
    }
}

impl InstancedMaterial for LeafMaterial {
    fn vertex_shader() -> ShaderRef {
        LEAF_SHADER.into()
    }

    fn fragment_shader() -> ShaderRef {
        LEAF_SHADER.into()
    }

    fn disable_prepass(&self) -> bool {
        true
    }

    /// A blade is seen from both sides.
    fn double_sided(&self) -> bool {
        true
    }

    fn specialize(
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayoutRef,
        _key: Self::Data,
    ) -> Result<(), SpecializedMeshPipelineError> {
        descriptor.primitive.cull_mode = None;
        // The blades' outlines become multisample coverage.
        descriptor.multisample.alpha_to_coverage_enabled = descriptor.multisample.count > 1;
        Ok(())
    }
}

/// A species as the shaders read it: bark and leaf colour, sRGB, and the
/// foliage kind in the leaf colour's fourth component.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub(super) struct SpeciesRecord {
    bark: [f32; 4],
    leaf: [f32; 4],
}

impl From<&SpeciesInfo> for SpeciesRecord {
    fn from(info: &SpeciesInfo) -> Self {
        let [r, g, b] = info.bark;
        let [lr, lg, lb] = info.leaf;
        Self {
            bark: [r, g, b, 1.0],
            leaf: [lr, lg, lb, foliage_kind(info.foliage)],
        }
    }
}

fn foliage_kind(foliage: Foliage) -> f32 {
    f32::from(foliage as u8)
}

/// Where a snapshot's data goes: the materials and their storage buffers.
#[derive(bevy::ecs::system::SystemParam)]
pub(super) struct ForestStores<'w> {
    branch_materials: ResMut<'w, Assets<BranchMaterial>>,
    leaf_materials: ResMut<'w, Assets<LeafMaterial>>,
    buffers: ResMut<'w, Assets<ShaderBuffer>>,
    writes: ResMut<'w, PendingWrites>,
}

/// One level of detail's meshes and materials.
pub(super) struct LevelAssets {
    pub(super) branch_mesh: Handle<Mesh>,
    pub(super) leaf_mesh: Handle<Mesh>,
    pub(super) branches: Handle<BranchMaterial>,
    pub(super) leaves: Handle<LeafMaterial>,
}

/// The forest's meshes and materials at every level of detail, shared by
/// every chunk, and the storage buffers they all read.
#[derive(Resource)]
pub(super) struct ForestMaterials {
    levels: [LevelAssets; 3],
    buffers: ForestBuffers,
}

pub(super) fn setup(
    mut commands: Commands,
    detail: Res<ForestDetail>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut branch_materials: ResMut<Assets<BranchMaterial>>,
    mut leaf_materials: ResMut<Assets<LeafMaterial>>,
    mut buffers: ResMut<Assets<ShaderBuffer>>,
) {
    let stored = ForestBuffers::new(&mut buffers);
    let levels = Level::ALL.map(|level| {
        let shape = level.shape(&detail);
        LevelAssets {
            branch_mesh: meshes.add(shapes::branch_grid(shape.sides, shape.spans)),
            leaf_mesh: meshes.add(shapes::leaf_blades(shape.blades)),
            branches: branch_materials.add(BranchMaterial {
                lod: branch_lod(&shape),
                branches: stored.branches(),
                species: stored.species(),
                plants: stored.plants(),
            }),
            leaves: leaf_materials.add(LeafMaterial {
                lod: leaf_lod(&shape),
                leaves: stored.leaves(),
                species: stored.species(),
            }),
        }
    });
    commands.insert_resource(ForestMaterials {
        levels,
        buffers: stored,
    });
}

/// Rebuild the full level's meshes when the detail changes; the coarser
/// levels are fixed.
pub(super) fn follow_detail(
    detail: Res<ForestDetail>,
    forest: Option<Res<ForestMaterials>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut branch_materials: ResMut<Assets<BranchMaterial>>,
    mut leaf_materials: ResMut<Assets<LeafMaterial>>,
) {
    let Some(forest) = forest else {
        return;
    };
    if !detail.is_changed() || detail.is_added() {
        return;
    }
    let full = forest.level(Level::Full);
    let shape = Level::Full.shape(&detail);
    let _ = meshes.insert(
        &full.branch_mesh,
        shapes::branch_grid(shape.sides, shape.spans),
    );
    let _ = meshes.insert(&full.leaf_mesh, shapes::leaf_blades(shape.blades));
    if let Some(mut material) = branch_materials.get_mut(&full.branches) {
        material.lod = branch_lod(&shape);
    }
    if let Some(mut material) = leaf_materials.get_mut(&full.leaves) {
        material.lod = leaf_lod(&shape);
    }
}

impl ForestMaterials {
    pub(super) fn level(&self, level: Level) -> &LevelAssets {
        &self.levels[level.index()]
    }

    /// Write a snapshot's records into the storage buffers, binding any
    /// buffer that had to be replaced.
    pub(super) fn store(&mut self, records: Records, stores: &mut ForestStores) {
        let ForestStores {
            branch_materials,
            leaf_materials,
            buffers,
            writes,
        } = stores;
        if !self.buffers.write(records, buffers, writes) {
            return;
        }
        let bound = &self.buffers;
        for level in &self.levels {
            if let Some(mut material) = branch_materials.get_mut(&level.branches) {
                material.branches = bound.branches();
                material.plants = bound.plants();
                material.species = bound.species();
            }
            if let Some(mut material) = leaf_materials.get_mut(&level.leaves) {
                material.leaves = bound.leaves();
                material.species = bound.species();
            }
        }
    }
}

fn branch_lod(shape: &Shape) -> UVec4 {
    UVec4::new(shape.sides, shape.spans, 0, 0)
}

fn leaf_lod(shape: &Shape) -> UVec4 {
    UVec4::new(shape.blades, 0, 0, 0)
}
