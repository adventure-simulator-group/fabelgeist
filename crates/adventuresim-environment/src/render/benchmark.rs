//! Opt-in observation of one frozen forest snapshot on the GPU.
//!
//! Successful draw commands and completed uploads are component evidence.
//! Pixel readbacks still have to establish that the intended forest is visible.

use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::time::Instant;

use bevy::prelude::*;
use bevy::render::mesh::{RenderMesh, RenderMeshBufferInfo};
use bevy::render::render_asset::RenderAssets;
use bevy::render::{Render, RenderApp, RenderSystems};
use bevy_eidolon::prelude::{InstancedMaterial, MaterialBatchRanges};

use super::{BranchMaterial, ForestPart, LeafMaterial};

mod ground;
mod observe;
#[cfg(feature = "viewer")]
pub mod profile;
mod validation;

pub use validation::{
    ValidationEvidence, ValidationPhase, ValidationPolicy, ValidationWindow, WindowStart,
};

/// Zero is reserved for a GPU submission not observed yet.
const UNOBSERVED: u64 = 0;

/// Counters for a single immutable snapshot, shared with its render world.
#[derive(Resource, Clone)]
pub struct ForestBenchmark {
    inner: Arc<Counters>,
}

struct Counters {
    started: Instant,
    queued: AtomicUsize,
    submitted: AtomicUsize,
    superseded: AtomicUsize,
    discarded: AtomicUsize,
    bytes: AtomicU64,
    branch_draws: AtomicUsize,
    leaf_draws: AtomicUsize,
    completed_micros: AtomicU64,
    failures: AtomicUsize,
    expected: Mutex<Expected>,
    validation: ValidationWindow,
}

#[derive(Default)]
struct Expected {
    branches: usize,
    leaves: usize,
    chunks: Vec<ExpectedChunk>,
    meshes: Vec<ExpectedMesh>,
}

/// Per-frame status; endpoint sampling avoids constructing full LOD histograms.
#[derive(Debug)]
pub struct ForestStatus {
    pub first_submission_micros: Option<u64>,
    pub failed: bool,
}

struct ExpectedMesh {
    id: AssetId<Mesh>,
    vertices: usize,
    indices: usize,
}

/// Counts of draw instances, distinct from botanical source-record counts.
pub struct ForestInstances {
    pub branches: usize,
    pub leaves: usize,
}

#[derive(Debug, Default)]
#[cfg_attr(feature = "prepared", derive(serde::Serialize))]
pub struct LodCounts {
    pub full: usize,
    pub middle: usize,
    pub coarse: usize,
}

/// Record work submitted by a batch, before per-tree GPU culling.
pub struct ForestWork {
    pub records: usize,
    pub indexed_slots: usize,
}

#[derive(Debug, Default)]
pub struct LodWork {
    pub instances: LodCounts,
    pub records: LodCounts,
    pub indexed_slots: LodCounts,
}

struct ExpectedChunk {
    entity: Entity,
    instances: usize,
    part: ForestPart,
    level: super::Level,
    work: ForestWork,
}

/// A diagnostic snapshot; counts include only work observed by this probe.
#[derive(Debug)]
pub struct ForestBenchmarkCounts {
    pub queued_writes: usize,
    pub submitted_writes: usize,
    pub superseded_writes: usize,
    pub discarded_writes: usize,
    pub uploaded_bytes: u64,
    pub first_submission_micros: Option<u64>,
    pub gpu_failures: usize,
    pub expected_branch_instances: usize,
    pub expected_leaf_instances: usize,
    pub expected_batches: usize,
    pub expected_meshes: usize,
    pub branch_lod: LodWork,
    pub leaf_lod: LodWork,
}

impl LodCounts {
    fn add(&mut self, level: super::Level, count: usize) {
        match level {
            super::Level::Full => self.full += count,
            super::Level::Middle => self.middle += count,
            super::Level::Coarse => self.coarse += count,
        }
    }
}

impl LodWork {
    fn add(&mut self, chunk: &ExpectedChunk) {
        self.instances.add(chunk.level, chunk.instances);
        self.records.add(chunk.level, chunk.work.records);
        self.indexed_slots
            .add(chunk.level, chunk.work.indexed_slots);
    }
}

impl Expected {
    fn active(&self, chunk: &ExpectedChunk) -> bool {
        match chunk.part {
            ForestPart::Branches => self.branches > 0,
            ForestPart::Leaves => self.leaves > 0,
        }
    }
}

