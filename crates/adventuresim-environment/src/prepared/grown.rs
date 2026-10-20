//! Checked grown geometry from admitted records, with explicit vertex encoding.
//!
//! Address meshes need no authoring device. Optional native expansion owns a
//! separate compute device and never borrows the presenting renderer.

use std::{num::NonZeroU32, sync::Arc};

use anyhow::Result;
use bevy::mesh::Mesh;
use fabelgeist_botany::identity::PlantId;

use super::{RecordSpan, TreeBank, VariantIndex};
use crate::render::{ForestPart, Level};

#[cfg(all(feature = "grown-authoring", not(target_family = "wasm")))]
pub use native::GrownAuthoringContext;

mod geometry;
#[cfg(all(feature = "grown-authoring", not(target_family = "wasm")))]
mod gpu;
mod mesh;
#[cfg(all(feature = "grown-authoring", not(target_family = "wasm")))]
mod native;
mod plan;
#[cfg(all(feature = "grown-authoring", not(target_family = "wasm")))]
mod shader;

/// An explicit allocation limit, independent of source botanical identity.
#[derive(Clone, Copy, Debug)]
pub struct GrownPartVertexLimit(NonZeroU32);

/// Bounds final mesh payload, separately from transient GPU/readback memory.
#[derive(Clone, Copy, Debug)]
pub struct GrownMeshBudget {
    pub part_vertices: GrownPartVertexLimit,
    pub total_mesh_bytes: GrownMeshBytes,
}

/// Logical mesh channel/index payload, not allocation, RAM or GPU memory.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GrownMeshBytes(u64);

/// Checked meshes plus their actual channel encoding and immutable source.
/// Address evaluation uses placeholder attributes and original bank addresses.
pub struct GrownGeometry {
    bank: Arc<TreeBank>,
    variant: VariantIndex,
    source: PlantId,
    level: Level,
    parts: Vec<GrownGeometryPart>,
    mesh_bytes: GrownMeshBytes,
    encoding: GrownVertexEncoding,
}

/// An allocation partition of a source record span, not a botanical group.
pub struct GrownGeometryPart {
    role: ForestPart,
    records: RecordSpan,
    mesh: Mesh,
}

/// Channel evaluation is owned by construction, never chosen during installation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum GrownVertexEncoding {
    Address,
    Expanded,
}

impl GrownPartVertexLimit {
    /// Native vertex-count boundary; at least one complete record must fit.
    pub fn from_vertices(vertices: u32) -> Result<Self> {
        let vertices = NonZeroU32::new(vertices)
            .ok_or_else(|| anyhow::anyhow!("grown part vertex limit is zero"))?;
        Ok(Self(vertices))
    }

    pub fn vertices(self) -> u32 {
        self.0.get()
    }
}

impl GrownMeshBytes {
    /// Logical payload accounting boundary, in bytes of retained mesh channels.
    pub fn from_bytes(bytes: u64) -> Self {
        Self(bytes)
    }

    pub fn bytes(self) -> u64 {
        self.0
    }
}

impl TreeBank {
    /// Construct exact live address topology without a compute device/readback.
    pub fn address_grown(
        self: &Arc<Self>,
        source: PlantId,
        level: Level,
        budget: GrownMeshBudget,
    ) -> Result<GrownGeometry> {
        let plan = plan::GeometryPlan::new(self, source, level, budget, |geometry| {
            Ok(budget.part_vertices.vertices() / geometry.vertices_per_record()?)
        })?;
        let mut parts = Vec::with_capacity(plan.parts.len());
        for part in &plan.parts {
            parts.push(GrownGeometryPart {
                role: part.geometry.role(),
                records: part.records,
                mesh: mesh::address(self.library(), part)?,
            });
        }
        Ok(GrownGeometry {
            bank: self.clone(),
            variant: plan.variant,
            source,
            level: plan.level,
            parts,
            mesh_bytes: plan.mesh_bytes,
            encoding: GrownVertexEncoding::Address,
        })
    }
}

impl GrownGeometry {
    pub fn bank(&self) -> &Arc<TreeBank> {
        &self.bank
    }

    pub fn variant(&self) -> VariantIndex {
        self.variant
    }

    pub fn source(&self) -> PlantId {
        self.source
    }

    pub fn level(&self) -> Level {
        self.level
    }

    pub fn mesh_bytes(&self) -> GrownMeshBytes {
        self.mesh_bytes
    }

    pub fn encoding(&self) -> GrownVertexEncoding {
        self.encoding
    }

    /// Move meshes into a renderer without cloning their CPU channels.
    pub fn into_parts(self) -> impl ExactSizeIterator<Item = GrownGeometryPart> {
        self.parts.into_iter()
    }

    pub fn parts(&self) -> impl ExactSizeIterator<Item = &GrownGeometryPart> {
        self.parts.iter()
    }
}

impl GrownGeometryPart {
    pub fn role(&self) -> ForestPart {
        self.role
    }

    pub fn records(&self) -> RecordSpan {
        self.records
    }

    pub fn into_mesh(self) -> Mesh {
        self.mesh
    }

    pub fn mesh(&self) -> &Mesh {
        &self.mesh
    }
}
