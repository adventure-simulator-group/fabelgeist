//! Per-query searches of a point cloud or a triangle mesh: the nearest point,
//! the closest point on a surface, and where a ray crosses a surface.
//!
//! Each query is answered by one invocation that visits every target, so the
//! cost is queries times targets. For the thousands of queries against tens
//! of thousands of targets that fitting one mesh to another asks, that is a
//! few milliseconds -- and it is exact, with no hierarchy to build or
//! refit, and with ties resolved exactly as a host loop resolves them: the
//! lowest target index wins.
//!
//! Positions and queries are tightly packed `f32` triples, so `&[[f32; 3]]`
//! uploads as-is. Triangles are `u32` triples into the positions. Either
//! target set can be narrowed to a list of candidate indices. Counts describe
//! populations; callers supply native indices within their underlying buffers.

use fabelgeist_gpu::prelude::BufferUpload;

use crate::prelude::*;
use std::sync::Arc;
pub use output::{Hit, QueryHits};
pub use quantities::{CandidateCount, QueryCount, QueryLayoutError};
pub use selection::{PointSelection, PointTargets, TriangleSelection, TriangleTargets};

mod output;
mod quantities;
mod selection;
mod wgsl;

/// Which crossing of a ray a query keeps.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Crossing {
    First,
    Last,
}

/// Rays cast by [`MeshQuery::record_ray_triangles`].
#[derive(Clone, Copy, Debug)]
pub struct Rays<'a> {
    /// `f32` triples.
    pub origins: &'a Buffer,
    /// `f32` triples. Distances are reported in units of each direction's
    /// length, so unit directions give metres.
    pub directions: &'a Buffer,
    pub count: QueryCount,
    /// Crossings outside this span along the ray are ignored.
    pub span: [f32; 2],
    pub crossing: Crossing,
    /// Triangles seen this close to edge-on are ignored.
    pub parallel_epsilon: f32,
}

/// The compiled query kernels.
#[derive(Clone, Debug)]
pub struct MeshQuery {
    nearest_points: Arc<Kernel>,
    closest_triangles: Arc<Kernel>,
    ray_triangles: Arc<Kernel>,
    /// Bound in place of a candidate list when a search covers every target.
    all_targets: Buffer,
}

impl MeshQuery {
    pub fn new(context: &WgpuContext) -> Result<Self> {
        Self::with_cache(context, &KernelCache::new())
    }

    pub fn with_cache(context: &WgpuContext, cache: &KernelCache) -> Result<Self> {
        Ok(Self {
            nearest_points: cache.get(context, &wgsl::nearest_points())?,
            closest_triangles: cache.get(context, &wgsl::closest_triangles())?,
            ray_triangles: cache.get(context, &wgsl::ray_triangles())?,
            all_targets: Buffer::from_upload(
                context,
                BufferUpload::from_elements(&[0u32]),
                BufferDefinition::storage().with_label(("query all targets").into()),
            )?,
        })
    }

    fn parameters(
        &self,
        query_count: QueryCount,
        hits: &QueryHits,
    ) -> Result<PassParameters> {
        if query_count > hits.capacity {
            return Err(anyhow!(
                "MeshQuery: {query_count} queries do not fit {} answers",
                hits.capacity
            ));
        }
        let mut parameters = PassParameters::new();
        parameters.insert("query_count", query_count);
        parameters.insert("mode", 0u32);
        parameters.insert("nearest", hits.nearest.clone());
        parameters.insert("weights", hits.weights.clone());
        parameters.insert("distances", hits.distances.clone());
        Ok(parameters)
    }

    /// The nearest target point to each query point.
    pub fn record_nearest_points(
        &self,
        batch: &mut KernelBatch,
        queries: &Buffer,
        query_count: QueryCount,
        targets: PointTargets,
        hits: &QueryHits,
    ) -> Result<()> {
        let mut parameters = self.parameters(query_count, hits)?;
        if query_count.is_empty() {
            return Ok(());
        }
        targets.selection.bind(&mut parameters, &self.all_targets)?;
        parameters.insert("queries", queries.clone());
        parameters.insert("positions", targets.positions.clone());
        batch.dispatch_items(&self.nearest_points, &parameters, query_count.into())?;
        Ok(())
    }

    /// The closest point on the target triangles to each query point.
    pub fn record_closest_triangles(
        &self,
        batch: &mut KernelBatch,
        queries: &Buffer,
        query_count: QueryCount,
        targets: TriangleTargets,
        hits: &QueryHits,
    ) -> Result<()> {
        let mut parameters = self.parameters(query_count, hits)?;
        if query_count.is_empty() {
            return Ok(());
        }
        targets.selection.bind(&mut parameters, &self.all_targets)?;
        parameters.insert("queries", queries.clone());
        parameters.insert("positions", targets.positions.clone());
        parameters.insert("triangles", targets.triangles.clone());
        batch.dispatch_items(&self.closest_triangles, &parameters, query_count.into())?;
        Ok(())
    }

    /// Where each ray crosses the target triangles, from either side.
    pub fn record_ray_triangles(
        &self,
        batch: &mut KernelBatch,
        rays: Rays,
        targets: TriangleTargets,
        hits: &QueryHits,
    ) -> Result<()> {
        let mut parameters = self.parameters(rays.count, hits)?;
        if rays.count.is_empty() {
            return Ok(());
        }
        targets.selection.bind(&mut parameters, &self.all_targets)?;
        parameters.insert(
            "mode",
            match rays.crossing {
                Crossing::First => 0u32,
                Crossing::Last => 1,
            },
        );
        parameters.insert("minimum", rays.span[0]);
        parameters.insert("maximum", rays.span[1]);
        parameters.insert("epsilon", rays.parallel_epsilon);
        parameters.insert("pad", 0u32);
        parameters.insert("queries", rays.origins.clone());
        parameters.insert("directions", rays.directions.clone());
        parameters.insert("positions", targets.positions.clone());
        parameters.insert("triangles", targets.triangles.clone());
        batch.dispatch_items(&self.ray_triangles, &parameters, rays.count.into())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests;
