//! Shared conversion of generator geometry into production render assets.
use super::*;
use adventuresim_building_generator::{BuildingLodMaterial, LodMesh};

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
