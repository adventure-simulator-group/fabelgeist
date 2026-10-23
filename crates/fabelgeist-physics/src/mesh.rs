//! A triangle mesh collider, with the hierarchy that makes it queryable.
//!
//! The vertices live on the GPU and can be rewritten every frame, which is
//! what an animated body is. Rewriting them refits the hierarchy rather than
//! rebuilding it -- the topology has not changed, and a refit is a handful of
//! passes instead of a sort.

use anyhow::anyhow;
use fabelgeist_bvh::gpu::{BvhKernels, BvhLayout, BvhPrimitiveCount, GpuBvh, TraversalConfig, traversal_source};
use fabelgeist_compute::prelude::*;
use fabelgeist_gpu::prelude::BufferUpload;
use fabelgeist_gpu::prelude::*;
use fabelgeist_math::Vec3;
use fabelgeist_mesh::{MeshTriangleCount, MeshVertexCount};
use std::fmt;

use crate::wgsl;

/// How far outside the surface a particle is still considered in contact, and
/// how much friction that contact has.
#[derive(Clone, Copy, Debug)]
pub struct MeshSurface {
    /// The collision shell. A garment held half a centimetre off the skin
    /// looks like cloth; held at zero it looks like paint.
    pub thickness: f32,
    pub friction: f32,
}

impl Default for MeshSurface {
    fn default() -> Self {
        Self {
            thickness: 0.006,
            friction: 0.3,
        }
    }
}

/// A collider layout whose native triangle indices cannot be addressed.
#[derive(Debug)]
pub struct MeshColliderLayoutError {
    triangles: MeshTriangleCount,
}

pub struct MeshCollider {
    /// xyz = vertex position. `vec4` because that is what the kernels index.
    positions: Buffer,
    /// Three vertex indices per triangle.
    triangles: Buffer,
    /// Two `vec4` per triangle: the lower then the upper corner.
    bounds: Buffer,

    bvh: GpuBvh,
    surface: MeshSurface,

    triangle_bounds: std::sync::Arc<Kernel>,
    vertex_count: MeshVertexCount,
    triangle_count: MeshTriangleCount,
}

impl MeshCollider {
    pub fn new(
        context: &WgpuContext,
        cache: &KernelCache,
        bvh_kernels: BvhKernels,
        positions: &[Vec3],
        triangles: &[[u32; 3]],
        surface: MeshSurface,
    ) -> Result<Self> {
        if triangles.is_empty() {
            return Err(anyhow!("MeshCollider: no triangles"));
        }
        for (index, triangle) in triangles.iter().enumerate() {
            if triangle.iter().any(|&v| v as usize >= positions.len()) {
                return Err(anyhow!(
                    "MeshCollider: triangle {index} indexes past the {} vertices",
                    positions.len()
                ));
            }
        }

        let vertex_count = MeshVertexCount::try_from(positions.len())?;
        let triangle_count = MeshTriangleCount::try_from(triangles.len())?;
        let corner_count = triangle_count.checked_corners()
            .ok_or(MeshColliderLayoutError { triangles: triangle_count })?;
        let primitives = BvhPrimitiveCount::from(triangle_count);
        let bvh_layout = BvhLayout::try_from(primitives)?;
        let storage = BufferDefinition::storage();

        let packed_positions = pack_positions(positions);
        let mut flat_triangles = Vec::with_capacity(usize::from(corner_count));
        flat_triangles.extend(triangles.iter().flatten().copied());

        let mut collider = Self {
            positions: Buffer::from_upload(
                context,
                BufferUpload::from_elements(&packed_positions),
                storage
                    .clone()
                    .with_label(("mesh collider positions").into()),
            )?,
            triangles: Buffer::from_upload(
                context,
                BufferUpload::from_elements(&flat_triangles),
                storage
                    .clone()
                    .with_label(("mesh collider triangles").into()),
            )?,
            bounds: Buffer::new(
                context,
                primitives.bounds_bytes(),
                storage.with_label(("mesh collider triangle bounds").into()),
            )?,
            bvh: GpuBvh::new(context, bvh_kernels, bvh_layout)?,
            surface,
            triangle_bounds: cache.get(context, wgsl::TRIANGLE_BOUNDS)?,
            vertex_count,
            triangle_count,
        };

        collider.rebuild(context)?;
        Ok(collider)
    }

