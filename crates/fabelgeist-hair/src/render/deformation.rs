//! Each strand level retains its own simulation-update obligation.

use super::HairRenderer;
use crate::lod::{Representation, StrandLevelIndex};

pub(super) struct DeformationViews {
    pub(super) main: Representation,
    pub(super) shadow: Option<Representation>,
    pub(super) blend: Option<Representation>,
}

/// Levels whose deformation commands were recorded into the frame encoder.
pub(super) struct RecordedDeformations {
    levels: [Option<StrandLevelIndex>; 3],
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum DeformationState {
    Pending,
    Current,
}

impl HairRenderer {
    pub(super) fn record_deformations(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        views: DeformationViews,
    ) -> RecordedDeformations {
        let mut recorded = RecordedDeformations { levels: [None; 3] };
        for (slot, view) in [Some(views.main), views.shadow, views.blend]
            .into_iter()
            .enumerate()
        {
            let Some(Representation::Strands { level, .. }) = view else {
                continue;
            };
            if recorded.levels.contains(&Some(level)) {
                continue;
            }
            let pending = self
                .dynamics
                .as_ref()
                .and_then(|dynamics| dynamics.levels.get(level.index()))
                .is_some_and(|level| level.state == DeformationState::Pending);
            if !pending {
                continue;
            }
            self.deform(encoder, level);
            recorded.levels[slot] = Some(level);
        }
        recorded
    }

    /// Publish only after the frame's encoder has been submitted normally.
    pub(super) fn publish_deformations(&mut self, recorded: RecordedDeformations) {
        for index in recorded.levels.into_iter().flatten() {
            if let Some(level) = self
                .dynamics
                .as_mut()
                .and_then(|dynamics| dynamics.levels.get_mut(index.index()))
            {
                level.state = DeformationState::Current;
            }
        }
    }
}
