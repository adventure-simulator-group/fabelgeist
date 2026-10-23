//! A linear bounding volume hierarchy (BVH), built on the GPU.
//!
//! Morton-code order, Karras hierarchy, atomic bottom-up refit -- the whole
//! build records into a [`KernelBatch`] and never reads anything back, so a
//! deforming mesh can be rebuilt inside the frame that deforms it.
//!
//! Morton order splits on a fixed grid. The surface-area heuristic in
//! [`crate::cpu`] instead chooses splits from the primitives' bounds. This
//! build keeps the data and the work on the device.
//!
//! When only the vertices move and the topology does not -- a posed body, a
//! simulated cloth -- [`RecordedBvh::record_refit`] keeps the existing Morton
//! ordering and skips straight to the refit. That ordering slowly stops matching
//! the geometry as the mesh deforms, so a full [`GpuBvh::record_build`] is still
//! worth doing periodically.

use crate::morton;
use fabelgeist_compute::prelude::*;
use fabelgeist_gpu::prelude::*;

pub use layout::{BvhBoundsError, BvhLayout, BvhLayoutError, BvhNodeCount, BvhPrimitiveCount};
pub use recorded::{EmptyBvh, RecordedBvh};
pub use shaders::{TraversalConfig, traversal_source};

mod layout;
mod recorded;
mod storage;
pub mod shaders;

/// The compiled build passes, shared by every hierarchy on a device.
#[derive(Clone, Debug)]
pub struct BvhKernels {
    bounds: std::sync::Arc<Kernel>,
    codes: std::sync::Arc<Kernel>,
    hierarchy: std::sync::Arc<Kernel>,
    clear_bounds: std::sync::Arc<Kernel>,
    refit: std::sync::Arc<Kernel>,
    gather_bounds: std::sync::Arc<Kernel>,
    sort: RadixSort,
}

impl BvhKernels {
    pub fn new(context: &WgpuContext) -> Result<Self> {
        Self::with_cache(context, &KernelCache::new())
    }

    pub fn with_cache(context: &WgpuContext, cache: &KernelCache) -> Result<Self> {
        Ok(Self {
            bounds: cache.get(context, &shaders::bounds_source())?,
            codes: cache.get(context, &shaders::codes_source())?,
            hierarchy: cache.get(context, &shaders::hierarchy_source())?,
            clear_bounds: cache.get(context, &shaders::clear_bounds_source())?,
            refit: cache.get(context, &shaders::refit_source())?,
            gather_bounds: cache.get(context, &shaders::gather_bounds_source())?,
            sort: RadixSort::with_cache(context, cache)?,
        })
    }
}

/// Reusable build storage for a checked primitive capacity.
///
/// The caller owns `primitive_bounds` -- two `vec4<f32>` per primitive, lower
/// then upper -- because it is usually written by the caller's own kernel
/// (triangle bounds from deformed vertices, particle bounds from positions).
/// Recording a build returns a [`RecordedBvh`] for its live layout.
pub struct GpuBvh {
    kernels: BvhKernels,

    /// Two `vec4<f32>` per node. See [`shaders`] for the layout.
    nodes: Buffer,
    /// Right child of each internal node.
    right_children: Buffer,
    /// Primitives in Morton order.
    indices: Buffer,
    /// Parent of each node. The root's entry is never read.
    parents: Buffer,

    codes: Buffer,
    scene_bounds: Buffer,
    node_bounds: Buffer,
    sort_scratch: SortScratch,

    capacity_layout: BvhLayout,
}

