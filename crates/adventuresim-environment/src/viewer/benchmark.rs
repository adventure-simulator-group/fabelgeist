//! Reproducible component measurements, not browser-area acceptance.

use std::path::PathBuf;
use std::time::{Duration, Instant};

use adventuresim_environment::prepared::ForestFrame;
use adventuresim_environment::render::ForestDetail;
use adventuresim_environment::render::benchmark::{ForestBenchmark, ForestBenchmarkCounts};
use bevy::prelude::*;
use fabelgeist_botany::viewer::sim::{BranchInstance, LeafInstance, PlantInstance};
use serde_json::json;

use crate::stage::Orbit;

/// Consecutive readbacks must agree within this bounded number of attempts.
const MAX_READBACKS: usize = 8;
const REPORT_VERSION: u32 = 3;
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
    pub asset_load: Duration,
    pub asset_decode: Duration,
    pub asset_bytes: usize,
    pub mesh_bytes: usize,
    pub library_models: usize,
    pub grid_classes: usize,
    pub library_branches: usize,
    pub library_leaves: usize,
    pub placed_branches: usize,
    pub placed_leaves: usize,
    pub branch_slots: usize,
    pub leaf_slots: usize,
}

pub enum ArrivalPath {
    GrowthReplay,
    PreparedUnique,
    PreparedShared,
}

#[derive(Resource)]
pub struct Measurement {
    arrival: ArrivalPath,
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
    pixels: PixelStability,
    first_readback: Option<Duration>,
}

pub enum Readback {
    Repeat,
    Stable,
}

/// Consecutive GPU readbacks must agree, including untimed correctness captures.
#[derive(Resource, Default)]
pub struct PixelStability {
    attempts: usize,
    previous: Option<Vec<u8>>,
}

