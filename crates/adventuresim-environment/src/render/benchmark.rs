//! Opt-in observation of one frozen forest snapshot on the GPU.
//!
//! Successful draw commands and completed uploads are component evidence.
//! Pixel readbacks still have to establish that the intended forest is visible.

use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::time::Instant;

use bevy::core_pipeline::core_3d::Opaque3d;
use bevy::ecs::system::{SystemParamItem, lifetimeless::SRes};
use bevy::prelude::*;
use bevy::render::render_phase::{
    DrawFunctions, PhaseItem, RenderCommand, RenderCommandResult, RenderCommandState,
    TrackedRenderPass,
};
use bevy::render::render_resource::{CachedPipelineState, PipelineCache, PollType};
use bevy::render::renderer::RenderDevice;
use bevy::render::{Render, RenderApp, RenderSystems};
use bevy::shader::ShaderCacheError;
use bevy_eidolon::cull::pipeline::InstancedComputePipeline;
use bevy_eidolon::prelude::{InstancedMaterial, MaterialBatchRanges};
use bevy_eidolon::render::draw::DrawInstancedMaterial;

use super::{BranchMaterial, LeafMaterial};

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
}

#[derive(Default)]
struct Expected {
    branches: usize,
    leaves: usize,
    chunks: Vec<ExpectedChunk>,
}

struct ExpectedChunk {
    entity: Entity,
    instances: usize,
    part: ForestPart,
}

pub(super) enum ForestPart {
    Branches,
    Leaves,
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
}

struct RecordDraw<const FOLIAGE: bool>;

impl ForestBenchmark {
    pub fn new(started: Instant) -> Self {
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
            }),
        }
    }

    /// Install before material assets are prepared. This probe adds a GPU wait
    /// per frame and must not be enabled in ordinary play.
    pub fn install(&self, app: &mut App) {
        app.insert_resource(self.clone());
        app.world_mut()
            .resource_mut::<super::records::PendingWrites>()
            .observe(self.clone());
        let Some(render) = app.get_sub_app_mut(RenderApp) else {
            self.inner.failures.fetch_add(1, Ordering::Relaxed);
            return;
        };
        render.insert_resource(self.clone());
        observe_draws::<BranchMaterial, false>(render.world_mut());
        observe_draws::<LeafMaterial, true>(render.world_mut());
        render.add_systems(Render, complete.in_set(RenderSystems::Cleanup));
    }

    pub fn counts(&self) -> ForestBenchmarkCounts {
        let read = |counter: &AtomicUsize| counter.load(Ordering::Acquire);
        let micros = self.inner.completed_micros.load(Ordering::Acquire);
        ForestBenchmarkCounts {
            queued_writes: read(&self.inner.queued),
            submitted_writes: read(&self.inner.submitted),
            superseded_writes: read(&self.inner.superseded),
            discarded_writes: read(&self.inner.discarded),
            uploaded_bytes: self.inner.bytes.load(Ordering::Acquire),
            first_submission_micros: (micros != UNOBSERVED).then_some(micros),
            gpu_failures: read(&self.inner.failures),
        }
    }

    pub fn expect_snapshot(&self, branches: usize, leaves: usize) {
        match self.inner.expected.lock() {
            Ok(mut expected) => {
                expected.branches = branches;
                expected.leaves = leaves;
            }
            Err(_) => {
                self.inner.failures.fetch_add(1, Ordering::Release);
            }
        }
    }

    pub(super) fn expect_chunk(&self, entity: Entity, instances: usize, part: ForestPart) {
        match self.inner.expected.lock() {
            Ok(mut expected) => expected.chunks.push(ExpectedChunk {
                entity,
                instances,
                part,
            }),
            Err(_) => {
                self.inner.failures.fetch_add(1, Ordering::Release);
            }
        }
    }

    fn complete_chunks(
        &self,
        branches: &MaterialBatchRanges<BranchMaterial>,
        leaves: &MaterialBatchRanges<LeafMaterial>,
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
        for chunk in &expected.chunks {
            let (entities, total) = match chunk.part {
                ForestPart::Branches => (&branch_entities, &mut branch_count),
                ForestPart::Leaves => (&leaf_entities, &mut leaf_count),
            };
            if !entities.contains(&chunk.entity) {
                return false;
            }
            *total += chunk.instances;
        }
        branch_count == expected.branches
            && leaf_count == expected.leaves
            && expected.chunks.len() == branch_entities.len() + leaf_entities.len()
    }

    pub(super) fn queued(&self) {
        self.inner.queued.fetch_add(1, Ordering::Release);
    }

    pub(super) fn superseded(&self) {
        self.inner.superseded.fetch_add(1, Ordering::Release);
    }

    pub(super) fn discarded(&self) {
        self.inner.discarded.fetch_add(1, Ordering::Release);
    }

    pub(super) fn submitted(&self, bytes: usize) {
        self.inner.bytes.fetch_add(bytes as u64, Ordering::Release);
        self.inner.submitted.fetch_add(1, Ordering::Release);
    }
}

