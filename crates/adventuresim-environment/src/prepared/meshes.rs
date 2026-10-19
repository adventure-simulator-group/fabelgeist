//! Canonical indexed integer grids shared by model capacity and LOD.

use std::collections::BTreeSet;

use bevy::{mesh::Indices, prelude::Mesh};

use super::{AddressMesh, MeshClass, PreparedError, Result, TreeGrid, TreeModel};
use crate::render::{
    ForestDetail, ForestPart,
    levels::{Level, Shape},
    shapes,
};

/// Bound offline address payload before allocating any expanded grids.
const MAX_MESH_BYTES: u64 = 256 * 1024 * 1024;

pub(super) struct GridCardinality {
    pub vertices: u32,
    pub indices: u32,
}

impl MeshClass {
    fn primitive(self, shape: &Shape) -> Mesh {
        match self.part {
            ForestPart::Branches => shapes::branch_grid(shape.sides, shape.spans),
            ForestPart::Leaves => shapes::leaf_blades(shape.blades),
        }
    }
    pub(super) fn cardinality(self, shape: &Shape) -> Result<GridCardinality> {
        // GPU/file count boundary. Arithmetic is widened before multiplication.
        let (vertices, indices) = match self.part {
            ForestPart::Branches => (
                (u64::from(shape.sides) + 1) * (u64::from(shape.spans) + 1),
                u64::from(shape.sides) * u64::from(shape.spans) * 6,
            ),
            ForestPart::Leaves => (u64::from(shape.blades) * 4, u64::from(shape.blades) * 6),
        };
        let vertices = vertices * u64::from(self.capacity.records());
        let indices = indices * u64::from(self.capacity.records());
        Ok(GridCardinality {
            vertices: u32::try_from(vertices)
                .map_err(|_| PreparedError::Invalid("grid vertex overflow"))?,
            indices: u32::try_from(indices)
                .map_err(|_| PreparedError::Invalid("grid index overflow"))?,
        })
    }
}

pub(super) fn classes(models: &[TreeModel]) -> Result<BTreeSet<MeshClass>> {
    let mut classes = BTreeSet::new();
    for model in models {
        for (part, span) in [
            (ForestPart::Branches, model.branches),
            (ForestPart::Leaves, model.leaves),
        ] {
            if let Some(class) = MeshClass::for_records(part, span.count)? {
                classes.insert(class);
            }
        }
    }
    Ok(classes)
}

pub(super) fn preflight(classes: &BTreeSet<MeshClass>, detail: &ForestDetail) -> Result<()> {
    super::validation::detail(detail)?;
    let mut bytes = 0_u64;
    for &class in classes {
        for level in Level::ALL {
            let counts = class.cardinality(&level.shape(detail))?;
            bytes = bytes
                .checked_add(
                    (u64::from(counts.vertices) + u64::from(counts.indices))
                        * size_of::<u32>() as u64,
                )
                .ok_or(PreparedError::Invalid("mesh size overflow"))?;
        }
    }
    admit_address_mesh_bytes(bytes)
}

pub(crate) fn admit_address_mesh_bytes(bytes: u64) -> Result<()> {
    if bytes > MAX_MESH_BYTES {
        return Err(PreparedError::Invalid(
            "address meshes exceed prototype's 256 MiB preparation limit",
        ));
    }
    Ok(())
}

pub(super) fn build(models: &[TreeModel], detail: &ForestDetail) -> Result<Vec<TreeGrid>> {
    let classes = classes(models)?;
    preflight(&classes, detail)?;
    classes
        .into_iter()
        .map(|class| {
            let levels = Level::ALL
                .into_iter()
                .map(|level| expand(class, &level.shape(detail)))
                .collect::<Result<Vec<_>>>()?;
            Ok(TreeGrid {
                class,
                levels: levels
                    .try_into()
                    .map_err(|_| PreparedError::Invalid("LOD cardinality"))?,
            })
        })
        .collect()
}

pub(super) fn expand(class: MeshClass, shape: &Shape) -> Result<AddressMesh> {
    let counts = class.cardinality(shape)?;
    let grid = class.primitive(shape);
    let Some(Indices::U32(triangles)) = grid.indices() else {
        return Err(PreparedError::Invalid("canonical grid indices"));
    };
    let vertices_per_record = grid.count_vertices() as u32;
    let indices = (0..class.capacity.records())
        .flat_map(|record| {
            triangles
                .iter()
                .map(move |&index| record * vertices_per_record + index)
        })
        .collect();
    Ok(AddressMesh {
        vertices: (0..counts.vertices).collect(),
        indices,
    })
}

pub(super) fn validate(class: MeshClass, mesh: &AddressMesh, shape: &Shape) -> Result<()> {
    let counts = class.cardinality(shape)?;
    if mesh.vertices.len() as u64 != u64::from(counts.vertices)
        || mesh.indices.len() as u64 != u64::from(counts.indices)
    {
        return Err(PreparedError::MeshCardinality);
    }
    if !mesh
        .vertices
        .iter()
        .enumerate()
        .all(|(index, &address)| index as u64 == u64::from(address))
    {
        return Err(PreparedError::Invalid("noncanonical GPU vertex address"));
    }
    let grid = class.primitive(shape);
    let Some(Indices::U32(triangles)) = grid.indices() else {
        return Err(PreparedError::Invalid("canonical grid indices"));
    };
    let vertices_per_record = grid.count_vertices() as u64;
    for (index, &actual) in mesh.indices.iter().enumerate() {
        let record = index / triangles.len();
        let expected =
            u64::from(triangles[index % triangles.len()]) + record as u64 * vertices_per_record;
        if u64::from(actual) != expected {
            return Err(PreparedError::Invalid(
                "noncanonical address mesh triangles",
            ));
        }
    }
    Ok(())
}
