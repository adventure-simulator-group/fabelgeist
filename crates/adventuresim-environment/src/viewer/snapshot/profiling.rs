//! Install and save renderer diagnostics for frozen snapshots.

use std::path::Path;
use std::time::Instant;

use crate::benchmark::{ArrivalPath, READINESS_TIMEOUT};
use adventuresim_environment::render::benchmark::profile::{
    ProfileMode, ProfileReportError, RenderProfile,
};
use anyhow::Result;
use bevy::app::PluginsState;
use bevy::diagnostic::DiagnosticsStore;
use bevy::prelude::App;

pub(super) fn install(
    app: &mut App,
    mode: Option<ProfileMode>,
    arrival: ArrivalPath,
) -> Result<()> {
    let Some(mode) = mode else {
        return Ok(());
    };
    let started = Instant::now();
    while app.plugins_state() == PluginsState::Adding {
        anyhow::ensure!(
            started.elapsed() < READINESS_TIMEOUT,
            "render plugins exceeded capture readiness deadline"
        );
        bevy::tasks::tick_global_task_pools_on_main_thread();
    }
    app.finish();
    let profile = RenderProfile::new(mode);
    match arrival {
        ArrivalPath::PreparedShared => profile.install_shared(app)?,
        ArrivalPath::PreparedUnique => profile.install(app)?,
        ArrivalPath::GrowthReplay => anyhow::bail!("render profiling requires prepared input"),
    }
    // Pipelined cleanup transfers the render app to its thread. Install first.
    app.cleanup();
    Ok(())
}

pub(super) fn save(
    profile: Option<&RenderProfile>,
    diagnostics: Option<&DiagnosticsStore>,
    path: &Path,
) -> Result<()> {
    let Some(profile) = profile else {
        return Ok(());
    };
    let report = profile.report(diagnostics.ok_or(ProfileReportError::MissingStore)?)?;
    std::fs::write(
        path.with_extension("profile.json"),
        serde_json::to_vec_pretty(&report)?,
    )?;
    println!(
        "forest render profile: {:?}, {} sampled frames, {} wrapped cull systems",
        report.mode, report.sampled_frames, report.wrapped_cull_systems
    );
    Ok(())
}
