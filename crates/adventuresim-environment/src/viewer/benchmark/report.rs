//! Stable capture report serialization, outside the sampled frame interval.

use super::{ArrivalPath, Measurement, MeasurementError, REPORT_VERSION};
use adventuresim_environment::render::{
    benchmark::{
        ForestBenchmark, ValidationPhase,
        profile::{ProfileReportError, RenderProfile},
    },
    trees::TreeMeshStats,
};
use bevy::{diagnostic::DiagnosticsStore, prelude::Image};
use serde_json::json;
use std::time::Duration;

impl Measurement {
    pub(super) fn write_report(
        &self,
        now: Duration,
        image: &Image,
        probe: &ForestBenchmark,
        topology: Option<&TreeMeshStats>,
        profile: Option<&RenderProfile>,
        diagnostics: Option<&DiagnosticsStore>,
    ) -> Result<(), MeasurementError> {
        let validation = probe
            .validation()
            .evidence()
            .filter(|evidence| evidence.phase == ValidationPhase::Complete)
            .ok_or(MeasurementError::MissingFinalValidation)?;
        let profile = profile
            .map(|profile| profile.report(diagnostics.ok_or(ProfileReportError::MissingStore)?))
            .transpose()?;
        let counts = probe.counts();
        if counts.discarded_writes > 0 || counts.gpu_failures > 0 || counts.superseded_writes > 0 {
            return Err(MeasurementError::FailedSubmission(Box::new(counts)));
        }
        let submission = counts
            .first_submission_micros
            .ok_or(MeasurementError::MissingSubmission)?;
        let milliseconds = |duration: Duration| duration.as_secs_f64() * 1_000.0;
        let arrival_frame = self.arrival_evidence()?;
        let report = json!({
            "schema_version": REPORT_VERSION,
            "request_id": self.request_id(),
            "scope": "native_offscreen_forest_component",
            "area_acceptance": "unassessed",
            "arrival_path": match self.arrival { ArrivalPath::GrowthReplay => "growth_replay", ArrivalPath::PreparedUnique => "prepared_unique", ArrivalPath::PreparedShared => "prepared_shared" },
            "scene": self.scene,
            "years": self.years,
            "resolution": [image.width(), image.height()],
            "camera": {
                "focus_metres": self.camera.orbit.focus.to_array(),
                "radius_metres": self.camera.orbit.radius,
                "yaw_radians": self.camera.orbit.yaw,
                "pitch_radians": self.camera.orbit.pitch,
            },
            "view": self.camera.evidence(),
            "counts": self.workload(&counts),
            "effective_detail": {
                "tube_sides": self.detail.tube_sides, "tube_spans": self.detail.tube_spans,
                "leaf_blades": self.detail.leaf_blades,
                "full_pixels": self.detail.full_pixels,
                "middle_pixels": self.detail.middle_pixels, "leaves": self.detail.leaves,
                "level_selection": self.detail.level,
            },
            "shared_meshes": topology,
            "native_residency": probe.native_residency(),
            "probe_validation": validation,
            "renderer_profile": profile,
            "arrival_frame": arrival_frame,
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
                "settled readbacks wait at least one second after arrival receipt and thirty frames (sixty in persistent sequences)",
                "readbacks occur after the requested steady-frame interval and may include earlier PNG writes",
                "endpoint validation does not establish intermediate-frame structural or pipeline-error coverage",
                "both policies retain per-draw counters and GPU waits; their difference is not total observer cost",
                "GPU waits serialize frames; report separately from ordinary presentation",
                "uploaded_record_bytes include compact placements but exclude meshes, instance buffers and textures",
                "library mesh bytes describe canonical source grids; shared_meshes describes actual requested grid payload, not peak memory",
                "LOD record/slot counts describe submitted batches before per-tree GPU culling, not actual visible vertex work",
                "fresh file reads may hit operating-system storage caches; this is not a network measurement",
            ],
        });
        let bytes = serde_json::to_vec_pretty(&report)?;
        std::fs::write(&self.request.report, bytes).map_err(|source| MeasurementError::Write {
            path: self.request.report.clone(),
            source,
        })?;
        Ok(())
    }
}
