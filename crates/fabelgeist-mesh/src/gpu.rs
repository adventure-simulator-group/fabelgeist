mod box_shape;
mod displacement;
mod plane;
mod sphere;
use crate::{FrontFace, PrimitiveTopology};
pub use box_shape::BoxDimensions;
pub use displacement::{DisplacementProjection, DisplacementScale, MeshDisplacement};
use fabelgeist_gpu::prelude::Buffer;
pub use plane::{PlaneGeometry, PlaneSubdivisions};
pub use sphere::{SphereGeometry, SphereRadius, SphereRings, SphereSectors};
#[derive(Debug, Clone, PartialEq)]
pub struct GpuMesh {
    pub positions: Buffer,
    pub normals: Buffer,
    pub tex_coords: Buffer,
    pub indices: Option<Buffer>,
    pub joints: Option<Buffer>,
    pub weights: Option<Buffer>,
    pub vertex_count: crate::DrawVertexCount,
    pub topology: PrimitiveTopology,
    pub front_face: FrontFace,
}

#[cfg(test)]
mod tests;