impl ForestBenchmark {
    pub fn new(started: Instant, policy: ValidationPolicy) -> Self {
        Self {
            inner: Arc::new(Counters {
                started,
                queued: AtomicUsize::new(0),
                submitted: AtomicUsize::new(0),
                superseded: AtomicUsize::new(0),
                discarded: AtomicUsize::new(0),
                bytes: AtomicU64::new(0),
                branch_draws: AtomicUsize::new(0),
                leaf_draws: AtomicUsize::new(0),
                completed_micros: AtomicU64::new(UNOBSERVED),
                failures: AtomicUsize::new(0),
                expected: Mutex::new(Expected::default()),
                validation: ValidationWindow::new(policy),
            }),
        }
    }

    /// Install before material assets are prepared. This probe adds a GPU wait
    /// per frame and must not be enabled in ordinary play.
    pub fn install(&self, app: &mut App) {
        self.install_materials::<BranchMaterial, LeafMaterial>(app);
    }

    #[cfg(feature = "prepared")]
    pub fn install_shared(&self, app: &mut App) {
        self.install_materials::<super::trees::TreeBranchMaterial, super::trees::TreeLeafMaterial>(
            app,
        );
    }

    fn install_materials<B: InstancedMaterial, L: InstancedMaterial>(&self, app: &mut App) {
        app.insert_resource(self.clone());
        app.world_mut()
            .resource_mut::<super::records::PendingWrites>()
            .observe(self.clone());
        let Some(render) = app.get_sub_app_mut(RenderApp) else {
            self.inner.failures.fetch_add(1, Ordering::Relaxed);
            return;
        };
        render.insert_resource(self.clone());
        ground::install(render.world_mut());
        render.add_systems(bevy::render::ExtractSchedule, ground::extract);
        observe::draws::<B, false>(render.world_mut());
        observe::draws::<L, true>(render.world_mut());
        render.init_resource::<observe::FrameCheck>();
        render.add_systems(
            Render,
            observe::latch.in_set(RenderSystems::PrepareResources),
        );
        render.add_systems(
            Render,
            observe::complete::<B, L>.in_set(RenderSystems::Cleanup),
        );
    }

    pub fn validation(&self) -> &ValidationWindow {
        &self.inner.validation
    }

    pub fn status(&self) -> ForestStatus {
        if !self.validation().skip_structure() {
            let counts = self.counts();
            return ForestStatus {
                first_submission_micros: counts.first_submission_micros,
                failed: counts.gpu_failures > 0
                    || counts.discarded_writes > 0
                    || counts.superseded_writes > 0,
            };
        }
        let counters = &self.inner;
        let micros = counters.completed_micros.load(Ordering::Acquire);
        ForestStatus {
            first_submission_micros: (micros != UNOBSERVED).then_some(micros),
            failed: counters.failures.load(Ordering::Acquire) > 0
                || counters.discarded.load(Ordering::Acquire) > 0
                || counters.superseded.load(Ordering::Acquire) > 0,
        }
    }

    fn mutate(&self, mutation: impl FnOnce()) {
        self.validation().with_mutation(|frozen| {
            if frozen {
                self.inner.failures.fetch_add(1, Ordering::Release);
            }
            mutation();
        });
    }

    pub fn counts(&self) -> ForestBenchmarkCounts {
        let read = |counter: &AtomicUsize| counter.load(Ordering::Acquire);
        let micros = self.inner.completed_micros.load(Ordering::Acquire);
        let expected = self.inner.expected.lock().ok();
        let mut branch_lod = LodWork::default();
        let mut leaf_lod = LodWork::default();
        if let Some(expected) = &expected {
            for chunk in expected
                .chunks
                .iter()
                .filter(|chunk| expected.active(chunk))
            {
                match chunk.part {
                    ForestPart::Branches => branch_lod.add(chunk),
                    ForestPart::Leaves => leaf_lod.add(chunk),
                }
            }
        }
        ForestBenchmarkCounts {
            queued_writes: read(&self.inner.queued),
            submitted_writes: read(&self.inner.submitted),
            superseded_writes: read(&self.inner.superseded),
            discarded_writes: read(&self.inner.discarded),
            uploaded_bytes: self.inner.bytes.load(Ordering::Acquire),
            first_submission_micros: (micros != UNOBSERVED).then_some(micros),
            gpu_failures: read(&self.inner.failures),
            expected_branch_instances: expected.as_ref().map_or(0, |e| e.branches),
            expected_leaf_instances: expected.as_ref().map_or(0, |e| e.leaves),
            expected_batches: expected.as_ref().map_or(0, |e| {
                e.chunks.iter().filter(|chunk| e.active(chunk)).count()
            }),
            expected_meshes: expected.as_ref().map_or(0, |e| e.meshes.len()),
            branch_lod,
            leaf_lod,
        }
    }

