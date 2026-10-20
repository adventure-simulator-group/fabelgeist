//! Ordinary Full materials retain the exact grown source used by each mesh.

use std::sync::Arc;

use bevy::{camera::primitives::Aabb, prelude::*, render::storage::ShaderBuffer};
use fabelgeist_botany::identity::PlantId;

use super::ForestPart;
use crate::prepared::{RecordSpan, TreeBank, VariantIndex};

pub use materials::{GrownLeafMaterial, GrownWoodMaterial};

mod assets;
mod materials;
mod source;

/// Registers ordinary material pipelines without an instanced forest feed.
pub struct GrownFullMaterialPlugin;

/// Source role and render handle remain bound to the admitted bank owner.
pub struct GrownSourceBuffer {
    role: GrownBufferRole,
    handle: Handle<ShaderBuffer>,
}

/// A complete ownership record for one fixed Full geometry, not a scene cache.
#[derive(Resource)]
pub struct GrownFullAssets {
    bank: Arc<TreeBank>,
    source: PlantId,
    variant: VariantIndex,
    encoding: GrownVertexEncoding,
    parts: Vec<GrownPartAsset>,
    buffers: Vec<GrownSourceBuffer>,
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

/// Only vertex evaluation changes; all surface and pass semantics are shared.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum GrownVertexEncoding {
    Address,
    Expanded,
}

/// Immutable source storage roles, distinct from mesh part roles.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GrownBufferRole {
    Wood,
    Leaves,
    Species,
    Plants,
}

impl Plugin for GrownFullMaterialPlugin {
    fn build(&self, app: &mut App) {
        super::register_forest_libraries(app);
        bevy::asset::embedded_asset!(app, "full/shaders/material.wgsl");
        bevy::shader::load_shader_library!(app, "full/shaders/vertices.wgsl");
        app.add_plugins((
            bevy::pbr::MaterialPlugin::<GrownWoodMaterial>::default(),
            bevy::pbr::MaterialPlugin::<GrownLeafMaterial>::default(),
        ));
    }
}

impl GrownFullAssets {
    pub fn bank(&self) -> &Arc<TreeBank> {
        &self.bank
    }
    pub fn source(&self) -> PlantId {
        self.source
    }
    pub fn variant(&self) -> VariantIndex {
        self.variant
    }
    pub fn encoding(&self) -> GrownVertexEncoding {
        self.encoding
    }
    pub fn parts(&self) -> impl ExactSizeIterator<Item = &GrownPartAsset> {
        self.parts.iter()
    }
    pub fn buffers(&self) -> impl ExactSizeIterator<Item = &GrownSourceBuffer> {
        self.buffers.iter()
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
