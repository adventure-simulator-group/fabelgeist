//! Native buffer extents for admitted card topology.

use super::{CardError, CardLayout, CardResult, CardStripLayout, CardVertex};
use fabelgeist_gpu::prelude::{BufferByteLength, BufferElementCount};

impl CardLayout {
    pub fn vertices_bytes(self) -> BufferByteLength {
        (u64::from(self.vertices().native_elements()) * size_of::<CardVertex>() as u64).into()
    }

    pub fn indices_bytes(self) -> BufferByteLength {
        self.corners().word_bytes()
    }

    /// One native vertex even when the logical triangle mesh is empty.
    pub fn vertices_binding_bytes(self) -> BufferByteLength {
        self.vertices_bytes().max(BufferByteLength::from(size_of::<CardVertex>()))
    }

    /// One native index even when no logical corners are drawn.
    pub fn indices_binding_bytes(self) -> BufferByteLength {
        self.indices_bytes().max(BufferByteLength::from(size_of::<u32>()))
    }
}

impl CardStripLayout {
    /// card_build writes each vertex as four separately indexed vec4 records.
    pub(crate) fn admit_native_writer(self) -> CardResult<Self> {
        BufferElementCount::from(self.layout().vertices().native_elements())
            .checked_mul((size_of::<CardVertex>() / size_of::<[f32; 4]>()) as u32)
            .ok_or(CardError::VertexWords { vertices: self.layout().vertices() })?;
        Ok(self)
    }
}