    pub fn expect_instances(&self, instances: ForestInstances) {
        self.mutate(|| match self.inner.expected.lock() {
            Ok(mut expected) => {
                expected.branches = instances.branches;
                expected.leaves = instances.leaves;
            }
            Err(_) => {
                self.inner.failures.fetch_add(1, Ordering::Release);
            }
        });
    }

    pub(super) fn expect_mesh(&self, id: AssetId<Mesh>, vertices: usize, indices: usize) {
        self.mutate(|| match self.inner.expected.lock() {
            Ok(mut expected) => expected.meshes.push(ExpectedMesh {
                id,
                vertices,
                indices,
            }),
            Err(_) => {
                self.inner.failures.fetch_add(1, Ordering::Release);
            }
        });
    }

    fn complete_meshes(&self, meshes: &RenderAssets<RenderMesh>) -> bool {
        let Ok(expected) = self.inner.expected.lock() else {
            return false;
        };
        expected.meshes.iter().all(|expected| meshes.get(expected.id).is_some_and(|mesh| {
            mesh.vertex_count as usize == expected.vertices && matches!(mesh.buffer_info, RenderMeshBufferInfo::Indexed { count, .. } if count as usize == expected.indices)
        }))
    }

    pub(super) fn expect_chunk(
        &self,
        entity: Entity,
        instances: usize,
        part: ForestPart,
        level: super::Level,
        work: ForestWork,
    ) {
        self.mutate(|| match self.inner.expected.lock() {
            Ok(mut expected) => expected.chunks.push(ExpectedChunk {
                entity,
                instances,
                part,
                level,
                work,
            }),
            Err(_) => {
                self.inner.failures.fetch_add(1, Ordering::Release);
            }
        });
    }

    pub(super) fn expect_level(&self, entity: Entity, level: super::Level) {
        self.mutate(|| {
            if let Ok(mut expected) = self.inner.expected.lock() {
                if let Some(chunk) = expected
                    .chunks
                    .iter_mut()
                    .find(|chunk| chunk.entity == entity)
                {
                    chunk.level = level;
                }
            } else {
                self.inner.failures.fetch_add(1, Ordering::Release);
            }
        });
    }

    fn complete_chunks<B: InstancedMaterial, L: InstancedMaterial>(
        &self,
        branches: &MaterialBatchRanges<B>,
        leaves: &MaterialBatchRanges<L>,
    ) -> bool {
        let Ok(expected) = self.inner.expected.lock() else {
            return false;
        };
        let branch_entities: std::collections::HashSet<_> = branches
            .entities
            .iter()
            .map(|(_, main)| main.id())
            .collect();
        let leaf_entities: std::collections::HashSet<_> =
            leaves.entities.iter().map(|(_, main)| main.id()).collect();
        let mut branch_count = 0;
        let mut leaf_count = 0;
        let mut visible_batches = 0;
        for chunk in expected
            .chunks
            .iter()
            .filter(|chunk| expected.active(chunk))
        {
            let (entities, total) = match chunk.part {
                ForestPart::Branches => (&branch_entities, &mut branch_count),
                ForestPart::Leaves => (&leaf_entities, &mut leaf_count),
            };
            if !entities.contains(&chunk.entity) {
                return false;
            }
            *total += chunk.instances;
            visible_batches += 1;
        }
        branch_count == expected.branches
            && leaf_count == expected.leaves
            && visible_batches == branch_entities.len() + leaf_entities.len()
    }

    pub(super) fn queued(&self) {
        self.mutate(|| {
            self.inner.queued.fetch_add(1, Ordering::Release);
        });
    }

    pub(super) fn superseded(&self) {
        self.inner.superseded.fetch_add(1, Ordering::Release);
    }

    pub(super) fn discarded(&self) {
        self.inner.discarded.fetch_add(1, Ordering::Release);
    }

    pub(super) fn submitted(&self, bytes: usize) {
        self.mutate(|| {
            self.inner.bytes.fetch_add(bytes as u64, Ordering::Release);
            self.inner.submitted.fetch_add(1, Ordering::Release);
        });
    }
}
