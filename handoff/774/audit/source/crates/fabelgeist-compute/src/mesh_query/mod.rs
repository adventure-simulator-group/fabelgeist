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
//! target set can be narrowed to a list of candidate indices.

use fabelgeist_gpu::prelude::{BufferUpload, BufferUse};
mod wgsl;

use crate::prelude::*;
use std::sync::Arc;

/// Points searched by [`MeshQuery::record_nearest_points`].
#[derive(Clone, Copy, Debug)]
pub struct PointTargets<'a> {
    /// `f32` triples.
    pub positions: &'a Buffer,
    /// When given, only these point indices are searched.
    pub candidates: Option<&'a Buffer>,
    /// Points searched: the candidate count when there are candidates.
    pub count: u32,
}

/// Triangles searched by the surface queries.
#[derive(Clone, Copy, Debug)]
pub struct TriangleTargets<'a> {
    /// `f32` triples.
    pub positions: &'a Buffer,
    /// `u32` triples indexing `positions`.
    pub triangles: &'a Buffer,
    /// When given, only these triangle indices are searched.
    pub candidates: Option<&'a Buffer>,
    /// Triangles searched: the candidate count when there are candidates.
    pub count: u32,
}

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
    pub count: u32,
    /// Crossings outside this span along the ray are ignored.
    pub span: [f32; 2],
    pub crossing: Crossing,
    /// Triangles seen this close to edge-on are ignored.
    pub parallel_epsilon: f32,
}

/// One answer per query.
#[derive(Clone, Debug)]
pub struct QueryHits {
    /// The target found, or `u32::MAX` for none.
    pub nearest: Buffer,
    /// Barycentric weights on the triangle found, as `f32` triples. Unused by
    /// point searches.
    pub weights: Buffer,
    /// Squared distance for point and surface searches; distance along the
    /// ray for ray casts.
    pub distances: Buffer,
    capacity: u32,
}

/// A query's answer, read back to the host.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Hit {
    pub target: Option<u32>,
    pub weights: [f32; 3],
    pub distance: f32,
}

impl QueryHits {
    pub fn new(context: &WgpuContext, capacity: u32) -> Result<Self> {
        let capacity = capacity.max(1);
        let storage = BufferDefinition::storage().with_usage(BufferUse::CopySource);
        Ok(Self {
            nearest: Buffer::new(
                context,
                (capacity as u64 * 4).into(),
                storage.clone().with_label(("query nearest").into()),
            )?,
            weights: Buffer::new(
                context,
                (capacity as u64 * 12).into(),
                storage.clone().with_label(("query weights").into()),
            )?,
            distances: Buffer::new(
                context,
                (capacity as u64 * 4).into(),
                storage.with_label(("query distances").into()),
            )?,
            capacity,
        })
    }

    pub fn capacity(&self) -> u32 {
        self.capacity
    }

    /// Read the first `count` answers. Stalls on the device.
    pub async fn read(&self, context: &WgpuContext, count: u32) -> Result<Vec<Hit>> {
        let count = count.min(self.capacity) as usize;
        let nearest: Vec<u32> = self.nearest.read(context).await?;
        let weights: Vec<f32> = self.weights.read(context).await?;
        let distances: Vec<f32> = self.distances.read(context).await?;
        Ok((0..count)
            .map(|i| Hit {
                target: (nearest[i] != u32::MAX).then_some(nearest[i]),
                weights: [weights[i * 3], weights[i * 3 + 1], weights[i * 3 + 2]],
                distance: distances[i],
            })
            .collect())
    }
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
        query_count: u32,
        target_count: u32,
        candidates: Option<&Buffer>,
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
        parameters.insert("target_count", target_count);
        parameters.insert("use_candidates", u32::from(candidates.is_some()));
        parameters.insert("mode", 0u32);
        parameters.insert(
            "candidates",
            candidates.unwrap_or(&self.all_targets).clone(),
        );
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
        query_count: u32,
        targets: PointTargets,
        hits: &QueryHits,
    ) -> Result<()> {
        let mut parameters =
            self.parameters(query_count, targets.count, targets.candidates, hits)?;
        parameters.insert("queries", queries.clone());
        parameters.insert("positions", targets.positions.clone());
        batch.dispatch_items(&self.nearest_points, &parameters, query_count)?;
        Ok(())
    }

    /// The closest point on the target triangles to each query point.
    pub fn record_closest_triangles(
        &self,
        batch: &mut KernelBatch,
        queries: &Buffer,
        query_count: u32,
        targets: TriangleTargets,
        hits: &QueryHits,
    ) -> Result<()> {
        let mut parameters =
            self.parameters(query_count, targets.count, targets.candidates, hits)?;
        parameters.insert("queries", queries.clone());
        parameters.insert("positions", targets.positions.clone());
        parameters.insert("triangles", targets.triangles.clone());
        batch.dispatch_items(&self.closest_triangles, &parameters, query_count)?;
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
        let mut parameters =
            self.parameters(rays.count, targets.count, targets.candidates, hits)?;
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
        batch.dispatch_items(&self.ray_triangles, &parameters, rays.count)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests;
