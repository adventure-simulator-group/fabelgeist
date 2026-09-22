//! Frame statistics: a rolling window of CPU frame times (mean, median, p95,
//! worst 1 %), the GPU pass timings bevy's render diagnostics expose on
//! native, an always-on egui overlay, and on native a CSV log plus
//! screenshot / timed-exit hooks so a run can be scripted from the shell:
//!
//! ```text
//! BENCH_LOG=run.csv BENCH_EXIT_AFTER=30 BENCH_SCREENSHOT=shot.png bench.exe
//! ```

use std::collections::VecDeque;

use bevy::diagnostic::DiagnosticsStore;
use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};

use crate::settings::BenchSettings;

const WINDOW: usize = 300;

#[derive(Resource, Default)]
pub struct FrameStats {
    frames_ms: VecDeque<f32>,
    /// Sorted copy of the window, rebuilt when read.
    sorted: Vec<f32>,
    pub mean_ms: f32,
    pub p50_ms: f32,
    pub p95_ms: f32,
    pub p99_ms: f32,
    pub gpu: Vec<(String, f32)>,
}

impl FrameStats {
    fn push(&mut self, ms: f32) {
        if self.frames_ms.len() == WINDOW {
            self.frames_ms.pop_front();
        }
        self.frames_ms.push_back(ms);
        self.sorted.clear();
        self.sorted.extend(self.frames_ms.iter().copied());
        self.sorted.sort_by(|a, b| a.total_cmp(b));
        let n = self.sorted.len().max(1);
        let at = |q: f32| self.sorted[((n as f32 - 1.0) * q).round() as usize];
        self.mean_ms = self.sorted.iter().sum::<f32>() / n as f32;
        self.p50_ms = at(0.5);
        self.p95_ms = at(0.95);
        self.p99_ms = at(0.99);
    }
    pub fn fps(&self) -> f32 {
        if self.mean_ms > 0.0 { 1000.0 / self.mean_ms } else { 0.0 }
    }
    pub fn window_len(&self) -> usize {
        self.frames_ms.len()
    }
}

pub struct StatsPlugin;

impl Plugin for StatsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FrameStats>()
            .add_systems(First, collect)
            .add_systems(EguiPrimaryContextPass, overlay);
        #[cfg(not(target_family = "wasm"))]
        app.init_resource::<NativeHooks>()
            .add_systems(Update, native_hooks);
    }
}

fn collect(time: Res<Time<Real>>, store: Res<DiagnosticsStore>, mut stats: ResMut<FrameStats>) {
    let ms = time.delta_secs() * 1000.0;
    if ms > 0.0 {
        stats.push(ms);
    }
    // Render diagnostics: every `<pass>/elapsed_gpu` bevy records, smoothed.
    let mut gpu: Vec<(String, f32)> = store
        .iter()
        .filter(|d| d.path().as_str().ends_with("elapsed_gpu"))
        .filter_map(|d| {
            d.smoothed().map(|v| {
                let path = d.path().as_str();
                let name = path
                    .trim_end_matches("/elapsed_gpu")
                    .trim_start_matches("render/");
                (name.to_string(), v as f32)
            })
        })
        .collect();
    gpu.sort_by(|a, b| b.1.total_cmp(&a.1));
    gpu.truncate(8);
    stats.gpu = gpu;
}

fn overlay(mut contexts: EguiContexts, stats: Res<FrameStats>, settings: Res<BenchSettings>) {
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };
    egui::Area::new(egui::Id::new("bench-stats"))
        .fixed_pos((8.0, 8.0))
        .show(ctx, |ui| {
            egui::Frame::popup(ui.style())
                .fill(egui::Color32::from_black_alpha(160))
                .show(ui, |ui| {
                    ui.monospace(format!(
                        "{:6.1} fps  mean {:5.2} ms  p50 {:5.2}  p95 {:5.2}  p99 {:5.2}  ({} frames)",
                        stats.fps(),
                        stats.mean_ms,
                        stats.p50_ms,
                        stats.p95_ms,
                        stats.p99_ms,
                        stats.window_len()
                    ));
                    for (name, ms) in &stats.gpu {
                        ui.monospace(format!("gpu {name:<32} {ms:6.3} ms"));
                    }
                    ui.monospace(settings.summary());
                });
        });
}

