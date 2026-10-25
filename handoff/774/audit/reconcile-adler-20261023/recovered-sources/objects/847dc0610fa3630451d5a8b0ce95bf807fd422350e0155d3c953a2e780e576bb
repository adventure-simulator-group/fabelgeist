//! Fixed enclosure geometry shares material draws within spatial cells.
use super::{BuildingLodMaterial, TacticalBuildingMaterials};
use adventuresim_tactical_core::{
    city_layout::grounding::{BoundarySupportCell, BoundarySupportElement},
    prelude::{CityBoundaryMaterial, SceneBoundary},
    scene_coordinates::GateDatum,
};
use bevy::mesh::{GenerateTangentsError, MeshMergeError};
use bevy::prelude::*;
use std::collections::HashMap;

// Keep culling spatially useful without retaining an entity for every wall cap.
const BOUNDARY_BATCH_METRES: f32 = 128.0;

type BoundaryBatchResult<T> = std::result::Result<T, BoundaryBatchError>;

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

#[derive(Clone, Copy)]
pub(super) enum BoundaryPlacement {
    GateRelative,
    Scene(GateDatum),
}

#[derive(Debug, thiserror::Error)]
pub(super) enum BoundaryBatchError {
    #[error("{element:?} has nonfinite boundary mesh positions or texture coordinates")]
    NonFiniteVertex { element: BoundarySupportElement },
    #[error(transparent)]
    Tangents(#[from] GenerateTangentsError),
    #[error(transparent)]
    Merge(#[from] MeshMergeError),
}

impl BoundaryBatches {
    pub fn insert_boundary(
        &mut self,
        boundary: &SceneBoundary,
        placement: BoundaryPlacement,
        materials: &TacticalBuildingMaterials,
    ) -> BoundaryBatchResult<()> {
        for cell in boundary.fixed_support().cells() {
            let material = match cell.material() {
                CityBoundaryMaterial::Masonry => BuildingLodMaterial::Wall(
                    adventuresim_building_generator::WallMaterialClass::RubbleMasonry,
                ),
                CityBoundaryMaterial::Timber => BuildingLodMaterial::Timber,
                CityBoundaryMaterial::Iron => BuildingLodMaterial::Iron,
            };
            self.insert(
                cell,
                placement,
                materials
                    .for_building(boundary.front_building_id().0)
                    .get(material),
            )?;
        }
        Ok(())
    }

    pub fn insert(
        &mut self,
        member: &BoundarySupportCell,
        placement: BoundaryPlacement,
        material: Handle<StandardMaterial>,
    ) -> BoundaryBatchResult<()> {
        // The parent supplies the gate transform for relative batches. Distant
        // batches encode that same datum in scene-space vertices instead.
        let elevation = match placement {
            BoundaryPlacement::GateRelative => 0.0,
            BoundaryPlacement::Scene(datum) => datum.elevation().metres(),
        };
        let vertices = member.native_positions();
        let centre =
            vertices.iter().copied().sum::<Vec3>() / vertices.len() as f32 + Vec3::Y * elevation;
        let cell = (centre.xz() / BOUNDARY_BATCH_METRES).floor().as_ivec2();
        let origin = Vec3::new(cell.x as f32, 0.0, cell.y as f32) * BOUNDARY_BATCH_METRES;
        let mesh = mesh(member, origin - Vec3::Y * elevation)?;
        match self.meshes.entry((cell, material.id())) {
            std::collections::hash_map::Entry::Occupied(mut entry) => {
                entry.get_mut().mesh.merge(&mesh)?;
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
        Ok(())
    }

    pub fn finish(self) -> impl ExactSizeIterator<Item = BoundaryMesh> {
        self.meshes.into_values()
    }
}

/// Every rendered face uses the shared cell's exact collision vertices.
fn mesh(cell: &BoundarySupportCell, origin: Vec3) -> BoundaryBatchResult<Mesh> {
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
            let position = point - origin;
            let uv = uv / adventuresim_building_generator::BUILDING_DETAIL_UV_METRES_PER_UNIT;
            if !position.is_finite() || !uv.is_finite() {
                return Err(BoundaryBatchError::NonFiniteVertex {
                    element: cell.element(),
                });
            }
            positions.push(position.to_array());
            normals.push(normal.to_array());
            uvs.push(uv.to_array());
        }
    }
    // Each cell has eight faces, so at most 24 native vertex ordinals fit u32.
    let indices: Vec<_> = (0..positions.len() as u32).collect();
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));
    mesh.generate_tangents()?;
    Ok(mesh)
}

#[cfg(test)]
mod tests;
