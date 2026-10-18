//! Reproducible component measurements, not browser-area acceptance.

use std::path::PathBuf;
use std::time::{Duration, Instant};

use adventuresim_environment::render::ForestDetail;
use adventuresim_environment::render::benchmark::{ForestBenchmark, ForestBenchmarkCounts};
use bevy::prelude::*;
use fabelgeist_botany::viewer::sim::{BranchInstance, LeafInstance, PlantInstance, Snapshot};
use serde_json::json;

use crate::stage::Orbit;

/// Consecutive readbacks must agree within this bounded number of attempts.
const MAX_READBACKS: usize = 8;
const REPORT_VERSION: u32 = 1;
const READINESS_TIMEOUT: Duration = Duration::from_secs(240);

pub struct Request {
    pub report: PathBuf,
    pub started: Instant,
}

#[derive(Default)]
pub struct Preparation {
    pub scene: Duration,
    pub gpu_setup: Duration,
    pub growth: Duration,
    pub packing: Duration,
}

#[derive(Resource)]
pub struct Measurement {
    request: Request,
    preparation: Preparation,
    scene: String,
    years: f32,
    plants: usize,
    branches: usize,
    leaves: usize,
    record_bytes: usize,
    detail: ForestDetail,
    camera: Orbit,
    handoff: Option<Duration>,
    previous_frame: Option<Instant>,
    frames: Vec<Duration>,
    readbacks: usize,
    previous_pixels: Option<Vec<u8>>,
    first_readback: Option<Duration>,
}

pub enum Readback {
    Repeat,
    Stable,
}

