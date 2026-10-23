//! Fixed collider topology, source storage and triangle-bounds recording.

use super::MeshSurface;
use fabelgeist_compute::prelude::*;
use fabelgeist_gpu::prelude::*;
use fabelgeist_math::Vec3;
use fabelgeist_mesh::{MeshTriangleCount, MeshVertexCount};

pub(super) struct MeshGeometry {
    /// xyz = vertex position. `vec4` because that is what the kernels index.
    pub(super) positions: Buffer,
    /// Three vertex indices per triangle.
    pub(super) triangles: Buffer,
    /// Two `vec4` per triangle: the lower then the upper corner.
    pub(super) bounds: Buffer,

    pub(super) surface: MeshSurface,

    pub(super) triangle_bounds: std::sync::Arc<Kernel>,
    pub(super) vertex_count: MeshVertexCount,
    pub(super) triangle_count: MeshTriangleCount,
}

impl MeshGeometry {
    pub(super) fn record_bounds(&self, batch: &mut KernelBatch) -> Result<()> {
        let mut parameters = PassParameters::new();
        parameters.insert("mesh_positions", self.positions.clone());
        parameters.insert("mesh_triangles", self.triangles.clone());
        parameters.insert("primitive_bounds", self.bounds.clone());
        parameters.insert("count", self.triangle_count);
        // The bounds carry the collision shell, so a query for a point inside
        // the shell can use the point's own box rather than an expanded one.
        parameters.insert("margin", self.surface.thickness);
        parameters.insert("pad0", 0u32);
        parameters.insert("pad1", 0u32);
        batch.dispatch_items(&self.triangle_bounds, &parameters, self.triangle_count.into())?;
        Ok(())
    }
}

pub fn pack_positions(positions: &[Vec3]) -> Vec<f32> {
    let mut packed = Vec::with_capacity(positions.len() * 4);
    for position in positions {
        packed.extend_from_slice(&[position.x, position.y, position.z, 0.0]);
    }
    packed
}
