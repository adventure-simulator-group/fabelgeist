//! Shared conversion of generator geometry into production render assets.
use super::*;
use adventuresim_building_generator::{BuildingLodMaterial, LodMesh};

/// Cuboid surfaces tile at the generator's material scale, irrespective of wall length.
pub(super) fn metric_cuboid(size: Vec3) -> Mesh {
    use bevy::mesh::VertexAttributeValues;
    let mut mesh = Mesh::from(Cuboid::from_size(size));
    let Some(VertexAttributeValues::Float32x3(positions)) =
        mesh.attribute(Mesh::ATTRIBUTE_POSITION)
    else {
        unreachable!()
    };
    let Some(VertexAttributeValues::Float32x3(normals)) = mesh.attribute(Mesh::ATTRIBUTE_NORMAL)
    else {
        unreachable!()
    };
    let uvs = positions
        .iter()
        .zip(normals)
        .map(|(position, normal)| {
            let p = Vec3::from_array(*position);
            let n = Vec3::from_array(*normal).abs();
            let uv = if n.x > 0.5 {
                Vec2::new(p.z, p.y)
            } else if n.y > 0.5 {
                Vec2::new(p.x, p.z)
            } else {
                Vec2::new(p.x, p.y)
            };
            (uv / adventuresim_building_generator::BUILDING_DETAIL_UV_METRES_PER_UNIT).to_array()
        })
        .collect::<Vec<_>>();
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.generate_tangents()
        .expect("metric box faces have valid UVs");
    mesh
}

pub(super) fn recipe_mesh(batch: &LodMesh, local_origin: Vec3) -> Mesh {
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
    );
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_POSITION,
        batch
            .vertices
            .iter()
            .map(|vertex| (vertex.position - local_origin).to_array())
            .collect::<Vec<_>>(),
    );
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_NORMAL,
        batch
            .vertices
            .iter()
            .map(|vertex| vertex.normal.to_array())
            .collect::<Vec<_>>(),
    );
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_UV_0,
        batch
            .vertices
            .iter()
            .map(|vertex| vertex.uv.to_array())
            .collect::<Vec<_>>(),
    );
    mesh.insert_indices(Indices::U32(batch.indices.clone()));
    if matches!(
        batch.material,
        BuildingLodMaterial::Wall(_)
            | BuildingLodMaterial::CrownMasonry
            | BuildingLodMaterial::Roof(_)
            | BuildingLodMaterial::Timber
            | BuildingLodMaterial::InteriorTimber
            | BuildingLodMaterial::FurnitureWood(_)
            | BuildingLodMaterial::Iron
            | BuildingLodMaterial::InteriorPlaster
            | BuildingLodMaterial::Floor
            | BuildingLodMaterial::Glass
    ) {
        mesh.generate_tangents()
            .expect("interior building UVs must support tangent-space normal maps");
    }
    mesh
}
