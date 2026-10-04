use crate::{FrontFace, PrimitiveTopology};

mod transfer;
mod triangles;
mod validation;

/// CPU mesh attributes. Empty normal/UV arrays mean those attributes are absent.
/// Indices are optional; an unindexed mesh uses consecutive vertices.
#[derive(Clone, Debug, PartialEq)]
pub struct MeshData {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub tex_coords: Vec<[f32; 2]>,
    pub indices: Option<Vec<u32>>,
    pub joints: Option<Vec<[u32; 4]>>,
    pub weights: Option<Vec<[f32; 4]>>,
    pub topology: PrimitiveTopology,
    pub front_face: FrontFace,
}

impl Default for MeshData {
    fn default() -> Self {
        Self {
            positions: vec![],
            normals: vec![],
            tex_coords: vec![],
            indices: None,
            joints: None,
            weights: None,
            topology: PrimitiveTopology::TriangleList,
            front_face: FrontFace::Ccw,
        }
    }
}

#[cfg(test)]
mod tests;
