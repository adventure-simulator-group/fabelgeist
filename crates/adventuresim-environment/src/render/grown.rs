//! Ordinary grown materials retain the exact grown source used by each mesh.

use std::sync::Arc;

use bevy::{
    camera::primitives::Aabb, ecs::world::WorldId, prelude::*, render::storage::ShaderBuffer,
};
use fabelgeist_botany::identity::PlantId;

use super::{ForestPart, Level};
use crate::prepared::{GrownVertexEncoding, RecordSpan, TreeBank, VariantIndex};

pub use materials::{GrownLeafMaterial, GrownWoodMaterial};

mod assets;
mod materials;
mod source;

/// Registers ordinary material pipelines without an instanced forest feed.
pub struct GrownMaterialPlugin;

/// Source role and render handle remain bound to the admitted bank owner.
pub struct GrownSourceBuffer {
    role: GrownBufferRole,
    handle: Handle<ShaderBuffer>,
}

/// One immutable bank's source uploads in one presenting asset world.
/// Share the returned Arc across model, level and encoding installations.
pub struct GrownBankAssets {
    source: Arc<TreeBank>,
    world: WorldId,
    buffers: Vec<GrownSourceBuffer>,
}

/// A complete ownership record for one fixed grown geometry, not a scene cache.
#[derive(Resource)]
pub struct GrownAssets {
    bank_assets: Arc<GrownBankAssets>,
    source: PlantId,
    variant: VariantIndex,
    level: Level,
    encoding: GrownVertexEncoding,
    parts: Vec<GrownPartAsset>,
}

pub struct GrownPartAsset {
    role: ForestPart,
    records: RecordSpan,
    mesh: Handle<Mesh>,
    material: GrownMaterialHandle,
    bounds: Aabb,
}

#[derive(Clone, Debug, PartialEq)]
pub enum GrownMaterialHandle {
    Wood(Handle<GrownWoodMaterial>),
    Leaves(Handle<GrownLeafMaterial>),
}

/// Immutable source storage roles, distinct from mesh part roles.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GrownBufferRole {
    Wood,
    Leaves,
    Species,
    Plants,
}

impl GrownBufferRole {
    pub const ALL: [Self; 4] = [Self::Wood, Self::Leaves, Self::Species, Self::Plants];
}

impl Plugin for GrownMaterialPlugin {
    fn build(&self, app: &mut App) {
        super::register_forest_libraries(app);
        bevy::asset::embedded_asset!(app, "grown/shaders/material.wgsl");
        bevy::shader::load_shader_library!(app, "grown/shaders/vertices.wgsl");
        app.add_plugins((
            bevy::pbr::MaterialPlugin::<GrownWoodMaterial>::default(),
            bevy::pbr::MaterialPlugin::<GrownLeafMaterial>::default(),
        ));
    }
}

impl GrownBankAssets {
    pub fn bank(&self) -> &Arc<TreeBank> {
        &self.source
    }
}

impl GrownAssets {
    pub fn bank_assets(&self) -> &Arc<GrownBankAssets> {
        &self.bank_assets
    }

    pub fn bank(&self) -> &Arc<TreeBank> {
        self.bank_assets.bank()
    }
    pub fn source(&self) -> PlantId {
        self.source
    }
    pub fn variant(&self) -> VariantIndex {
        self.variant
    }
    pub fn level(&self) -> Level {
        self.level
    }
    pub fn encoding(&self) -> GrownVertexEncoding {
        self.encoding
    }
    pub fn parts(&self) -> impl ExactSizeIterator<Item = &GrownPartAsset> {
        self.parts.iter()
    }
    pub fn buffers(&self) -> impl ExactSizeIterator<Item = &GrownSourceBuffer> {
        self.bank_assets.buffers.iter()
    }
}

impl GrownSourceBuffer {
    pub fn role(&self) -> GrownBufferRole {
        self.role
    }
    pub fn handle(&self) -> &Handle<ShaderBuffer> {
        &self.handle
    }
}

impl GrownPartAsset {
    pub fn role(&self) -> ForestPart {
        self.role
    }
    pub fn records(&self) -> RecordSpan {
        self.records
    }
    pub fn mesh(&self) -> &Handle<Mesh> {
        &self.mesh
    }
    pub fn material(&self) -> &GrownMaterialHandle {
        &self.material
    }
    /// Source-conservative for address evaluation, actual mesh bounds otherwise.
    /// Attach this with `NoAutoAabb` so Bevy keeps the authored bounds, including
    /// when address meshes have deliberately empty position channels.
    pub fn bounds(&self) -> Aabb {
        self.bounds
    }
}
