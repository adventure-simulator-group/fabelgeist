//! Fixed enclosure geometry shares material draws within spatial cells.
use adventuresim_tactical_core::city_layout::grounding::BoundarySupportCell;
use bevy::prelude::*;
use std::collections::HashMap;

// Keep culling spatially useful without retaining an entity for every wall cap.
const BOUNDARY_BATCH_METRES: f32 = 128.0;

pub(super) struct BoundaryMesh {
    pub mesh: Mesh,
    pub material: Handle<StandardMaterial>,
    pub origin: Vec3,
}

#[derive(Default)]
pub(super) struct BoundaryBatches {
    meshes: HashMap<(IVec2, AssetId<StandardMaterial>), BoundaryMesh>,
    pub members: usize,
}

impl BoundaryBatches {
    pub fn insert(
        &mut self,
        member: &BoundarySupportCell,
        elevation: f32,
        material: Handle<StandardMaterial>,
    ) {
        let centre =
            member.native_positions().iter().copied().sum::<Vec3>() / 6.0 + Vec3::Y * elevation;
        let cell = (centre.xz() / BOUNDARY_BATCH_METRES).floor().as_ivec2();
        let origin = Vec3::new(cell.x as f32, 0.0, cell.y as f32) * BOUNDARY_BATCH_METRES;
        let mesh = mesh(member, origin - Vec3::Y * elevation);
        match self.meshes.entry((cell, material.id())) {
            std::collections::hash_map::Entry::Occupied(mut entry) => {
                entry
                    .get_mut()
                    .mesh
                    .merge(&mesh)
                    .expect("boundary cuboids share indexed vertex attributes");
            }
            std::collections::hash_map::Entry::Vacant(entry) => {
                entry.insert(BoundaryMesh {
                    mesh,
                    material,
                    origin,
                });
            }
        }
        self.members += 1;
    }

    pub fn finish(self) -> impl ExactSizeIterator<Item = BoundaryMesh> {
        self.meshes.into_values()
    }
}

#[cfg(test)]
mod tests;

/// Every rendered face uses the shared cell's exact collision vertices.
fn mesh(cell: &BoundarySupportCell, origin: Vec3) -> Mesh {
    use bevy::{
        asset::RenderAssetUsages, mesh::Indices, render::render_resource::PrimitiveTopology,
    };
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    for triangle in cell.triangles() {
        let normal = (triangle[1] - triangle[0])
            .cross(triangle[2] - triangle[0])
            .normalize();
        // Omit only zero-area faces produced by coincident f32 intersection
        // vertices. These have no surface or collider bearing area.
        if !normal.is_finite() {
            continue;
        }
        let absolute = normal.abs();
        for point in triangle {
            let uv = if absolute.x >= absolute.y && absolute.x >= absolute.z {
                Vec2::new(point.z, point.y)
            } else if absolute.y >= absolute.z {
                Vec2::new(point.x, point.z)
            } else {
                Vec2::new(point.x, point.y)
            };
            positions.push((point - origin).to_array());
            normals.push(normal.to_array());
            uvs.push(
                (uv / adventuresim_building_generator::BUILDING_DETAIL_UV_METRES_PER_UNIT)
                    .to_array(),
            );
        }
    }
    let indices: Vec<_> = (0..positions.len())
        .map(|i| u32::try_from(i).expect("bounded boundary mesh"))
        .collect();
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));
    mesh.generate_tangents()
        .expect("nondegenerate enclosure faces have metric UVs");
    mesh
}
