//! Render-device telemetry shared by scene and regional-map readiness.
use bevy::{
    prelude::*,
    render::{Render, RenderApp, RenderSystems, render_resource::PipelineCache},
};
use std::sync::atomic::{AtomicUsize, Ordering};

static WAITING_PIPELINES: AtomicUsize = AtomicUsize::new(0);

pub(super) fn install(app: &mut App) {
    if let Some(render) = app.get_sub_app_mut(RenderApp) {
        render.add_systems(Render, pipeline_status.after(RenderSystems::Prepare));
    }
}

pub(crate) fn waiting_pipelines() -> usize {
    WAITING_PIPELINES.load(Ordering::Relaxed)
}

fn pipeline_status(cache: Res<PipelineCache>) {
    WAITING_PIPELINES.store(cache.waiting_pipelines().count(), Ordering::Relaxed);
}
