//! Controller contact classification, separate from richer posture state.

use super::SkeletonState;

/// Contact reported by the controller, distinct from a downed body's surface support.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GroundContact {
    Grounded,
    Airborne,
}

impl From<bool> for GroundContact {
    fn from(grounded: bool) -> Self {
        if grounded {
            Self::Grounded
        } else {
            Self::Airborne
        }
    }
}

impl From<&SkeletonState> for GroundContact {
    fn from(state: &SkeletonState) -> Self {
        Self::from(state.is_grounded())
    }
}
