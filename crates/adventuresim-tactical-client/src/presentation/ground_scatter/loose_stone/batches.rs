//! Chunk-instanced batches for loose-stone pebble patches.
//!
//! The per-entity renderer spawned two or three entities for every qualifying
//! ground sample - one per square metre over the whole playable scene. These
//! batches carry the same placements as `ChunkInstance`s instead, so the ECS
//! holds one entity per (patch mesh, LOD, ground cell) rather than tens of
//! thousands.

use bevy::prelude::{Commands, Handle, Mesh, Name};

use super::{
    LooseStonePebblePatch, MESH_VARIANTS, PEBBLE_BATCH_FOOTPRINT_METRES, PebbleMeshLod,
    TacticalPebbleBillboardMaterial, TacticalPebbleMaterial, pebble_lod_visibility,
};
use crate::presentation::{
    chunk_instancing::{ChunkInstance, ChunkInstancedMaterial, ChunkSize, FadeBand, InstanceBatch},
    ground_scatter::{GroundScatterLayer, instanced_grass::ground_scatter_reach},
};

/// Pebbles reach at most ~180 m as billboards; the dense hero and near bands
/// end within 24 m, so mid-sized cells keep their culling tight.
const PEBBLE_CHUNK_SIZE: ChunkSize = ChunkSize(24.0);

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

/// One LOD of one patch variant.
struct PebbleBatch<M: ChunkInstancedMaterial> {
    name: &'static str,
    mesh: Handle<Mesh>,
    material: Handle<M>,
    lod: PebbleMeshLod,
    /// Pebbles per placed patch, or zero for LODs that must not be counted.
    pebbles_per_patch: usize,
}

pub(super) fn spawn(
    commands: &mut Commands<'_, '_>,
    assets: PebbleBatchAssets<'_>,
    per_variant: Vec<Vec<ChunkInstance>>,
) {
    for (variant, instances) in per_variant.into_iter().enumerate() {
        if instances.is_empty() {
            continue;
        }
        let woodland = variant < MESH_VARIANTS as usize;
        let stone = if woodland {
            assets.woodland_stone.clone()
        } else {
            assets.stone.clone()
        };
        // Only the hero batch reports a pebble count: the three LODs draw the
        // same placements, so summing all of them would treble the scene total
        // the review viewer reads back.
        PebbleBatch {
            name: if woodland {
                "Instanced woodland hero pebble patches"
            } else {
                "Instanced loose-stone hero pebble patches"
            },
            mesh: assets.hero[variant].clone(),
            material: stone.clone(),
            lod: PebbleMeshLod::Hero,
            pebbles_per_patch: assets.pebble_counts[variant],
        }
        .spawn(commands, &instances);
        PebbleBatch {
            name: if woodland {
                "Instanced woodland near pebble patches"
            } else {
                "Instanced loose-stone near pebble patches"
            },
            mesh: assets.near[variant].clone(),
            material: stone,
            lod: PebbleMeshLod::Near,
            pebbles_per_patch: 0,
        }
        .spawn(commands, &instances);
        if !woodland {
            PebbleBatch {
                name: "Instanced loose-stone billboard pebble patches",
                mesh: assets.billboard[variant].clone(),
                material: assets.billboard_material.clone(),
                lod: PebbleMeshLod::Billboard,
                pebbles_per_patch: 0,
            }
            .spawn(commands, &instances);
        }
    }
}

impl<M: ChunkInstancedMaterial> PebbleBatch<M> {
    fn spawn(self, commands: &mut Commands<'_, '_>, instances: &[ChunkInstance]) {
        let range = pebble_lod_visibility(self.lod);
        let Self {
            name,
            pebbles_per_patch,
            ..
        } = self;
        InstanceBatch {
            mesh: self.mesh,
            material: self.material,
            instances: instances.to_vec(),
            band: FadeBand::new(range.start_margin, range.end_margin),
            reach: ground_scatter_reach(PEBBLE_BATCH_FOOTPRINT_METRES),
            chunk_size: PEBBLE_CHUNK_SIZE,
        }
        .spawn(commands, |chunk| {
            (
                Name::new(format!("{name} ({})", chunk.len())),
                GroundScatterLayer::LooseStone,
                LooseStonePebblePatch {
                    physical_pebbles: pebbles_per_patch * chunk.len(),
                },
            )
        });
    }
}
