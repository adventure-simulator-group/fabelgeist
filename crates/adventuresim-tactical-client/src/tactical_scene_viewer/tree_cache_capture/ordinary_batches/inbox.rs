//! CPU-only delivery channel bound to its original observation identity.
use super::*;
use snapshots::SnapshotBytes;
use types::RenderFrameId;

pub(super) type ReadbackOutcome = std::result::Result<Vec<SnapshotBytes>, String>;

#[derive(Clone)]
pub(super) struct Inbox {
    view: usize,
    frame: RenderFrameId,
    result: Arc<Mutex<Option<ReadbackOutcome>>>,
}

impl Inbox {
    pub(super) fn new(view: usize, frame: RenderFrameId) -> Self {
        Self {
            view,
            frame,
            result: Arc::new(Mutex::new(None)),
        }
    }

    pub(super) fn verify_owner(&self, view: usize, frame: RenderFrameId) -> Result<()> {
        ensure!(
            self.view == view && self.frame == frame,
            "stale GPU readback destination"
        );
        Ok(())
    }

    pub(super) fn deliver(
        &self,
        view: usize,
        frame: RenderFrameId,
        outcome: ReadbackOutcome,
    ) -> Result<()> {
        self.verify_owner(view, frame)?;
        let mut result = self
            .result
            .lock()
            .map_err(|_| anyhow::anyhow!("GPU readback inbox lock"))?;
        ensure!(result.is_none(), "duplicate GPU readback delivery");
        *result = Some(outcome);
        Ok(())
    }

    pub(super) fn take(
        &self,
        view: usize,
        frame: RenderFrameId,
    ) -> Result<Option<ReadbackOutcome>> {
        self.verify_owner(view, frame)?;
        Ok(self
            .result
            .lock()
            .map_err(|_| anyhow::anyhow!("GPU readback inbox lock"))?
            .take())
    }
}
