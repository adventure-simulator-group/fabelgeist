//! Mesh vertex roles and storage-owned membership.

use derive_more::{Display, From, IntoIterator};
use std::ops::{Index, IndexMut};

/// Number of vertices in an MHR mesh, distinct from a generic tensor axis.
/// Equal counts do not establish mesh-instance identity.
#[derive(Debug, Clone, Copy, Display, PartialEq, Eq)]
pub struct MeshVertexCount(usize);

/// Candidate vertex ordinal. Membership is established by the selected table,
/// not by this role alone; the same ordinal may name different mesh vertices.
#[derive(Debug, Clone, Copy, Display, PartialEq, Eq)]
pub struct MeshVertexIndex(usize);

/// Values in mesh vertex order. Native allocation and indexing stay here.
#[derive(Debug, Clone, From, IntoIterator)]
#[into_iterator(owned, ref, ref_mut)]
pub struct MeshVertexTable<T>(Vec<T>);

impl MeshVertexCount {
    /// Admits a count at a mesh-storage or literal boundary, including zero.
    pub const fn from_native_vertex_count(vertices: usize) -> Self {
        Self(vertices)
    }

    /// Projects at a native numerical, allocation or serialization boundary.
    pub const fn native_vertex_count(self) -> usize {
        self.0
    }

    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }
}

impl MeshVertexIndex {
    /// Admits a decoded or native-buffer ordinal without claiming membership.
    pub const fn from_native_vertex_index(vertex: usize) -> Self {
        Self(vertex)
    }

    /// Native vertex-buffer addressing at a numerical or encoding boundary.
    pub const fn native_vertex_index(self) -> usize {
        self.0
    }
}

impl<T> MeshVertexTable<T> {
    pub fn vertex_count(&self) -> MeshVertexCount {
        MeshVertexCount(self.0.len())
    }

    pub fn get(&self, vertex: MeshVertexIndex) -> Option<&T> {
        self.0.get(vertex.0)
    }

    pub fn get_mut(&mut self, vertex: MeshVertexIndex) -> Option<&mut T> {
        self.0.get_mut(vertex.0)
    }

    pub fn indexed(&self) -> impl ExactSizeIterator<Item = (MeshVertexIndex, &T)> {
        self.0
            .iter()
            .enumerate()
            .map(|(vertex, value)| (MeshVertexIndex(vertex), value))
    }

    pub fn indexed_mut(&mut self) -> impl ExactSizeIterator<Item = (MeshVertexIndex, &mut T)> {
        self.0
            .iter_mut()
            .enumerate()
            .map(|(vertex, value)| (MeshVertexIndex(vertex), value))
    }

    /// Contiguous mesh-order values for numerical or model-agnostic geometry ports.
    pub fn as_native_slice(&self) -> &[T] {
        &self.0
    }

    pub fn into_native_values(self) -> Vec<T> {
        self.0
    }
}

impl<T: Clone> MeshVertexTable<T> {
    pub fn filled(vertices: MeshVertexCount, value: T) -> Self {
        Self(vec![value; vertices.0])
    }
}

impl<T> Default for MeshVertexTable<T> {
    fn default() -> Self {
        Self(Vec::new())
    }
}

impl<T> FromIterator<T> for MeshVertexTable<T> {
    fn from_iter<I: IntoIterator<Item = T>>(values: I) -> Self {
        Self(values.into_iter().collect())
    }
}

impl<T> Index<MeshVertexIndex> for MeshVertexTable<T> {
    type Output = T;

    /// Use only after this table's construction or traversal establishes membership.
    fn index(&self, vertex: MeshVertexIndex) -> &T {
        &self.0[vertex.0]
    }
}

impl<T> IndexMut<MeshVertexIndex> for MeshVertexTable<T> {
    fn index_mut(&mut self, vertex: MeshVertexIndex) -> &mut T {
        &mut self.0[vertex.0]
    }
}
