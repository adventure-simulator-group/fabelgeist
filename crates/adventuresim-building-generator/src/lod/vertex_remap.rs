//! Exact attribute remapping, retaining texture seams and tangent discontinuities.
use std::collections::HashMap;

use super::{LodMesh, LodVertex};

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
struct VertexKey {
    attributes: [u32; 8],
    basis: [u32; 6],
}

fn bits(value: f32) -> u32 {
    if value == 0.0 { 0 } else { value.to_bits() }
}

impl LodMesh {
    pub(crate) fn remap_vertices(&mut self) {
        let mut bases = vec![None; self.vertices.len()];
        let mut seams = vec![false; self.vertices.len()];
        for triangle in self.indices.as_chunks::<3>().0 {
            let basis = tangent_basis(triangle.map(|i| self.vertices[i as usize]));
            for &index in triangle {
                let index = index as usize;
                if let Some(previous) = bases[index] {
                    seams[index] |= Some(previous) != basis;
                }
                seams[index] |= basis.is_none();
                bases[index] = basis;
            }
        }
        let mut unique = HashMap::with_capacity(self.vertices.len());
        let mut remap = Vec::with_capacity(self.vertices.len());
        let mut vertices = Vec::with_capacity(self.vertices.len());
        for (index, vertex) in self.vertices.iter().copied().enumerate() {
            let next = vertices.len() as u32;
            let mapped = if !seams[index]
                && let Some(basis) = bases[index]
            {
                let key = VertexKey::new(vertex, basis);
                *unique.entry(key).or_insert(next)
            } else {
                next
            };
            if mapped == next {
                vertices.push(vertex);
            }
            remap.push(mapped);
        }
        for index in &mut self.indices {
            *index = remap[*index as usize];
        }
        self.vertices = vertices;
    }
}

impl VertexKey {
    fn new(vertex: LodVertex, basis: [u32; 6]) -> Self {
        let p = vertex.position;
        let n = vertex.normal;
        let uv = vertex.uv;
        Self {
            attributes: [p.x, p.y, p.z, n.x, n.y, n.z, uv.x, uv.y].map(bits),
            basis,
        }
    }
}

fn tangent_basis([a, b, c]: [LodVertex; 3]) -> Option<[u32; 6]> {
    let edge_b = b.position - a.position;
    let edge_c = c.position - a.position;
    let uv_b = b.uv - a.uv;
    let uv_c = c.uv - a.uv;
    let determinant = uv_b.perp_dot(uv_c);
    if determinant == 0.0 || !determinant.is_finite() {
        return None;
    }
    let tangent = ((edge_b * uv_c.y - edge_c * uv_b.y) / determinant).normalize_or_zero();
    let bitangent = ((edge_c * uv_b.x - edge_b * uv_c.x) / determinant).normalize_or_zero();
    (tangent.is_finite() && bitangent.is_finite()).then(|| {
        [
            tangent.x,
            tangent.y,
            tangent.z,
            bitangent.x,
            bitangent.y,
            bitangent.z,
        ]
        .map(bits)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::BuildingLodMaterial;
    use bevy::math::{Vec2, Vec3};

    #[test]
    fn remapping_preserves_indexed_attributes_and_mirrored_texture_seams() {
        let mut mesh = LodMesh::new(BuildingLodMaterial::Timber);
        for (points, uvs) in [
            (
                [Vec3::ZERO, Vec3::X, Vec3::Y],
                [Vec2::ZERO, Vec2::X, Vec2::Y],
            ),
            (
                [Vec3::X, Vec3::X + Vec3::Y, Vec3::Y],
                [Vec2::X, Vec2::ONE, Vec2::Y],
            ),
            (
                [Vec3::ZERO, -Vec3::X, Vec3::Y],
                [Vec2::ZERO, Vec2::X, Vec2::Y],
            ),
        ] {
            mesh.push_triangle(points, Vec3::Z, uvs);
        }
        let before: Vec<_> = mesh
            .indices
            .iter()
            .map(|&i| format!("{:?}", mesh.vertices[i as usize]))
            .collect();
        mesh.remap_vertices();
        let after: Vec<_> = mesh
            .indices
            .iter()
            .map(|&i| format!("{:?}", mesh.vertices[i as usize]))
            .collect();
        assert_eq!(before, after);
        assert_eq!(mesh.vertices.len(), 7);
        assert_ne!(mesh.indices[0], mesh.indices[6]);
        assert_ne!(mesh.indices[2], mesh.indices[8]);
    }
}
