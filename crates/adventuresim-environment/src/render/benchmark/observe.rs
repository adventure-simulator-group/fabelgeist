//! Render-frame observation with the same draw counters and GPU wait in both policies.

use super::*;
use bevy::core_pipeline::core_3d::Opaque3d;
use bevy::ecs::system::{SystemParam, SystemParamItem, lifetimeless::SRes};
use bevy::render::render_phase::{
    DrawFunctions, PhaseItem, RenderCommand, RenderCommandResult, RenderCommandState,
    TrackedRenderPass,
};
use bevy::render::render_resource::{CachedPipelineState, PipelineCache, PollType};
use bevy::render::renderer::RenderDevice;
use bevy::shader::ShaderCacheError;
use bevy_eidolon::cull::pipeline::InstancedComputePipeline;
use bevy_eidolon::render::draw::DrawInstancedMaterial;

#[derive(Resource, Default)]
pub(super) struct FrameCheck(Option<validation::RenderCheck>);

#[derive(SystemParam)]
pub(super) struct ObservedFrame<'w> {
    check: Res<'w, FrameCheck>,
    ground: Res<'w, super::ground::GroundFrame>,
    #[cfg(feature = "viewer")]
    profile: Option<Res<'w, super::profile::RenderProfile>>,
}

struct RecordDraw<const FOLIAGE: bool>;

#[derive(SystemParam)]
pub(super) struct GeometryReady<'w, B: InstancedMaterial, L: InstancedMaterial> {
    branches: Res<'w, InstancedComputePipeline<B>>,
    leaves: Res<'w, InstancedComputePipeline<L>>,
    meshes: Res<'w, RenderAssets<RenderMesh>>,
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

pub(super) fn draws<M: InstancedMaterial, const FOLIAGE: bool>(world: &mut World) {
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

pub(super) fn complete<B: InstancedMaterial, L: InstancedMaterial>(
    probe: Res<ForestBenchmark>,
    frame: ObservedFrame,
    device: Res<RenderDevice>,
    cache: Res<PipelineCache>,
    branches: Res<MaterialBatchRanges<B>>,
    leaves: Res<MaterialBatchRanges<L>>,
    geometry: GeometryReady<B, L>,
) {
    let counters = &probe.inner;
    let branch_draws = counters.branch_draws.swap(0, Ordering::AcqRel);
    let leaf_draws = counters.leaf_draws.swap(0, Ordering::AcqRel);
    #[cfg(feature = "viewer")]
    if let Some(profile) = &frame.profile {
        profile.begin_wait();
    }
    let waited = device.poll(PollType::wait_indefinitely());
    #[cfg(feature = "viewer")]
    if let Some(profile) = &frame.profile {
        profile.end_wait();
    }
    if let Err(error) = waited {
        error!("forest benchmark GPU wait failed: {error}");
        counters.failures.fetch_add(1, Ordering::Release);
        return;
    }
    let Some(check) = frame.check.0 else {
        counters.failures.fetch_add(1, Ordering::Release);
        return;
    };
    if !check.full {
        probe.validation().acknowledge(check, true);
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
    let draws_done = branch_draws == branches.entities.len()
        && leaf_draws == leaves.entities.len()
        && probe.complete_chunks(&branches, &leaves);
    let valid = writes_done
        && draws_done
        && frame.ground.complete()
        && counts.gpu_failures == 0
        && (counts.expected_branch_instances == 0 || compute_ready(&geometry.branches, &cache))
        && (counts.expected_leaf_instances == 0 || compute_ready(&geometry.leaves, &cache))
        && probe.complete_meshes(&geometry.meshes);
    probe.validation().acknowledge(check, valid);
    if valid {
        let micros = counters.started.elapsed().as_micros().min(u64::MAX as u128) as u64;
        let _ = counters.completed_micros.compare_exchange(
            UNOBSERVED,
            micros.max(1),
            Ordering::Release,
            Ordering::Relaxed,
        );
    }
}

pub(super) fn latch(probe: Res<ForestBenchmark>, mut frame: ResMut<FrameCheck>) {
    frame.0 = probe.validation().render_check();
}
