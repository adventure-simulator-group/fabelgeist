//! Sticky held-source and successful-submission evidence through final callback.

use super::super::capture_state::SceneCaptureState;
use super::*;
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Mutex,
};

#[derive(Resource, Clone)]
pub(in super::super) struct Evidence {
    data: Arc<Mutex<Data>>,
    role: Role,
    source: PlantId,
}

struct Data {
    metadata: serde_json::Value,
    poses: BTreeMap<usize, serde_json::Value>,
    draws: ordinary_batches::OrdinaryBatches,
    camera_drift: BTreeSet<usize>,
    input_faults: BTreeMap<usize, String>,
}

impl Evidence {
    pub(super) fn new(
        metadata: serde_json::Value,
        role: Role,
        source: PlantId,
        draws: ordinary_batches::OrdinaryBatches,
    ) -> Self {
        Self {
            role,
            source,
            data: Arc::new(Mutex::new(Data {
                metadata,
                poses: BTreeMap::new(),
                draws,
                camera_drift: BTreeSet::new(),
                input_faults: BTreeMap::new(),
            })),
        }
    }

    pub(in super::super) fn input_fault(&self, view: usize, reason: String) -> Result<()> {
        let mut data = self
            .data
            .lock()
            .map_err(|_| anyhow::anyhow!("grown evidence lock"))?;
        data.input_faults.entry(view).or_insert(reason);
        data.draws.invalidate(view);
        Ok(())
    }

    pub(super) fn camera_drift(&self, view: usize) -> Result<()> {
        let mut data = self
            .data
            .lock()
            .map_err(|_| anyhow::anyhow!("grown evidence lock"))?;
        data.camera_drift.insert(view);
        data.draws.invalidate(view);
        Ok(())
    }

    pub(super) fn camera_intact(&self) -> bool {
        self.data
            .lock()
            .is_ok_and(|data| data.camera_drift.is_empty() && data.input_faults.is_empty())
    }

    pub(super) fn record(&self, view: usize, proof: serde_json::Value) -> Result<()> {
        self.data
            .lock()
            .map_err(|_| anyhow::anyhow!("grown evidence lock"))?
            .poses
            .entry(view)
            .or_insert(proof);
        Ok(())
    }

    pub(in super::super) fn capability(
        &self,
        presented_roots: usize,
    ) -> super::super::manifest::TreePresentationCapability {
        let clean = self.data.lock().is_ok_and(|data| {
            data.camera_drift.is_empty()
                && data.input_faults.is_empty()
                && !data.poses.is_empty()
                && data.draws.inventory_verified()
                && data.draws.all_valid(data.poses.len())
        });
        super::super::manifest::TreePresentationCapability::FixedGrownFull {
            role: self.role,
            source_id: self.source.word(),
            owned_inventory_verified: clean && presented_roots == 0,
        }
    }

    pub(in super::super) fn finish(&self, state: &SceneCaptureState) -> Result<()> {
        let data = self
            .data
            .lock()
            .map_err(|_| anyhow::anyhow!("grown evidence lock"))?;
        let poses_complete = data.poses.len() == state.views.len()
            && (0..state.views.len()).all(|view| data.poses.contains_key(&view));
        let draws_clean = data.draws.all_valid(state.views.len());
        let passed = poses_complete
            && draws_clean
            && data.camera_drift.is_empty()
            && data.input_faults.is_empty();
        let report = serde_json::json!({"metadata":data.metadata,"poses":data.poses,"post_screenshots":{"poses_complete":poses_complete,"draws":draws_clean,"camera_drift_views":data.camera_drift,"input_faults":data.input_faults},"capture_files":state.captures.iter().map(|capture|&capture.screenshot).collect::<Vec<_>>(),"passed":passed,"limits":"held source/asset/camera bindings and actual main-pass indirect commands plus compacted GPU instance membership; no vertex/source storage-byte readback, shadow/prepass counts, exact screenshot timestamps, memory, motion, performance or arrival"});
        fs::write(
            state.output.join("grown-full-review.json"),
            serde_json::to_vec_pretty(&report)?,
        )?;
        ensure!(passed, "incomplete/failed grown Full capture evidence");
        Ok(())
    }
}
