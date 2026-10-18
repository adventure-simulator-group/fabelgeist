//! Offline address-grid expansion; arrival only decodes these meshes.

use bevy::mesh::{Indices, VertexAttributeValues};
use bevy::prelude::Mesh;

use super::{AddressMesh, LevelMeshes, PreparedError, RecordSpan, Result};
use crate::render::{ForestDetail, levels::Level, shapes};

/// Every integer through this value is exactly representable in a GPU f32.
pub(super) const EXACT_ADDRESS: u64 = 1 << 24;
/// Bound the prototype's offline address payload before allocating its grids.
const MAX_MESH_BYTES: u64 = 256 * 1024 * 1024;

pub(super) fn preflight(branches: usize, leaves: usize, detail: &ForestDetail) -> Result<()> {
    let mut bytes = 0_u64;
    for level in Level::ALL {
        let shape = level.shape(detail);
        let branch = u64::from((shape.sides + 1) * (shape.spans + 1)) * 12
            + u64::from(shape.sides * shape.spans * 6) * 4;
        let leaf = u64::from(shape.blades) * (4 * 12 + 6 * 4);
        bytes = bytes
            .checked_add(branch * branches as u64 + leaf * leaves as u64)
            .ok_or(PreparedError::Invalid("mesh size overflow"))?;
    }
    if bytes > MAX_MESH_BYTES {
        return Err(PreparedError::Invalid(
            "address meshes exceed prototype's 256 MiB preparation limit; use a smaller oracle or variant budget",
        ));
    }
    Ok(())
}

pub(super) fn levels(
    branches: RecordSpan,
    leaves: RecordSpan,
    detail: &ForestDetail,
) -> Result<[LevelMeshes; 3]> {
    let mut output = Vec::new();
    for level in Level::ALL {
        let shape = level.shape(detail);
        output.push(LevelMeshes {
            branches: expand(shapes::branch_grid(shape.sides, shape.spans), branches, 1)?,
            leaves: expand(shapes::leaf_blades(shape.blades), leaves, shape.blades)?,
        });
    }
    output
        .try_into()
        .map_err(|_| PreparedError::Invalid("LOD cardinality"))
}

fn expand(grid: Mesh, records: RecordSpan, stride: u32) -> Result<AddressMesh> {
    let Some(VertexAttributeValues::Float32x3(positions)) =
        grid.attribute(Mesh::ATTRIBUTE_POSITION)
    else {
        return Err(PreparedError::Invalid("address grid positions"));
    };
    let Some(Indices::U32(triangles)) = grid.indices() else {
        return Err(PreparedError::Invalid("address grid indices"));
    };
    if u64::from(records.start + records.count) * u64::from(stride) > EXACT_ADDRESS {
        return Err(PreparedError::Invalid(
            "GPU address exceeds exact f32 integer range",
        ));
    }
    let mut vertices = Vec::with_capacity(positions.len() * records.count as usize);
    let mut indices = Vec::with_capacity(triangles.len() * records.count as usize);
    for record in records.start..records.start + records.count {
        let base = u32::try_from(vertices.len())
            .map_err(|_| PreparedError::Invalid("mesh vertex index overflow"))?;
        vertices.extend(
            positions
                .iter()
                .map(|&[x, y, z]| [x, y, z + (record * stride) as f32]),
        );
        indices.extend(triangles.iter().map(|&index| base + index));
    }
    Ok(AddressMesh { vertices, indices })
}
