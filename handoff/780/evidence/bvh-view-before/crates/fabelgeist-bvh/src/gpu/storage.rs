//! Allocation and native readback of a hierarchy's owned storage.

use super::{BvhKernels, BvhLayout, BvhNodeCount, BvhPrimitiveCount, GpuBvh, HostNode};
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
            recorded_layout: BvhLayout::EMPTY,
        })
    }

    pub fn capacity(&self) -> BvhPrimitiveCount {
        self.capacity_layout.primitive_count()
    }

    /// Primitive population as of the last recorded build.
    pub fn count(&self) -> BvhPrimitiveCount {
        self.recorded_layout.primitive_count()
    }

    pub fn node_count(&self) -> BvhNodeCount {
        self.recorded_layout.node_count()
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

    /// Read the hierarchy back as host-side nodes. For tests and debugging --
    /// it stalls on the device.
    pub async fn read_nodes(&self, context: &WgpuContext) -> Result<Vec<HostNode>> {
        let raw: Vec<f32> = self.nodes.read(context).await?;
        let count = usize::from(self.node_count());
        Ok((0..count)
            .map(|i| {
                let lower = &raw[i * 8..i * 8 + 4];
                let upper = &raw[i * 8 + 4..i * 8 + 8];
                HostNode {
                    bounds: crate::Aabb::new(
                        fabelgeist_math::Vec3::new(lower[0], lower[1], lower[2]),
                        fabelgeist_math::Vec3::new(upper[0], upper[1], upper[2]),
                    ),
                    left_or_first: lower[3].to_bits(),
                    count: upper[3].to_bits(),
                }
            })
            .collect())
    }

    pub async fn read_indices(&self, context: &WgpuContext) -> Result<Vec<u32>> {
        let all: Vec<u32> = self.indices.read(context).await?;
        Ok(all[..usize::from(self.count())].to_vec())
    }

    pub async fn read_right_children(&self, context: &WgpuContext) -> Result<Vec<u32>> {
        let all: Vec<u32> = self.right_children.read(context).await?;
        Ok(all[..usize::from(self.count().internal_nodes())].to_vec())
    }
}
