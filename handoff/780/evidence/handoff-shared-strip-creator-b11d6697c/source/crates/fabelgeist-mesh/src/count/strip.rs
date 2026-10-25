//! Uniform two-edge strips, independent of the material drawn on them.

use super::{MeshCornerCount, MeshTriangleCount, MeshVertexCount};
use std::num::{NonZeroU32, TryFromIntError};

pub type MeshStripResult<T> = Result<T, MeshStripError>;

/// Strips in one indexed mesh, distinct from their rows and vertices.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct MeshStripCount(u32);

/// Rows of two edge vertices, with at least one segment between them.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct MeshStripRows(u32);

/// A complete strip population whose vertices and corners fit native indices.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MeshStripLayout {
    strips: MeshStripCount,
    rows: MeshStripRows,
}

#[derive(Debug, thiserror::Error)]
pub enum MeshStripError {
    #[error("strip row population {requested} must fit native addresses and contain two rows")]
    Rows { requested: usize },
    #[error("strip population {requested} exceeds native addresses")]
    Count { requested: usize, #[source] source: TryFromIntError },
    #[error("subdividing {rows:?} into {pieces} pieces per segment exceeds native rows")]
    Subdivision { rows: MeshStripRows, pieces: NonZeroU32 },
    #[error("{strips:?} of {rows:?} exceed native vertex or corner addresses")]
    Extent { strips: MeshStripCount, rows: MeshStripRows },
}

impl From<u32> for MeshStripCount {
    fn from(strips: u32) -> Self {
        Self(strips)
    }
}

impl TryFrom<usize> for MeshStripCount {
    type Error = MeshStripError;

    fn try_from(strips: usize) -> MeshStripResult<Self> {
        u32::try_from(strips).map(Self)
            .map_err(|source| MeshStripError::Count { requested: strips, source })
    }
}

impl MeshStripCount {
    /// Native population boundary; this is not a launch quantity.
    pub const fn native_strips(self) -> u32 {
        self.0
    }
}

impl TryFrom<usize> for MeshStripRows {
    type Error = MeshStripError;

    fn try_from(rows: usize) -> MeshStripResult<Self> {
        match u32::try_from(rows) {
            Ok(rows) if rows >= 2 => Ok(Self(rows)),
            _ => Err(MeshStripError::Rows { requested: rows }),
        }
    }
}

impl MeshStripRows {
    /// Admit a segment policy at the owning topology conversion boundary.
    pub fn from_segments(segments: usize) -> MeshStripResult<Self> {
        let rows = segments.checked_add(1)
            .ok_or(MeshStripError::Rows { requested: segments })?;
        Self::try_from(rows)
    }

    pub fn subdivided(self, pieces: NonZeroU32) -> MeshStripResult<Self> {
        (self.0 - 1).checked_mul(pieces.get())
            .and_then(|segments| segments.checked_add(1))
            .map(Self)
            .ok_or(MeshStripError::Subdivision { rows: self, pieces })
    }

    /// Native row-index and numerical subdivision boundary.
    pub const fn native_rows(self) -> u32 {
        self.0
    }
}

impl MeshStripLayout {
    pub fn new(strips: MeshStripCount, rows: MeshStripRows) -> MeshStripResult<Self> {
        let invalid = || MeshStripError::Extent { strips, rows };
        strips.0.checked_mul(rows.0).and_then(|vertices| vertices.checked_mul(2))
            .ok_or_else(invalid)?;
        strips.0.checked_mul(rows.0 - 1).and_then(|quads| quads.checked_mul(6))
            .ok_or_else(invalid)?;
        Ok(Self { strips, rows })
    }

    pub fn strips(self) -> MeshStripCount {
        self.strips
    }

    pub fn rows(self) -> MeshStripRows {
        self.rows
    }

    pub fn vertices(self) -> MeshVertexCount {
        MeshVertexCount(self.strips.0 * self.rows.0 * 2)
    }

    pub fn triangles(self) -> MeshTriangleCount {
        MeshTriangleCount(self.strips.0 * (self.rows.0 - 1) * 2)
    }

    pub fn corners(self) -> MeshCornerCount {
        MeshCornerCount(self.strips.0 * (self.rows.0 - 1) * 6)
    }

    /// Strip-major triangles, each segment wound from root to tip.
    pub fn faces(self) -> impl Iterator<Item = [u32; 3]> {
        (0..self.strips.0).flat_map(move |strip| {
            // The admitted complete population bounds every intermediate.
            let base = strip * self.rows.0 * 2;
            (0..self.rows.0 - 1).flat_map(move |segment| {
                let [a, b] = [base + 2 * segment, base + 2 * segment + 1];
                [[a, a + 2, b], [b, a + 2, b + 2]]
            })
        })
    }
}
