//! Controlled indexed padding: identical model meshes with full or live indices.

use std::collections::BTreeMap;

use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use serde::Serialize;

use super::{TreeBatchOccupancy, TreeResidencyEvidence};
use crate::prepared::{
    MeshClass, PreparedError, RecordCount, Result, TreeLibrary, VariantIndex,
    admit_address_mesh_bytes,
};
use crate::render::address::VERTEX_ADDRESS;
use crate::render::{ForestPart, Level, benchmark::ForestBenchmark};

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
#[derive(Resource, Clone, Debug, Serialize)]
pub struct TreeMeshStats {
    pub layout: TreeLayout,
    pub occupancy: TreeBatchOccupancy,
    pub residency: Option<TreeResidencyEvidence>,
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
    readiness: Vec<MeshReadiness>,
}

struct MeshReadiness {
    mesh: AssetId<Mesh>,
    vertices: usize,
    indices: usize,
}

struct GridPlan {
    key: GridKey,
    class: MeshClass,
    indexed_records: RecordCount,
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

impl UploadedGrids {
    pub(super) fn observe(&self, probe: &ForestBenchmark) {
        for mesh in &self.readiness {
            probe.expect_mesh(mesh.mesh, mesh.vertices, mesh.indices);
        }
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
        occupancy: TreeBatchOccupancy::default(),
        residency: None,
        grid_meshes: plans
            .len()
            .checked_mul(3)
            .ok_or(PreparedError::MeshCardinality)?,
        vertex_bytes: 0,
        index_bytes: 0,
        batch_digest: [0; 32],
    };
    for plan in &plans {
        for level in Level::ALL {
            let counts = plan
                .class
                .mesh_counts(level, &library.detail, plan.indexed_records)?;
            stats.vertex_bytes = stats
                .vertex_bytes
                .checked_add(word_bytes(counts.vertices as usize)?)
                .ok_or(PreparedError::MeshCardinality)?;
            stats.index_bytes = stats
                .index_bytes
                .checked_add(word_bytes(counts.indices as usize)?)
                .ok_or(PreparedError::MeshCardinality)?;
        }
    }
    // All planned allocations are admitted before the first asset insertion.
    // No bank-owned CPU address/index arrays remain to count or clone.
    let bytes = (stats.vertex_bytes as u64)
        .checked_add(stats.index_bytes as u64)
        .ok_or(PreparedError::MeshCardinality)?;
    admit_address_mesh_bytes(bytes)?;
    let mut handles = BTreeMap::new();
    let mut readiness = Vec::new();
    for plan in plans {
        let mut levels = Vec::new();
        for level in Level::ALL {
            let addresses =
                plan.class
                    .address_mesh(level, &library.detail, plan.indexed_records)?;
            let vertices = addresses.vertices.len();
            let count = addresses.indices.len();
            let mesh = meshes.add(
                Mesh::new(
                    PrimitiveTopology::TriangleList,
                    RenderAssetUsages::RENDER_WORLD,
                )
                .with_inserted_attribute(VERTEX_ADDRESS, addresses.vertices)
                .with_inserted_indices(Indices::U32(addresses.indices)),
            );
            if let Some(probe) = probe {
                probe.expect_mesh(mesh.id(), vertices, count);
            }
            readiness.push(MeshReadiness {
                mesh: mesh.id(),
                vertices,
                indices: count,
            });
            levels.push(mesh);
        }
        handles.insert(
            plan.key,
            levels
                .try_into()
                .map_err(|_| PreparedError::MeshCardinality)?,
        );
    }
    Ok(UploadedGrids {
        handles,
        stats,
        readiness,
    })
}

fn plan(library: &TreeLibrary, layout: TreeLayout) -> Result<Vec<GridPlan>> {
    let requirements = library.topology_requirements()?;
    if layout == TreeLayout::CapacityBatches {
        return requirements
            .classes()
            .map(|class| {
                Ok(GridPlan {
                    key: GridKey::Capacity(class),
                    class,
                    indexed_records: RecordCount::from_records(class.capacity.records())?,
                })
            })
            .collect();
    }
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
            plans.push(GridPlan {
                key: layout.key(variant, class),
                class,
                indexed_records: RecordCount::from_records(
                    layout.indexed_records(class, span.count),
                )?,
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
