//! Splits a batch's instances into ground chunks and spawns one culled entity
//! per chunk.
use std::{collections::BTreeMap, sync::Arc};

use bevy::{
    camera::{
        primitives::Aabb,
        visibility::{NoAutoAabb, VisibilityRange},
    },
    light::NotShadowCaster,
    prelude::*,
};

use super::{ChunkInstance, ChunkInstancedMaterial, ChunkMaterial, FadeBand, InstanceChunk};

/// Every instance of one mesh and material within one distance band.
pub(in crate::presentation) struct InstanceBatch<M: ChunkInstancedMaterial> {
    pub mesh: Handle<Mesh>,
    pub material: Handle<M>,
    pub instances: Vec<ChunkInstance>,
    pub band: FadeBand,
    pub reach: InstanceReach,
    pub chunk_size: ChunkSize,
}

/// How far a unit-scale instance's geometry extends from its root, in
/// metres. Bounds scale with the largest instance in each chunk.
#[derive(Clone, Copy, Debug)]
pub(in crate::presentation) struct InstanceReach {
    pub horizontal: f32,
    pub above: f32,
    pub below: f32,
}

/// The edge of the square ground cell one chunk entity covers, in metres.
/// Larger cells mean fewer entities to cull; smaller cells cull tighter.
#[derive(Clone, Copy, Debug)]
pub(in crate::presentation) struct ChunkSize(pub f32);

/// The instances of one ground cell.
struct Cell {
    instances: Vec<ChunkInstance>,
}

impl<M: ChunkInstancedMaterial> InstanceBatch<M> {
    /// Spawns one entity per occupied ground cell. `components` adds the
    /// caller's markers to each chunk, given that chunk's instances.
    pub(in crate::presentation) fn spawn<B: Bundle>(
        self,
        commands: &mut Commands,
        mut components: impl FnMut(&[ChunkInstance]) -> B,
    ) {
        let Self {
            mesh,
            material,
            instances,
            band,
            reach,
            chunk_size,
        } = self;
        let mut cells = BTreeMap::<(i32, i32), Cell>::new();
        for instance in instances {
            let key = (
                (instance.position.x / chunk_size.0).floor() as i32,
                (instance.position.z / chunk_size.0).floor() as i32,
            );
            cells
                .entry(key)
                .or_insert_with(|| Cell {
                    instances: Vec::new(),
                })
                .instances
                .push(instance);
        }
        for cell in cells.into_values() {
            let bounds = cell.bounds(reach);
            let centre = Vec3::from(bounds.center);
            // Bevy measures the range from the entity's translation, so pad
            // the band by the chunk's half-diagonal: no instance inside the
            // band is culled with its chunk. Each instance dithers its own fade.
            let pad = Vec3::from(bounds.half_extents).length();
            let start = (band.fade_in().start - pad).max(0.0);
            let end = band.fade_out().end + pad;
            let extra = components(&cell.instances);
            commands.spawn((
                Mesh3d(mesh.clone()),
                ChunkMaterial(material.clone()),
                InstanceChunk {
                    instances: Arc::from(cell.instances),
                    band: band.clone(),
                },
                Transform::from_translation(centre),
                Visibility::Inherited,
                Aabb {
                    center: Vec3::ZERO.into(),
                    half_extents: bounds.half_extents,
                },
                // Bevy otherwise replaces the chunk bounds with the mesh's.
                NoAutoAabb,
                VisibilityRange {
                    start_margin: start..start,
                    end_margin: end..end,
                    use_aabb: false,
                },
                NotShadowCaster,
                extra,
            ));
        }
    }
}

impl Cell {
    fn bounds(&self, reach: InstanceReach) -> Aabb {
        let mut minimum = Vec3::splat(f32::INFINITY);
        let mut maximum = Vec3::splat(f32::NEG_INFINITY);
        let mut largest_scale = 1.0_f32;
        for instance in &self.instances {
            minimum = minimum.min(instance.position);
            maximum = maximum.max(instance.position);
            largest_scale = largest_scale.max(instance.scale);
        }
        let horizontal = reach.horizontal * largest_scale;
        let minimum = minimum - Vec3::new(horizontal, reach.below * largest_scale, horizontal);
        let maximum = maximum + Vec3::new(horizontal, reach.above * largest_scale, horizontal);
        Aabb::from_min_max(minimum, maximum)
    }
}
