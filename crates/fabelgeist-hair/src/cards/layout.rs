//! Checked triangle populations and generated card-strip relationships.

use super::CardVertex;
use fabelgeist_mesh::{MeshCornerCount, MeshTriangleCount, MeshVertexCount};
use std::num::TryFromIntError;

#[cfg(feature = "gpu")]
mod gpu;

pub type CardResult<T> = Result<T, CardError>;

/// Physical strips, independent of triangles and index corners.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct CardCount(usize);

/// Requested quads along a strip; producers apply their minimum/detail policy.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct CardSegments(usize);

/// Complete indexed triangles; this does not certify index membership or buffers.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CardLayout {
    vertices: MeshVertexCount,
    corners: MeshCornerCount,
}

/// An admitted uniform strip topology with at least one segment per card.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CardStripLayout {
    layout: CardLayout,
    segments: CardSegments,
}

#[derive(Debug, thiserror::Error)]
pub enum CardError {
    #[error("invalid card source shape")]
    Source(#[from] crate::strands::StrandShapeError),
    #[error("card source {requested:?} exceeds guide map addresses")]
    SourceCount { requested: crate::parameters::StrandCount, #[source] source: TryFromIntError },
    #[error("card vertex population {requested} exceeds native addresses")]
    Vertices { requested: usize, #[source] source: TryFromIntError },
    #[error("card triangle population {requested} exceeds native addresses")]
    Triangles { requested: usize, #[source] source: TryFromIntError },
    #[error("card triangle population {triangles:?} exceeds native corner addresses")]
    Corners { triangles: MeshTriangleCount },
    #[error("card index list has {indices} corners, which is not a whole triangle list")]
    PartialTriangle { indices: usize },
    #[error("card corner {corner} addresses vertex {index}, outside {vertices:?}")]
    Index { corner: usize, index: u32, vertices: MeshVertexCount },
    #[error("{cards:?} cards of {segments:?} segments exceed indexed strip addresses")]
    Strips { cards: CardCount, segments: CardSegments },
    #[error("card strips need at least one segment")]
    MissingSegment,
    #[error("card geometry allocation failed")]
    Allocation(#[from] std::collections::TryReserveError),
    #[cfg(feature = "gpu")]
    #[error("native card vertices {vertices:?} exceed vec4 storage addresses")]
    VertexWords { vertices: MeshVertexCount },
    #[cfg(feature = "gpu")]
    #[error("native card lock or scan addresses overflow for {strands:?} strands of {segments:?} segments")]
    LockAddresses { strands: crate::parameters::StrandCount, segments: CardSegments },
    #[cfg(feature = "gpu")]
    #[error("native card compaction returned {cards:?} cards for {strands:?} strands")]
    Compaction { cards: CardCount, strands: crate::parameters::StrandCount },
}

impl CardCount {
    pub const fn from_count(count: usize) -> Self {
        Self(count)
    }

    /// Collection, native geometry or observation boundary.
    pub const fn count(self) -> usize {
        self.0
    }
}

impl CardSegments {
    pub const fn from_count(count: usize) -> Self {
        Self(count)
    }

    /// Collection or native strip-kernel boundary.
    pub const fn count(self) -> usize {
        self.0
    }
}

impl CardLayout {
    pub fn new(vertices: MeshVertexCount, triangles: MeshTriangleCount) -> CardResult<Self> {
        let corners = triangles.checked_corners().ok_or(CardError::Corners { triangles })?;
        Ok(Self { vertices, corners })
    }

    pub fn vertices(self) -> MeshVertexCount {
        self.vertices
    }

    pub fn triangles(self) -> MeshTriangleCount {
        // The sole admission route constructs corners as three per triangle.
        MeshTriangleCount::from(self.corners.native_elements() / 3)
    }

    pub fn corners(self) -> MeshCornerCount {
        self.corners
    }

    pub(super) fn for_triangles(vertices: &[CardVertex], indices: &[u32]) -> CardResult<Self> {
        if !indices.len().is_multiple_of(3) {
            return Err(CardError::PartialTriangle { indices: indices.len() });
        }
        let vertex_count = MeshVertexCount::try_from(vertices.len())
            .map_err(|source| CardError::Vertices { requested: vertices.len(), source })?;
        let triangle_count = MeshTriangleCount::try_from(indices.len() / 3)
            .map_err(|source| CardError::Triangles { requested: indices.len() / 3, source })?;
        let layout = Self::new(vertex_count, triangle_count)?;
        for (corner, &index) in indices.iter().enumerate() {
            if index >= vertex_count.native_elements() {
                return Err(CardError::Index { corner, index, vertices: vertex_count });
            }
        }
        Ok(layout)
    }
}

impl CardStripLayout {
    pub fn new(cards: CardCount, segments: CardSegments) -> CardResult<Self> {
        if segments.count() == 0 {
            return Err(CardError::MissingSegment);
        }
        let invalid = || CardError::Strips { cards, segments };
        let rows = segments.count().checked_add(1).ok_or_else(invalid)?;
        let per_card = rows.checked_mul(2).ok_or_else(invalid)?;
        let vertices = cards.count().checked_mul(per_card).ok_or_else(invalid)?;
        let triangles = cards.count().checked_mul(segments.count())
            .and_then(|quads| quads.checked_mul(2)).ok_or_else(invalid)?;
        let vertices = MeshVertexCount::try_from(vertices)
            .map_err(|source| CardError::Vertices { requested: vertices, source })?;
        let triangles = MeshTriangleCount::try_from(triangles)
            .map_err(|source| CardError::Triangles { requested: triangles, source })?;
        Ok(Self { layout: CardLayout::new(vertices, triangles)?, segments })
    }

    pub fn layout(self) -> CardLayout {
        self.layout
    }

    pub fn cards(self) -> CardCount {
        CardCount::from_count(usize::from(self.layout.vertices()) / ((self.segments.count() + 1) * 2))
    }

    pub fn segments(self) -> CardSegments {
        self.segments
    }
}