impl PixelStability {
    pub fn attempts(&self) -> usize {
        self.attempts
    }
    pub fn observe(&mut self, image: &Image) -> Result<Readback, MeasurementError> {
        self.attempts += 1;
        let pixels = image.data.as_ref().ok_or(MeasurementError::MissingPixels)?;
        let stable = self.previous.as_ref() == Some(pixels);
        self.previous = Some(pixels.clone());
        if stable {
            return Ok(Readback::Stable);
        }
        if self.attempts < MAX_READBACKS {
            Ok(Readback::Repeat)
        } else {
            Err(MeasurementError::UnstablePixels)
        }
    }
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
    FailedSubmission(Box<ForestBenchmarkCounts>),
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
        frame: &ForestFrame,
        detail: ForestDetail,
        camera: Orbit,
        arrival: ArrivalPath,
    ) -> Self {
        Self {
            request,
            arrival,
            preparation,
            scene: frame.label.clone(),
            years: frame.time,
            plants: frame.source.plants,
            branches: frame.source.branches,
            leaves: frame.source.leaves,
            record_bytes: frame.source.branches * size_of::<BranchInstance>()
                + frame.source.leaves * size_of::<LeafInstance>()
                + frame.source.plants * size_of::<PlantInstance>(),
            detail,
            camera,
            handoff: None,
            previous_frame: None,
            frames: Vec::new(),
            pixels: PixelStability::default(),
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
            return Err(MeasurementError::FailedSubmission(Box::new(counts)));
        }
        if self.request.started.elapsed() > READINESS_TIMEOUT {
            return Err(MeasurementError::ReadinessTimeout);
        }
        Ok(())
    }

    pub fn readback_number(&self) -> usize {
        self.pixels.attempts()
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
        let now = self.request.started.elapsed();
        self.first_readback.get_or_insert(now);
        if matches!(self.pixels.observe(image)?, Readback::Repeat) {
            return Ok(Readback::Repeat);
        }
        let counts = probe.counts();
        if counts.discarded_writes > 0 || counts.gpu_failures > 0 || counts.superseded_writes > 0 {
            return Err(MeasurementError::FailedSubmission(Box::new(counts)));
        }
        let submission = counts
            .first_submission_micros
            .ok_or(MeasurementError::MissingSubmission)?;
        let milliseconds = |duration: Duration| duration.as_secs_f64() * 1_000.0;
        let report = json!({
            "schema_version": REPORT_VERSION,
            "scope": "native_offscreen_forest_component",
            "area_acceptance": "unassessed",
            "arrival_path": match self.arrival { ArrivalPath::GrowthReplay => "growth_replay", ArrivalPath::PreparedUnique => "prepared_unique", ArrivalPath::PreparedShared => "prepared_shared" },
            "scene": self.scene,
            "years": self.years,
            "resolution": [image.width(), image.height()],
            "camera": {
                "focus_metres": self.camera.focus.to_array(),
                "radius_metres": self.camera.radius,
                "yaw_radians": self.camera.yaw,
                "pitch_radians": self.camera.pitch,
            },
            "counts": self.workload(&counts),
            "effective_detail": {
                "tube_sides": self.detail.tube_sides, "tube_spans": self.detail.tube_spans,
                "leaf_blades": self.detail.leaf_blades,
                "full_pixels": self.detail.full_pixels,
                "middle_pixels": self.detail.middle_pixels, "leaves": self.detail.leaves,
                "level_selection": self.detail.level,
            },
            "stages_ms": {
                "scene_construction": milliseconds(self.preparation.scene),
                "simulation_gpu_setup": milliseconds(self.preparation.gpu_setup),
                "growth": milliseconds(self.preparation.growth),
                "snapshot_packing": milliseconds(self.preparation.packing),
                "required_asset_file_read": milliseconds(self.preparation.asset_load),
                "required_asset_decode_validate": milliseconds(self.preparation.asset_decode),
            },
            "elapsed_from_request_ms": {
                "cpu_handoff": self.handoff.map(milliseconds),
                "first_completed_forest_draw_commands": submission as f64 / 1_000.0,
                "first_readback": self.first_readback.map(milliseconds),
                "stable_readback": milliseconds(now),
            },
            "cpu_gpu_waited_frame_ms": self.frames.iter().copied().map(milliseconds).collect::<Vec<_>>(),
            "stable_readbacks": 2,
            "readback_attempts": self.pixels.attempts(),
            "limits": [
                "native offscreen component; no browser, area transfer or production travel",
                "submission acknowledgement is not pixel coverage; review both saved readbacks",
                "encoded indirect draws do not establish nonzero visible counts or observed cull dispatch",
                "settled readbacks include the existing one-second and thirty-frame warmup",
                "readbacks occur after the requested steady-frame interval and may include earlier PNG writes",
                "GPU waits serialize frames; report separately from ordinary presentation",
                "uploaded_record_bytes include compact placements but exclude meshes, instance buffers and textures",
                "library mesh bytes count vertex/index payloads requested for GPU residency, not peak resident memory",
                "LOD record/slot counts describe submitted batches before per-tree GPU culling, not actual visible vertex work",
                "fresh file reads may hit operating-system storage caches; this is not a network measurement",
            ],
        });
        let bytes = serde_json::to_vec_pretty(&report)?;
        std::fs::write(&self.request.report, bytes).map_err(|source| MeasurementError::Write {
            path: self.request.report.clone(),
            source,
        })?;
        Ok(Readback::Stable)
    }

    fn workload(&self, counts: &ForestBenchmarkCounts) -> serde_json::Value {
        json!({
            "plants": self.plants, "branches": self.branches,
            "leaf_clusters": self.leaves, "structural_record_bytes": self.record_bytes,
            "queued_writes": counts.queued_writes,
            "submitted_writes": counts.submitted_writes,
            "superseded_writes": counts.superseded_writes,
            "discarded_writes": counts.discarded_writes,
            "uploaded_record_bytes": counts.uploaded_bytes,
            "gpu_failures": counts.gpu_failures,
            "required_asset_file_bytes": self.preparation.asset_bytes,
            "library_mesh_vertex_index_bytes": self.preparation.mesh_bytes,
            "library_models": self.preparation.library_models,
            "shared_grid_classes": self.preparation.grid_classes,
            "library_branches": self.preparation.library_branches,
            "library_leaf_clusters": self.preparation.library_leaves,
            "records_implied_by_shared_placements": {
                "branches": self.preparation.placed_branches,
                "leaf_clusters": self.preparation.placed_leaves,
            },
            "capacity_slots_implied_by_shared_placements": {
                "branches": self.preparation.branch_slots,
                "leaf_clusters": self.preparation.leaf_slots,
            },
            "expected_branch_draw_instances": counts.expected_branch_instances,
            "expected_leaf_draw_instances": counts.expected_leaf_instances,
            "completed_spatial_batches": counts.expected_batches,
            "required_gpu_meshes": counts.expected_meshes,
            "settled_draw_instances_by_lod": {
                "branches": counts.branch_lod.instances,
                "leaves": counts.leaf_lod.instances,
            },
            "settled_batch_records_by_lod": {
                "branches": { "valid_records": counts.branch_lod.records, "allocated_slots": counts.branch_lod.slots },
                "leaves": { "valid_records": counts.leaf_lod.records, "allocated_slots": counts.leaf_lod.slots },
            },
        })
    }
}