impl<P: PhaseItem, const FOLIAGE: bool> RenderCommand<P> for RecordDraw<FOLIAGE> {
    type Param = SRes<ForestBenchmark>;
    type ViewQuery = ();
    type ItemQuery = ();

    fn render<'w>(
        _item: &P,
        _view: (),
        _entity: Option<()>,
        probe: SystemParamItem<'w, '_, Self::Param>,
        _pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let counters = &probe.into_inner().inner;
        let counter = if FOLIAGE {
            &counters.leaf_draws
        } else {
            &counters.branch_draws
        };
        counter.fetch_add(1, Ordering::Relaxed);
        RenderCommandResult::Success
    }
}

fn observe_draws<M: InstancedMaterial, const FOLIAGE: bool>(world: &mut World) {
    let command =
        RenderCommandState::<Opaque3d, (DrawInstancedMaterial<M>, RecordDraw<FOLIAGE>)>::new(world);
    world
        .resource::<DrawFunctions<Opaque3d>>()
        .write()
        .add_with::<DrawInstancedMaterial<M>, _>(command);
}

fn compute_ready<M: InstancedMaterial>(
    pipeline: &InstancedComputePipeline<M>,
    cache: &PipelineCache,
) -> bool {
    pipeline
        .pipeline_id
        .is_some_and(|id| cache.get_compute_pipeline(id).is_some())
}

fn complete(
    probe: Res<ForestBenchmark>,
    device: Res<RenderDevice>,
    cache: Res<PipelineCache>,
    branches: Res<MaterialBatchRanges<BranchMaterial>>,
    leaves: Res<MaterialBatchRanges<LeafMaterial>>,
    branch_compute: Res<InstancedComputePipeline<BranchMaterial>>,
    leaf_compute: Res<InstancedComputePipeline<LeafMaterial>>,
) {
    let counters = &probe.inner;
    let branch_draws = counters.branch_draws.swap(0, Ordering::AcqRel);
    let leaf_draws = counters.leaf_draws.swap(0, Ordering::AcqRel);
    if let Err(error) = device.poll(PollType::wait_indefinitely()) {
        error!("forest benchmark GPU wait failed: {error}");
        counters.failures.fetch_add(1, Ordering::Release);
        return;
    }
    if cache.pipelines().any(|pipeline| match &pipeline.state {
        CachedPipelineState::Err(
            ShaderCacheError::ShaderNotLoaded(_) | ShaderCacheError::ShaderImportNotYetAvailable,
        ) => false,
        CachedPipelineState::Err(_) => true,
        _ => false,
    }) {
        counters.failures.fetch_add(1, Ordering::Release);
        return;
    }
    let counts = probe.counts();
    let writes_done = counts.queued_writes > 0
        && counts.queued_writes == counts.submitted_writes
        && counts.superseded_writes == 0
        && counts.discarded_writes == 0;
    let draws_done = !branches.entities.is_empty()
        && !leaves.entities.is_empty()
        && branch_draws == branches.entities.len()
        && leaf_draws == leaves.entities.len()
        && probe.complete_chunks(&branches, &leaves);
    if writes_done
        && draws_done
        && counts.gpu_failures == 0
        && compute_ready(&branch_compute, &cache)
        && compute_ready(&leaf_compute, &cache)
    {
        let micros = counters.started.elapsed().as_micros().min(u64::MAX as u128) as u64;
        let _ = counters.completed_micros.compare_exchange(
            UNOBSERVED,
            micros.max(1),
            Ordering::Release,
            Ordering::Relaxed,
        );
    }
}
