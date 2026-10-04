use super::GpuMesh;
use crate::{FrontFace, PrimitiveTopology};
use fabelgeist_gpu::globals::WgpuContext;
use fabelgeist_gpu::prelude::{BufferUpload, BufferUse};

/// Signed mesh-space radius, rounded to binary32 at admission.
/// Negative radii and nonfinite values remain admitted.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SphereRadius(f32);
impl From<f64> for SphereRadius {
    fn from(radius: f64) -> Self {
        Self(radius as f32)
    }
}
impl Default for SphereRadius {
    fn default() -> Self {
        Self(0.5)
    }
}
/// Latitude intervals, including the pole rows in the generated grid.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SphereRings(u32);
impl From<f64> for SphereRings {
    fn from(rings: f64) -> Self {
        Self(rings.max(2.0) as u32)
    }
}
impl Default for SphereRings {
    fn default() -> Self {
        Self(16)
    }
}
/// Longitude intervals, including a duplicated seam column.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SphereSectors(u32);
impl From<f64> for SphereSectors {
    fn from(sectors: f64) -> Self {
        Self(sectors.max(3.0) as u32)
    }
}
impl Default for SphereSectors {
    fn default() -> Self {
        Self(16)
    }
}
/// A sphere's independent geometric and tessellation choices.
/// Float cell counts preserve maximum, truncation and saturation; they do not
/// promise that the resulting geometry fits host memory or draw arithmetic.
///
/// ```compile_fail
/// use fabelgeist_mesh::{SphereGeometry, SphereRings};
/// SphereGeometry::default().with_sectors(SphereRings::from(16.0));
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SphereGeometry {
    radius: SphereRadius,
    rings: SphereRings,
    sectors: SphereSectors,
}
impl SphereGeometry {
    pub fn with_radius(mut self, radius: SphereRadius) -> Self {
        self.radius = radius;
        self
    }
    pub fn with_rings(mut self, rings: SphereRings) -> Self {
        self.rings = rings;
        self
    }
    pub fn with_sectors(mut self, sectors: SphereSectors) -> Self {
        self.sectors = sectors;
        self
    }
    pub fn upload(self, context: &WgpuContext) -> Result<GpuMesh, crate::MeshUploadError> {
        let r = self.radius.0;
        let num_rings = self.rings.0;
        let num_sectors = self.sectors.0;

        let mut positions = Vec::new();
        let mut normals = Vec::new();
        let mut tex_coords = Vec::new();
        let mut indices = Vec::new();

        let r_step = 1.0 / (num_rings as f32);
        let s_step = 1.0 / (num_sectors as f32);

        for y in 0..=num_rings {
            let phi = std::f32::consts::PI * (y as f32) * r_step;
            let sin_phi = phi.sin();
            let cos_phi = phi.cos();

            for x in 0..=num_sectors {
                let theta = 2.0 * std::f32::consts::PI * (x as f32) * s_step;
                let sin_theta = theta.sin();
                let cos_theta = theta.cos();

                let nx = cos_theta * sin_phi;
                let ny = cos_phi;
                let nz = sin_theta * sin_phi;

                positions.push(nx * r);
                positions.push(ny * r);
                positions.push(nz * r);

                normals.push(nx);
                normals.push(ny);
                normals.push(nz);

                tex_coords.push(x as f32 * s_step);
                tex_coords.push(y as f32 * r_step);
            }
        }

        for y in 0..num_rings {
            for x in 0..num_sectors {
                let r0 = y * (num_sectors + 1) + x;
                let r1 = r0 + 1;
                let r2 = (y + 1) * (num_sectors + 1) + x;
                let r3 = r2 + 1;

                indices.push(r0);
                indices.push(r1);
                indices.push(r2);

                indices.push(r1);
                indices.push(r3);
                indices.push(r2);
            }
        }

        let vertex_count = (num_rings + 1) * (num_sectors + 1);

        let pos_buf = crate::MeshAttribute::Positions.allocate(
            context,
            BufferUpload::from_elements(&positions),
            fabelgeist_gpu::data::gpu::buffer::BufferDefinition::storage()
                .with_label(("Sphere Positions").into())
                .with_usage(BufferUse::Vertex),
        )?;
        let norm_buf = crate::MeshAttribute::Normals.allocate(
            context,
            BufferUpload::from_elements(&normals),
            fabelgeist_gpu::data::gpu::buffer::BufferDefinition::storage()
                .with_label(("Sphere Normals").into())
                .with_usage(BufferUse::Vertex),
        )?;
        let uv_buf = crate::MeshAttribute::TextureCoordinates.allocate(
            context,
            BufferUpload::from_elements(&tex_coords),
            fabelgeist_gpu::data::gpu::buffer::BufferDefinition::storage()
                .with_label(("Sphere TexCoords").into())
                .with_usage(BufferUse::Vertex),
        )?;
        let index_buf = crate::MeshAttribute::Indices.allocate(
            context,
            BufferUpload::from_elements(&indices),
            fabelgeist_gpu::data::gpu::buffer::BufferDefinition::storage()
                .with_label(("Sphere Indices").into())
                .with_usage(BufferUse::Index),
        )?;

        Ok(GpuMesh {
            positions: pos_buf,
            normals: norm_buf,
            tex_coords: uv_buf,
            indices: Some(index_buf),
            joints: None,
            weights: None,
            vertex_count: crate::DrawVertexCount::from(vertex_count),
            topology: PrimitiveTopology::TriangleList,
            front_face: FrontFace::Ccw,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[derive(serde::Deserialize)]
    struct AdmissionFixture {
        input: u64,
        radius: u32,
        rings: u32,
        sectors: u32,
    }
    #[test]
    fn scalar_admission_preserves_original_signed_nonfinite_and_saturating_words() {
        let fixtures: Vec<AdmissionFixture> =
            serde_json::from_str(include_str!("../../tests/fixtures/geometry_inputs.json"))
                .unwrap();
        for fixture in fixtures {
            let input = f64::from_bits(fixture.input);
            assert_eq!(SphereRadius::from(input).0.to_bits(), fixture.radius);
            assert_eq!(SphereRings::from(input).0, fixture.rings);
            assert_eq!(SphereSectors::from(input).0, fixture.sectors);
        }
    }
}
