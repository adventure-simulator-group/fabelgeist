//! Render-world pipeline telemetry shared by every loading gate.
use bevy::{
    prelude::*,
    render::{Render, RenderApp, RenderSystems, render_resource::PipelineCache},
};
use std::sync::atomic::{AtomicUsize, Ordering};

static WAITING_PIPELINES: AtomicUsize = AtomicUsize::new(0);

/// Consecutive ready frames required before a loading gate trusts readiness.
pub(crate) const SETTLED_RENDER_FRAMES: usize = 4;

pub(crate) fn install(app: &mut App) {
    if let Some(render) = app.get_sub_app_mut(RenderApp) {
        render.add_systems(Render, pipeline_status.after(RenderSystems::Prepare));
    }
}

/// Number of pipelines the render world has queued but not finished compiling.
pub(crate) fn waiting_pipelines() -> usize {
    WAITING_PIPELINES.load(Ordering::Relaxed)
}

fn pipeline_status(cache: Res<PipelineCache>) {
    WAITING_PIPELINES.store(cache.waiting_pipelines().count(), Ordering::Relaxed);
}
