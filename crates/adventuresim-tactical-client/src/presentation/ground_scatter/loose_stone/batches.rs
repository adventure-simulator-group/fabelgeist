//! Instanced batch entities for loose-stone pebble patches.
//!
//! The per-entity renderer spawned two or three entities for every qualifying
//! ground sample - one per square metre over the whole playable scene. These
//! batches carry the same placements as `InstanceData` instead, so the ECS
//! holds one entity per (patch mesh, LOD) rather than tens of thousands, and
//! the per-instance distance cull runs on the GPU.

use std::sync::Arc;

use bevy::{
    camera::{primitives::Aabb, visibility::NoFrustumCulling},
    color::LinearRgba,
    ecs::system::EntityCommands,
    prelude::{Commands, Handle, Mesh, Mesh3d, Name, Transform, Vec4, Visibility},
};
use bevy_eidolon::prelude::*;

use super::{
    LooseStonePebblePatch, MESH_VARIANTS, PEBBLE_BATCH_FOOTPRINT_METRES, PebbleMeshLod,
    TacticalPebbleBillboardMaterial, TacticalPebbleMaterial, pebble_lod_visibility,
};
use crate::presentation::ground_scatter::GroundScatterLayer;

/// The shared meshes and materials every pebble batch draws from.
pub(super) struct PebbleBatchAssets<'a> {
    pub(super) hero: &'a [Handle<Mesh>],
    pub(super) near: &'a [Handle<Mesh>],
    pub(super) billboard: &'a [Handle<Mesh>],
    pub(super) pebble_counts: &'a [usize],
    pub(super) stone: Handle<TacticalPebbleMaterial>,
    pub(super) woodland_stone: Handle<TacticalPebbleMaterial>,
    pub(super) billboard_material: Handle<TacticalPebbleBillboardMaterial>,
}

pub(super) fn spawn(
    commands: &mut Commands<'_, '_>,
    assets: PebbleBatchAssets<'_>,
    mut per_variant: Vec<Vec<InstanceData>>,
) {
    for variant in 0..per_variant.len() {
        let instances = Arc::new(std::mem::take(&mut per_variant[variant]));
        if instances.is_empty() {
            continue;
        }
        let woodland = variant < MESH_VARIANTS as usize;
        let aabb = crate::presentation::ground_scatter::instanced_grass::fitted_batch_aabb(
            &instances,
            PEBBLE_BATCH_FOOTPRINT_METRES,
        );
        let stone = if woodland {
            assets.woodland_stone.clone()
        } else {
            assets.stone.clone()
        };
        // Only the hero batch reports a pebble count: the three LODs draw the
        // same placements, so summing all of them would treble the scene total
        // the review viewer reads back.
        spawn_batch(
            commands,
            PebbleBatch {
                name: if woodland {
                    "Instanced woodland hero pebble patches"
                } else {
                    "Instanced loose-stone hero pebble patches"
                },
                mesh: assets.hero[variant].clone(),
                lod: PebbleMeshLod::Hero,
                instances: instances.clone(),
                aabb,
                physical_pebbles: assets.pebble_counts[variant] * instances.len(),
            },
        )
        .insert(InstancedMeshMaterial(stone.clone()));
        spawn_batch(
            commands,
            PebbleBatch {
                name: if woodland {
                    "Instanced woodland near pebble patches"
                } else {
                    "Instanced loose-stone near pebble patches"
                },
                mesh: assets.near[variant].clone(),
                lod: PebbleMeshLod::Near,
                instances: instances.clone(),
                aabb,
                physical_pebbles: 0,
            },
        )
        .insert(InstancedMeshMaterial(stone));
        if !woodland {
            spawn_batch(
                commands,
                PebbleBatch {
                    name: "Instanced loose-stone billboard pebble patches",
                    mesh: assets.billboard[variant].clone(),
                    lod: PebbleMeshLod::Billboard,
                    instances: instances.clone(),
                    aabb,
                    physical_pebbles: 0,
                },
            )
            .insert(InstancedMeshMaterial(assets.billboard_material.clone()));
        }
    }
}

struct PebbleBatch {
    name: &'static str,
    mesh: Handle<Mesh>,
    lod: PebbleMeshLod,
    instances: Arc<Vec<InstanceData>>,
    aabb: Aabb,
    physical_pebbles: usize,
}

fn spawn_batch<'a>(commands: &'a mut Commands<'_, '_>, batch: PebbleBatch) -> EntityCommands<'a> {
    let count = batch.instances.len();
    commands.spawn((
        Name::new(format!("{} ({count})", batch.name)),
        GroundScatterLayer::LooseStone,
        LooseStonePebblePatch {
            physical_pebbles: batch.physical_pebbles,
        },
        GpuCullCompute,
        // Whole-scene batches: culling belongs to the GPU compute pass, and CPU
        // frustum culling would free the retained instance buffers (see the
        // grass batches).
        NoFrustumCulling,
        Mesh3d(batch.mesh),
        batch.aabb,
        InstanceMaterialData {
            instances: batch.instances,
            color: LinearRgba::WHITE,
            visibility_range: band(batch.lod),
        },
        Transform::default(),
        Visibility::Inherited,
    ))
}

/// The eidolon fade band (`xy` fade-in, `zw` fade-out) for one pebble LOD,
/// taken from the distances the per-entity `VisibilityRange` used.
fn band(lod: PebbleMeshLod) -> Vec4 {
    let range = pebble_lod_visibility(lod);
    Vec4::new(
        range.start_margin.start,
        range.start_margin.end,
        range.end_margin.start,
        range.end_margin.end,
    )
}
