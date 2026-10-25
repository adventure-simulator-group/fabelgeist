//! Rod storage follows the shader's two-word frames and fourteen-word factors.

use crate::gpu::StrandLayout;
use crate::gpu::buffers::storage;
use crate::gpu::error::{HairGpuError, Result};
use fabelgeist_gpu::prelude::{Buffer, BufferElementCount, WgpuContext};

/// Arrays allocated together for the same admitted simulation population.
pub(super) struct RodBuffers {
    pub rest: Buffer,
    pub shape: Buffer,
    pub frames: Buffer,
    pub factors: Buffer,
    pub position: Buffer,
    pub previous: Buffer,
}

impl RodBuffers {
    pub fn new(context: &WgpuContext, layout: StrandLayout) -> Result<Self> {
        // sim_step addresses fourteen vec4 factors per point. This also
        // bounds the two-vec4 shape/frame addresses before allocation.
        BufferElementCount::from(layout.native_samples())
            .checked_mul(14)
            .ok_or(HairGpuError::SimulationFactors { layout })?;
        let bytes = u64::from(layout.positions_bytes());
        Ok(Self {
            rest: storage(context, layout.positions_bytes(), "hair sim rest".into())?,
            shape: storage(context, (bytes * 2).into(), "hair sim rod rest".into())?,
            frames: storage(context, (bytes * 2).into(), "hair sim frames".into())?,
            factors: storage(context, (bytes * 14).into(), "hair sim factors".into())?,
            position: storage(context, layout.positions_bytes(), "hair sim positions".into())?,
            previous: storage(context, layout.positions_bytes(), "hair sim previous".into())?,
        })
    }
}
