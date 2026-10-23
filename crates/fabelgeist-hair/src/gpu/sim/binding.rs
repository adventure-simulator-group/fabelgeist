//! Paired native binding arrays, derived from the actual drawn and guide sources.

use crate::gpu::buffers::storage;
use crate::gpu::error::Result;
use crate::gpu::grow::params;
use crate::gpu::GpuLevel;
use crate::parameters::StrandCount;
use fabelgeist_compute::{Kernel, KernelBatch};
use fabelgeist_gpu::prelude::{Buffer, BufferByteLength, WgpuContext};
use std::sync::Arc;

/// Both populations are checked as prefixes of their own source before recording.
pub(super) struct BindingRequest<'a> {
    pub drawn: &'a Arc<GpuLevel>,
    pub guides: &'a Arc<GpuLevel>,
    pub drawn_count: StrandCount,
    pub guide_count: StrandCount,
}

/// One four-guide index vector and weight vector per drawn strand.
pub(super) struct BindingBuffers {
    pub ids: Buffer,
    pub weights: Buffer,
}

impl BindingBuffers {
    pub fn record(
        context: &WgpuContext,
        batch: &mut KernelBatch,
        kernel: &Kernel,
        request: BindingRequest<'_>,
    ) -> Result<Self> {
        request.drawn.ensure_device(context)?;
        request.guides.ensure_device(context)?;
        let drawn = request.drawn.layout().prefix(request.drawn_count)?;
        let guides = request.guides.layout().prefix(request.guide_count)?;
        // Both arrays contain one native four-component record per target strand.
        let bytes = BufferByteLength::from(
            u64::from(drawn.native_strands()) * size_of::<[u32; 4]>() as u64,
        );
        let ids = storage(context, bytes, "hair sim binding ids".into())?;
        let weights = storage(context, bytes, "hair sim binding weights".into())?;
        batch.dispatch_folded(
            kernel,
            &params! {
                "level" => request.drawn.positions().clone(),
                "guides" => request.guides.positions().clone(),
                "ids" => ids.clone(),
                "weights" => weights.clone(),
                "strands" => drawn.native_strands(),
                "points" => drawn.native_points(),
                "count" => guides.native_strands(),
                "guide_points" => guides.native_points(),
                "shared_prefix" => u32::from(Arc::ptr_eq(request.drawn, request.guides)),
                "pad0" => 0u32,
                "pad1" => 0u32,
                "pad2" => 0u32,
            },
            drawn.strand_invocations(),
        )?;
        Ok(Self { ids, weights })
    }
}
