//! Final native ownership/allocation snapshot, outside sampled frame intervals.

use std::{num::NonZeroU32, ops::Range};

use bevy::{
    prelude::*,
    render::{
        mesh::{
            RenderMesh,
            allocator::{MeshAllocator, MeshBufferSlice},
        },
        render_asset::RenderAssets,
        storage::{GpuShaderBuffer, ShaderBuffer},
    },
};

use super::{ForestBenchmark, ForestRequestId};

/// Native entity bits include generation. Buffer ordinals identify Bevy buffer
/// objects within this process; capacity is not total device VRAM or a peak.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "prepared", derive(serde::Serialize))]
pub struct NativeResidency {
    request: ForestRequestId,
    tree_entities: Vec<u64>,
    ground_entities: Vec<u64>,
    owned_scene_entities: Vec<u64>,
    meshes: Vec<MeshAllocation>,
    storage: Vec<StorageAllocation>,
}

#[derive(Clone, Copy, Debug)]
pub struct NativeAssetCounts {
    pub meshes: usize,
    pub storage: usize,
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "prepared", derive(serde::Serialize))]
struct MeshAllocation {
    asset: AssetId<Mesh>,
    vertex: Option<BufferAllocation>,
    index: Option<BufferAllocation>,
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "prepared", derive(serde::Serialize))]
struct BufferAllocation {
    buffer_ordinal: u32,
    element_range: Range<u32>,
    capacity_bytes: u64,
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "prepared", derive(serde::Serialize))]
struct StorageAllocation {
    asset: AssetId<ShaderBuffer>,
    buffer_ordinal: u32,
    capacity_bytes: u64,
}

impl From<MeshBufferSlice<'_>> for BufferAllocation {
    fn from(slice: MeshBufferSlice) -> Self {
        Self {
            buffer_ordinal: NonZeroU32::from(slice.buffer.id()).get(),
            element_range: slice.range,
            capacity_bytes: slice.buffer.size(),
        }
    }
}

impl ForestBenchmark {
    pub(crate) fn expect_ground_entity(&self, entity: Entity) {
        self.mutate(|| match self.inner.expected.lock() {
            Ok(mut expected) => expected.ground_entities.push(entity),
            Err(_) => self.failure(),
        });
    }

    pub fn native_residency(&self) -> Option<NativeResidency> {
        self.inner
            .native
            .lock()
            .ok()
            .and_then(|native| native.clone())
    }

    pub(super) fn capture_native(
        &self,
        meshes: &RenderAssets<RenderMesh>,
        buffers: &RenderAssets<GpuShaderBuffer>,
        allocator: &MeshAllocator,
    ) {
        let Ok(expected) = self.inner.expected.lock() else {
            self.failure();
            return;
        };
        let Ok(ground) = self.inner.ground.lock() else {
            self.failure();
            return;
        };
        let Ok(mut native) = self.inner.native.lock() else {
            self.failure();
            return;
        };
        let mut tree_entities: Vec<_> = expected
            .chunks
            .iter()
            .filter(|chunk| expected.active(chunk))
            .map(|chunk| chunk.entity.to_bits())
            .collect();
        tree_entities.sort_unstable();
        let mut ground_entities = ground.clone();
        ground_entities.sort_unstable();
        let mut owned_scene_entities: Vec<_> = expected
            .chunks
            .iter()
            .map(|chunk| chunk.entity.to_bits())
            .chain(
                expected
                    .ground_entities
                    .iter()
                    .map(|entity| entity.to_bits()),
            )
            .collect();
        owned_scene_entities.sort_unstable();
        let mut mesh_allocations: Vec<_> = meshes
            .iter()
            .map(|(asset, _)| MeshAllocation {
                asset,
                vertex: allocator
                    .mesh_vertex_slice(&asset)
                    .map(BufferAllocation::from),
                index: allocator
                    .mesh_index_slice(&asset)
                    .map(BufferAllocation::from),
            })
            .collect();
        mesh_allocations.sort_by_key(|allocation| allocation.asset);
        let mut storage: Vec<_> = buffers
            .iter()
            .map(|(asset, stored)| StorageAllocation {
                asset,
                buffer_ordinal: NonZeroU32::from(stored.buffer.id()).get(),
                capacity_bytes: stored.buffer.size(),
            })
            .collect();
        storage.sort_by_key(|allocation| allocation.asset);
        *native = Some(NativeResidency {
            request: self.request_id(),
            tree_entities,
            ground_entities,
            owned_scene_entities,
            meshes: mesh_allocations,
            storage,
        });
    }

    fn failure(&self) {
        self.inner
            .failures
            .fetch_add(1, std::sync::atomic::Ordering::Release);
    }
}

impl NativeResidency {
    /// Match all scene meshes against this request's owned membership,
    /// including hidden leaves and offscreen ground that require no draw.
    pub fn matches_scene_entities(&self, entities: impl IntoIterator<Item = Entity>) -> bool {
        let mut actual: Vec<_> = entities.into_iter().map(Entity::to_bits).collect();
        actual.sort_unstable();
        actual == self.owned_scene_entities
    }

    pub fn asset_counts(&self) -> NativeAssetCounts {
        NativeAssetCounts {
            meshes: self.meshes.len(),
            storage: self.storage.len(),
        }
    }
}
