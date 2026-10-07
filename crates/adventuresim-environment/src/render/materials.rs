//! The branch and leaf materials, and the snapshot data they read.
//!
//! Both read their instances from storage buffers by the index each
//! instance carries; a new snapshot replaces the buffers.

use bevy::asset::RenderAssetUsages;
use bevy::mesh::MeshVertexBufferLayoutRef;
use bevy::prelude::*;
use bevy::render::render_resource::{
    AsBindGroup, RenderPipelineDescriptor, SpecializedMeshPipelineError,
};
use bevy::render::storage::ShaderBuffer;
use bevy::shader::ShaderRef;
use bevy_eidolon::prelude::*;
use fabelgeist_botany::viewer::sim::{Foliage, Snapshot, SpeciesInfo};

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
struct SpeciesRecord {
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
}

/// The forest's meshes and materials, shared by every chunk.
#[derive(Resource)]
pub(super) struct ForestMaterials {
    pub(super) branch_mesh: Handle<Mesh>,
    pub(super) leaf_mesh: Handle<Mesh>,
    pub(super) branches: Handle<BranchMaterial>,
    pub(super) leaves: Handle<LeafMaterial>,
}

pub(super) fn setup(
    mut commands: Commands,
    detail: Res<ForestDetail>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut branch_materials: ResMut<Assets<BranchMaterial>>,
    mut leaf_materials: ResMut<Assets<LeafMaterial>>,
    mut buffers: ResMut<Assets<ShaderBuffer>>,
) {
    let mut empty = || buffers.add(storage::<SpeciesRecord>(&[]));
    let species = empty();
    let branches = branch_materials.add(BranchMaterial {
        lod: branch_lod(&detail),
        branches: empty(),
        species: species.clone(),
        plants: empty(),
    });
    let leaves = leaf_materials.add(LeafMaterial {
        lod: leaf_lod(&detail),
        leaves: empty(),
        species,
    });
    commands.insert_resource(ForestMaterials {
        branch_mesh: meshes.add(shapes::branch_grid(detail.tube_sides, detail.tube_spans)),
        leaf_mesh: meshes.add(shapes::leaf_blades(detail.leaf_blades)),
        branches,
        leaves,
    });
}

/// Rebuild the base meshes when the detail changes.
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
    let _ = meshes.insert(
        &forest.branch_mesh,
        shapes::branch_grid(detail.tube_sides, detail.tube_spans),
    );
    let _ = meshes.insert(&forest.leaf_mesh, shapes::leaf_blades(detail.leaf_blades));
    if let Some(mut material) = branch_materials.get_mut(&forest.branches) {
        material.lod = branch_lod(&detail);
    }
    if let Some(mut material) = leaf_materials.get_mut(&forest.leaves) {
        material.lod = leaf_lod(&detail);
    }
}

impl ForestMaterials {
    /// Hand a snapshot's instances, plants and species to the materials.
    pub(super) fn store(&self, snapshot: &Snapshot, stores: &mut ForestStores) {
        let ForestStores {
            branch_materials,
            leaf_materials,
            buffers,
        } = stores;
        let species: Vec<SpeciesRecord> =
            snapshot.species.iter().map(SpeciesRecord::from).collect();
        let species = buffers.add(storage(&species));
        if let Some(mut material) = branch_materials.get_mut(&self.branches) {
            material.branches = buffers.add(storage(&snapshot.branches));
            material.plants = buffers.add(storage(&snapshot.plant_bounds));
            material.species = species.clone();
        }
        if let Some(mut material) = leaf_materials.get_mut(&self.leaves) {
            material.leaves = buffers.add(storage(&snapshot.leaves));
            material.species = species;
        }
    }
}

fn branch_lod(detail: &ForestDetail) -> UVec4 {
    UVec4::new(detail.tube_sides.max(3), detail.tube_spans.max(1), 0, 0)
}

fn leaf_lod(detail: &ForestDetail) -> UVec4 {
    UVec4::new(detail.leaf_blades.max(1), 0, 0, 0)
}

/// A storage buffer of `records`; never empty, which a binding may not be.
fn storage<T: bytemuck::Pod>(records: &[T]) -> ShaderBuffer {
    let mut bytes = bytemuck::cast_slice(records).to_vec();
    if bytes.is_empty() {
        bytes.resize(std::mem::size_of::<T>().max(16), 0);
    }
    ShaderBuffer::new(&bytes, RenderAssetUsages::RENDER_WORLD)
}
