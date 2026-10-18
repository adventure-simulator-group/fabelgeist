//! A snapshot's records in the storage buffers the forest's materials read.
//!
//! The buffers persist from snapshot to
//! snapshot, with room to grow, and each snapshot's records are written
//! straight into them on the render thread: only the bytes in use, and
//! only when they changed -- the simulation hands over the same snapshot
//! again while it is paused. A snapshot that outgrows a buffer replaces it,
//! and the materials then bind the new one.

use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_asset::RenderAssets;
use bevy::render::renderer::RenderQueue;
use bevy::render::storage::{GpuShaderBuffer, ShaderBuffer};
use bevy::render::{MainWorld, Render, RenderApp, RenderSystems};
use fabelgeist_botany::viewer::sim::Snapshot;

use super::benchmark::ForestBenchmark;
use super::materials::SpeciesRecord;

/// A buffer's smallest size, bytes: a binding may not be empty.
const MINIMUM: u64 = 256;
/// A replaced buffer's room to grow, as a share of what it must hold.
const HEADROOM: f64 = 1.5;
/// Frames a write waits for its buffer before it is dropped: a buffer
/// replaced again before it was ever created never will be.
const PATIENCE: u32 = 8;

/// A snapshot's records as bytes, and each set's fingerprint.
pub(super) struct Records {
    species: Bytes,
    branches: Bytes,
    plants: Bytes,
    leaves: Bytes,
}

/// One record set's bytes and their fingerprint.
pub(super) struct Bytes {
    bytes: Vec<u8>,
    fingerprint: u64,
}

impl Bytes {
    pub(super) fn new<T: bytemuck::Pod>(records: &[T]) -> Self {
        let bytes = bytemuck::cast_slice(records).to_vec();
        Self {
            fingerprint: fingerprint(&bytes),
            bytes,
        }
    }
}

impl Records {
    /// The bytes of `snapshot`'s records. It copies and reads every record,
    /// so it runs off the main thread.
    pub(super) fn new(snapshot: &Snapshot) -> Self {
        Self::from_parts(
            &snapshot.species,
            &snapshot.branches,
            &snapshot.plant_bounds,
            &snapshot.leaves,
        )
    }

    pub(super) fn from_parts(
        species: &[fabelgeist_botany::viewer::sim::SpeciesInfo],
        branches: &[fabelgeist_botany::viewer::sim::BranchInstance],
        plants: &[fabelgeist_botany::viewer::sim::PlantInstance],
        leaves: &[fabelgeist_botany::viewer::sim::LeafInstance],
    ) -> Self {
        let species: Vec<SpeciesRecord> = species.iter().map(SpeciesRecord::from).collect();
        Self {
            species: Bytes::new(&species),
            branches: Bytes::new(branches),
            plants: Bytes::new(plants),
            leaves: Bytes::new(leaves),
        }
    }
}

/// A fingerprint of `bytes`, to tell a changed record set from the last:
/// fast rather than strong, as `FxHash` mixes, a word at a time.
pub(super) fn fingerprint(bytes: &[u8]) -> u64 {
    const MIX: u64 = 0x51_7c_c1_b7_27_22_0a_95;
    let (words, tail) = bytes.as_chunks::<8>();
    let mut hash = bytes.len() as u64;
    for word in words {
        hash = (hash.rotate_left(5) ^ u64::from_le_bytes(*word)).wrapping_mul(MIX);
    }
    for &byte in tail {
        hash = (hash.rotate_left(5) ^ u64::from(byte)).wrapping_mul(MIX);
    }
    hash
}

/// One storage buffer: its handle, its size and what was last written.
pub(super) struct Stored {
    pub(super) handle: Handle<ShaderBuffer>,
    capacity: u64,
    written: Option<u64>,
}

impl Stored {
    pub(super) fn new(buffers: &mut Assets<ShaderBuffer>) -> Self {
        Self {
            handle: buffers.add(empty(MINIMUM)),
            capacity: MINIMUM,
            written: None,
        }
    }

    /// Queue `bytes` to be written, making room first; true when the
    /// buffer was replaced to make room.
    pub(super) fn write(
        &mut self,
        bytes: Bytes,
        buffers: &mut Assets<ShaderBuffer>,
        writes: &mut PendingWrites,
    ) -> bool {
        let size = bytes.bytes.len() as u64;
        let replaced = size > self.capacity;
        if replaced {
            self.capacity = ((size as f64 * HEADROOM) as u64).max(MINIMUM);
            self.handle = buffers.add(empty(self.capacity));
            self.written = None;
        }
        if self.written != Some(bytes.fingerprint) {
            self.written = Some(bytes.fingerprint);
            writes.queue(self.handle.id(), bytes.bytes);
        }
        replaced
    }
}

