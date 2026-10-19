//! Fixed-plate evidence retained through the last actual screenshot callback.

use super::super::capture_state::SceneCaptureState;
use super::*;
use adventuresim_environment::render::benchmark::{ForestBenchmark, ValidationPhase};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Mutex,
};

#[derive(Resource, Clone)]
pub(in super::super) struct Evidence(Arc<Mutex<Data>>);

struct Data {
    metadata: serde_json::Value,
    poses: BTreeMap<usize, serde_json::Value>,
    draws: DrawEvidence,
    camera_drift_views: BTreeSet<usize>,
}

struct DrawStatus {
    clean: bool,
    report: serde_json::Value,
}

enum DrawEvidence {
    Source(source_draws::SourceDraws),
    Candidate(Vec<ForestBenchmark>),
}

impl Evidence {
    pub(super) fn new(
        metadata: serde_json::Value,
        source: Option<source_draws::SourceDraws>,
    ) -> Self {
        Self(Arc::new(Mutex::new(Data {
            metadata,
            draws: match source {
                Some(source) => DrawEvidence::Source(source),
                None => DrawEvidence::Candidate(Vec::new()),
            },
            poses: BTreeMap::new(),
            camera_drift_views: BTreeSet::new(),
        })))
    }

    pub(super) fn track(&self, probe: ForestBenchmark) -> Result<()> {
        let mut data = self
            .0
            .lock()
            .map_err(|_| anyhow::anyhow!("capture evidence lock"))?;
        let DrawEvidence::Candidate(probes) = &mut data.draws else {
            anyhow::bail!("source plate cannot own a bank probe");
        };
        probes.push(probe);
        Ok(())
    }

    pub(super) fn camera_drift(&self, view: usize) -> Result<()> {
        self.0
            .lock()
            .map_err(|_| anyhow::anyhow!("capture evidence lock"))?
            .camera_drift_views
            .insert(view);
        Ok(())
    }

    pub(super) fn camera_intact(&self) -> bool {
        self.0
            .lock()
            .is_ok_and(|data| data.camera_drift_views.is_empty())
    }

    pub(super) fn record(&self, view: usize, proof: serde_json::Value) -> Result<()> {
        self.0
            .lock()
            .map_err(|_| anyhow::anyhow!("capture evidence lock"))?
            .poses
            .entry(view)
            .or_insert(proof);
        Ok(())
    }

    pub(in super::super) fn finish(&self, state: &SceneCaptureState) -> Result<()> {
        let data = self
            .0
            .lock()
            .map_err(|_| anyhow::anyhow!("capture evidence lock"))?;
        let poses_complete = data.poses.len() == state.views.len()
            && (0..state.views.len()).all(|view| data.poses.contains_key(&view));
        let draw_status = match &data.draws {
            DrawEvidence::Source(source) => {
                let clean = source.all_valid(state.views.len());
                DrawStatus {
                    clean,
                    report: serde_json::json!({"role": Role::Source, "source_prefixes_clean": clean}),
                }
            }
            DrawEvidence::Candidate(probes) => {
                let clean = probes.len() == state.views.len()
                    && probes.iter().all(|probe| {
                        !probe.status().failed
                            && probe
                                .validation()
                                .evidence()
                                .is_some_and(|status| status.phase == ValidationPhase::Complete)
                            && probe.final_completed_frame().is_some()
                            && probe.native_residency().is_some()
                    });
                DrawStatus {
                    clean,
                    report: serde_json::json!({"role": Role::Candidate, "bank_probes": probes.len(), "bank_probes_clean": clean}),
                }
            }
        };
        let passed = poses_complete && draw_status.clean && data.camera_drift_views.is_empty();
        let report = serde_json::json!({"metadata": data.metadata, "poses": data.poses,
            "post_screenshots": {"poses_complete": poses_complete, "draws": draw_status.report, "camera_drift_views": data.camera_drift_views},
            "capture_files": state.captures.iter().map(|capture| &capture.screenshot).collect::<Vec<_>>(),
            "passed": passed,
            "limits": "queue-prefix prerequisites and held images, not exact capture-frame timestamps, motion, geographic arrival, physical VRAM or quality approval"});
        fs::write(
            state.output.join("bank-review.json"),
            serde_json::to_vec_pretty(&report)?,
        )?;
        ensure!(passed, "incomplete or failed fixed-plate draw evidence");
        Ok(())
    }
}
