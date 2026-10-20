//! Canonical address topology derived only at an actual mesh consumer.

use std::collections::BTreeSet;

use bevy::{mesh::Indices, prelude::Mesh};

use super::{AddressMesh, MeshClass, PreparedError, RecordCount, Result, TreeModel};
use crate::render::{
    ForestDetail, ForestPart,
    levels::{Level, Shape},
    shapes,
};

/// Prospective address topology admission; not physical RAM, VRAM or wire bytes.
const MAX_MESH_BYTES: u64 = 256 * 1024 * 1024;

#[derive(Clone, Copy, Debug)]
pub struct CanonicalTopologyBytes(u64);

/// Derived class set and total all-level topology, with no retained mesh arrays.
pub struct TopologyRequirements {
    classes: BTreeSet<MeshClass>,
    bytes: CanonicalTopologyBytes,
}

pub(crate) struct GridCardinality {
    pub vertices: u32,
    pub indices: u32,
}

impl CanonicalTopologyBytes {
    /// Numerical report boundary, in bytes of canonical u32 address/index words.
    pub fn bytes(self) -> u64 {
        self.0
    }
}

impl TopologyRequirements {
    pub(super) fn from_models(models: &[TreeModel], detail: &ForestDetail) -> Result<Self> {
        super::validation::detail(detail)?;
        let classes = classes(models)?;
        let mut bytes = 0_u64;
        for &class in &classes {
            for level in Level::ALL {
                let counts = class.cardinality(&level.shape(detail))?;
                bytes = bytes
                    .checked_add(counts.bytes())
                    .ok_or(PreparedError::Invalid("mesh size overflow"))?;
            }
        }
        admit_address_mesh_bytes(bytes)?;
        Ok(Self {
            classes,
            bytes: CanonicalTopologyBytes(bytes),
        })
    }

    pub fn classes(&self) -> impl ExactSizeIterator<Item = MeshClass> + '_ {
        self.classes.iter().copied()
    }

    pub fn projected_bytes(&self) -> CanonicalTopologyBytes {
        self.bytes
    }
}

impl GridCardinality {
    pub(crate) fn bytes(&self) -> u64 {
        (u64::from(self.vertices) + u64::from(self.indices)) * size_of::<u32>() as u64
    }
}

impl MeshClass {
    fn cardinality(self, shape: &Shape) -> Result<GridCardinality> {
        cardinality(self.part, shape, self.capacity.records())
    }

    pub(crate) fn mesh_counts(
        self,
        level: Level,
        detail: &ForestDetail,
        indexed_records: RecordCount,
    ) -> Result<GridCardinality> {
        super::validation::detail(detail)?;
        if indexed_records.records() > self.capacity.records() {
            return Err(PreparedError::MeshCardinality);
        }
        let shape = level.shape(detail);
        let capacity = self.cardinality(&shape)?;
        let indexed = cardinality(self.part, &shape, indexed_records.records())?;
        let counts = GridCardinality {
            vertices: capacity.vertices,
            indices: indexed.indices,
        };
        admit_address_mesh_bytes(counts.bytes())?;
        Ok(counts)
    }

    /// Capacity vertex addresses and an exact complete-record index prefix.
    /// Private shape parameters are derived from the public level/detail contract.
    pub fn address_mesh(
        self,
        level: Level,
        detail: &ForestDetail,
        indexed_records: RecordCount,
    ) -> Result<AddressMesh> {
        let counts = self.mesh_counts(level, detail, indexed_records)?;
        let indices = index_prefix(self.part, level, detail, indexed_records)?;
        Ok(AddressMesh {
            vertices: (0..counts.vertices).collect(),
            indices,
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

pub(crate) fn admit_address_mesh_bytes(bytes: u64) -> Result<()> {
    if bytes > MAX_MESH_BYTES {
        return Err(PreparedError::Invalid(
            "address meshes exceed prototype's 256 MiB preparation limit",
        ));
    }
    Ok(())
}

/// Native grown meshes need only live triangles, never padded address vertices.
pub(super) fn index_prefix(
    part: ForestPart,
    level: Level,
    detail: &ForestDetail,
    records: RecordCount,
) -> Result<Vec<u32>> {
    super::validation::detail(detail)?;
    let shape = level.shape(detail);
    let counts = cardinality(part, &shape, records.records())?;
    admit_address_mesh_bytes(counts.bytes())?;
    let grid = primitive(part, &shape);
    let Some(Indices::U32(triangles)) = grid.indices() else {
        return Err(PreparedError::Invalid("canonical grid indices"));
    };
    let vertices_per_record =
        u32::try_from(grid.count_vertices()).map_err(|_| PreparedError::MeshCardinality)?;
    let mut indices = Vec::with_capacity(counts.indices as usize);
    for record in 0..records.records() {
        indices.extend(
            triangles
                .iter()
                .map(|&index| record * vertices_per_record + index),
        );
    }
    Ok(indices)
}

fn primitive(part: ForestPart, shape: &Shape) -> Mesh {
    match part {
        ForestPart::Branches => shapes::branch_grid(shape.sides, shape.spans),
        ForestPart::Leaves => shapes::leaf_blades(shape.blades),
    }
}

fn cardinality(part: ForestPart, shape: &Shape, records: u32) -> Result<GridCardinality> {
    // Native allocation/kernel boundary. Widen every shape/count product first.
    let (vertices, indices) = match part {
        ForestPart::Branches => (
            (u64::from(shape.sides) + 1) * (u64::from(shape.spans) + 1),
            u64::from(shape.sides) * u64::from(shape.spans) * 6,
        ),
        ForestPart::Leaves => (u64::from(shape.blades) * 4, u64::from(shape.blades) * 6),
    };
    Ok(GridCardinality {
        vertices: u32::try_from(vertices * u64::from(records))
            .map_err(|_| PreparedError::Invalid("grid vertex overflow"))?,
        indices: u32::try_from(indices * u64::from(records))
            .map_err(|_| PreparedError::Invalid("grid index overflow"))?,
    })
}
