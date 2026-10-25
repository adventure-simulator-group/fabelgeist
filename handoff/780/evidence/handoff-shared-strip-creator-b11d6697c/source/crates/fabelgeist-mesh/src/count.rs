//! Mesh populations admitted before fixed-width GPU layout and dispatch.
//!
//! These quantities describe topology, independently of its GPU allocation.
//! They do not certify index membership or attribute lengths.

use std::fmt;
use std::num::TryFromIntError;

pub use strip::{MeshStripCount, MeshStripError, MeshStripLayout, MeshStripResult, MeshStripRows};

#[cfg(feature = "gpu")]
mod gpu;
mod strip;

/// Vertices in a mesh or in one contiguous mesh arena.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct MeshVertexCount(u32);

/// Indexed triangles, distinct from their vertices and corners.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct MeshTriangleCount(u32);

/// Three corners for every triangle in an admitted indexed layout.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct MeshCornerCount(u32);

impl From<u32> for MeshVertexCount {
    fn from(vertices: u32) -> Self {
        Self(vertices)
    }
}

impl TryFrom<usize> for MeshVertexCount {
    type Error = TryFromIntError;

    fn try_from(vertices: usize) -> Result<Self, Self::Error> {
        u32::try_from(vertices).map(Self)
    }
}

impl From<u32> for MeshTriangleCount {
    fn from(triangles: u32) -> Self {
        Self(triangles)
    }
}

impl TryFrom<usize> for MeshTriangleCount {
    type Error = TryFromIntError;

    fn try_from(triangles: usize) -> Result<Self, Self::Error> {
        u32::try_from(triangles).map(Self)
    }
}

impl MeshVertexCount {
    pub fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// Combine adjacent vertex populations without wrapping native addresses.
    pub fn checked_add(self, other: Self) -> Option<Self> {
        self.0.checked_add(other.0).map(Self)
    }

    /// Fixed-width representation for native uniforms and owning quantity
    /// conversions. Keep this extraction inside those boundary adapters.
    pub const fn native_elements(self) -> u32 {
        self.0
    }
}

impl MeshTriangleCount {
    pub fn is_empty(self) -> bool {
        self.0 == 0
    }

    pub fn checked_add(self, other: Self) -> Option<Self> {
        self.0.checked_add(other.0).map(Self)
    }

    /// The corner layout must fit the same fixed-width native address space.
    pub fn checked_corners(self) -> Option<MeshCornerCount> {
        self.0.checked_mul(3).map(MeshCornerCount)
    }

    /// Fixed-width representation for native uniforms and owning quantity
    /// conversions. Keep this extraction inside those boundary adapters.
    pub const fn native_elements(self) -> u32 {
        self.0
    }
}

impl MeshCornerCount {
    /// Fixed-width representation for native uniforms and owning quantity
    /// conversions. Keep this extraction inside those boundary adapters.
    pub const fn native_elements(self) -> u32 {
        self.0
    }
}

impl From<MeshVertexCount> for usize {
    fn from(vertices: MeshVertexCount) -> Self {
        vertices.0 as usize
    }
}

impl From<MeshTriangleCount> for usize {
    fn from(triangles: MeshTriangleCount) -> Self {
        triangles.0 as usize
    }
}

impl From<MeshCornerCount> for usize {
    fn from(corners: MeshCornerCount) -> Self {
        corners.0 as usize
    }
}

impl fmt::Display for MeshVertexCount {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

impl fmt::Display for MeshTriangleCount {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

impl fmt::Display for MeshCornerCount {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}
