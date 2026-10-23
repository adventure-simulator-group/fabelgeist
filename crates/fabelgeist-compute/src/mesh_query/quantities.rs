//! Search populations and representable native result layouts.

use fabelgeist_gpu::prelude::{BufferByteLength, InvocationCount, PassParameter};
use fabelgeist_mesh::{MeshTriangleCount, MeshVertexCount};
use std::fmt;
use std::num::TryFromIntError;

/// Queries answered by a search, independently of its target population.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct QueryCount(u32);

/// Entries in a supplied list of candidate target indices.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct CandidateCount(u32);

/// Native scalar-word addressing cannot represent the requested layout.
#[derive(Debug, thiserror::Error)]
pub enum QueryLayoutError {
    #[error("result words for {queries} queries exceed native addresses")]
    Results { queries: QueryCount, #[source] source: TryFromIntError },
    #[error("target words for {vertices} points exceed native addresses")]
    Points { vertices: MeshVertexCount, #[source] source: TryFromIntError },
    #[error("target words for {triangles} triangles exceed native addresses")]
    Triangles { triangles: MeshTriangleCount, #[source] source: TryFromIntError },
}

pub(super) struct QueryLayout {
    capacity: QueryCount,
}

impl From<u32> for QueryCount {
    fn from(queries: u32) -> Self { Self(queries) }
}

impl From<MeshVertexCount> for QueryCount {
    fn from(vertices: MeshVertexCount) -> Self { Self(vertices.native_elements()) }
}

impl TryFrom<usize> for QueryCount {
    type Error = TryFromIntError;
    fn try_from(queries: usize) -> Result<Self, Self::Error> {
        u32::try_from(queries).map(Self)
    }
}

impl From<u32> for CandidateCount {
    fn from(candidates: u32) -> Self { Self(candidates) }
}

impl TryFrom<usize> for CandidateCount {
    type Error = TryFromIntError;
    fn try_from(candidates: usize) -> Result<Self, Self::Error> {
        u32::try_from(candidates).map(Self)
    }
}

impl QueryCount {
    pub fn is_empty(self) -> bool { self.0 == 0 }
}

impl TryFrom<QueryCount> for QueryLayout {
    type Error = QueryLayoutError;
    fn try_from(queries: QueryCount) -> Result<Self, Self::Error> {
        let capacity = QueryCount(queries.0.max(1));
        u32::try_from(u64::from(capacity.0) * 3)
            .map_err(|source| QueryLayoutError::Results { queries, source })?;
        Ok(Self { capacity })
    }
}

impl QueryLayout {
    pub(super) fn capacity(&self) -> QueryCount { self.capacity }
    pub(super) fn word_bytes(&self) -> BufferByteLength {
        BufferByteLength::from(u64::from(self.capacity.0) * 4)
    }
    pub(super) fn weight_bytes(&self) -> BufferByteLength {
        BufferByteLength::from(u64::from(self.capacity.0) * 12)
    }
}

impl From<QueryCount> for usize {
    fn from(queries: QueryCount) -> Self { queries.0 as usize }
}

impl From<QueryCount> for InvocationCount {
    fn from(queries: QueryCount) -> Self { Self::from(queries.0) }
}

impl From<QueryCount> for PassParameter {
    fn from(queries: QueryCount) -> Self { Self::Unsigned(queries.0) }
}

impl From<CandidateCount> for PassParameter {
    fn from(candidates: CandidateCount) -> Self { Self::Unsigned(candidates.0) }
}

impl fmt::Display for QueryCount {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

impl fmt::Display for CandidateCount {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}