/// An empty storage buffer of `size` bytes, to be written on the render
/// thread.
fn empty(size: u64) -> ShaderBuffer {
    ShaderBuffer::with_size(size as usize, RenderAssetUsages::RENDER_WORLD)
}

/// The storage buffers.
pub(super) struct ForestBuffers {
    species: Stored,
    branches: Stored,
    plants: Stored,
    leaves: Stored,
}

impl ForestBuffers {
    pub(super) fn new(buffers: &mut Assets<ShaderBuffer>) -> Self {
        Self {
            species: Stored::new(buffers),
            branches: Stored::new(buffers),
            plants: Stored::new(buffers),
            leaves: Stored::new(buffers),
        }
    }

    pub(super) fn species(&self) -> Handle<ShaderBuffer> {
        self.species.handle.clone()
    }

    pub(super) fn branches(&self) -> Handle<ShaderBuffer> {
        self.branches.handle.clone()
    }

    pub(super) fn plants(&self) -> Handle<ShaderBuffer> {
        self.plants.handle.clone()
    }

    pub(super) fn leaves(&self) -> Handle<ShaderBuffer> {
        self.leaves.handle.clone()
    }

    /// Queue `records` to be written; true when a buffer was replaced,
    /// which the materials must then bind.
    pub(super) fn write(
        &mut self,
        records: Records,
        buffers: &mut Assets<ShaderBuffer>,
        writes: &mut PendingWrites,
    ) -> bool {
        let species = self.species.write(records.species, buffers, writes);
        let branches = self.branches.write(records.branches, buffers, writes);
        let plants = self.plants.write(records.plants, buffers, writes);
        let leaves = self.leaves.write(records.leaves, buffers, writes);
        species || branches || plants || leaves
    }
}

/// Bytes to write into a storage buffer from its start, and the frames
/// they have waited for it.
struct BufferWrite {
    buffer: AssetId<ShaderBuffer>,
    bytes: Vec<u8>,
    waited: u32,
    probe: Option<ForestBenchmark>,
}

/// Writes queued on the main thread, until the render thread takes them.
#[derive(Resource, Default)]
pub(super) struct PendingWrites {
    writes: Vec<BufferWrite>,
    probe: Option<ForestBenchmark>,
}

impl PendingWrites {
    pub(super) fn observe(&mut self, probe: ForestBenchmark) {
        self.probe = Some(probe);
    }

    /// Queue `bytes` for `buffer`, in place of any older bytes for it.
    fn queue(&mut self, buffer: AssetId<ShaderBuffer>, bytes: Vec<u8>) {
        self.writes.retain(|write| {
            if write.buffer == buffer {
                if let Some(probe) = &write.probe {
                    probe.superseded();
                }
                return false;
            }
            true
        });
        if let Some(probe) = &self.probe {
            probe.queued();
        }
        self.writes.push(BufferWrite {
            buffer,
            bytes,
            waited: 0,
            probe: self.probe.clone(),
        });
    }
}

/// Writes taken to the render thread, until their buffers exist.
#[derive(Resource, Default)]
struct RenderWrites(Vec<BufferWrite>);

/// Write the queued records on the render thread.
pub(super) fn install(app: &mut App) {
    app.init_resource::<PendingWrites>();
    let Some(render) = app.get_sub_app_mut(RenderApp) else {
        return;
    };
    render
        .init_resource::<RenderWrites>()
        .add_systems(ExtractSchedule, take_writes)
        .add_systems(Render, write.in_set(RenderSystems::PrepareResources));
}

fn take_writes(mut main: ResMut<MainWorld>, mut taken: ResMut<RenderWrites>) {
    let mut pending = main.resource_mut::<PendingWrites>();
    for write in pending.writes.drain(..) {
        taken.0.retain(|older| {
            if older.buffer == write.buffer {
                if let Some(probe) = &older.probe {
                    probe.superseded();
                }
                return false;
            }
            true
        });
        taken.0.push(write);
    }
}

/// Write into each buffer that exists by now; a new one is written once it
/// does.
fn write(
    mut writes: ResMut<RenderWrites>,
    buffers: Res<RenderAssets<GpuShaderBuffer>>,
    queue: Res<RenderQueue>,
) {
    writes.0.retain_mut(|write| {
        let Some(gpu) = buffers.get(write.buffer) else {
            write.waited += 1;
            if write.waited >= PATIENCE
                && let Some(probe) = &write.probe
            {
                probe.discarded();
            }
            return write.waited < PATIENCE;
        };
        queue.write_buffer(&gpu.buffer, 0, &write.bytes);
        if let Some(probe) = &write.probe {
            probe.submitted(write.bytes.len());
        }
        false
    });
}
