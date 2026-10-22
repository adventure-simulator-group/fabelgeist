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
use super::records::{Bytes, ForestBuffers, PendingWrites, Records};
use super::impostors::ImpostorSettings;
use super::{ForestDetail, LEAF_COVERAGE_PIXELS, LeafCoverage, shapes};

pub(super) const BRANCH_SHADER: &str =
    "embedded://adventuresim_environment/render/shaders/branch.wgsl";
pub(super) const LEAF_SHADER: &str = "embedded://adventuresim_environment/render/shaders/leaf.wgsl";
pub(super) const CULL_SHADER: &str =
    "embedded://adventuresim_environment/render/shaders/forest_cull.wgsl";

/// Branch segments, built as tubes.
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct BranchMaterial {
    /// x: sides around a tube; y: spans along it; z: below how many pixels
    /// of radius a plant with an impostor is drawn as it instead, as f32 bits.
    #[uniform(0)]
    lod: UVec4,
    #[storage(1, read_only, visibility(vertex, fragment))]
    branches: Handle<ShaderBuffer>,
    #[storage(2, read_only, visibility(vertex, fragment))]
    species: Handle<ShaderBuffer>,
    #[storage(3, read_only, visibility(vertex, fragment))]
    plants: Handle<ShaderBuffer>,
    /// Each plant's impostor: its centre and baked radius, zero without one.
    #[storage(5, read_only, visibility(vertex))]
    impostors: Handle<ShaderBuffer>,
}

/// Leaf clusters, built as blades.
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct LeafMaterial {
    /// x: blades a cluster; y: 1 when sub-pixel blades keep their coverage
    /// ([`LeafCoverage`]); z: the smallest blade then drawn, pixels, as f32
    /// bits; w: below how many pixels of radius a plant with an impostor is
    /// drawn as it instead, as f32 bits.
    #[uniform(0)]
    lod: UVec4,
    #[storage(1, read_only, visibility(vertex, fragment))]
    leaves: Handle<ShaderBuffer>,
    #[storage(2, read_only, visibility(vertex, fragment))]
    species: Handle<ShaderBuffer>,
    #[storage(5, read_only, visibility(vertex))]
    impostors: Handle<ShaderBuffer>,
}

