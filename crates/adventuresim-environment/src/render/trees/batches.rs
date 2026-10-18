//! Spatial batches retain whole-tree mesh reuse and multi-tree draw instancing.

use std::{collections::BTreeMap, sync::Arc};

use bevy::camera::{primitives::Aabb, visibility::NoFrustumCulling};
use bevy::prelude::*;
use bevy_eidolon::prelude::*;
use fabelgeist_math::units::Metres;

use super::super::{
    ForestDetail,
    benchmark::{ForestBenchmark, ForestPart},
    chunks::{CHUNK, FAR, size_class},
    levels::{Level, LevelView},
};
use super::{SharedForest, upload::TreeAssets};

#[derive(Component)]
pub(super) struct TreeChunk {
    variant: usize,
    foliage: bool,
    radius: f32,
    level: Level,
}

struct Gathering {
    instances: Vec<InstanceData>,
    low: Vec3,
    high: Vec3,
    radius: f32,
}

pub(super) fn show(
    commands: &mut Commands,
    forest: &SharedForest,
    assets: &TreeAssets,
    view: &LevelView,
    detail: &ForestDetail,
    probe: Option<&ForestBenchmark>,
) {
    let mut batches = BTreeMap::new();
    for (index, placement) in forest.population.placements.iter().enumerate() {
        let variant = placement.variant.index();
        let model = &forest.library.models[variant];
        let bounds = model.bounds.transformed(placement.pose);
        let low = Vec3::from_array(bounds.low.map(Metres::metres));
        let high = Vec3::from_array(bounds.high.map(Metres::metres));
        let center = (low + high) * 0.5;
        let radius = (high - low).length() * 0.5;
        let root = Vec3::from_array(placement.pose.origin().map(Metres::metres));
        let cell = (root.xz() / CHUNK).floor().as_ivec2();
        for (foliage, count) in [(false, model.branches.count), (true, model.leaves.count)] {
            if count == 0 {
                continue;
            }
            let batch = batches
                .entry((variant, cell.x, cell.y, size_class(radius), foliage))
                .or_insert_with(|| Gathering {
                    instances: Vec::new(),
                    low,
                    high,
                    radius,
                });
            batch.low = batch.low.min(low);
            batch.high = batch.high.max(high);
            batch.radius = batch.radius.max(radius);
            // Only GPU culling reads this centre/radius. Geometry reads the
            // separate placement buffer by index, never this scalar transform.
            batch.instances.push(InstanceData {
                position: center,
                scale: radius,
                index: index as u32,
                ..default()
            });
        }
    }
    for ((variant, _, _, _, foliage), gathered) in batches {
        let aabb = Aabb::from_min_max(gathered.low, gathered.high);
        let level = view.level(&aabb, gathered.radius, Level::Coarse, detail);
        let chunk = TreeChunk {
            variant,
            foliage,
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
                if foliage {
                    ForestPart::Leaves
                } else {
                    ForestPart::Branches
                },
                level,
            );
        }
        entity.insert(chunk);
    }
}

pub(super) fn pick(
    mut commands: Commands,
    detail: Res<ForestDetail>,
    view: Res<LevelView>,
    assets: Option<Res<TreeAssets>>,
    mut chunks: Query<(Entity, &mut TreeChunk, &Aabb, &mut Visibility)>,
    probe: Option<Res<ForestBenchmark>>,
) {
    let Some(assets) = assets else {
        return;
    };
    for (entity, mut chunk, aabb, mut visibility) in &mut chunks {
        *visibility = if chunk.foliage && !detail.leaves {
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
    let level = &assets.models[chunk.variant][chunk.level.index()];
    if chunk.foliage {
        let Some(mesh) = &level.leaf_mesh else {
            error!("tree batch requested absent foliage geometry");
            return;
        };
        entity.insert((
            Mesh3d(mesh.clone()),
            InstancedMeshMaterial(level.leaves.clone()),
        ));
    } else {
        let Some(mesh) = &level.branch_mesh else {
            error!("tree batch requested absent branch geometry");
            return;
        };
        entity.insert((
            Mesh3d(mesh.clone()),
            InstancedMeshMaterial(level.branches.clone()),
        ));
    }
}
