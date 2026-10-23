//! Query bindings tied to the resources and layout of one recorded build.

use crate::{CellScale, HashBucketCount, HashPointCount};
use fabelgeist_gpu::data::gpu::buffer::Buffer;
use fabelgeist_gpu::data::gpu::parameters::PassParameters;

/// A build's recorded source and query bindings, not a submission receipt.
///
/// Record queries after the build and propagate submission errors before using
/// their results. Buffers retain their native mutable semantics: later writes
/// can change their contents. Reallocating the builder leaves this view tied to
/// its original buffers and layout.
#[derive(Clone, Debug)]
pub struct RecordedHashGrid {
    pub(super) points: Buffer,
    pub(super) indices: Buffer,
    pub(super) ranges: Buffer,
    pub(super) count: HashPointCount,
    pub(super) scale: CellScale,
    pub(super) buckets: HashBucketCount,
}

impl RecordedHashGrid {
    /// Source population used by the recorded build, distinct from query count.
    pub fn count(&self) -> HashPointCount {
        self.count
    }

    /// Bind the actual hashed source, index/range buffers and query layout.
    /// The query invocation count remains the caller's own population.
    pub fn bind(&self, parameters: &mut PassParameters) {
        parameters.insert("points", self.points.clone());
        parameters.insert("hash_indices", self.indices.clone());
        parameters.insert("hash_ranges", self.ranges.clone());
        self.bind_layout(parameters);
    }

    pub(super) fn build_parameters(&self) -> PassParameters {
        let mut parameters = PassParameters::new();
        parameters.insert("count", self.count);
        self.bind_layout(&mut parameters);
        parameters.insert("pad", 0u32);
        parameters
    }

    fn bind_layout(&self, parameters: &mut PassParameters) {
        parameters.insert("table_size", self.buckets);
        parameters.insert("inverse_cell", self.scale);
    }
}
