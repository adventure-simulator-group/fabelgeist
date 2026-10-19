//! Fixed-plate evidence retained through the last actual screenshot callback.

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
    representation: TreeCacheRepresentation,
}

struct Data {
    metadata: serde_json::Value,
    poses: BTreeMap<usize, serde_json::Value>,
    draws: material_draws::SpecimenDraws,
    camera_drift_views: BTreeSet<usize>,
    input_faults: BTreeMap<usize, String>,
}

impl Evidence {
    pub(super) fn new(
        metadata: serde_json::Value,
        role: Role,
        representation: TreeCacheRepresentation,
        draws: material_draws::SpecimenDraws,
    ) -> Self {
        Self {
            role,
            representation,
            data: Arc::new(Mutex::new(Data {
                metadata,
                draws,
                poses: BTreeMap::new(),
                camera_drift_views: BTreeSet::new(),
                input_faults: BTreeMap::new(),
            })),
        }
    }

    pub(in super::super) fn input_fault(&self, view: usize, reason: String) -> Result<()> {
        let mut data = self
            .data
            .lock()
            .map_err(|_| anyhow::anyhow!("capture evidence lock"))?;
        data.input_faults.entry(view).or_insert(reason);
        data.draws.invalidate(view);
        Ok(())
    }

    pub(super) fn camera_drift(&self, view: usize) -> Result<()> {
        self.data
            .lock()
            .map_err(|_| anyhow::anyhow!("capture evidence lock"))?
            .camera_drift_views
            .insert(view);
        Ok(())
    }

    pub(super) fn camera_intact(&self) -> bool {
        self.data
            .lock()
            .is_ok_and(|data| data.camera_drift_views.is_empty() && data.input_faults.is_empty())
    }

    pub(super) fn record(&self, view: usize, proof: serde_json::Value) -> Result<()> {
        self.data
            .lock()
            .map_err(|_| anyhow::anyhow!("capture evidence lock"))?
            .poses
            .entry(view)
            .or_insert(proof);
        Ok(())
    }

    pub(in super::super) fn selected_cache_roots(&self) -> usize {
        1
    }

    pub(in super::super) fn capability(
        &self,
        presented_roots: usize,
    ) -> super::super::manifest::TreePresentationCapability {
        use super::super::manifest::TreePresentationCapability;
        let clean = self.data.lock().is_ok_and(|data| {
            data.camera_drift_views.is_empty()
                && data.input_faults.is_empty()
                && data.draws.inventory_verified()
        });
        TreePresentationCapability::FixedProductionRepresentation {
            role: self.role,
            representation: self.representation,
            owned_inventory_verified: clean && presented_roots == 1,
        }
    }

    pub(in super::super) fn finish(&self, state: &SceneCaptureState) -> Result<()> {
        let data = self
            .data
            .lock()
            .map_err(|_| anyhow::anyhow!("capture evidence lock"))?;
        let poses_complete = data.poses.len() == state.views.len()
            && (0..state.views.len()).all(|view| data.poses.contains_key(&view));
        let draws_clean = data.draws.all_valid(state.views.len());
        let passed = poses_complete
            && draws_clean
            && data.camera_drift_views.is_empty()
            && data.input_faults.is_empty();
        let report = serde_json::json!({"metadata": data.metadata, "poses": data.poses,
            "post_screenshots": {"poses_complete": poses_complete, "draws": draws_clean, "camera_drift_views": data.camera_drift_views, "input_faults": data.input_faults},
            "capture_files": state.captures.iter().map(|capture| &capture.screenshot).collect::<Vec<_>>(),
            "passed": passed,
            "limits": "queue-prefix prerequisites and held images, not exact capture-frame timestamps, motion, geographic arrival, physical VRAM or quality approval"});
        fs::write(
            state.output.join("cache-review.json"),
            serde_json::to_vec_pretty(&report)?,
        )?;
        ensure!(passed, "incomplete or failed fixed-plate draw evidence");
        Ok(())
    }
}
