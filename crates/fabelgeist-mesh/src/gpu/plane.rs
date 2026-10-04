use super::GpuMesh;
use crate::{FrontFace, PrimitiveTopology};
use fabelgeist_gpu::data::vector::{Vec2, Vec3};
use fabelgeist_gpu::globals::WgpuContext;
use fabelgeist_gpu::prelude::{BufferUpload, BufferUse};

/// A plane's admitted cell counts, distinct from spatial coordinates.
///
/// Native floating input keeps the original per-axis maximum and saturating
/// truncation. Admission does not promise that allocation arithmetic fits.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PlaneSubdivisions {
    horizontal: u32,
    vertical: u32,
}
impl Default for PlaneSubdivisions {
    fn default() -> Self {
        Self {
            horizontal: 1,
            vertical: 1,
        }
    }
}
impl From<Vec2> for PlaneSubdivisions {
    fn from(axes: Vec2) -> Self {
        Self {
            horizontal: axes.x.max(1.0) as u32,
            vertical: axes.y.max(1.0) as u32,
        }
    }
}

/// Plane construction owns its subdivisions, half-spans and supplied normal.
/// The basis is intentionally neither normalized nor orthogonalized.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlaneGeometry {
    subdivisions: PlaneSubdivisions,
    right: Vec3,
    up: Vec3,
    normal: Vec3,
}
impl Default for PlaneGeometry {
    fn default() -> Self {
        Self {
            subdivisions: PlaneSubdivisions::default(),
            right: Vec3::new(1.0, 0.0, 0.0),
            up: Vec3::new(0.0, 0.0, 1.0),
            normal: Vec3::new(0.0, 1.0, 0.0),
        }
    }
}
impl PlaneGeometry {
    pub fn with_subdivisions(mut self, subdivisions: PlaneSubdivisions) -> Self {
        self.subdivisions = subdivisions;
        self
    }
    pub fn with_right(mut self, half_span: Vec3) -> Self {
        self.right = half_span;
        self
    }
    pub fn with_up(mut self, half_span: Vec3) -> Self {
        self.up = half_span;
        self
    }
    pub fn with_normal(mut self, normal: Vec3) -> Self {
        self.normal = normal;
        self
    }
    pub fn upload(self, context: &WgpuContext) -> Result<GpuMesh, crate::MeshUploadError> {
        let res_x = self.subdivisions.horizontal;
        let res_y = self.subdivisions.vertical;
        let r_vec = self.right;
        let u_vec = self.up;
        let n_vec = self.normal;
        let mut positions = Vec::with_capacity(((res_x + 1) * (res_y + 1) * 3) as usize);
        let mut normals = Vec::with_capacity(((res_x + 1) * (res_y + 1) * 3) as usize);
        let mut tex_coords = Vec::with_capacity(((res_x + 1) * (res_y + 1) * 2) as usize);
        let mut indices = Vec::with_capacity((res_x * res_y * 6) as usize);

        for y in 0..=res_y {
            for x in 0..=res_x {
                let u = x as f32 / res_x as f32;
                let v = y as f32 / res_y as f32;

                let ux = (u - 0.5) * 2.0;
                let uy = (v - 0.5) * 2.0;

                let p = r_vec * ux + u_vec * uy;

                positions.push(p.x);
                positions.push(p.y);
                positions.push(p.z);

                normals.push(n_vec.x);
                normals.push(n_vec.y);
                normals.push(n_vec.z);

                tex_coords.push(u);
                tex_coords.push(v);
            }
        }

        for y in 0..res_y {
            for x in 0..res_x {
                let i0 = y * (res_x + 1) + x;
                let i1 = i0 + 1;
                let i2 = (y + 1) * (res_x + 1) + x;
                let i3 = i2 + 1;

                indices.push(i0);
                indices.push(i1);
                indices.push(i2);

                indices.push(i1);
                indices.push(i3);
                indices.push(i2);
            }
        }

        let pos_buf = crate::MeshAttribute::Positions.allocate(
            context,
            BufferUpload::from_elements(&positions),
            fabelgeist_gpu::data::gpu::buffer::BufferDefinition::storage()
                .with_label(("Plane Positions").into())
                .with_usage(BufferUse::Vertex),
        )?;

        let norm_buf = crate::MeshAttribute::Normals.allocate(
            context,
            BufferUpload::from_elements(&normals),
            fabelgeist_gpu::data::gpu::buffer::BufferDefinition::storage()
                .with_label(("Plane Normals").into())
                .with_usage(BufferUse::Vertex),
        )?;

        let uv_buf = crate::MeshAttribute::TextureCoordinates.allocate(
            context,
            BufferUpload::from_elements(&tex_coords),
            fabelgeist_gpu::data::gpu::buffer::BufferDefinition::storage()
                .with_label(("Plane TexCoords").into())
                .with_usage(BufferUse::Vertex),
        )?;

        let index_buf = crate::MeshAttribute::Indices.allocate(
            context,
            BufferUpload::from_elements(&indices),
            fabelgeist_gpu::data::gpu::buffer::BufferDefinition::storage()
                .with_label(("Plane Indices").into())
                .with_usage(BufferUse::Index),
        )?;

        Ok(GpuMesh {
            positions: pos_buf,
            normals: norm_buf,
            tex_coords: uv_buf,
            indices: Some(index_buf),
            joints: None,
            weights: None,
            vertex_count: crate::DrawVertexCount::from((res_x + 1) * (res_y + 1)),
            topology: PrimitiveTopology::TriangleList,
            front_face: FrontFace::Cw,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[derive(serde::Deserialize)]
    struct AdmissionFixture {
        input: u32,
        count: u32,
    }
    #[test]
    fn cells_preserve_original_fractional_signed_nonfinite_and_saturating_admission() {
        let fixtures: Vec<AdmissionFixture> =
            serde_json::from_str(include_str!("../../tests/fixtures/plane_inputs.json")).unwrap();
        for fixture in fixtures {
            let input = f32::from_bits(fixture.input);
            let subdivisions = PlaneSubdivisions::from(Vec2::new(input, input));
            assert_eq!(subdivisions.horizontal, fixture.count);
            assert_eq!(subdivisions.vertical, fixture.count);
        }
    }
}