#[derive(Debug, thiserror::Error)]
pub enum MeasurementError {
    #[error("the screenshot contains no pixel data")]
    MissingPixels,
    #[error("the frozen snapshot did not settle after {MAX_READBACKS} readbacks")]
    UnstablePixels,
    #[error("the forest GPU submission was not acknowledged")]
    MissingSubmission,
    #[error("the forest upload or GPU probe failed: {0:?}")]
    FailedSubmission(ForestBenchmarkCounts),
    #[error("the forest readiness probe exceeded its bounded deadline")]
    ReadinessTimeout,
    #[error("could not serialize the foliage measurement")]
    Serialize(#[from] serde_json::Error),
    #[error("could not write the foliage measurement to {path}")]
    Write {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
}

impl Measurement {
    pub fn new(
        request: Request,
        preparation: Preparation,
        scene: String,
        snapshot: &Snapshot,
        detail: ForestDetail,
        camera: Orbit,
    ) -> Self {
        Self {
            request,
            preparation,
            scene,
            years: snapshot.time,
            plants: snapshot.plants,
            branches: snapshot.branches.len(),
            leaves: snapshot.leaves.len(),
            record_bytes: snapshot.branches.len() * size_of::<BranchInstance>()
                + snapshot.leaves.len() * size_of::<LeafInstance>()
                + snapshot.plant_bounds.len() * size_of::<PlantInstance>(),
            detail,
            camera,
            handoff: None,
            previous_frame: None,
            frames: Vec::new(),
            readbacks: 0,
            previous_pixels: None,
            first_readback: None,
        }
    }

    pub fn handoff_finished(&mut self) {
        self.handoff
            .get_or_insert_with(|| self.request.started.elapsed());
    }

    pub fn check(&self, probe: &ForestBenchmark) -> Result<(), MeasurementError> {
        let counts = probe.counts();
        if counts.discarded_writes > 0 || counts.gpu_failures > 0 || counts.superseded_writes > 0 {
            return Err(MeasurementError::FailedSubmission(counts));
        }
        if self.request.started.elapsed() > READINESS_TIMEOUT {
            return Err(MeasurementError::ReadinessTimeout);
        }
        Ok(())
    }

    pub fn readback_number(&self) -> usize {
        self.readbacks
    }

    /// Call only for the fixed-camera interval being measured.
    pub fn frame(&mut self) {
        let now = Instant::now();
        if let Some(previous) = self.previous_frame.replace(now) {
            self.frames.push(now - previous);
        }
    }

    pub fn readback(
        &mut self,
        image: &Image,
        probe: &ForestBenchmark,
    ) -> Result<Readback, MeasurementError> {
        self.readbacks += 1;
        let now = self.request.started.elapsed();
        self.first_readback.get_or_insert(now);
        let pixels = image.data.as_ref().ok_or(MeasurementError::MissingPixels)?;
        let stable = self.previous_pixels.as_ref() == Some(pixels);
        self.previous_pixels = Some(pixels.clone());
        if !stable {
            return if self.readbacks < MAX_READBACKS {
                Ok(Readback::Repeat)
            } else {
                Err(MeasurementError::UnstablePixels)
            };
        }
        let counts = probe.counts();
        if counts.discarded_writes > 0 || counts.gpu_failures > 0 || counts.superseded_writes > 0 {
            return Err(MeasurementError::FailedSubmission(counts));
        }
        let submission = counts
            .first_submission_micros
            .ok_or(MeasurementError::MissingSubmission)?;
        let milliseconds = |duration: Duration| duration.as_secs_f64() * 1_000.0;
        let report = json!({
            "schema_version": REPORT_VERSION,
            "scope": "native_offscreen_forest_component",
            "area_acceptance": "unassessed",
            "scene": self.scene,
            "years": self.years,
            "resolution": [image.width(), image.height()],
            "camera": {
                "focus_metres": self.camera.focus.to_array(),
                "radius_metres": self.camera.radius,
                "yaw_radians": self.camera.yaw,
                "pitch_radians": self.camera.pitch,
            },
            "counts": {
                "plants": self.plants, "branches": self.branches,
                "leaf_clusters": self.leaves, "structural_record_bytes": self.record_bytes,
                "queued_writes": counts.queued_writes,
                "submitted_writes": counts.submitted_writes,
                "superseded_writes": counts.superseded_writes,
                "discarded_writes": counts.discarded_writes,
                "uploaded_record_bytes": counts.uploaded_bytes,
                "gpu_failures": counts.gpu_failures,
            },
            "effective_detail": {
                "tube_sides": self.detail.tube_sides, "tube_spans": self.detail.tube_spans,
                "leaf_blades": self.detail.leaf_blades,
                "full_pixels": self.detail.full_pixels,
                "middle_pixels": self.detail.middle_pixels, "leaves": self.detail.leaves,
            },
            "stages_ms": {
                "scene_construction": milliseconds(self.preparation.scene),
                "simulation_gpu_setup": milliseconds(self.preparation.gpu_setup),
                "growth": milliseconds(self.preparation.growth),
                "snapshot_packing": milliseconds(self.preparation.packing),
            },
            "elapsed_from_request_ms": {
                "cpu_handoff": self.handoff.map(milliseconds),
                "first_completed_forest_draw_commands": submission as f64 / 1_000.0,
                "first_readback": self.first_readback.map(milliseconds),
                "stable_readback": milliseconds(now),
            },
            "cpu_gpu_waited_frame_ms": self.frames.iter().copied().map(milliseconds).collect::<Vec<_>>(),
            "stable_readbacks": 2,
            "readback_attempts": self.readbacks,
            "limits": [
                "native offscreen component; no browser, area transfer or production travel",
                "submission acknowledgement is not pixel coverage; review both saved readbacks",
                "encoded indirect draws do not establish nonzero visible counts or observed cull dispatch",
                "settled readbacks include the existing one-second and thirty-frame warmup",
                "readbacks occur after the requested steady-frame interval and may include earlier PNG writes",
                "GPU waits serialize frames; report separately from ordinary presentation",
                "record bytes exclude instance buffers, textures and peak resident memory",
            ],
        });
        let bytes = serde_json::to_vec_pretty(&report)?;
        std::fs::write(&self.request.report, bytes).map_err(|source| MeasurementError::Write {
            path: self.request.report.clone(),
            source,
        })?;
        Ok(Readback::Stable)
    }
}
