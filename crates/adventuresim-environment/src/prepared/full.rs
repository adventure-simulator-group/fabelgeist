//! Bounded Full mesh authoring from admitted grown records, before presentation.
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
use crate::render::ForestPart;

mod geometry;
mod gpu;
mod mesh;
mod plan;
mod shader;

/// An explicit allocation limit, independent of source botanical identity.
#[derive(Clone, Copy, Debug)]
pub struct FullPartVertexLimit(NonZeroU32);

/// Bounds final mesh payload, separately from transient GPU/readback memory.
#[derive(Clone, Copy, Debug)]
pub struct FullExpansionBudget {
    pub part_vertices: FullPartVertexLimit,
    pub total_mesh_bytes: BufferByteLength,
}

/// The only construction path owns a dedicated, non-presenting compute device.
pub struct FullAuthoringContext {
    gpu: gpu::NativeExpansion,
}

/// Real positions/normals/UVs plus the original integer bank vertex address.
/// The retained admitted bank binds the source records used by surface shaders.
pub struct GrownFullGeometry {
    bank: Arc<TreeBank>,
    variant: VariantIndex,
    source: PlantId,
    parts: Vec<GrownFullPart>,
    mesh_bytes: BufferByteLength,
}

/// An allocation partition of a source record span, not a botanical group.
pub struct GrownFullPart {
    role: ForestPart,
    records: RecordSpan,
    mesh: Mesh,
}

impl FullPartVertexLimit {
    /// Native vertex-count boundary; at least one complete record must fit.
    pub fn from_vertices(vertices: u32) -> Result<Self> {
        let vertices = NonZeroU32::new(vertices)
            .ok_or_else(|| anyhow::anyhow!("Full part vertex limit is zero"))?;
        Ok(Self(vertices))
    }

    pub fn vertices(self) -> u32 {
        self.0.get()
    }
}

impl FullAuthoringContext {
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
    pub async fn expand_full(
        self: &Arc<Self>,
        author: &mut FullAuthoringContext,
        source: PlantId,
        budget: FullExpansionBudget,
    ) -> Result<GrownFullGeometry> {
        let plan = plan::ExpansionPlan::new(self, source, budget, &author.gpu)?;
        let mut parts = Vec::with_capacity(plan.parts.len());
        for part in &plan.parts {
            let vertices = author.gpu.expand(self.library(), part).await?;
            let mesh = mesh::construct(self.library(), part, vertices)?;
            parts.push(GrownFullPart {
                role: part.geometry.role(),
                records: part.records,
                mesh,
            });
        }
        Ok(GrownFullGeometry {
            bank: self.clone(),
            variant: plan.variant,
            source,
            parts,
            mesh_bytes: plan.mesh_bytes,
        })
    }
}

impl GrownFullGeometry {
    pub fn bank(&self) -> &Arc<TreeBank> {
        &self.bank
    }

    pub fn variant(&self) -> VariantIndex {
        self.variant
    }

    pub fn source(&self) -> PlantId {
        self.source
    }

    pub fn mesh_bytes(&self) -> BufferByteLength {
        self.mesh_bytes
    }

    pub fn parts(&self) -> impl ExactSizeIterator<Item = &GrownFullPart> {
        self.parts.iter()
    }
}

impl GrownFullPart {
    pub fn role(&self) -> ForestPart {
        self.role
    }

    pub fn records(&self) -> RecordSpan {
        self.records
    }

    pub fn mesh(&self) -> &Mesh {
        &self.mesh
    }
}
