use crate::{GenerateError, PartMesh};

/// A sewn surface before lining and opening rims are added.
#[derive(Default)]
pub(super) struct Pattern {
    pub positions: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
}

impl Pattern {
    pub fn vertex(&mut self, position: [f32; 3]) -> u32 {
        let index = self.positions.len() as u32;
        self.positions.push(position);
        index
    }

    pub fn quad(&mut self, a: u32, b: u32, c: u32, d: u32) {
        self.indices.extend([a, b, c, a, c, d]);
    }

    pub fn lined(self, thickness: f32) -> Result<PartMesh, GenerateError> {
        PartMesh::from_surface(self.positions, self.indices, thickness)
    }
}
