//! Bounded grown mesh authoring from admitted grown records, before presentation.
//!
//! The author owns a separate compute device. Readback may block that device;
//! it never borrows a continuously submitting renderer. This is not a complete
//! tactical cache, a resumable ecosystem, or a background scheduler.

use std::{num::NonZeroU32, sync::Arc};

use anyhow::Result;
use bevy::mesh::Mesh;
use fabelgeist_botany::identity::PlantId;
use fabelgeist_gpu::prelude::BufferByteLength;

use super::{RecordSpan, TreeBank, VariantIndex};
use crate::render::{ForestPart, Level};

mod geometry;
mod gpu;
mod mesh;
mod plan;
mod shader;

/// An explicit allocation limit, independent of source botanical identity.
#[derive(Clone, Copy, Debug)]
pub struct GrownPartVertexLimit(NonZeroU32);

/// Bounds final mesh payload, separately from transient GPU/readback memory.
#[derive(Clone, Copy, Debug)]
pub struct GrownExpansionBudget {
    pub part_vertices: GrownPartVertexLimit,
    pub total_mesh_bytes: BufferByteLength,
}

/// The only construction path owns a dedicated, non-presenting compute device.
pub struct GrownAuthoringContext {
    gpu: gpu::NativeExpansion,
}

/// Real positions/normals/UVs plus the original integer bank vertex address.
/// The retained admitted bank binds the source records used by surface shaders.
pub struct GrownGeometry {
    bank: Arc<TreeBank>,
    variant: VariantIndex,
    source: PlantId,
    level: Level,
    parts: Vec<GrownGeometryPart>,
    mesh_bytes: BufferByteLength,
}

/// An allocation partition of a source record span, not a botanical group.
pub struct GrownGeometryPart {
    role: ForestPart,
    records: RecordSpan,
    mesh: Mesh,
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

impl GrownAuthoringContext {
    pub async fn new() -> Result<Self> {
        Ok(Self {
            gpu: gpu::NativeExpansion::new().await?,
        })
    }

    pub fn adapter_info(&self) -> wgpu::AdapterInfo {
        self.gpu.adapter_info()
    }
}

impl TreeBank {
    /// Expand the exact selected grown model; never generate replacement trees.
    pub async fn expand_grown(
        self: &Arc<Self>,
        author: &mut GrownAuthoringContext,
        source: PlantId,
        level: Level,
        budget: GrownExpansionBudget,
    ) -> Result<GrownGeometry> {
        let plan = plan::ExpansionPlan::new(self, source, level, budget, &author.gpu)?;
        let mut parts = Vec::with_capacity(plan.parts.len());
        for part in &plan.parts {
            let vertices = author.gpu.expand(self.library(), part).await?;
            let mesh = mesh::construct(self.library(), part, vertices)?;
            parts.push(GrownGeometryPart {
                role: part.geometry.role(),
                records: part.records,
                mesh,
            });
        }
        Ok(GrownGeometry {
            bank: self.clone(),
            variant: plan.variant,
            source,
            level: plan.level,
            parts,
            mesh_bytes: plan.mesh_bytes,
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

    pub fn mesh_bytes(&self) -> BufferByteLength {
        self.mesh_bytes
    }

    /// Move real meshes into a renderer without cloning their CPU channels.
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
