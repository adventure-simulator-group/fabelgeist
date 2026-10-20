//! Explicit native expansion on a dedicated non-presenting compute device.

use std::sync::Arc;

use anyhow::Result;
use fabelgeist_botany::identity::PlantId;

use super::{
    GrownGeometry, GrownGeometryPart, GrownMeshBudget, GrownVertexEncoding, gpu, mesh, plan,
};
use crate::{prepared::TreeBank, render::Level};

/// Readback may block this owned device, never a continuously submitting renderer.
pub struct GrownAuthoringContext {
    gpu: gpu::NativeExpansion,
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
        budget: GrownMeshBudget,
    ) -> Result<GrownGeometry> {
        let plan = plan::GeometryPlan::new(self, source, level, budget, |geometry| {
            author.gpu.record_capacity(
                geometry.role(),
                geometry.vertices_per_record()?,
                geometry.record_stride(),
                budget.part_vertices.vertices(),
            )
        })?;
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
            encoding: GrownVertexEncoding::Expanded,
        })
    }
}
