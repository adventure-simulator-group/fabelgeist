//! Main-world data of chunk-instanced batches: the per-instance record, the
//! distance band its shader dithers within, and the chunk entity components.
use std::{ops::Range, sync::Arc};

use bevy::{
    ecs::query::QueryItem,
    prelude::*,
    render::{extract_component::ExtractComponent, sync_component::SyncComponent},
};
use bytemuck::{Pod, Zeroable};

use super::ChunkInstancedMaterial;

/// One placed copy of a batch's mesh in world space. The layout is the
/// `@location(8..10)` instance stream of `chunk_instancing.wgsl`.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Pod, Zeroable)]
pub(crate) struct ChunkInstance {
    pub position: Vec3,
    pub scale: f32,
    /// Yaw about +Y, in radians.
    pub rotation: f32,
    /// Content-defined bits; grass keeps placement coverage in the low byte.
    pub seed: u32,
}

/// The camera-distance band, in metres, an instance draws in. It dithers in
/// over `fade_in` and out over `fade_out` per instance, so bands that share
/// endpoints hand each pixel to exactly one of them.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct FadeBand {
    fade_in: Range<f32>,
    fade_out: Range<f32>,
}

/// One chunk of a batch: the instances inside one ground cell, drawn with
/// the entity's `Mesh3d` and [`ChunkMaterial`].
#[derive(Component, Clone, Debug)]
pub(crate) struct InstanceChunk {
    pub instances: Arc<[ChunkInstance]>,
    pub band: FadeBand,
}

/// The chunk-instanced material a chunk entity draws with.
#[derive(Component, Debug)]
pub(in crate::presentation) struct ChunkMaterial<M: ChunkInstancedMaterial>(pub Handle<M>);

impl FadeBand {
    /// The width every always-visible fade-in keeps: Bevy's dither-level
    /// mapping divides by the margin width, so it must never be zero.
    const OPEN_START_METRES: f32 = 0.001;

    pub(in crate::presentation) fn new(fade_in: Range<f32>, fade_out: Range<f32>) -> Self {
        let fade_in = fade_in.start..fade_in.end.max(fade_in.start + Self::OPEN_START_METRES);
        Self { fade_in, fade_out }
    }

    /// A band visible from the camera out to `fade_out`.
    pub(in crate::presentation) fn until(fade_out: Range<f32>) -> Self {
        Self::new(0.0..0.0, fade_out)
    }

    pub(in crate::presentation) fn fade_in(&self) -> Range<f32> {
        self.fade_in.clone()
    }

    pub(in crate::presentation) fn fade_out(&self) -> Range<f32> {
        self.fade_out.clone()
    }

    /// The `visibility_range` vector of `chunk_instancing.wgsl`.
    pub(crate) fn shader_range(&self) -> Vec4 {
        Vec4::new(
            self.fade_in.start,
            self.fade_in.end,
            self.fade_out.start,
            self.fade_out.end,
        )
    }
}

impl SyncComponent for InstanceChunk {
    type Target = Self;
}

impl ExtractComponent for InstanceChunk {
    type QueryData = &'static Self;
    /// The render world keeps its copy, and the uploaded buffers, until the
    /// chunk changes.
    type QueryFilter = Changed<Self>;
    type Out = Self;

    fn extract_component(item: QueryItem<'_, '_, Self::QueryData>) -> Option<Self> {
        Some(item.clone())
    }
}

impl<M: ChunkInstancedMaterial> Clone for ChunkMaterial<M> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<M: ChunkInstancedMaterial> SyncComponent for ChunkMaterial<M> {
    type Target = Self;
}

impl<M: ChunkInstancedMaterial> ExtractComponent for ChunkMaterial<M> {
    type QueryData = &'static Self;
    type QueryFilter = ();
    type Out = Self;

    fn extract_component(item: QueryItem<'_, '_, Self::QueryData>) -> Option<Self> {
        Some(item.clone())
    }
}
