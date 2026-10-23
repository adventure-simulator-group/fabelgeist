//! Admitted native extraction outputs, with their physical capacities.

use super::ExtractionIndexCapacity;
use fabelgeist_gpu::prelude::Buffer;
use fabelgeist_mesh::MeshVertexCount;

/// Packed xyz positions and normals, sharing one admitted vertex capacity.
pub struct ExtractedVertices {
    capacity: MeshVertexCount,
    positions: Buffer,
    normals: Buffer,
}

/// Triangle vertices and the four-word native draw-indirect arguments.
pub struct TriangleExtraction {
    vertices: ExtractedVertices,
    indirect: Buffer,
}

/// Projected vertices, index slots and five-word indexed-draw arguments.
pub struct IndexedExtraction {
    vertices: ExtractedVertices,
    index_capacity: ExtractionIndexCapacity,
    indices: Buffer,
    indirect: Buffer,
}

impl ExtractedVertices {
    pub(super) fn new(capacity: MeshVertexCount, positions: Buffer, normals: Buffer) -> Self {
        Self {
            capacity,
            positions,
            normals,
        }
    }

    /// Physical allocation capacity; the written prefix may be smaller.
    pub fn capacity(&self) -> MeshVertexCount {
        self.capacity
    }

    pub fn positions(&self) -> &Buffer {
        &self.positions
    }

    pub fn normals(&self) -> &Buffer {
        &self.normals
    }
}

impl TriangleExtraction {
    pub(crate) fn new(vertices: ExtractedVertices, indirect: Buffer) -> Self {
        Self { vertices, indirect }
    }

    pub fn vertices(&self) -> &ExtractedVertices {
        &self.vertices
    }

    pub fn indirect(&self) -> &Buffer {
        &self.indirect
    }
}

impl IndexedExtraction {
    pub(crate) fn new(
        vertices: ExtractedVertices,
        index_capacity: ExtractionIndexCapacity,
        indices: Buffer,
        indirect: Buffer,
    ) -> Self {
        Self {
            vertices,
            index_capacity,
            indices,
            indirect,
        }
    }

    pub fn vertices(&self) -> &ExtractedVertices {
        &self.vertices
    }

    pub fn index_capacity(&self) -> ExtractionIndexCapacity {
        self.index_capacity
    }

    pub fn indices(&self) -> &Buffer {
        &self.indices
    }

    pub fn indirect(&self) -> &Buffer {
        &self.indirect
    }
}