impl InstancedMaterial for BranchMaterial {
    fn cull_shader() -> ShaderRef {
        CULL_SHADER.into()
    }
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
    fn cull_shader() -> ShaderRef {
        CULL_SHADER.into()
    }
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
    coverage: Res<LeafCoverage>,
    impostors: Res<ImpostorSettings>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut branch_materials: ResMut<Assets<BranchMaterial>>,
    mut leaf_materials: ResMut<Assets<LeafMaterial>>,
    mut buffers: ResMut<Assets<ShaderBuffer>>,
    probe: Option<Res<super::benchmark::ForestBenchmark>>,
) {
    let stored = ForestBuffers::new(&mut buffers);
    let levels = Level::ALL.map(|level| {
        let shape = level.shape(&detail);
        let branch_grid = shapes::branch_grid(shape.sides, shape.spans);
        let leaf_grid = shapes::leaf_blades(shape.blades);
        let branch_count = branch_grid.count_vertices();
        let leaf_count = leaf_grid.count_vertices();
        let branch_mesh = meshes.add(branch_grid);
        let leaf_mesh = meshes.add(leaf_grid);
        if let Some(probe) = &probe {
            probe.expect_mesh(
                branch_mesh.id(),
                branch_count,
                (shape.sides * shape.spans * 6) as usize,
            );
            probe.expect_mesh(leaf_mesh.id(), leaf_count, (shape.blades * 6) as usize);
        }
        LevelAssets {
            branch_mesh,
            leaf_mesh,
            branches: branch_materials.add(BranchMaterial {
                lod: branch_lod(&shape, &impostors),
                branches: stored.branches(),
                species: stored.species(),
                plants: stored.plants(),
                impostors: stored.impostors(),
            }),
            leaves: leaf_materials.add(LeafMaterial {
                lod: leaf_lod(&shape, *coverage, &impostors),
                leaves: stored.leaves(),
                species: stored.species(),
                impostors: stored.impostors(),
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
    coverage: Res<LeafCoverage>,
    impostors: Res<ImpostorSettings>,
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
        material.lod = branch_lod(&shape, &impostors);
    }
    if let Some(mut material) = leaf_materials.get_mut(&full.leaves) {
        material.lod = leaf_lod(&shape, *coverage, &impostors);
    }
}

/// Switch every level's leaves when [`LeafCoverage`] changes.
pub(super) fn follow_leaf_coverage(
    coverage: Res<LeafCoverage>,
    forest: Option<Res<ForestMaterials>>,
    mut leaf_materials: ResMut<Assets<LeafMaterial>>,
) {
    let Some(forest) = forest else {
        return;
    };
    if !coverage.is_changed() || coverage.is_added() {
        return;
    }
    for level in &forest.levels {
        if let Some(mut material) = leaf_materials.get_mut(&level.leaves) {
            material.lod.y = u32::from(coverage.preserve);
        }
    }
}

/// Switch every level's plants to their impostors at the size asked for.
pub(super) fn follow_impostor_switch(
    impostors: Res<ImpostorSettings>,
    forest: Option<Res<ForestMaterials>>,
    mut branch_materials: ResMut<Assets<BranchMaterial>>,
    mut leaf_materials: ResMut<Assets<LeafMaterial>>,
) {
    let Some(forest) = forest else {
        return;
    };
    if !impostors.is_changed() || impostors.is_added() {
        return;
    }
    let bits = impostors.switch_pixels().to_bits();
    for level in &forest.levels {
        if let Some(mut material) = branch_materials.get_mut(&level.branches)
            && material.lod.z != bits
        {
            material.lod.z = bits;
        }
        if let Some(mut material) = leaf_materials.get_mut(&level.leaves)
            && material.lod.w != bits
        {
            material.lod.w = bits;
        }
    }
}

impl ForestMaterials {
    pub(super) fn level(&self, level: Level) -> &LevelAssets {
        &self.levels[level.index()]
    }

    /// Write a snapshot's records into the storage buffers, binding any
    /// buffer that had to be replaced.
    pub(super) fn store(&mut self, records: Records, stores: &mut ForestStores) {
        if !self
            .buffers
            .write(records, &mut stores.buffers, &mut stores.writes)
        {
            return;
        }
        self.rebind(stores);
    }

    /// Write each plant's impostor, binding the buffer if it had to be
    /// replaced.
    pub(super) fn store_impostors(&mut self, bytes: Bytes, stores: &mut ForestStores) {
        if self
            .buffers
            .write_impostors(bytes, &mut stores.buffers, &mut stores.writes)
        {
            self.rebind(stores);
        }
    }

    fn rebind(&self, stores: &mut ForestStores) {
        let ForestStores {
            branch_materials,
            leaf_materials,
            ..
        } = stores;
        let bound = &self.buffers;
        for level in &self.levels {
            if let Some(mut material) = branch_materials.get_mut(&level.branches) {
                material.branches = bound.branches();
                material.plants = bound.plants();
                material.species = bound.species();
                material.impostors = bound.impostors();
            }
            if let Some(mut material) = leaf_materials.get_mut(&level.leaves) {
                material.leaves = bound.leaves();
                material.species = bound.species();
                material.impostors = bound.impostors();
            }
        }
    }
}

fn branch_lod(shape: &Shape, impostors: &ImpostorSettings) -> UVec4 {
    UVec4::new(
        shape.sides,
        shape.spans,
        impostors.switch_pixels().to_bits(),
        0,
    )
}

fn leaf_lod(shape: &Shape, coverage: LeafCoverage, impostors: &ImpostorSettings) -> UVec4 {
    UVec4::new(
        shape.blades,
        u32::from(coverage.preserve),
        LEAF_COVERAGE_PIXELS.to_bits(),
        impostors.switch_pixels().to_bits(),
    )
}
