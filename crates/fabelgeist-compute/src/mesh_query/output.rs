//! Owned query result buffers, with native addressing checked before allocation.

use super::QueryCount;
use super::quantities::QueryLayout;
use fabelgeist_gpu::prelude::*;

/// One answer per query; held allocations cannot be replaced independently.
#[derive(Clone, Debug)]
pub struct QueryHits {
    pub(super) nearest: Buffer,
    pub(super) weights: Buffer,
    pub(super) distances: Buffer,
    pub(super) capacity: QueryCount,
}

/// A query's answer, read back from native words.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Hit {
    pub target: Option<u32>,
    pub weights: [f32; 3],
    pub distance: f32,
}

impl QueryHits {
    pub fn new(context: &WgpuContext, capacity: QueryCount) -> Result<Self> {
        let layout = QueryLayout::try_from(capacity)?;
        let storage = BufferDefinition::storage().with_usage(BufferUse::CopySource);
        Ok(Self {
            nearest: Buffer::new(context, layout.word_bytes(),
                storage.clone().with_label("query nearest".into()))?,
            weights: Buffer::new(context, layout.weight_bytes(),
                storage.clone().with_label("query weights".into()))?,
            distances: Buffer::new(context, layout.word_bytes(),
                storage.with_label("query distances".into()))?,
            capacity: layout.capacity(),
        })
    }

    pub fn capacity(&self) -> QueryCount { self.capacity }

    /// Native target indices, with `u32::MAX` representing a miss.
    pub fn nearest(&self) -> &Buffer { &self.nearest }

    /// Native barycentric triples, unused by point searches.
    pub fn weights(&self) -> &Buffer { &self.weights }

    /// Native squared surface distances, or distances along ray directions.
    pub fn distances(&self) -> &Buffer { &self.distances }

    /// Read up to the first count answers, waiting for the device.
    pub async fn read(&self, context: &WgpuContext, count: QueryCount) -> Result<Vec<Hit>> {
        let count = usize::from(count.min(self.capacity));
        let nearest: Vec<u32> = self.nearest.read(context).await?;
        let weights: Vec<f32> = self.weights.read(context).await?;
        let distances: Vec<f32> = self.distances.read(context).await?;
        Ok((0..count).map(|i| Hit {
            target: (nearest[i] != u32::MAX).then_some(nearest[i]),
            weights: [weights[i * 3], weights[i * 3 + 1], weights[i * 3 + 2]],
            distance: distances[i],
        }).collect())
    }
}