    /// Fixed material configuration used to expand the cached primitive bounds.
    pub fn surface(&self) -> MeshSurface {
        self.surface
    }

    pub fn triangle_count(&self) -> MeshTriangleCount {
        self.triangle_count
    }

    pub fn vertex_count(&self) -> MeshVertexCount {
        self.vertex_count
    }

    /// Replace the vertex positions. The topology is unchanged, so the caller
    /// then wants [`MeshCollider::record_refit`] rather than a rebuild.
    pub fn write_positions(&mut self, context: &WgpuContext, positions: &[Vec3]) -> Result<()> {
        if MeshVertexCount::try_from(positions.len())? != self.vertex_count {
            return Err(anyhow!(
                "MeshCollider: holds {} vertices, given {}",
                self.vertex_count,
                positions.len()
            ));
        }
        self.positions.write(
            context,
            BufferUpload::from_elements(&pack_positions(positions)),
        )?;
        Ok(())
    }

    fn record_bounds(&self, batch: &mut KernelBatch) -> Result<()> {
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

    /// Recompute the bounds and rebuild the hierarchy from scratch. Needed
    /// once, and again whenever the mesh has deformed far enough that its
    /// Morton ordering no longer reflects where the triangles are.
    pub fn record_rebuild(&mut self, batch: &mut KernelBatch) -> Result<()> {
        self.record_bounds(batch)?;
        let bounds = self.bounds.clone();
        self.bvh.record_build(batch, &bounds, self.triangle_count.into())?;
        Ok(())
    }

    /// Recompute the bounds and refit, keeping the ordering. What an animated
    /// body does every frame.
    pub fn record_refit(&mut self, batch: &mut KernelBatch) -> Result<()> {
        self.record_bounds(batch)?;
        let bounds = self.bounds.clone();
        self.bvh.record_refit(batch, &bounds)?;
        Ok(())
    }

    pub fn rebuild(&mut self, context: &WgpuContext) -> Result<()> {
        let mut batch = KernelBatch::labelled(context, "mesh collider rebuild");
        self.record_rebuild(&mut batch)?;
        batch.submit();
        Ok(())
    }

    pub fn refit(&mut self, context: &WgpuContext) -> Result<()> {
        let mut batch = KernelBatch::labelled(context, "mesh collider refit");
        self.record_refit(&mut batch)?;
        batch.submit();
        Ok(())
    }

    /// The WGSL for the resolve kernel, with this mesh's traversal pasted in.
    pub(crate) fn kernel_source() -> String {
        let config = TraversalConfig {
            callback: "bvh_hit".into(),
            ..Default::default()
        };
        wgsl::mesh_source(&traversal_source(&config))
    }

    pub(crate) fn bind(&self, parameters: &mut PassParameters) {
        parameters.insert("mesh_positions", self.positions.clone());
        parameters.insert("mesh_triangles", self.triangles.clone());
        self.bvh.bind(parameters, &TraversalConfig::default());
    }
}

pub fn pack_positions(positions: &[Vec3]) -> Vec<f32> {
    let mut packed = Vec::with_capacity(positions.len() * 4);
    for position in positions {
        packed.extend_from_slice(&[position.x, position.y, position.z, 0.0]);
    }
    packed
}

impl fmt::Display for MeshColliderLayoutError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "mesh collider corners for {} triangles exceed native addresses", self.triangles)
    }
}

impl std::error::Error for MeshColliderLayoutError {}
