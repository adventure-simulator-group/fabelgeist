//! Original-frame readback ownership and fail-closed screenshot readiness.
use super::*;
use bindings::RenderMember;
use inbox::Inbox;
use types::RenderFrameId;

pub(super) struct Submission {
    pub frame: RenderFrameId,
    pub members: Vec<RenderMember>,
    pub commands: Vec<commands::Receipt>,
    pub inbox: Inbox,
    state: Readback,
}

enum Readback {
    Awaiting,
    Validated(serde_json::Value),
    Failed,
}

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
enum FailureStage {
    Ownership,
    Readback,
    Membership,
}

impl Submission {
    pub(super) fn new(
        view: usize,
        frame: RenderFrameId,
        members: Vec<RenderMember>,
        commands: Vec<commands::Receipt>,
    ) -> Self {
        Self {
            frame,
            members,
            commands,
            inbox: Inbox::new(view, frame),
            state: Readback::Awaiting,
        }
    }

    pub(super) fn check_commands(
        &self,
        current: &[commands::Receipt],
        frame: RenderFrameId,
        camera: RetainedViewEntity,
    ) -> Result<()> {
        ensure!(
            current.len() == self.commands.len(),
            "held successful draw count drift"
        );
        let mut unmatched: Vec<_> = self.commands.iter().collect();
        for command in current {
            ensure!(
                command.frame == frame && command.camera == camera,
                "stale held-frame draw"
            );
            let index = unmatched
                .iter()
                .position(|held| held.same_draw(command))
                .context("held successful draw command drift")?;
            unmatched.swap_remove(index);
        }
        ensure!(unmatched.is_empty(), "held successful draw missing");
        Ok(())
    }

    fn header(&self, pose: &Pose) -> serde_json::Value {
        serde_json::json!({"view":pose.view, "frame":self.frame, "role":pose.role,
            "camera_entity_bits":pose.camera_bits,
            "expected_membership":pose.expected.iter().map(TaggedMember::report).collect::<Vec<_>>(),
            "render_members":self.members.iter().map(RenderMember::report).collect::<Vec<_>>(),
            "actual_color_dispatches":self.commands.iter().map(commands::Receipt::report).collect::<Vec<_>>()})
    }
}

impl Pose {
    pub(super) fn poll(&self) -> Result<Option<serde_json::Value>> {
        if self.invalid.load(Ordering::Acquire) {
            let failure = self
                .failure
                .lock()
                .map_err(|_| anyhow::anyhow!("GPU failure lock"))?
                .clone()
                .unwrap_or_else(|| "invalid ordinary GPU observation".to_owned());
            self.write_failure(FailureStage::Ownership, &failure)?;
            anyhow::bail!(failure);
        }
        let mut submission = self
            .submission
            .lock()
            .map_err(|_| anyhow::anyhow!("GPU submission lock"))?;
        let Some(submission) = submission.as_mut() else {
            return Ok(None);
        };
        let snapshots = match &submission.state {
            Readback::Awaiting => match submission.inbox.take(self.view, submission.frame)? {
                None => return Ok(None),
                Some(Ok(snapshots)) => snapshots,
                Some(Err(reason)) => {
                    submission.state = Readback::Failed;
                    self.fail(format!("GPU readback: {reason}"));
                    self.write_failure(FailureStage::Readback, &reason)?;
                    anyhow::bail!(reason);
                }
            },
            Readback::Validated(proof) => return Ok(Some(proof.clone())),
            Readback::Failed => anyhow::bail!("invalid GPU readback lifecycle"),
        };
        // The delivery is consumed. Every error below remains terminal.
        submission.state = Readback::Failed;
        let directory = self
            .output
            .join("gpu-batches")
            .join(format!("view-{:03}", self.view));
        fs::create_dir_all(&directory)?;
        fs::write(
            directory.join("receipts.json"),
            serde_json::to_vec_pretty(&submission.header(self))?,
        )?;
        let mut manifest = Vec::new();
        for (index, snapshot) in snapshots.iter().enumerate() {
            let file = format!("storage-{index:02}.bin");
            fs::write(directory.join(&file), &snapshot.data)?;
            manifest.push(
                serde_json::json!({"key":snapshot.key, "file":file, "bytes":snapshot.data.len()}),
            );
        }
        fs::write(
            directory.join("storage.json"),
            serde_json::to_vec_pretty(&manifest)?,
        )?;
        let validated = decode::PhaseData::decode(&snapshots)
            .and_then(|phases| validate::verify(self, submission, &phases));
        let proof = match validated {
            Ok(proof) => proof,
            Err(error) => {
                submission.state = Readback::Failed;
                self.fail(format!("GPU membership: {error:#}"));
                self.write_failure(FailureStage::Membership, &format!("{error:#}"))?;
                return Err(error);
            }
        };
        fs::write(
            directory.join("proof.json"),
            serde_json::to_vec_pretty(&proof)?,
        )?;
        submission.state = Readback::Validated(proof.clone());
        self.complete.store(true, Ordering::Release);
        Ok(Some(proof))
    }

    fn write_failure(&self, stage: FailureStage, reason: &str) -> Result<()> {
        fs::write(
            self.output
                .join(format!("gpu-batch-failure-view-{:03}.json", self.view)),
            serde_json::to_vec_pretty(&serde_json::json!({"view":self.view, "stage":stage,
                "reason":reason, "passed":false, "scope":"ordinary main-color GPU proof"}))?,
        )?;
        Ok(())
    }
}
