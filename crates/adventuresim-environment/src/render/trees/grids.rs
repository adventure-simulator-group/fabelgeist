//! Controlled indexed padding: identical model meshes with full or live indices.

use std::collections::BTreeMap;

use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use serde::Serialize;

use super::VERTEX_ADDRESS;
use crate::prepared::{
    AddressMesh, MeshClass, PreparedError, Result, TreeGrid, TreeLibrary, VariantIndex,
    admit_address_mesh_bytes,
};
use crate::render::{ForestPart, benchmark::ForestBenchmark};

/// Default reuse, or matched per-model mesh/batch partitions for diagnostics.
#[derive(Resource, Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "viewer", derive(clap::ValueEnum))]
pub enum TreeLayout {
    #[default]
    CapacityBatches,
    ModelPadded,
    ModelLiveIndices,
}

/// Requested grid payload, excluding common primitives and allocator overhead.
#[derive(Resource, Clone, Copy, Debug, Serialize)]
pub struct TreeMeshStats {
    pub layout: TreeLayout,
    pub grid_meshes: usize,
    pub vertex_bytes: usize,
    pub index_bytes: usize,
    /// SHA-256 of batch keys, bounds and complete instance payloads.
    pub batch_digest: [u8; 32],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub(super) enum GridKey {
    Capacity(MeshClass),
    Model {
        variant: VariantIndex,
        class: MeshClass,
    },
}

pub(super) struct UploadedGrids {
    pub handles: BTreeMap<GridKey, [Handle<Mesh>; 3]>,
    pub stats: TreeMeshStats,
}

struct GridPlan<'a> {
    key: GridKey,
    grid: &'a TreeGrid,
    indexed_records: u32,
}

impl TreeLayout {
    pub(super) fn key(self, variant: VariantIndex, class: MeshClass) -> GridKey {
        match self {
            Self::CapacityBatches => GridKey::Capacity(class),
            Self::ModelPadded | Self::ModelLiveIndices => GridKey::Model { variant, class },
        }
    }

    pub(super) fn indexed_records(self, class: MeshClass, live: u32) -> u32 {
        match self {
            Self::ModelLiveIndices => live,
            Self::CapacityBatches | Self::ModelPadded => class.capacity.records(),
        }
    }
}

impl GridKey {
    pub(super) fn class(self) -> MeshClass {
        match self {
            Self::Capacity(class) | Self::Model { class, .. } => class,
        }
    }
}

impl GridPlan<'_> {
    fn index_count(&self, addresses: &AddressMesh) -> Result<usize> {
        let capacity = self.grid.class.capacity.records() as usize;
        if self.indexed_records == 0
            || self.indexed_records as usize > capacity
            || !addresses.indices.len().is_multiple_of(capacity)
        {
            return Err(PreparedError::MeshCardinality);
        }
        (addresses.indices.len() / capacity)
            .checked_mul(self.indexed_records as usize)
            .ok_or(PreparedError::MeshCardinality)
    }
}

pub(super) fn upload(
    library: &TreeLibrary,
    layout: TreeLayout,
    meshes: &mut Assets<Mesh>,
    probe: Option<&ForestBenchmark>,
) -> Result<UploadedGrids> {
    let plans = plan(library, layout)?;
    let mut stats = TreeMeshStats {
        layout,
        grid_meshes: plans
            .len()
            .checked_mul(3)
            .ok_or(PreparedError::MeshCardinality)?,
        vertex_bytes: 0,
        index_bytes: 0,
        batch_digest: [0; 32],
    };
    for plan in &plans {
        for addresses in &plan.grid.levels {
            stats.vertex_bytes = stats
                .vertex_bytes
                .checked_add(word_bytes(addresses.vertices.len())?)
                .ok_or(PreparedError::MeshCardinality)?;
            stats.index_bytes = stats
                .index_bytes
                .checked_add(word_bytes(plan.index_count(addresses)?)?)
                .ok_or(PreparedError::MeshCardinality)?;
        }
    }
    let source_bytes = library.mesh_bytes();
    let bytes = (source_bytes as u64)
        .checked_add(stats.vertex_bytes as u64)
        .and_then(|bytes| bytes.checked_add(stats.index_bytes as u64))
        .ok_or(PreparedError::MeshCardinality)?;
    admit_address_mesh_bytes(bytes)?;
    let mut handles = BTreeMap::new();
    for plan in plans {
        let mut levels = Vec::new();
        for addresses in &plan.grid.levels {
            let count = plan.index_count(addresses)?;
            let mesh = meshes.add(
                Mesh::new(
                    PrimitiveTopology::TriangleList,
                    RenderAssetUsages::RENDER_WORLD,
                )
                .with_inserted_attribute(VERTEX_ADDRESS, addresses.vertices.clone())
                .with_inserted_indices(Indices::U32(addresses.indices[..count].to_vec())),
            );
            if let Some(probe) = probe {
                probe.expect_mesh(mesh.id(), addresses.vertices.len(), count);
            }
            levels.push(mesh);
        }
        handles.insert(
            plan.key,
            levels
                .try_into()
                .map_err(|_| PreparedError::MeshCardinality)?,
        );
    }
    Ok(UploadedGrids { handles, stats })
}

fn plan(library: &TreeLibrary, layout: TreeLayout) -> Result<Vec<GridPlan<'_>>> {
    if layout == TreeLayout::CapacityBatches {
        return Ok(library
            .grids
            .iter()
            .map(|grid| GridPlan {
                key: GridKey::Capacity(grid.class),
                grid,
                indexed_records: grid.class.capacity.records(),
            })
            .collect());
    }
    let grids: BTreeMap<_, _> = library
        .grids
        .iter()
        .map(|grid| (grid.class, grid))
        .collect();
    let mut plans = Vec::new();
    for variant in library.variant_indices() {
        let model = &library.models[variant.index()];
        for (part, span) in [
            (ForestPart::Branches, model.branches),
            (ForestPart::Leaves, model.leaves),
        ] {
            let Some(class) = MeshClass::for_records(part, span.count)? else {
                continue;
            };
            let grid = grids
                .get(&class)
                .ok_or(PreparedError::Invalid("missing diagnostic source grid"))?;
            plans.push(GridPlan {
                key: layout.key(variant, class),
                grid,
                indexed_records: layout.indexed_records(class, span.count),
            });
        }
    }
    Ok(plans)
}

fn word_bytes(words: usize) -> Result<usize> {
    words
        .checked_mul(size_of::<u32>())
        .ok_or(PreparedError::MeshCardinality)
}
