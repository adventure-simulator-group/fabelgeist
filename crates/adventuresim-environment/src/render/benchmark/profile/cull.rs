//! Retain Eidolon's original dispatch inside a same-thread diagnostic pipe.

use anyhow::{Result, anyhow, ensure};
use bevy::core_pipeline::schedule::camera_driver;
use bevy::ecs::schedule::ScheduleCleanupPolicy;
use bevy::prelude::*;
use bevy::render::RenderStartup;
use bevy::render::diagnostic::RecordDiagnostics;
use bevy::render::renderer::{RenderContext, RenderDevice, RenderGraph, RenderGraphSystems};
use bevy::render::settings::WgpuFeatures;
use bevy_eidolon::cull::node::instanced_compute_node;
use bevy_eidolon::prelude::InstancedMaterial;

use super::{ProfileFailure, RenderProfile};

pub(super) fn install<B: InstancedMaterial, L: InstancedMaterial>(world: &mut World) -> Result<()> {
    world.try_schedule_scope(RenderGraph, |world, schedule| -> Result<()> {
        let branches = schedule.remove_systems_in_set(
            instanced_compute_node::<B>,
            world,
            ScheduleCleanupPolicy::RemoveSystemsOnly,
        )?;
        ensure!(
            branches == 1,
            "expected one original branch cull system, found {branches}"
        );
        let leaves = schedule.remove_systems_in_set(
            instanced_compute_node::<L>,
            world,
            ScheduleCleanupPolicy::RemoveSystemsOnly,
        )?;
        ensure!(
            leaves == 1,
            "expected one original leaf cull system, found {leaves}"
        );
        schedule.add_systems((
            begin::<false>
                .pipe(instanced_compute_node::<B>)
                .pipe(end)
                .in_set(RenderGraphSystems::Render)
                .before(camera_driver),
            begin::<true>
                .pipe(instanced_compute_node::<L>)
                .pipe(end)
                .in_set(RenderGraphSystems::Render)
                .before(camera_driver),
        ));
        let profile = world.resource::<RenderProfile>();
        profile
            .data
            .lock()
            .map_err(|_| anyhow!("profile lock poisoned"))?
            .wrapped_cull_systems = branches + leaves;
        Ok(())
    })??;
    world
        .resource_mut::<Schedules>()
        .add_systems(RenderStartup, require_timestamps);
    Ok(())
}

fn require_timestamps(device: Res<RenderDevice>, profile: Res<RenderProfile>) {
    if !device
        .features()
        .contains(WgpuFeatures::TIMESTAMP_QUERY | WgpuFeatures::TIMESTAMP_QUERY_INSIDE_ENCODERS)
    {
        error!("render profile requires native encoder timestamp queries");
        profile.fail(ProfileFailure::UnsupportedTimestamps);
    }
}

fn begin<const LEAVES: bool>(mut context: RenderContext) {
    let recorder = context.diagnostic_recorder();
    if let Some(recorder) = recorder {
        let name = if LEAVES {
            "forest_cull_leaves"
        } else {
            "forest_cull_branches"
        };
        // A pipe keeps this begin and its end on the same worker thread.
        // Its three deferred encoders are queued in begin/dispatch/end order.
        recorder.begin_time_span(context.command_encoder(), name.into());
    }
}

fn end(mut context: RenderContext) {
    if let Some(recorder) = context.diagnostic_recorder() {
        recorder.end_time_span(context.command_encoder());
    }
}
