use super::GpuMesh;
use crate::{FrontFace, PrimitiveTopology};
use fabelgeist_gpu::data::vector::Vec3;
use fabelgeist_gpu::globals::WgpuContext;
use fabelgeist_gpu::prelude::{BufferUpload, BufferUse};

/// Signed full box dimensions in mesh coordinates.
/// Negative and nonfinite components retain their original admission policy.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BoxDimensions(Vec3);
impl Default for BoxDimensions {
    fn default() -> Self {
        Self(Vec3::new(1.0, 1.0, 1.0))
    }
}
impl From<Vec3> for BoxDimensions {
    fn from(dimensions: Vec3) -> Self {
        Self(dimensions)
    }
}
impl BoxDimensions {
    pub fn upload(self, context: &WgpuContext) -> Result<GpuMesh, crate::MeshUploadError> {
        let h = self.0 * 0.5;

        let mut data = BoxAttributes::new(h);

        // Front face (+Z)
        data.append_face(
            Vec3::new(0.0, 0.0, 1.0),
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
        );
        // Back face (-Z)
        data.append_face(
            Vec3::new(0.0, 0.0, -1.0),
            Vec3::new(-1.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
        );
        // Top face (+Y)
        data.append_face(
            Vec3::new(0.0, 1.0, 0.0),
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(0.0, 0.0, -1.0),
        );
        // Bottom face (-Y)
        data.append_face(
            Vec3::new(0.0, -1.0, 0.0),
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(0.0, 0.0, 1.0),
        );
        // Right face (+X)
        data.append_face(
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(0.0, 0.0, -1.0),
            Vec3::new(0.0, 1.0, 0.0),
        );
        // Left face (-X)
        data.append_face(
            Vec3::new(-1.0, 0.0, 0.0),
            Vec3::new(0.0, 0.0, 1.0),
            Vec3::new(0.0, 1.0, 0.0),
        );

        let pos_buf = crate::MeshAttribute::Positions.allocate(
            context,
            BufferUpload::from_elements(&data.positions),
            fabelgeist_gpu::data::gpu::buffer::BufferDefinition::storage()
                .with_label(("Box Positions").into())
                .with_usage(BufferUse::Vertex),
        )?;
        let norm_buf = crate::MeshAttribute::Normals.allocate(
            context,
            BufferUpload::from_elements(&data.normals),
            fabelgeist_gpu::data::gpu::buffer::BufferDefinition::storage()
                .with_label(("Box Normals").into())
                .with_usage(BufferUse::Vertex),
        )?;
        let uv_buf = crate::MeshAttribute::TextureCoordinates.allocate(
            context,
            BufferUpload::from_elements(&data.tex_coords),
            fabelgeist_gpu::data::gpu::buffer::BufferDefinition::storage()
                .with_label(("Box TexCoords").into())
                .with_usage(BufferUse::Vertex),
        )?;
        let index_buf = crate::MeshAttribute::Indices.allocate(
            context,
            BufferUpload::from_elements(&data.indices),
            fabelgeist_gpu::data::gpu::buffer::BufferDefinition::storage()
                .with_label(("Box Indices").into())
                .with_usage(BufferUse::Index),
        )?;

        Ok(GpuMesh {
            positions: pos_buf,
            normals: norm_buf,
            tex_coords: uv_buf,
            indices: Some(index_buf),
            joints: None,
            weights: None,
            vertex_count: crate::DrawVertexCount::from(24),
            topology: PrimitiveTopology::TriangleList,
            front_face: FrontFace::Cw,
        })
    }
}

// This owner retains the box's interleaved face construction through ABI upload.
struct BoxAttributes {
    half_size: Vec3,
    positions: Vec<f32>,
    normals: Vec<f32>,
    tex_coords: Vec<f32>,
    indices: Vec<u32>,
}
impl BoxAttributes {
    fn new(half_size: Vec3) -> Self {
        Self {
            half_size,
            positions: Vec::new(),
            normals: Vec::new(),
            tex_coords: Vec::new(),
            indices: Vec::new(),
        }
    }
    fn append_face(&mut self, n: Vec3, r: Vec3, u: Vec3) {
        let start_idx = (self.positions.len() / 3) as u32;
        let center = Vec3::new(
            n.x * self.half_size.x,
            n.y * self.half_size.y,
            n.z * self.half_size.z,
        );
        let r_vec = Vec3::new(
            r.x * self.half_size.x,
            r.y * self.half_size.y,
            r.z * self.half_size.z,
        );
        let u_vec = Vec3::new(
            u.x * self.half_size.x,
            u.y * self.half_size.y,
            u.z * self.half_size.z,
        );

        let p0 = center - r_vec - u_vec;
        let p1 = center + r_vec - u_vec;
        let p2 = center + r_vec + u_vec;
        let p3 = center - r_vec + u_vec;

        for p in &[p0, p1, p2, p3] {
            self.positions.push(p.x);
            self.positions.push(p.y);
            self.positions.push(p.z);
            self.normals.push(n.x);
            self.normals.push(n.y);
            self.normals.push(n.z);
        }

        self.tex_coords.push(0.0);
        self.tex_coords.push(0.0);
        self.tex_coords.push(1.0);
        self.tex_coords.push(0.0);
        self.tex_coords.push(1.0);
        self.tex_coords.push(1.0);
        self.tex_coords.push(0.0);
        self.tex_coords.push(1.0);

        self.indices.push(start_idx);
        self.indices.push(start_idx + 1);
        self.indices.push(start_idx + 2);
        self.indices.push(start_idx);
        self.indices.push(start_idx + 2);
        self.indices.push(start_idx + 3);
    }
}
