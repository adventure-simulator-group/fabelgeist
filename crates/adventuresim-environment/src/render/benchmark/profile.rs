//! Opt-in native phase diagnostics, with an uninstrumented control.

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use anyhow::{Result, anyhow};
use bevy::diagnostic::DiagnosticsStore;
use bevy::prelude::*;
use bevy::render::RenderApp;
use bevy_eidolon::prelude::InstancedMaterial;

use super::super::{BranchMaterial, ForestPart, LeafMaterial, trees};

mod cpu;
mod cull;

/// Control installs no clocks or cull wrappers. Both modes use the same build.
#[derive(Clone, Copy, Debug, PartialEq, Eq, clap::ValueEnum, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProfileMode {
    Control,
    Instrumented,
}

#[derive(Resource, Clone)]
pub struct RenderProfile {
    mode: ProfileMode,
    data: Arc<Mutex<cpu::ProfileData>>,
}

/// Wall spans include scheduler gaps; they are not exclusive CPU busy time.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RenderStage {
    Queue,
    Prepare,
    GraphEncode,
    GraphSubmit,
    GpuWait,
}

#[derive(Clone, Copy, Debug, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProfileFailure {
    UnsupportedTimestamps,
    IncompleteFrame,
    UnpairedStage,
    DuplicateStage,
}

#[derive(Clone, Copy, Debug)]
pub enum DiagnosticClock {
    Cpu,
    Gpu,
}

#[derive(Debug, thiserror::Error)]
pub enum ProfileReportError {
    #[error("render diagnostics store is missing")]
    MissingStore,
    #[error("render profile lock is poisoned")]
    Poisoned,
    #[error("render profile failed: {0:?}")]
    Failed(Vec<ProfileFailure>),
    #[error("render diagnostic history contains invalid values")]
    InvalidHistory,
    #[error("no CPU profile frames were observed during Sampling")]
    MissingFrames,
    #[error("missing {clock:?} cull diagnostic for {part:?}")]
    MissingDiagnostic {
        part: ForestPart,
        clock: DiagnosticClock,
    },
}

/// Bevy's asynchronous bounded history, deliberately separate from frame data.
#[derive(serde::Serialize)]
pub struct DiagnosticHistory {
    path: String,
    values_ms: Vec<f64>,
}

#[derive(serde::Serialize)]
pub struct ProfileReport {
    pub mode: ProfileMode,
    pub wrapped_cull_systems: usize,
    pub sampled_frames: usize,
    pub cpu_phase_wall_ms: Vec<BTreeMap<RenderStage, f64>>,
    pub diagnostic_histories: Vec<DiagnosticHistory>,
    pub failures: Vec<ProfileFailure>,
    limits: [&'static str; 4],
}

impl RenderProfile {
    pub fn new(mode: ProfileMode) -> Self {
        Self {
            mode,
            data: Arc::new(Mutex::new(cpu::ProfileData::default())),
        }
    }

    pub fn install(&self, app: &mut App) -> Result<()> {
        self.install_materials::<BranchMaterial, LeafMaterial>(app)
    }

    pub fn install_shared(&self, app: &mut App) -> Result<()> {
        self.install_materials::<trees::TreeBranchMaterial, trees::TreeLeafMaterial>(app)
    }

    fn install_materials<B: InstancedMaterial, L: InstancedMaterial>(
        &self,
        app: &mut App,
    ) -> Result<()> {
        app.insert_resource(self.clone());
        let render = app
            .get_sub_app_mut(RenderApp)
            .ok_or_else(|| anyhow!("render profiling requires the render app"))?;
        render.insert_resource(self.clone());
        if self.mode == ProfileMode::Instrumented {
            cull::install::<B, L>(render.world_mut())?;
            cpu::install(render);
        }
        Ok(())
    }

    pub fn healthy(&self) -> bool {
        self.data.lock().is_ok_and(|data| data.failures.is_empty())
    }

    pub fn ready(&self) -> bool {
        self.mode == ProfileMode::Control
            || self.data.lock().is_ok_and(|data| !data.frames.is_empty())
    }

    pub fn report(
        &self,
        diagnostics: &DiagnosticsStore,
    ) -> Result<ProfileReport, ProfileReportError> {
        let data = self.data.lock().map_err(|_| ProfileReportError::Poisoned)?;
        if !data.failures.is_empty() {
            return Err(ProfileReportError::Failed(data.failures.clone()));
        }
        let mut histories: Vec<_> = diagnostics
            .iter()
            .filter(|diagnostic| {
                let path = diagnostic.path().as_str();
                path.ends_with("elapsed_cpu") || path.ends_with("elapsed_gpu")
            })
            .map(|diagnostic| DiagnosticHistory {
                path: diagnostic.path().as_str().to_owned(),
                values_ms: diagnostic.values().copied().collect(),
            })
            .collect();
        histories.sort_by(|a, b| a.path.cmp(&b.path));
        if !histories.iter().all(|history| {
            history
                .values_ms
                .iter()
                .all(|value| value.is_finite() && *value >= 0.0)
        }) {
            return Err(ProfileReportError::InvalidHistory);
        }
        if self.mode == ProfileMode::Instrumented {
            if data.frames.is_empty() {
                return Err(ProfileReportError::MissingFrames);
            }
            for part in [ForestPart::Branches, ForestPart::Leaves] {
                let marker = match part {
                    ForestPart::Branches => "forest_cull_branches",
                    ForestPart::Leaves => "forest_cull_leaves",
                };
                for clock in [DiagnosticClock::Cpu, DiagnosticClock::Gpu] {
                    let unit = match clock {
                        DiagnosticClock::Cpu => "elapsed_cpu",
                        DiagnosticClock::Gpu => "elapsed_gpu",
                    };
                    let suffix = format!("/{marker}/{unit}");
                    if !histories.iter().any(|history| {
                        history.path.ends_with(&suffix) && !history.values_ms.is_empty()
                    }) {
                        return Err(ProfileReportError::MissingDiagnostic { part, clock });
                    }
                }
            }
        }
        let frames: Vec<_> = data
            .frames
            .iter()
            .map(|frame| {
                frame
                    .iter()
                    .map(|(stage, duration)| (*stage, duration.as_secs_f64() * 1_000.0))
                    .collect()
            })
            .collect();
        Ok(ProfileReport {
            mode: self.mode,
            wrapped_cull_systems: data.wrapped_cull_systems,
            sampled_frames: frames.len(),
            cpu_phase_wall_ms: frames,
            diagnostic_histories: histories,
            failures: data.failures.clone(),
            limits: [
                "CPU wall spans are latched at Queue during Sampling, not ordinal-matched to main-world frame intervals",
                "graph submit includes encoder finishing, queue submission and uncovered-target handling",
                "cull CPU histories cover the encoding body, excluding deferred encoder finishing and queue submission",
                "GPU/CPU diagnostic histories are asynchronous bounded aggregates, not the exact steady-window vector; do not sum CPU, GPU and wait spans",
            ],
        })
    }

    /// These clocks surround only the existing blocking GPU poll.
    pub(super) fn begin_wait(&self) {
        if self.mode == ProfileMode::Instrumented {
            self.begin(RenderStage::GpuWait);
        }
    }

    pub(super) fn end_wait(&self) {
        if self.mode == ProfileMode::Control {
            return;
        }
        self.end(RenderStage::GpuWait);
        if let Ok(mut data) = self.data.lock() {
            data.finish_frame();
        }
    }

    fn fail(&self, failure: ProfileFailure) {
        if let Ok(mut data) = self.data.lock() {
            data.failures.push(failure);
        }
    }
}
