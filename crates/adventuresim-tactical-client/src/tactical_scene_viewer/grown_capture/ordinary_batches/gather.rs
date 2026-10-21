//! Read this view's actual submitted buffers before another view can reuse them.
use super::*;
use bevy::{
    app::SubApp,
    core_pipeline::{
        Core3d, Core3dSystems,
        core_3d::{main_opaque_pass_3d, main_transparent_pass_3d},
    },
    ecs::system::SystemParam,
    pbr::{
        MeshInputUniform, MeshUniform, PreparedMaterial, RenderMaterialInstances,
        RenderMeshInstances, SpecializedMaterialPipelineCache,
    },
    render::{
        batching::gpu_preprocessing::{
            BatchedInstanceBuffers, GpuPreprocessingSupport, IndirectBatchSet,
            IndirectParametersBuffers, IndirectParametersIndexed, IndirectParametersNonIndexed,
        },
        erased_render_asset::ErasedRenderAssets,
        mesh::{RenderMesh, allocator::MeshAllocator},
        render_asset::RenderAssets,
        render_resource::{Buffer, PipelineCache, ShaderType},
        renderer::{RenderContext, ViewQuery},
        view::ExtractedView,
    },
};
use commands::{CommandCount, Dispatch, Receipt};
use snapshots::{CopyPipeline, SourcePrefix};
use std::num::NonZeroU64;
use types::{ColorPhase, SnapshotKey, SnapshotKind};

#[derive(SystemParam)]
struct RendererSources<'w> {
    meshes: Res<'w, RenderAssets<RenderMesh>>,
    source_buffers: Res<'w, RenderAssets<bevy::render::storage::GpuShaderBuffer>>,
    instances: Res<'w, RenderMeshInstances>,
    materials: Res<'w, RenderMaterialInstances>,
    prepared: Res<'w, ErasedRenderAssets<PreparedMaterial>>,
    allocator: Res<'w, MeshAllocator>,
    specializations: Res<'w, SpecializedMaterialPipelineCache>,
    pipelines: Res<'w, PipelineCache>,
    indirect: Res<'w, IndirectParametersBuffers>,
    buffers: Option<Res<'w, BatchedInstanceBuffers<MeshUniform, MeshInputUniform>>>,
    preprocessing: Res<'w, GpuPreprocessingSupport>,
}

pub(super) fn install(render: &mut SubApp) {
    render.add_systems(
        Core3d,
        gather
            .in_set(Core3dSystems::MainPass)
            .after(main_opaque_pass_3d)
            .before(main_transparent_pass_3d),
    );
}

fn gather(
    view: ViewQuery<&ExtractedView>,
    frame: Res<Frame>,
    sources: RendererSources,
    mut context: RenderContext,
    mut copier: Local<Option<CopyPipeline>>,
    mut readbacks: ResMut<readbacks::Queue>,
) -> bevy::prelude::Result {
    let Some(pose) = &frame.pose else {
        return Ok(());
    };
    let view = view.into_inner();
    if pose.camera != view.retained_view_entity || pose.invalid.load(Ordering::Acquire) {
        return Ok(());
    }
    if !pose.camera_intact(view, frame.serial)? {
        return Ok(());
    }
    let availability = bindings::RenderMember::collect(
        pose,
        &sources.meshes,
        &sources.instances,
        &sources.materials,
        &sources.prepared,
        &sources.allocator,
        &sources.specializations,
        &sources.pipelines,
    )?;
    let current = availability.as_ready();
    let commands = frame
        .commands
        .lock()
        .map_err(|_| "GPU command lock")?
        .clone();
    let mut held = pose.submission.lock().map_err(|_| "GPU submission lock")?;
    let prepared_sources =
        source_buffers::PreparedSource::resolve(&pose.buffers, &sources.source_buffers)?;
    if let Some(held) = held.as_ref() {
        if prepared_sources.as_ref() != Some(&held.sources) {
            pose.fail("held prepared source allocation changed or disappeared");
        }
        if current != Some(&held.members) {
            pose.fail("held renderer allocation/material/pipeline drift");
        }
        if let Err(error) = held.check_commands(&commands, frame.serial, pose.camera) {
            pose.fail(format!("held successful draws: {error:#}"));
            held.record_command_drift(pose, &commands, frame.serial, &error)?;
        }
        return Ok(());
    }
    if pose.readiness == Readiness::FirstTransitionFrame && frame.serial != pose.first_frame {
        pose.unready(frame.serial, ReadinessStage::MissedFirstFrame, &commands)?;
        return Ok(());
    }
    let Some(prepared_sources) = prepared_sources else {
        pose.unready(frame.serial, ReadinessStage::SourceBuffers, &commands)?;
        return Ok(());
    };
    let members = match availability {
        bindings::Availability::Ready(members) => members,
        bindings::Availability::Missing(reason) => {
            if pose.readiness == Readiness::FirstTransitionFrame {
                pose.fail("first transition frame is incomplete: MeshMaterialPipeline");
                pose.record_frame_failure(
                    frame.serial,
                    ReadinessStage::MeshMaterialPipeline,
                    &commands,
                    reason.report(),
                )?;
            }
            return Ok(());
        }
    };
    if !members
        .iter()
        .all(|member| commands.iter().any(|command| command.compatible(member)))
    {
        pose.unready(frame.serial, ReadinessStage::DrawCommands, &commands)?;
        return Ok(());
    }
    if !sources.preprocessing.is_available() {
        return Err("ordinary proof requires real GPU preprocessing".into());
    }
    let buffers = sources
        .buffers
        .as_deref()
        .context("ordinary GPU output buffers absent")?;
    for command in &commands {
        if command.frame != frame.serial || command.camera != pose.camera {
            return Err("stale ordinary command receipt".into());
        }
    }
    let prefixes = source_prefixes(&commands, buffers, &sources.indirect)?;
    let copier = copier.get_or_insert_with(|| CopyPipeline::new(context.render_device()));
    let encoded = copier.encode(&mut context, &prefixes)?;
    let submission =
        proof::Submission::new(pose.view, frame.serial, members, commands, prepared_sources);
    readbacks.enqueue(
        Arc::clone(pose),
        frame.serial,
        submission.inbox.clone(),
        encoded,
    )?;
    *held = Some(submission);
    Ok(())
}

