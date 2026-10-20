//! Missing first-transition work is a failure, not a later-ready submission.

use super::*;
use bevy::render::view::ExtractedView;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum Readiness {
    Settled,
    FirstTransitionFrame,
    FrozenDemandFault,
}

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum ReadinessStage {
    ExtractedCamera,
    SourceBuffers,
    MeshMaterialPipeline,
    DrawCommands,
    MissedFirstFrame,
}

impl Readiness {
    pub(super) fn for_view(
        subject: Subject,
        control: BatchControl,
        view: &CaptureViewSpec,
    ) -> Self {
        if view.warmup {
            return Self::Settled;
        }
        if control != BatchControl::Ordinary {
            return Self::FrozenDemandFault;
        }
        match subject {
            Subject::Generated {
                selection: LevelSelection::Adaptive,
                ..
            } => Self::FirstTransitionFrame,
            Subject::Generated { .. } | Subject::Specimens { .. } => Self::Settled,
        }
    }
}

impl Pose {
    pub(super) fn camera_intact(
        &self,
        view: &ExtractedView,
        frame: types::RenderFrameId,
    ) -> Result<bool> {
        if let Some(selection) = &self.selection
            && let Err(error) = selection.camera.check_extracted(view)
        {
            self.fail(format!("adaptive camera join: {error:#}"));
            self.record_frame_failure(frame,ReadinessStage::ExtractedCamera,&[],serde_json::json!({"world_from_view":view.world_from_view.to_matrix().to_cols_array(),"clip_from_view":view.clip_from_view.to_cols_array(),"clip_from_world":view.clip_from_world.map(|matrix|matrix.to_cols_array()),"viewport":view.viewport.to_array()}))?;
            return Ok(false);
        }
        Ok(true)
    }

    pub(super) fn unready(
        &self,
        frame: types::RenderFrameId,
        stage: ReadinessStage,
        commands: &[commands::Receipt],
    ) -> Result<()> {
        if self.readiness == Readiness::FirstTransitionFrame {
            self.fail(format!("first transition frame is incomplete: {stage:?}"));
            self.record_frame_failure(frame, stage, commands, serde_json::Value::Null)?;
        }
        Ok(())
    }

    pub(super) fn record_frame_failure(
        &self,
        frame: types::RenderFrameId,
        stage: ReadinessStage,
        commands: &[commands::Receipt],
        detail: serde_json::Value,
    ) -> Result<()> {
        // Callers latch the owning pose before this fallible diagnostic write.
        fs::write(
            self.output.join(format!(
                "first-transition-failure-view-{:03}.json",
                self.view
            )),
            serde_json::to_vec_pretty(
                &serde_json::json!({"view":self.view,"first_extracted_frame":self.first_frame,"observed_frame":frame,"requirement":self.readiness,"stage":stage,"selection":self.selection.as_ref().map(|selection|selection.report()),"actual_color_dispatches":commands.iter().map(commands::Receipt::report).collect::<Vec<_>>(),"detail":detail,"passed":false}),
            )?,
        )?;
        Ok(())
    }
}
