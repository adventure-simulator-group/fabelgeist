//! Each strand level retains its own simulation-update obligation.

use super::HairRenderer;
use crate::lod::Representation;

pub(super) struct DeformationViews {
    pub(super) main: Representation,
    pub(super) shadow: Option<Representation>,
    pub(super) blend: Option<Representation>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum DeformationState {
    Pending,
    Current,
}

impl HairRenderer {
    pub(super) fn record_deformations(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        views: DeformationViews,
    ) {
        for view in [Some(views.main), views.shadow, views.blend]
            .into_iter()
            .flatten()
        {
            let Representation::Strands { level, .. } = view else {
                continue;
            };
            let pending = self
                .dynamics
                .as_ref()
                .and_then(|dynamics| dynamics.levels.get(level.index()))
                .is_some_and(|level| level.state == DeformationState::Pending);
            if !pending {
                continue;
            }
            self.deform(encoder, level);
            if let Some(level) = self
                .dynamics
                .as_mut()
                .and_then(|dynamics| dynamics.levels.get_mut(level.index()))
            {
                level.state = DeformationState::Current;
            }
        }
    }
}
