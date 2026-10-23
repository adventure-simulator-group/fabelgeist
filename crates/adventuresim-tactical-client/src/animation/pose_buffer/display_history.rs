//! Retained authored-presentation history, independent of visibility and physics.

use super::PoseBufferRig;

/// Physics can pin buffered poses without establishing authored display history.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) enum PosePresentationMode {
    #[default]
    Inactive,
    Displaying,
}

impl PoseBufferRig {
    pub(crate) fn has_presented_pose(&self) -> bool {
        self.presentation == PosePresentationMode::Displaying
    }
}
