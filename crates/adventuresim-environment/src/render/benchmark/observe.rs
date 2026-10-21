//! Current-view draw validation with explicit waited or queued completion.

use super::*;
use bevy::camera::primitives::Frustum;
use bevy::core_pipeline::core_3d::Opaque3d;
use bevy::ecs::system::{SystemParam, SystemParamItem, lifetimeless::SRes};
use bevy::render::render_phase::{
    DrawFunctions, PhaseItem, RenderCommand, RenderCommandResult, RenderCommandState,
    TrackedRenderPass,
};
use bevy::render::render_resource::{CachedPipelineState, PipelineCache};
use bevy::render::renderer::RenderQueue;
use bevy::render::view::ExtractedView;
#[cfg(not(target_family = "wasm"))]
use bevy::render::{render_resource::PollType, renderer::RenderDevice};
use bevy::shader::ShaderCacheError;
use bevy_eidolon::cull::pipeline::InstancedComputePipeline;
use bevy_eidolon::prepass::CullComputeCamera;
use bevy_eidolon::render::draw::DrawInstancedMaterial;

#[derive(Resource, Default)]
pub(super) struct FrameCheck(Option<completion::RenderObservation>);

#[derive(SystemParam)]
pub(super) struct ObservedFrame<'w, 's> {
    check: Res<'w, FrameCheck>,
    ground: Res<'w, super::ground::GroundFrame>,
    #[cfg(not(target_family = "wasm"))]
    device: Res<'w, RenderDevice>,
    queue: Res<'w, RenderQueue>,
    cull_views: Query<
        'w,
        's,
        (&'static ExtractedView, &'static Frustum, &'static Camera),
        With<CullComputeCamera>,
    >,
    #[cfg(feature = "viewer")]
    profile: Option<Res<'w, super::profile::RenderProfile>>,
}

struct RecordDraw<const FOLIAGE: bool>;

#[derive(SystemParam)]
pub(super) struct GeometryReady<'w, B: InstancedMaterial, L: InstancedMaterial> {
    branches: Res<'w, InstancedComputePipeline<B>>,
    leaves: Res<'w, InstancedComputePipeline<L>>,
    meshes: Res<'w, RenderAssets<RenderMesh>>,
    buffers: Res<'w, RenderAssets<bevy::render::storage::GpuShaderBuffer>>,
    allocator: Res<'w, bevy::render::mesh::allocator::MeshAllocator>,
}

impl<P: PhaseItem, const FOLIAGE: bool> RenderCommand<P> for RecordDraw<FOLIAGE> {
    type Param = (SRes<ForestBenchmark>, SRes<super::ground::GroundFrame>);
    type ViewQuery = &'static ExtractedView;
    type ItemQuery = ();

    fn render<'w>(
        _item: &P,
        view: &'w ExtractedView,
        _entity: Option<()>,
        probe: SystemParamItem<'w, '_, Self::Param>,
        _pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let (probe, ground) = probe;
        if !ground.into_inner().owns_view(view.retained_view_entity) {
            return RenderCommandResult::Success;
        }
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
    cache: Res<PipelineCache>,
    branches: Res<MaterialBatchRanges<B>>,
    leaves: Res<MaterialBatchRanges<L>>,
    geometry: GeometryReady<B, L>,
) {
    let counters = &probe.inner;
    let branch_draws = counters.branch_draws.swap(0, Ordering::AcqRel);
    let leaf_draws = counters.leaf_draws.swap(0, Ordering::AcqRel);
    if !frame.wait_for_diagnostic(&probe) {
        return;
    }
    let Some(latched) = frame.check.0 else {
        counters.failures.fetch_add(1, Ordering::Release);
        return;
    };
    if latched.request != probe.request_id() {
        counters.failures.fetch_add(1, Ordering::Release);
        return;
    }
    let check = latched.validation;
    if !frame.cull_view_matches(&probe) {
        frame.acknowledge(&probe, latched, false, None);
        return;
    }
    if !check.full {
        frame.acknowledge(&probe, latched, true, None);
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
        && frame.ground.complete(&probe)
        && counts.gpu_failures == 0
        && (counts.expected_branch_instances == 0 || compute_ready(&geometry.branches, &cache))
        && (counts.expected_leaf_instances == 0 || compute_ready(&geometry.leaves, &cache))
        && probe.complete_meshes(&geometry.meshes);
    let inventory = if valid && check.phase() == ValidationPhase::FinalRequested {
        probe.capture_native(
            &geometry.meshes,
            &geometry.buffers,
            &geometry.allocator,
            latched.frame,
        )
    } else {
        None
    };
    frame.acknowledge(&probe, latched, valid, inventory);
}

impl ObservedFrame<'_, '_> {
    fn cull_view_matches(&self, probe: &ForestBenchmark) -> bool {
        // Match the pinned Eidolon producer's exact render-world candidate
        // query, and admit only a unique source with the owned view identity.
        let mut active = self
            .cull_views
            .iter()
            .filter(|(_, _, camera)| camera.is_active);
        let Some((view, _, _)) = active.next() else {
            return false;
        };
        let matches = active.next().is_none() && self.ground.owns_view(view.retained_view_entity);
        if !matches
            && self.ground.view().is_some()
            && probe
                .inner
                .failures
                .compare_exchange(0, 1, Ordering::AcqRel, Ordering::Acquire)
                .is_ok()
        {
            error!("forest GPU culling does not have one matching owned camera");
        }
        matches
    }

    fn wait_for_diagnostic(&self, probe: &ForestBenchmark) -> bool {
        if probe.completion_policy() == CompletionPolicy::Queued {
            return true;
        }
        #[cfg(target_family = "wasm")]
        {
            error!("waited forest diagnostics require a native backend");
            probe.failure();
            false
        }
        #[cfg(not(target_family = "wasm"))]
        {
            #[cfg(feature = "viewer")]
            if let Some(profile) = &self.profile {
                profile.begin_wait();
            }
            let waited = self.device.poll(PollType::wait_indefinitely());
            #[cfg(feature = "viewer")]
            if let Some(profile) = &self.profile {
                profile.end_wait();
            }
            if let Err(error) = waited {
                error!("forest benchmark GPU wait failed: {error}");
                probe.failure();
                return false;
            }
            true
        }
    }

    fn acknowledge(
        &self,
        probe: &ForestBenchmark,
        observation: completion::RenderObservation,
        valid: bool,
        inventory: Option<super::NativeResidency>,
    ) {
        match probe.completion_policy() {
            CompletionPolicy::WaitedDiagnostic => {
                probe.acknowledge_frame(observation, valid, inventory)
            }
            CompletionPolicy::Queued => {
                let probe = probe.clone();
                self.queue.on_submitted_work_done(move || {
                    probe.acknowledge_frame(observation, valid, inventory)
                });
            }
        }
    }
}

pub(super) fn latch(
    probe: Res<ForestBenchmark>,
    ground: Res<super::ground::GroundFrame>,
    mut frame: ResMut<FrameCheck>,
) {
    let Some(ordinal) = probe.inner.completion.next() else {
        probe.failure();
        frame.0 = None;
        return;
    };
    frame.0 = probe
        .validation()
        .render_check()
        .map(|validation| completion::RenderObservation {
            request: probe.request_id(),
            frame: ordinal,
            view: ground.view(),
            validation,
        });
}
