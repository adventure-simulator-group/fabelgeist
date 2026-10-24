//! One checked strip mesh and its four native attribute arrays.

use anyhow::Result;
use fabelgeist_gpu::prelude::{Buffer, BufferByteLength, BufferDefinition, WgpuContext};
use fabelgeist_hair::gpu::error::HairGpuError;
use fabelgeist_mesh::{MeshStripLayout, MeshVertexCount};

use super::super::HairGpu;
use crate::hair::HairGeometry;

/// Triangles on the allocating device, with one vec4 per vertex attribute.
pub struct GpuGeometry {
    positions: Buffer,
    normals: Buffer,
    tangents: Buffer,
    texcoords: Buffer,
    layout: MeshStripLayout,
    device: wgpu::Device,
}

/// Output attributes and the row-lift scratch required by the same strip pass.
pub(super) struct StripBuffers {
    pub(super) geometry: GpuGeometry,
    pub(super) lifts: Buffer,
}

impl StripBuffers {
    pub(super) fn new(context: &WgpuContext, layout: MeshStripLayout) -> Result<Self> {
        let attribute_bytes = layout.vertices().vec4_bytes();
        // One lift vec4 per row, versus two attribute vec4s per row.
        let lift_bytes = BufferByteLength::from(u64::from(attribute_bytes) / 2);
        let allocate = |bytes: BufferByteLength, label: &str| {
            Buffer::new(
                context,
                bytes.max(BufferByteLength::from(size_of::<[f32; 4]>())),
                BufferDefinition::storage().with_label(label.into()),
            )
        };
        Ok(Self {
            geometry: GpuGeometry {
                positions: allocate(attribute_bytes, "hair strip positions")?,
                normals: allocate(attribute_bytes, "hair strip normals")?,
                tangents: allocate(attribute_bytes, "hair strip tangents")?,
                texcoords: allocate(attribute_bytes, "hair strip texture coordinates")?,
                layout,
                device: context.device.as_ref().clone(),
            },
            lifts: allocate(lift_bytes, "hair strip lifts")?,
        })
    }
}

impl GpuGeometry {
    /// Position, and baked sky visibility.
    pub fn positions(&self) -> &Buffer {
        &self.positions
    }

    /// Normal, and the strip's own random value.
    pub fn normals(&self) -> &Buffer {
        &self.normals
    }

    /// Along the strands, root to tip.
    pub fn tangents(&self) -> &Buffer {
        &self.tangents
    }

    /// Texture coordinates.
    pub fn texcoords(&self) -> &Buffer {
        &self.texcoords
    }

    /// The triangles back on the host, without native allocation padding.
    pub fn read(&self, gpu: &HairGpu) -> Result<HairGeometry> {
        let context = gpu.context();
        if &self.device != context.device.as_ref() {
            return Err(HairGpuError::DeviceMismatch.into());
        }
        let read = |buffer: &Buffer| -> Result<Vec<[f32; 4]>> {
            let logical = buffer.prefix(self.layout.vertices().vec4_bytes())?;
            Ok(pollster::block_on(logical.read(context))?)
        };
        let positions = read(&self.positions)?;
        let normals = read(&self.normals)?;
        let tangents = read(&self.tangents)?;
        let texcoords = read(&self.texcoords)?;
        let xyz = |v: &[f32; 4]| [v[0], v[1], v[2]];
        Ok(HairGeometry {
            positions: positions.iter().map(xyz).collect(),
            normals: normals.iter().map(xyz).collect(),
            texcoords: texcoords.iter().map(|t| [t[0], t[1]]).collect(),
            occlusion: positions.iter().map(|p| p[3]).collect(),
            randoms: normals.iter().map(|n| n[3]).collect(),
            tangents: tangents.iter().map(xyz).collect(),
            faces: self.faces()?,
        })
    }

    pub fn vertices(&self) -> MeshVertexCount {
        self.layout.vertices()
    }

    /// Two triangles a segment, every strip's rows in order.
    pub fn faces(&self) -> Result<Vec<[u32; 3]>> {
        let mut faces = Vec::new();
        faces.try_reserve_exact(usize::from(self.layout.triangles()))?;
        faces.extend(self.layout.faces());
        Ok(faces)
    }
}