impl GpuBvh {
    /// Record a full rebuild: scene bounds, Morton codes, sort, hierarchy,
    /// refit.
    ///
    /// `primitive_bounds` holds two `vec4<f32>` per primitive, lower then
    /// upper. Its logical and native extents must contain that prefix. This
    /// checks byte availability, not the bounds' geometric validity.
    /// Nothing is read back, so this can sit in the middle of a frame's
    /// batch between the kernel that wrote the bounds and the one that
    /// queries them.
    pub fn record_build(
        &mut self,
        batch: &mut KernelBatch,
        primitive_bounds: &Buffer,
        count: BvhPrimitiveCount,
    ) -> Result<RecordedBvh> {
        if count > self.capacity() {
            return Err(anyhow::anyhow!(
                "GpuBvh: built for {} primitives, asked for {count}",
                self.capacity()
            ));
        }
        let layout = BvhLayout::try_from(count)?;
        count.check_bounds(primitive_bounds)?;
        let recorded = RecordedBvh::new(self, layout);
        if count.is_empty() {
            return Ok(recorded);
        }

        let params = count.parameters();

        // Scene bounds start at the empty box, not at zero, so the clear
        // kernel writes the sentinels rather than `clear_buffer` zeroing. One
        // "node" of six words is exactly the scene-bounds layout, so the same
        // kernel does this job with a count of one.
        let mut clear_scene = BvhPrimitiveCount::from(1).parameters();
        clear_scene.insert("node_bounds", self.scene_bounds.clone());
        batch.dispatch(&self.kernels.clear_bounds, &clear_scene, WorkgroupGrid::from([1, 1, 1]))?;

        let mut bounds = params.clone();
        bounds.insert("primitive_bounds", primitive_bounds.clone());
        bounds.insert("scene_bounds", self.scene_bounds.clone());
        batch.dispatch_items(&self.kernels.bounds, &bounds, count.into())?;

        let mut codes = params.clone();
        codes.insert("primitive_bounds", primitive_bounds.clone());
        codes.insert("scene_bounds", self.scene_bounds.clone());
        codes.insert("codes", self.codes.clone());
        codes.insert("indices", self.indices.clone());
        batch.dispatch_items(&self.kernels.codes, &codes, count.into())?;

        self.kernels.sort.record(
            batch,
            &self.codes,
            &self.indices,
            &mut self.sort_scratch,
            count.into(),
            morton::sort_key_width(),
        )?;

        let mut hierarchy = params.clone();
        hierarchy.insert("codes", self.codes.clone());
        hierarchy.insert("nodes", self.nodes.clone());
        hierarchy.insert("parents", self.parents.clone());
        hierarchy.insert("right_children", self.right_children.clone());
        batch.dispatch_items(&self.kernels.hierarchy, &hierarchy, count.internal_nodes().into())?;

        recorded.record_refit(batch, primitive_bounds)?;
        Ok(recorded)
    }

    /// Build on a batch of its own and submit. For a one-off; inside a frame,
    /// record into the frame's batch.
    pub fn build(
        &mut self,
        context: &WgpuContext,
        primitive_bounds: &Buffer,
        count: BvhPrimitiveCount,
    ) -> Result<RecordedBvh> {
        let mut batch = KernelBatch::labelled(context, "GpuBvh build");
        let recorded = self.record_build(&mut batch, primitive_bounds, count)?;
        batch.submit()?;
        Ok(recorded)
    }
}

/// A node read back to the host, for tests and debugging.
#[derive(Clone, Copy, Debug)]
pub struct HostNode {
    pub bounds: crate::Aabb,
    /// Left child for an internal node; slot in `indices` for a leaf.
    pub left_or_first: u32,
    /// Zero for an internal node, one for a leaf.
    pub count: u32,
}

impl HostNode {
    pub fn is_leaf(&self) -> bool {
        self.count != 0
    }
}

/// Pack host-side boxes into the `primitive_bounds` layout: two `vec4<f32>`
/// per primitive, lower then upper.
pub fn pack_bounds(bounds: &[crate::Aabb]) -> Vec<f32> {
    let mut packed = Vec::with_capacity(bounds.len() * 8);
    for box_ in bounds {
        packed.extend_from_slice(&[box_.min.x, box_.min.y, box_.min.z, 0.0]);
        packed.extend_from_slice(&[box_.max.x, box_.max.y, box_.max.z, 0.0]);
    }
    packed
}

#[cfg(test)]
mod tests;
