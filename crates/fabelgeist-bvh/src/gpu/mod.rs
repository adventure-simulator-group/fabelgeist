//! A linear BVH built on the GPU.
//!
//! Morton-code order, Karras hierarchy, atomic bottom-up refit -- the whole
//! build records into a [`KernelBatch`] and never reads anything back, so a
//! deforming mesh can be rebuilt inside the frame that deforms it.
//!
//! The tree it produces is worse than the surface-area-heuristic one in
//! [`crate::cpu`]: Morton order splits on a fixed grid rather than on where
//! the primitives actually are. It is built in milliseconds on the device the
//! primitives already live on, which is the trade being made.
//!
//! When only the vertices move and the topology does not -- a posed body, a
//! simulated cloth -- [`GpuBvh::record_refit`] skips straight to the refit and
//! keeps the existing Morton ordering. That ordering slowly stops matching the
//! geometry as the mesh deforms, so a full [`GpuBvh::record_build`] is still
//! worth doing periodically.

pub mod shaders;

use fabelgeist_compute::prelude::*;
use fabelgeist_gpu::prelude::*;

pub use shaders::{TraversalConfig, traversal_source};

use crate::morton;

/// Bytes per node: two `vec4<f32>`.
const NODE_BYTES: u64 = 32;
/// Words per node in the atomic scratch: lower xyz then upper xyz.
const BOUNDS_WORDS: u64 = 6;

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

/// A hierarchy over `count` primitives, and every buffer the build needs.
///
/// The caller owns `primitive_bounds` -- two `vec4<f32>` per primitive, lower
/// then upper -- because it is usually written by the caller's own kernel
/// (triangle bounds from deformed vertices, particle bounds from positions).
/// Everything else lives here.
pub struct GpuBvh {
    kernels: BvhKernels,

    /// Two `vec4<f32>` per node. See [`shaders`] for the layout.
    pub nodes: Buffer,
    /// Right child of each internal node.
    pub right_children: Buffer,
    /// Primitives in Morton order.
    pub indices: Buffer,
    /// Parent of each node. The root's entry is never read.
    pub parents: Buffer,

    codes: Buffer,
    scene_bounds: Buffer,
    node_bounds: Buffer,
    sort_scratch: SortScratch,

    capacity: u32,
    count: u32,
}

impl GpuBvh {
    /// Allocate for up to `capacity` primitives.
    pub fn new(context: &WgpuContext, kernels: BvhKernels, capacity: u32) -> Result<Self> {
        let capacity = capacity.max(1);
        let nodes = 2 * capacity as u64 - 1;
        let storage = BufferDefinition::storage();

        Ok(Self {
            kernels,
            nodes: Buffer::new(
                context,
                (nodes * NODE_BYTES).into(),
                storage.clone().with_label(("bvh nodes").into()),
            )?,
            right_children: Buffer::new(
                context,
                ((capacity as u64).max(1) * 4).into(),
                storage.clone().with_label(("bvh right children").into()),
            )?,
            indices: Buffer::new(
                context,
                (capacity as u64 * 4).into(),
                storage.clone().with_label(("bvh indices").into()),
            )?,
            parents: Buffer::new(
                context,
                (nodes * 4).into(),
                storage.clone().with_label(("bvh parents").into()),
            )?,
            codes: Buffer::new(
                context,
                (capacity as u64 * 4).into(),
                storage.clone().with_label(("bvh morton codes").into()),
            )?,
            scene_bounds: Buffer::new(
                context,
                (BOUNDS_WORDS * 4).into(),
                storage.clone().with_label(("bvh scene bounds").into()),
            )?,
            node_bounds: Buffer::new(
                context,
                (nodes * BOUNDS_WORDS * 4).into(),
                storage.with_label(("bvh node bounds").into()),
            )?,
            sort_scratch: SortScratch::new(context, capacity.into())?,
            capacity,
            count: 0,
        })
    }

    pub fn capacity(&self) -> u32 {
        self.capacity
    }

    /// Primitive count as of the last recorded build.
    pub fn count(&self) -> u32 {
        self.count
    }

    /// Node count for the current primitive count.
    pub fn node_count(&self) -> u32 {
        if self.count == 0 {
            0
        } else {
            2 * self.count - 1
        }
    }

    /// Reallocate if `capacity` no longer fits.
    pub fn ensure_capacity(&mut self, context: &WgpuContext, capacity: u32) -> Result<bool> {
        if capacity <= self.capacity {
            return Ok(false);
        }
        *self = Self::new(context, self.kernels.clone(), capacity)?;
        Ok(true)
    }

    /// The buffers a traversal generated by [`traversal_source`] expects,
    /// under the default [`TraversalConfig`] names.
    pub fn parameters(&self) -> PassParameters {
        let mut parameters = PassParameters::new();
        self.bind(&mut parameters, &TraversalConfig::default());
        parameters
    }

    /// The same, under whatever names `config` gives.
    pub fn bind(&self, parameters: &mut PassParameters, config: &TraversalConfig) {
        parameters.insert(
            PassParameterName::from(config.nodes.clone()),
            (self.nodes.clone()).into(),
        );
        parameters.insert(
            PassParameterName::from(config.right_children.clone()),
            (self.right_children.clone()).into(),
        );
        parameters.insert(
            PassParameterName::from(config.indices.clone()),
            (self.indices.clone()).into(),
        );
    }

    fn params(&self, count: u32) -> PassParameters {
        let mut parameters = PassParameters::new();
        parameters.insert("count".into(), (count).into());
        parameters.insert("pad0".into(), (0u32).into());
        parameters.insert("pad1".into(), (0u32).into());
        parameters.insert("pad2".into(), (0u32).into());
        parameters
    }

