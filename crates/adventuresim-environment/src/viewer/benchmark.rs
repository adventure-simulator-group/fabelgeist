//! Reproducible component measurements, not browser-area acceptance.

use std::path::PathBuf;
use std::time::{Duration, Instant};

use adventuresim_environment::prepared::ForestFrame;
use adventuresim_environment::render::ForestDetail;
use adventuresim_environment::render::benchmark::profile::{ProfileReportError, RenderProfile};
use adventuresim_environment::render::benchmark::{
    ForestBenchmark, ForestBenchmarkCounts, ForestRequestId,
};
use adventuresim_environment::render::trees::TreeMeshStats;
use bevy::diagnostic::DiagnosticsStore;
use bevy::prelude::*;
use fabelgeist_botany::viewer::sim::{BranchInstance, LeafInstance, PlantInstance};
use serde_json::json;

use crate::capture_view::CaptureView;

mod arrival;
mod report;

/// Consecutive readbacks must agree within this bounded number of attempts.
const MAX_READBACKS: usize = 8;
const REPORT_VERSION: u32 = 9;
pub(crate) const READINESS_TIMEOUT: Duration = Duration::from_secs(240);

pub struct Request {
    pub identity: ForestRequestId,
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

#[derive(Clone, Copy)]
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
    camera: CaptureView,
    handoff: Option<Duration>,
    previous_frame: Option<Instant>,
    frames: Vec<Duration>,
    pixels: PixelStability,
    first_readback: Option<Duration>,
    arrival_frame: arrival::ArrivalFrame,
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
    #[error("the first GPU-ready forest frame was not captured")]
    MissingArrival,
    #[error("the screenshot or render probe belongs to a different arrival request")]
    WrongRequest,
    #[error("the first-frame readback arrived outside its request")]
    UnexpectedArrival,
    #[error("could not encode or save the first forest frame")]
    ArrivalImage(#[source] Box<dyn std::error::Error + Send + Sync>),
    #[error("the screenshot contains no pixel data")]
    MissingPixels,
    #[error("the frozen snapshot did not settle after {MAX_READBACKS} readbacks")]
    UnstablePixels,
    #[error("the forest GPU submission was not acknowledged")]
    MissingSubmission,
    #[error("the final full forest validation was not acknowledged")]
    MissingFinalValidation,
    #[error(transparent)]
    RenderProfile(#[from] ProfileReportError),
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
    pub fn request_id(&self) -> ForestRequestId {
        self.request.identity
    }

    pub fn new(
        request: Request,
        preparation: Preparation,
        frame: &ForestFrame,
        detail: ForestDetail,
        camera: CaptureView,
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
            arrival_frame: arrival::ArrivalFrame::Pending,
        }
    }

    pub fn handoff_finished(&mut self) {
        self.handoff
            .get_or_insert_with(|| self.request.started.elapsed());
    }

    pub fn check(&self, probe: &ForestBenchmark) -> Result<(), MeasurementError> {
        self.require_request(probe.request_id())?;
        if probe.status().failed {
            return Err(MeasurementError::FailedSubmission(Box::new(probe.counts())));
        }
        if self.request.started.elapsed() > READINESS_TIMEOUT {
            return Err(MeasurementError::ReadinessTimeout);
        }
        Ok(())
    }

    fn require_request(&self, owner: ForestRequestId) -> Result<(), MeasurementError> {
        if self.request_id() != owner {
            return Err(MeasurementError::WrongRequest);
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
        owner: ForestRequestId,
        image: &Image,
        probe: &ForestBenchmark,
        topology: Option<&TreeMeshStats>,
        profile: Option<&RenderProfile>,
        diagnostics: Option<&DiagnosticsStore>,
    ) -> Result<Readback, MeasurementError> {
        self.require_request(owner)?;
        self.check(probe)?;
        let now = self.request.started.elapsed();
        self.first_readback.get_or_insert(now);
        if matches!(self.pixels.observe(image)?, Readback::Repeat) {
            return Ok(Readback::Repeat);
        }
        self.write_report(now, image, probe, topology, profile, diagnostics)?;
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
                "branches": { "valid_records": counts.branch_lod.records, "indexed_record_slots": counts.branch_lod.indexed_slots },
                "leaves": { "valid_records": counts.leaf_lod.records, "indexed_record_slots": counts.leaf_lod.indexed_slots },
            },
        })
    }
}
