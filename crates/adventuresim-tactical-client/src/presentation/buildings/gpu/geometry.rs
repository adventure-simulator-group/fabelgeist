//! Indexed geometry pages stay below the baseline WebGPU storage binding limit.
use bevy::{mesh::VertexAttributeValues, prelude::*};
use std::collections::HashMap;

// Leave headroom under the 128 MiB baseline binding limit. Vertices and indices
// are independently bounded, including a single unusually large source mesh.
pub(super) const PAGE_BYTES: usize = 64 * 1024 * 1024;

#[derive(Default)]
pub(super) struct Page {
    pub vertices: Vec<Vec4>,
    pub indices: Vec<u32>,
}

#[derive(Clone)]
pub(super) struct Slice {
    pub page: usize,
    pub start: u32,
    pub count: u32,
    pub radius: f32,
}

pub(super) struct Geometry {
    pub pages: Vec<Page>,
    limit: usize,
}

impl Default for Geometry {
    fn default() -> Self {
        Self {
            pages: vec![Page::default()],
            limit: PAGE_BYTES,
        }
    }
}

fn float3(mesh: &Mesh, attribute: bevy::mesh::MeshVertexAttribute) -> &[[f32; 3]] {
    let values = mesh
        .attribute(attribute)
        .expect("generated city vertex attribute");
    match values {
        VertexAttributeValues::Float32x3(values) => values,
        _ => panic!("generated city attribute must be float3"),
    }
}

impl Geometry {
    pub fn insert(&mut self, mesh: &Mesh) -> Vec<Slice> {
        let positions = float3(mesh, Mesh::ATTRIBUTE_POSITION);
        let normals = float3(mesh, Mesh::ATTRIBUTE_NORMAL);
        let Some(VertexAttributeValues::Float32x2(uvs)) = mesh.attribute(Mesh::ATTRIBUTE_UV_0)
        else {
            panic!("generated city requires UVs");
        };
        let tangents = mesh.attribute(Mesh::ATTRIBUTE_TANGENT);
        let source: Vec<_> = mesh
            .indices()
            .expect("generated city is indexed")
            .iter()
            .collect();
        assert_eq!(
            source.len() % 3,
            0,
            "city geometry must contain whole triangles"
        );
        let mut remap = HashMap::new();
        let mut slices: Vec<Slice> = Vec::new();
        for triangle in source.as_chunks::<3>().0 {
            let page = self.pages.last().expect("geometry page");
            let missing = triangle
                .iter()
                .filter(|index| !remap.contains_key(*index))
                .count();
            if (page.vertices.len() + missing * 3) * size_of::<Vec4>() > self.limit
                || (page.indices.len() + 3) * size_of::<u32>() > self.limit
            {
                self.pages.push(Page::default());
                remap.clear();
            }
            let page_index = self.pages.len() - 1;
            let page = self.pages.last_mut().expect("geometry page");
            if slices.last().is_none_or(|slice| slice.page != page_index) {
                slices.push(Slice {
                    page: page_index,
                    start: page.indices.len() as u32,
                    count: 0,
                    radius: 0.0,
                });
            }
            let slice = slices.last_mut().expect("mesh page slice");
            for &index in triangle {
                let vertex = *remap.entry(index).or_insert_with(|| {
                    let vertex = (page.vertices.len() / 3) as u32;
                    page.vertices
                        .push(Vec3::from_array(positions[index]).extend(uvs[index][0]));
                    page.vertices
                        .push(Vec3::from_array(normals[index]).extend(uvs[index][1]));
                    page.vertices.push(match tangents {
                        Some(VertexAttributeValues::Float32x4(values)) => {
                            Vec4::from_array(values[index])
                        }
                        _ => Vec3::from_array(normals[index])
                            .any_orthonormal_vector()
                            .extend(1.0),
                    });
                    vertex
                });
                page.indices.push(vertex);
                slice.radius = slice
                    .radius
                    .max(Vec3::from_array(positions[index]).length());
                slice.count += 1;
            }
        }
        slices
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn oversized_mesh_splits_without_losing_or_reordering_triangles() {
        let mesh = Mesh::from(Cuboid::new(2.0, 3.0, 4.0));
        let mut geometry = Geometry {
            pages: vec![Page::default()],
            limit: 3 * 3 * size_of::<Vec4>(),
        };
        let slices = geometry.insert(&mesh);
        assert!(slices.len() > 1);
        let positions = float3(&mesh, Mesh::ATTRIBUTE_POSITION);
        let expected: Vec<_> = mesh
            .indices()
            .unwrap()
            .iter()
            .map(|i| Vec3::from_array(positions[i]))
            .collect();
        let mut actual = Vec::new();
        for slice in slices {
            let page = &geometry.pages[slice.page];
            assert!(page.vertices.len() * size_of::<Vec4>() <= geometry.limit);
            assert!(page.indices.len() * size_of::<u32>() <= geometry.limit);
            for &index in &page.indices[slice.start as usize..(slice.start + slice.count) as usize]
            {
                actual.push(page.vertices[index as usize * 3].truncate());
            }
        }
        assert_eq!(actual, expected);
    }
}