    /// Record a full rebuild: scene bounds, Morton codes, sort, hierarchy,
    /// refit.
    ///
    /// `primitive_bounds` holds two `vec4<f32>` per primitive, lower then
    /// upper. Nothing is read back, so this can sit in the middle of a frame's
    /// batch between the kernel that wrote the bounds and the one that
    /// queries them.
    pub fn record_build(
        &mut self,
        batch: &mut KernelBatch,
        primitive_bounds: &Buffer,
        count: u32,
    ) -> Result<()> {
        if count > self.capacity {
            return Err(anyhow::anyhow!(
                "GpuBvh: built for {} primitives, asked for {count}",
                self.capacity
            ));
        }
        self.count = count;
        if count == 0 {
            return Ok(());
        }

        let params = self.params(count);

        // Scene bounds start at the empty box, not at zero, so the clear
        // kernel writes the sentinels rather than `clear_buffer` zeroing. One
        // "node" of six words is exactly the scene-bounds layout, so the same
        // kernel does this job with a count of one.
        let mut clear_scene = self.params(1);
        clear_scene.insert("node_bounds".into(), (self.scene_bounds.clone()).into());
        batch.dispatch(&self.kernels.clear_bounds, &clear_scene, ([1, 1, 1]).into())?;

        let mut bounds = params.clone();
        bounds.insert("primitive_bounds".into(), (primitive_bounds.clone()).into());
        bounds.insert("scene_bounds".into(), (self.scene_bounds.clone()).into());
        batch.dispatch_items(&self.kernels.bounds, &bounds, (count).into())?;

        let mut codes = params.clone();
        codes.insert("primitive_bounds".into(), (primitive_bounds.clone()).into());
        codes.insert("scene_bounds".into(), (self.scene_bounds.clone()).into());
        codes.insert("codes".into(), (self.codes.clone()).into());
        codes.insert("indices".into(), (self.indices.clone()).into());
        batch.dispatch_items(&self.kernels.codes, &codes, (count).into())?;

        self.kernels.sort.record(
            batch,
            &self.codes,
            &self.indices,
            &mut self.sort_scratch,
            count.into(),
            morton::BITS.into(),
        )?;

        let mut hierarchy = params.clone();
        hierarchy.insert("codes".into(), (self.codes.clone()).into());
        hierarchy.insert("nodes".into(), (self.nodes.clone()).into());
        hierarchy.insert("parents".into(), (self.parents.clone()).into());
        hierarchy.insert(
            "right_children".into(),
            (self.right_children.clone()).into(),
        );
        batch.dispatch_items(
            &self.kernels.hierarchy,
            &hierarchy,
            (count.saturating_sub(1)).into(),
        )?;

        self.record_refit_passes(batch, primitive_bounds, count)?;
        Ok(())
    }

    /// Record a refit only: keep the ordering and the topology, recompute
    /// every box.
    ///
    /// Valid only after a [`GpuBvh::record_build`] with the same primitive
    /// count, and only while the primitives are still roughly where the
    /// Morton order put them.
    pub fn record_refit(
        &mut self,
        batch: &mut KernelBatch,
        primitive_bounds: &Buffer,
    ) -> Result<()> {
        if self.count == 0 {
            return Ok(());
        }
        let count = self.count;
        self.record_refit_passes(batch, primitive_bounds, count)
    }

    fn record_refit_passes(
        &mut self,
        batch: &mut KernelBatch,
        primitive_bounds: &Buffer,
        count: u32,
    ) -> Result<()> {
        let nodes = 2 * count - 1;
        let params = self.params(count);

        let mut clear = params.clone();
        clear.insert("node_bounds".into(), (self.node_bounds.clone()).into());
        batch.dispatch_items(&self.kernels.clear_bounds, &clear, (nodes).into())?;

        let mut refit = params.clone();
        refit.insert("primitive_bounds".into(), (primitive_bounds.clone()).into());
        refit.insert("indices".into(), (self.indices.clone()).into());
        refit.insert("parents".into(), (self.parents.clone()).into());
        refit.insert("node_bounds".into(), (self.node_bounds.clone()).into());
        batch.dispatch_items(&self.kernels.refit, &refit, (count).into())?;

        let mut gather = params;
        gather.insert("node_bounds".into(), (self.node_bounds.clone()).into());
        gather.insert("nodes".into(), (self.nodes.clone()).into());
        batch.dispatch_items(&self.kernels.gather_bounds, &gather, (nodes).into())?;

        Ok(())
    }

    /// Build on a batch of its own and submit. For a one-off; inside a frame,
    /// record into the frame's batch.
    pub fn build(
        &mut self,
        context: &WgpuContext,
        primitive_bounds: &Buffer,
        count: u32,
    ) -> Result<()> {
        let mut batch = KernelBatch::labelled(context, ("GpuBvh build").into());
        self.record_build(&mut batch, primitive_bounds, count)?;
        batch.submit();
        Ok(())
    }

    /// Read the hierarchy back as host-side nodes. For tests and debugging --
    /// it stalls on the device.
    pub async fn read_nodes(&self, context: &WgpuContext) -> Result<Vec<HostNode>> {
        let raw: Vec<f32> = self.nodes.read(context).await?;
        let count = self.node_count() as usize;
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
        Ok(all[..self.count as usize].to_vec())
    }

    pub async fn read_right_children(&self, context: &WgpuContext) -> Result<Vec<u32>> {
        let all: Vec<u32> = self.right_children.read(context).await?;
        Ok(all[..self.count.saturating_sub(1) as usize].to_vec())
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
