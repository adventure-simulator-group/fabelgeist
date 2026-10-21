//! Exact live topology and global addresses for each checked whole-record part.

use anyhow::{Context, Result, ensure};
use bevy::{
    asset::RenderAssetUsages,
    mesh::{Indices, Mesh, PrimitiveTopology},
};

#[cfg(all(feature = "grown-authoring", not(target_family = "wasm")))]
use super::gpu::ExpandedVertex;
use super::plan::PartPlan;
use crate::{
    prepared::{RecordCount, TreeLibrary, meshes},
    render::address::VERTEX_ADDRESS,
};
#[cfg(all(feature = "grown-authoring", not(target_family = "wasm")))]
use bevy::camera::primitives::MeshAabb;

#[cfg(all(feature = "grown-authoring", not(target_family = "wasm")))]
pub(super) fn construct(
    library: &TreeLibrary,
    part: &PartPlan,
    vertices: Vec<ExpandedVertex>,
) -> Result<Mesh> {
    ensure!(
        vertices.len() == usize::try_from(part.vertices)?,
        "wrong grown readback length"
    );
    let base = part
        .records
        .start
        .checked_mul(part.geometry.vertices_per_record()?)
        .context("grown address overflow")?;
    let mut positions = Vec::with_capacity(vertices.len());
    let mut normals = Vec::with_capacity(vertices.len());
    let mut uvs = Vec::with_capacity(vertices.len());
    let mut addresses = Vec::with_capacity(vertices.len());
    for (ordinal, vertex) in vertices.into_iter().enumerate() {
        let address = base
            .checked_add(u32::try_from(ordinal)?)
            .context("grown address overflow")?;
        ensure!(
            vertex.address == address,
            "noncontiguous grown source address"
        );
        ensure!(
            vertex.position[3] == 1.0 && vertex.normal[3] == 0.0 && vertex.pad == 0,
            "invalid grown readback padding or homogeneous coordinates"
        );
        ensure!(
            vertex
                .position
                .iter()
                .chain(&vertex.normal)
                .chain(&vertex.uv)
                .all(|value| value.is_finite()),
            "nonfinite grown attribute"
        );
        let [nx, ny, nz, _] = vertex.normal;
        let squared_length = nx * nx + ny * ny + nz * nz;
        ensure!(
            squared_length.is_finite() && squared_length > f32::MIN_POSITIVE,
            "invalid grown normal"
        );
        let [x, y, z, _] = vertex.position;
        positions.push([x, y, z]);
        normals.push([nx, ny, nz]);
        uvs.push(vertex.uv);
        addresses.push(vertex.address);
    }
    let indices = topology(library, part)?;
    let mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
    .with_inserted_attribute(VERTEX_ADDRESS, addresses)
    .with_inserted_indices(Indices::U32(indices));
    let bounds = mesh.compute_aabb().context("missing grown bounds")?;
    ensure!(
        bounds.center.is_finite() && bounds.half_extents.is_finite(),
        "nonfinite grown bounds"
    );
    Ok(mesh)
}

pub(super) fn address(library: &TreeLibrary, part: &PartPlan) -> Result<Mesh> {
    let count = usize::try_from(part.vertices)?;
    let base = part
        .records
        .start
        .checked_mul(part.geometry.vertices_per_record()?)
        .context("grown address overflow")?;
    let end = base
        .checked_add(part.vertices)
        .context("grown address overflow")?;
    let indices = topology(library, part)?;
    Ok(Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vec![[0_f32; 3]; count])
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0_f32; 3]; count])
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0_f32; 2]; count])
    .with_inserted_attribute(VERTEX_ADDRESS, (base..end).collect::<Vec<_>>())
    .with_inserted_indices(Indices::U32(indices)))
}

fn topology(library: &TreeLibrary, part: &PartPlan) -> Result<Vec<u32>> {
    let indices = meshes::index_prefix(
        part.geometry.role(),
        part.level,
        &library.detail,
        RecordCount::from_records(part.records.count)?,
    )?;
    ensure!(
        indices.len() == usize::try_from(part.indices)?,
        "grown topology length differs"
    );
    ensure!(
        indices.iter().all(|index| *index < part.vertices),
        "grown index outside part"
    );
    Ok(indices)
}