fn source_prefixes(
    commands: &[Receipt],
    buffers: &BatchedInstanceBuffers<MeshUniform, MeshInputUniform>,
    indirect: &IndirectParametersBuffers,
) -> Result<Vec<SourcePrefix>> {
    let mut prefixes = Vec::new();
    for phase in [ColorPhase::Opaque, ColorPhase::AlphaMask] {
        let commands: Vec<_> = commands
            .iter()
            .filter(|command| command.phase == phase)
            .collect();
        if commands.is_empty() {
            continue;
        }
        let phase_buffers = buffers
            .phase_instance_buffers
            .get(&phase.type_id())
            .context("phase mesh output absent")?;
        let logical_instances = phase_buffers.data_buffer.len();
        let output = phase_buffers
            .data_buffer
            .buffer()
            .context("phase mesh storage absent")?;
        let stride = <Vec<MeshUniform> as ShaderType>::METADATA.stride().get();
        let bytes = u64::try_from(logical_instances)?
            .checked_mul(stride)
            .context("mesh snapshot byte overflow")?;
        prefixes.push(prefix(
            SnapshotKey {
                phase,
                kind: SnapshotKind::MeshOutput { logical_instances },
            },
            output,
            bytes,
        )?);
        let parameters = indirect
            .get(&phase.type_id())
            .context("phase indirect buffers absent")?;
        for indexed in [true, false] {
            let class_commands: Vec<_> = commands
                .iter()
                .copied()
                .filter(|command| command.binding.geometry.indexed() == indexed)
                .collect();
            let mut parameter_end = 0_u32;
            let mut batch_set_end = 0_u32;
            for command in class_commands {
                if let Dispatch::Indirect { parameters, count } = &command.dispatch {
                    ensure!(
                        parameters.start < parameters.end,
                        "empty/reversed indirect range"
                    );
                    parameter_end = parameter_end.max(parameters.end);
                    if let CommandCount::BatchSet(index) = count {
                        batch_set_end = batch_set_end
                            .max(index.checked_add(1).context("batch-set prefix overflow")?);
                    }
                }
            }
            if parameter_end == 0 {
                continue;
            }
            let (arguments, sets, argument_kind, set_kind, argument_stride) = if indexed {
                (
                    parameters.indexed.data_buffer(),
                    parameters.indexed.batch_sets_buffer(),
                    SnapshotKind::IndexedArguments,
                    SnapshotKind::IndexedBatchSets,
                    size_of::<IndirectParametersIndexed>(),
                )
            } else {
                (
                    parameters.non_indexed.data_buffer(),
                    parameters.non_indexed.batch_sets_buffer(),
                    SnapshotKind::NonIndexedArguments,
                    SnapshotKind::NonIndexedBatchSets,
                    size_of::<IndirectParametersNonIndexed>(),
                )
            };
            let bytes = u64::from(parameter_end)
                .checked_mul(u64::try_from(argument_stride)?)
                .context("argument prefix overflow")?;
            prefixes.push(prefix(
                SnapshotKey {
                    phase,
                    kind: argument_kind,
                },
                arguments.context("actual argument storage absent")?,
                bytes,
            )?);
            if batch_set_end > 0 {
                let bytes = u64::from(batch_set_end)
                    .checked_mul(size_of::<IndirectBatchSet>() as u64)
                    .context("batch-set prefix overflow")?;
                prefixes.push(prefix(
                    SnapshotKey {
                        phase,
                        kind: set_kind,
                    },
                    sets.context("actual batch-set storage absent")?,
                    bytes,
                )?);
            }
        }
    }
    Ok(prefixes)
}

fn prefix(key: SnapshotKey, source: &Buffer, bytes: u64) -> Result<SourcePrefix> {
    let bytes = NonZeroU64::new(bytes).context("empty GPU storage prefix")?;
    ensure!(
        bytes.get() <= source.size(),
        "actual GPU prefix exceeds source buffer"
    );
    Ok(SourcePrefix {
        key,
        source: source.clone(),
        bytes,
    })
}
