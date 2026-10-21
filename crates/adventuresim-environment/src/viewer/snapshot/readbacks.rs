//! Request-owned stable readbacks and capture completion.

use super::{profiling, sequence};
use crate::benchmark::{Measurement, PixelStability, Readback};
use adventuresim_environment::render::{
    benchmark::{ForestBenchmark, ForestRequestId, profile::RenderProfile},
    trees::TreeMeshStats,
};
use bevy::{
    diagnostic::DiagnosticsStore,
    prelude::*,
    render::view::screenshot::{Screenshot, ScreenshotCaptured, save_to_disk},
};
use std::path::PathBuf;

pub(super) fn capture(
    commands: &mut Commands,
    target: Handle<Image>,
    path: PathBuf,
    owner: Option<ForestRequestId>,
) {
    commands.spawn(Screenshot::image(target.clone())).observe(
        move |captured: On<ScreenshotCaptured>,
              mut commands: Commands,
              mut measurement: Option<ResMut<Measurement>>,
              mut pixels: Option<ResMut<PixelStability>>,
              probe: Option<Res<ForestBenchmark>>,
              topology: Option<Res<TreeMeshStats>>,
              profile: Option<Res<RenderProfile>>,
              diagnostics: Option<Res<DiagnosticsStore>>,
              mut sequence: Option<ResMut<sequence::Progress>>,
              inventory: sequence::Inventory,
              mut exit: MessageWriter<AppExit>| {
            if let Some(measurement) = measurement.as_mut() {
                let Some(probe) = probe.as_ref() else {
                    error!("the foliage benchmark probe is missing");
                    exit.write(AppExit::error());
                    return;
                };
                let Some(owner) = owner else { error!("missing screenshot request owner"); exit.write(AppExit::error()); return; };
                match measurement.readback(owner, &captured.image, probe, topology.as_deref(), profile.as_deref(), diagnostics.as_deref()) {
                    Ok(Readback::Repeat) => {
                        let attempt = measurement.readback_number();
                        save_to_disk(path.with_extension(format!("readback-{attempt:02}.png")))(
                            captured,
                        );
                        capture(&mut commands, target.clone(), path.clone(), Some(owner));
                        return;
                    }
                    Ok(Readback::Stable) => {}
                    Err(error) => {
                        error!("{error}");
                        exit.write(AppExit::error());
                        return;
                    }
                }
            }
            if let Some(pixels) = pixels.as_mut() {
                match pixels.observe(&captured.image) {
                    Ok(Readback::Repeat) => {
                        save_to_disk(path.with_extension(format!("readback-{:02}.png", pixels.attempts())))(captured);
                        capture(&mut commands, target.clone(), path.clone(), owner);
                        return;
                    }
                    Ok(Readback::Stable) => {}
                    Err(error) => { error!("{error}"); exit.write(AppExit::error()); return; }
                }
            }
            if let Err(error) = profiling::save(profile.as_deref(), diagnostics.as_deref(), &path) {
                error!("{error}"); exit.write(AppExit::error()); return;
            }
            if let Some(probe) = &probe {
                let counts = probe.counts();
                println!("forest validation: {:?}", probe.validation().evidence());
                println!("forest ready: {}/{} writes, {}/{} branch/leaf instances, {} batches, {} meshes", counts.submitted_writes, counts.queued_writes, counts.expected_branch_instances, counts.expected_leaf_instances, counts.expected_batches, counts.expected_meshes);
            }
            if let Some(topology) = topology {
                match serde_json::to_string(&*topology) {
                    Ok(json) => println!("forest topology: {json}"),
                    Err(error) => { error!("{error}"); exit.write(AppExit::error()); return; }
                }
            }
            if let Some(sequence) = sequence.as_mut() {
                let Some(owner) = owner else { error!("missing sequence request owner"); exit.write(AppExit::error()); return; };
                let Some(probe) = probe.as_ref() else { error!("missing sequence probe"); exit.write(AppExit::error()); return; };
                if let Err(error) = inventory.snapshot(probe, captured.entity).and_then(|assets| sequence.finish(owner, assets)) {
                    error!("{error:#}"); exit.write(AppExit::error()); return;
                }
            } else {
                exit.write(AppExit::Success);
            }
            save_to_disk(path.clone())(captured);
            println!("wrote {}", path.display());
        },
    );
}
