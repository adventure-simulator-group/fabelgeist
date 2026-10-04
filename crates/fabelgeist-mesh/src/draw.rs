//! Draw-address quantities, independent of simulation particle addresses.

use crate::MeshValidationError;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct MeshAttributeLength(usize);

impl From<usize> for MeshAttributeLength {
    fn from(length: usize) -> Self {
        Self(length)
    }
}

impl std::fmt::Display for MeshAttributeLength {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(formatter)
    }
}

/// Number of vertices in the mesh's 32-bit draw address space; zero is valid.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DrawVertexCount(u32);

impl From<u32> for DrawVertexCount {
    fn from(vertices: u32) -> Self {
        Self(vertices)
    }
}

impl From<DrawVertexCount> for usize {
    fn from(vertices: DrawVertexCount) -> Self {
        vertices.0 as usize
    }
}

impl std::fmt::Display for DrawVertexCount {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(formatter)
    }
}

impl DrawVertexCount {
    pub fn from_attribute_length(length: MeshAttributeLength) -> Result<Self, MeshValidationError> {
        match u32::try_from(length.0) {
            Ok(count) => Ok(Self(count)),
            Err(_) => Err(MeshValidationError::TooManyVertices { actual: length }),
        }
    }

    pub fn sequential_indices(self) -> Vec<DrawVertexIndex> {
        (0..self.0).map(DrawVertexIndex).collect()
    }
}

/// A draw address is not proof that a vertex exists in a particular mesh.
///
/// ```compile_fail
/// use fabelgeist_mesh::{DrawVertexCount, DrawVertexIndex};
/// let _: DrawVertexCount = DrawVertexIndex::from(3);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DrawVertexIndex(u32);

impl From<u32> for DrawVertexIndex {
    fn from(index: u32) -> Self {
        Self(index)
    }
}

impl From<DrawVertexIndex> for usize {
    fn from(index: DrawVertexIndex) -> Self {
        index.0 as usize
    }
}

impl From<DrawVertexIndex> for u32 {
    fn from(index: DrawVertexIndex) -> Self {
        index.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DrawVertexMembership {
    Present,
    Outside,
}

impl DrawVertexIndex {
    pub fn membership(self, vertices: DrawVertexCount) -> DrawVertexMembership {
        if self.0 < vertices.0 {
            DrawVertexMembership::Present
        } else {
            DrawVertexMembership::Outside
        }
    }
}
