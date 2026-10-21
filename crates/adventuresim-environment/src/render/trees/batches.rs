//! Spatial batches retain whole-tree mesh reuse and multi-tree draw instancing.

use std::{collections::BTreeMap, sync::Arc};

use bevy::camera::{primitives::Aabb, visibility::NoFrustumCulling};
use bevy::prelude::*;
use bevy_eidolon::prelude::*;
use fabelgeist_math::units::Metres;
use serde::Serialize;
use sha2::{Digest, Sha256};

use super::super::{
    ForestDetail, ForestPart,
    benchmark::{ForestBenchmark, ForestWork},
    chunks::{FAR, size_class},
    levels::{ForestLodView, Level},
};
use super::{TreeBatchOccupancy, TreeBatching, TreeLayout, grids::GridKey, upload::TreeAssets};
use crate::prepared::{MeshClass, Result, SharedForest};

#[derive(Component)]
pub(super) struct TreeChunk {
    grid: GridKey,
    radius: f32,
    level: Level,
}

struct Gathering {
    instances: Vec<InstanceData>,
    low: Vec3,
    high: Vec3,
    radius: f32,
    records: usize,
    indexed_slots: usize,
}

pub(super) struct SpawnedBatches {
    pub digest: [u8; 32],
    pub occupancy: TreeBatchOccupancy,
}

/// Integer spatial bins at the renderer's world-to-cell boundary.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
struct BatchKey {
    grid: GridKey,
    cell: [i32; 2],
    size_class: usize,
}

/// Native diagnostic wire boundary; bounds and radius are area-world metres.
#[derive(Serialize)]
struct BatchEvidence {
    key: BatchKey,
    low: [f32; 3],
    high: [f32; 3],
    radius: f32,
    instances: usize,
}

pub(super) fn show(
    commands: &mut Commands,
    forest: &SharedForest,
    assets: &TreeAssets,
    view: &ForestLodView,
    detail: &ForestDetail,
    probe: Option<&ForestBenchmark>,
    batching: TreeBatching,
) -> Result<SpawnedBatches> {
    let batches = gather(forest, assets.stats.layout, batching)?;
    let mut occupancy = TreeBatchOccupancy {
        policy: batching,
        ..Default::default()
    };
    let mut digest = Sha256::new();
    for (key, gathered) in batches {
        *occupancy
            .histogram
            .entry(gathered.instances.len())
            .or_default() += 1;
        let aabb = Aabb::from_min_max(gathered.low, gathered.high);
        let level = view.level(&aabb, gathered.radius, Level::Coarse, detail);
        // Metadata gives each variable-length instance stream a boundary.
        digest.update(serde_json::to_vec(&BatchEvidence {
            key,
            low: gathered.low.to_array(),
            high: gathered.high.to_array(),
            radius: gathered.radius,
            instances: gathered.instances.len(),
        })?);
        digest.update(bytemuck::cast_slice(&gathered.instances));
        let chunk = TreeChunk {
            grid: key.grid,
            radius: gathered.radius,
            level,
        };
        let count = gathered.instances.len();
        let mut entity = commands.spawn((
            InstanceMaterialData {
                instances: Arc::new(gathered.instances),
                color: LinearRgba::WHITE,
                visibility_range: Vec4::new(0.0, 0.0, FAR, FAR),
            },
            aabb,
            super::super::ForestGeometry,
            NoFrustumCulling,
            GpuCullCompute,
            Transform::default(),
            Visibility::Inherited,
        ));
        dress(&mut entity, assets, &chunk);
        if let Some(probe) = probe {
            probe.expect_chunk(
                entity.id(),
                count,
                key.grid.class().part,
                level,
                ForestWork {
                    records: gathered.records,
                    indexed_slots: gathered.indexed_slots,
                },
            );
        }
        entity.insert(chunk);
    }
    Ok(SpawnedBatches {
        digest: digest.finalize().into(),
        occupancy,
    })
}

fn gather(
    forest: &SharedForest,
    layout: TreeLayout,
    batching: TreeBatching,
) -> Result<BTreeMap<BatchKey, Gathering>> {
    let mut batches = BTreeMap::new();
    for (index, placement) in forest.population().placements.iter().enumerate() {
        let model = &forest.bank().library().models[placement.variant.index()];
        let bounds = model.bounds.transformed(placement.pose);
        let low = Vec3::from_array(bounds.low.map(Metres::metres));
        let high = Vec3::from_array(bounds.high.map(Metres::metres));
        let center = (low + high) * 0.5;
        // Native gathering uses scalar world metres from the shared source kernel.
        let radius = crate::render::source_bounds_radius(low, high).metres();
        let root = Vec3::from_array(placement.pose.origin().map(Metres::metres));
        // Native vector kernel: area-world root metres to integer spatial bins.
        let cell = (root.xz() / batching.cell_width().metres())
            .floor()
            .as_ivec2();
        for (part, count) in [
            (ForestPart::Branches, model.branches.count),
            (ForestPart::Leaves, model.leaves.count),
        ] {
            let Some(class) = MeshClass::for_records(part, count)? else {
                continue;
            };
            let batch = batches
                .entry(BatchKey {
                    grid: layout.key(placement.variant, class),
                    cell: cell.to_array(),
                    size_class: size_class(radius),
                })
                .or_insert_with(|| Gathering {
                    instances: Vec::new(),
                    low,
                    high,
                    radius,
                    records: 0,
                    indexed_slots: 0,
                });
            batch.low = batch.low.min(low);
            batch.high = batch.high.max(high);
            batch.radius = batch.radius.max(radius);
            batch.records += count as usize;
            batch.indexed_slots += layout.indexed_records(class, count) as usize;
            // GPU distance culling reads this centre; frustum tests use the
            // union batch bounds. Geometry reads the separate placement buffer
            // by index, never this scalar transform.
            batch.instances.push(InstanceData {
                position: center,
                scale: radius,
                index: index as u32,
                ..default()
            });
        }
    }
    Ok(batches)
}

pub(super) fn pick(
    mut commands: Commands,
    detail: Res<ForestDetail>,
    view: Res<ForestLodView>,
    assets: Option<Res<TreeAssets>>,
    mut chunks: Query<(Entity, &mut TreeChunk, &Aabb, &mut Visibility)>,
    probe: Option<Res<ForestBenchmark>>,
) {
    let Some(assets) = assets else {
        return;
    };
    for (entity, mut chunk, aabb, mut visibility) in &mut chunks {
        *visibility = if chunk.grid.class().part == ForestPart::Leaves && !detail.leaves {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        };
        let level = view.level(aabb, chunk.radius, chunk.level, &detail);
        if chunk.level != level {
            chunk.level = level;
            if let Some(probe) = &probe {
                probe.expect_level(entity, level);
            }
            dress(&mut commands.entity(entity), &assets, &chunk);
        }
    }
}

fn dress(entity: &mut EntityCommands, assets: &TreeAssets, chunk: &TreeChunk) {
    let Some(grid) = assets.grids.get(&chunk.grid) else {
        error!("tree batch requested absent capacity grid");
        return;
    };
    let mesh = &grid[chunk.level.index()];
    let materials = &assets.materials[chunk.level.index()];
    if chunk.grid.class().part == ForestPart::Leaves {
        entity.insert((
            Mesh3d(mesh.clone()),
            InstancedMeshMaterial(materials.leaves.clone()),
        ));
    } else {
        entity.insert((
            Mesh3d(mesh.clone()),
            InstancedMeshMaterial(materials.branches.clone()),
        ));
    }
}
