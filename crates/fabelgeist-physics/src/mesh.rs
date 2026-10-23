//! A triangle mesh collider, with the hierarchy that makes it queryable.
//!
//! The vertices live on the GPU and can be rewritten every frame, which is
//! what an animated body is. Rewriting them refits the hierarchy rather than
//! rebuilding it -- the topology has not changed, and a refit is a handful of
//! passes instead of a sort.

use anyhow::anyhow;
use fabelgeist_bvh::gpu::{
    BvhKernels, BvhLayout, BvhPrimitiveCount, GpuBvh, RecordedBvh, TraversalConfig,
    traversal_source,
};
use fabelgeist_compute::prelude::*;
use fabelgeist_gpu::prelude::BufferUpload;
use fabelgeist_gpu::prelude::*;
use fabelgeist_math::Vec3;
use fabelgeist_mesh::{MeshTriangleCount, MeshVertexCount};
use std::fmt;

use crate::wgsl;
use geometry::MeshGeometry;

pub use geometry::pack_positions;

mod geometry;

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
    geometry: MeshGeometry,
    workspace: GpuBvh,
    bvh: RecordedBvh,
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

        let positions = Buffer::from_upload(
            context,
            BufferUpload::from_elements(&packed_positions),
            storage
                .clone()
                .with_label(("mesh collider positions").into()),
        )?;
        let triangles = Buffer::from_upload(
            context,
            BufferUpload::from_elements(&flat_triangles),
            storage
                .clone()
                .with_label(("mesh collider triangles").into()),
        )?;
        let bounds = Buffer::new(
            context,
            primitives.bounds_bytes(),
            storage.with_label(("mesh collider triangle bounds").into()),
        )?;
        let mut workspace = GpuBvh::new(context, bvh_kernels, bvh_layout)?;
        let geometry = MeshGeometry {
            positions,
            triangles,
            bounds,
            surface,
            triangle_bounds: cache.get(context, wgsl::TRIANGLE_BOUNDS)?,
            vertex_count,
            triangle_count,
        };

        let mut batch = KernelBatch::labelled(context, "mesh collider rebuild");
        geometry.record_bounds(&mut batch)?;
        let bvh = workspace.record_build(&mut batch, &geometry.bounds, triangle_count.into())?;
        batch.submit()?;
        Ok(Self {
            geometry,
            workspace,
            bvh,
        })
    }

    /// Fixed material configuration used to expand the cached primitive bounds.
    pub fn surface(&self) -> MeshSurface {
        self.geometry.surface
    }

    pub fn triangle_count(&self) -> MeshTriangleCount {
        self.geometry.triangle_count
    }

    pub fn vertex_count(&self) -> MeshVertexCount {
        self.geometry.vertex_count
    }

    /// Replace the vertex positions. The topology is unchanged, so the caller
    /// then wants [`MeshCollider::record_refit`] rather than a rebuild.
    pub fn write_positions(&mut self, context: &WgpuContext, positions: &[Vec3]) -> Result<()> {
        if MeshVertexCount::try_from(positions.len())? != self.geometry.vertex_count {
            return Err(anyhow!(
                "MeshCollider: holds {} vertices, given {}",
                self.geometry.vertex_count,
                positions.len()
            ));
        }
        self.geometry.positions.write(
            context,
            BufferUpload::from_elements(&pack_positions(positions)),
        )?;
        Ok(())
    }

    /// Recompute the bounds and rebuild the hierarchy from scratch. Needed
    /// once, and again whenever the mesh has deformed far enough that its
    /// Morton ordering no longer reflects where the triangles are.
    pub fn record_rebuild(&mut self, batch: &mut KernelBatch) -> Result<()> {
        self.geometry.record_bounds(batch)?;
        let bounds = self.geometry.bounds.clone();
        // Topology and storage are fixed, so the original query view names
        // the same resources and layout after every rebuild.
        self.workspace
            .record_build(batch, &bounds, self.geometry.triangle_count.into())?;
        Ok(())
    }

    /// Recompute the bounds and refit, keeping the ordering. What an animated
    /// body does every frame.
    pub fn record_refit(&mut self, batch: &mut KernelBatch) -> Result<()> {
        self.geometry.record_bounds(batch)?;
        let bounds = self.geometry.bounds.clone();
        self.bvh.record_refit(batch, &bounds)?;
        Ok(())
    }

    pub fn rebuild(&mut self, context: &WgpuContext) -> Result<()> {
        let mut batch = KernelBatch::labelled(context, "mesh collider rebuild");
        self.record_rebuild(&mut batch)?;
        batch.submit()?;
        Ok(())
    }

    pub fn refit(&mut self, context: &WgpuContext) -> Result<()> {
        let mut batch = KernelBatch::labelled(context, "mesh collider refit");
        self.record_refit(&mut batch)?;
        batch.submit()?;
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

    pub(crate) fn bind(&self, parameters: &mut PassParameters) -> Result<()> {
        parameters.insert("mesh_positions", self.geometry.positions.clone());
        parameters.insert("mesh_triangles", self.geometry.triangles.clone());
        self.bvh.bind(parameters, &TraversalConfig::default())
    }
}

impl fmt::Display for MeshColliderLayoutError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "mesh collider corners for {} triangles exceed native addresses", self.triangles)
    }
}

impl std::error::Error for MeshColliderLayoutError {}
