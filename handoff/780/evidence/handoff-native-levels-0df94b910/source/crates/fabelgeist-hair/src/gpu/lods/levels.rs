//! Record each native strand level from one source and occlusion bake.

use super::{GpuGroom, GpuGroomer, GpuLevel, Grid, LodSettings, params, storage};
use crate::gpu::StrandLayout;
use crate::gpu::error::Result;
use crate::parameters::PointsPerStrand;
use fabelgeist_compute::KernelBatch;
use fabelgeist_gpu::prelude::Buffer;
use std::sync::Arc;

/// Buffers and native uniforms from the same level-preparation batch.
pub(super) struct LevelBake<'a> {
    pub groom: &'a GpuGroom,
    pub grid: Grid,
    pub order: &'a Buffer,
    pub visibility: &'a Buffer,
    pub head: &'a Buffer,
    pub body: u32,
}

impl GpuGroomer {
    pub(super) fn record_levels(
        &self,
        batch: &mut KernelBatch,
        bake: LevelBake<'_>,
        settings: &LodSettings,
    ) -> Result<Vec<Arc<GpuLevel>>> {
        let groom = bake.groom;
        let mut counts = vec![groom.layout.points()];
        for &points in &settings.strand_points {
            if points >= PointsPerStrand::from_count(2)
                && points < counts[counts.len() - 1]
                && points <= PointsPerStrand::from_count(32)
            {
                counts.push(points);
            }
        }
        let mut levels = Vec::with_capacity(counts.len());
        for points in counts {
            let layout = StrandLayout::new(groom.layout.strands(), points)?;
            let positions = storage(
                &self.context,
                layout.positions_bytes(),
                "hair level positions".into(),
            )?;
            let infos = storage(&self.context, layout.info_bytes(), "hair level infos".into())?;
            let mut parameters = params! {
                "positions" => groom.positions.clone(),
                "strands" => groom.attributes.clone(),
                "order" => bake.order.clone(),
                "grid_values" => bake.visibility.clone(),
                "head_shape" => bake.head.clone(),
                "level_positions" => positions.clone(),
                "infos" => infos.clone(),
                "points" => groom.layout.native_points(),
                "level_points" => layout.native_points(),
                "body" => bake.body,
                "pad0" => 0u32,
                "clearance" => 0.0012f32,
                "pad1" => 0u32,
                "pad2" => 0u32,
                "pad3" => 0u32,
            };
            bake.grid.insert(&mut parameters, layout.native_strands());
            batch.dispatch_folded(&self.kernels.level, &parameters, layout.strand_invocations())?;
            levels.push(Arc::new(GpuLevel {
                layout,
                device: self.context.device.as_ref().clone(),
                positions,
                infos,
            }));
        }
        Ok(levels)
    }
}
