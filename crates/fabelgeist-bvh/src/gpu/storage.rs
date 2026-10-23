//! Allocation and growth of reusable hierarchy build storage.

use super::{BvhKernels, BvhLayout, BvhPrimitiveCount, GpuBvh};
use fabelgeist_compute::SortScratch;
use fabelgeist_gpu::prelude::*;

impl GpuBvh {
    /// Allocate a checked native layout, keeping one record for empty requests.
    pub fn new(context: &WgpuContext, kernels: BvhKernels, capacity: BvhLayout) -> Result<Self> {
        let capacity = capacity.with_sentinel();
        let primitives = capacity.primitive_count();
        let storage = BufferDefinition::storage();
        Ok(Self {
            kernels,
            nodes: Buffer::new(context, capacity.node_bytes(),
                storage.clone().with_label("bvh nodes".into()))?,
            right_children: Buffer::new(context, primitives.word_bytes(),
                storage.clone().with_label("bvh right children".into()))?,
            indices: Buffer::new(context, primitives.word_bytes(),
                storage.clone().with_label("bvh indices".into()))?,
            parents: Buffer::new(context, capacity.parent_bytes(),
                storage.clone().with_label("bvh parents".into()))?,
            codes: Buffer::new(context, primitives.word_bytes(),
                storage.clone().with_label("bvh morton codes".into()))?,
            scene_bounds: Buffer::new(context, BvhLayout::scene_bounds_bytes(),
                storage.clone().with_label("bvh scene bounds".into()))?,
            node_bounds: Buffer::new(context, capacity.atomic_bounds_bytes(),
                storage.with_label("bvh node bounds".into()))?,
            sort_scratch: SortScratch::new(context, primitives.into())?,
            capacity_layout: capacity,
        })
    }

    pub fn capacity(&self) -> BvhPrimitiveCount {
        self.capacity_layout.primitive_count()
    }

    /// Replace all storage together if the checked capacity no longer fits.
    pub fn ensure_capacity(
        &mut self,
        context: &WgpuContext,
        capacity: BvhLayout,
    ) -> Result<BufferCapacityChange> {
        if capacity.primitive_count() <= self.capacity() {
            return Ok(BufferCapacityChange::Unchanged);
        }
        *self = Self::new(context, self.kernels.clone(), capacity)?;
        Ok(BufferCapacityChange::Grown)
    }
}