#[cfg(not(target_family = "wasm"))]
#[derive(Resource)]
struct NativeHooks {
    log: Option<std::path::PathBuf>,
    exit_after: Option<f32>,
    screenshot: Option<String>,
    /// `BENCH_SCREENSHOT_AT` seconds; default 6 so pipelines have compiled.
    screenshot_at: f32,
    next_log: f32,
    screenshot_done: bool,
    /// `BENCH_SWITCH="5={\"tree_model\":\"Oak\"};12={\"shading\":\"Custom\"}"`:
    /// settings patches applied at those seconds, for scripted A/B runs.
    switches: Vec<(f32, serde_json::Value)>,
}

#[cfg(not(target_family = "wasm"))]
impl Default for NativeHooks {
    fn default() -> Self {
        let env = |k: &str| std::env::var(k).ok().filter(|v| !v.is_empty());
        Self {
            log: env("BENCH_LOG").map(Into::into),
            exit_after: env("BENCH_EXIT_AFTER").and_then(|v| v.parse().ok()),
            screenshot: env("BENCH_SCREENSHOT"),
            screenshot_at: env("BENCH_SCREENSHOT_AT")
                .and_then(|v| v.parse().ok())
                .unwrap_or(6.0),
            next_log: 3.0,
            screenshot_done: false,
            switches: env("BENCH_SWITCH")
                .map(|spec| {
                    spec.split(';')
                        .filter_map(|item| {
                            let (at, json) = item.split_once('=')?;
                            Some((at.trim().parse().ok()?, serde_json::from_str(json).ok()?))
                        })
                        .collect()
                })
                .unwrap_or_default(),
        }
    }
}

#[cfg(not(target_family = "wasm"))]
fn native_hooks(
    mut commands: Commands,
    time: Res<Time<Real>>,
    stats: Res<FrameStats>,
    mut settings: ResMut<BenchSettings>,
    mut hooks: ResMut<NativeHooks>,
    cull: Option<Res<crate::grass::culled::GrassCullStats>>,
    mut exit: MessageWriter<AppExit>,
) {
    use bevy::render::view::screenshot::{Screenshot, save_to_disk};
    use std::io::Write;

    let t = time.elapsed_secs();
    if let Some(index) = hooks.switches.iter().position(|(at, _)| t >= *at) {
        let (_, patch) = hooks.switches.remove(index);
        if let Ok(mut current) = serde_json::to_value(&*settings)
            && let (Some(current_map), Some(patch_map)) = (current.as_object_mut(), patch.as_object())
        {
            for (key, value) in patch_map {
                current_map.insert(key.clone(), value.clone());
            }
            match serde_json::from_value::<BenchSettings>(current) {
                Ok(next) => {
                    *settings = next;
                    info!("bench switch at {t:.1}s: {patch}");
                }
                Err(err) => warn!("BENCH_SWITCH patch ignored: {err}"),
            }
        }
    }
    if let Some(path) = hooks.log.clone()
        && t >= hooks.next_log
    {
        hooks.next_log = t + 2.0;
        let mut gpu: Vec<String> = stats
            .gpu
            .iter()
            .take(4)
            .map(|(name, ms)| format!("{name}={ms:.2}"))
            .collect();
        if let Some(cull) = &cull {
            gpu.push(format!(
                "cpu_cull={:.2} survivors={}",
                cull.last_ms,
                cull.survivors.iter().sum::<u32>()
            ));
        }
        let line = format!(
            "{t:.1},{:.2},{:.2},{:.2},{:.2},{:.1},\"{}\",\"{}\"\n",
            stats.mean_ms,
            stats.p50_ms,
            stats.p95_ms,
            stats.p99_ms,
            stats.fps(),
            settings.summary(),
            gpu.join(" ")
        );
        let new = !path.exists();
        if let Ok(mut file) = std::fs::OpenOptions::new().create(true).append(true).open(path) {
            if new {
                let _ = file.write_all(b"t,mean_ms,p50_ms,p95_ms,p99_ms,fps,settings,gpu_ms\n");
            }
            let _ = file.write_all(line.as_bytes());
        }
        info!("bench {}", line.trim_end());
    }
    if let Some(path) = hooks.screenshot.clone()
        && !hooks.screenshot_done
        && t >= hooks.screenshot_at
    {
        hooks.screenshot_done = true;
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk(path));
    }
    if let Some(limit) = hooks.exit_after
        && t >= limit
    {
        exit.write(AppExit::Success);
    }
}
