//! Carried playback choices, separate from local selection predicates.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum PoseSource {
    Bind,
    Clips,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum TimelineDirection {
    Forward,
    Reversed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum OrdinaryLocomotion {
    Inactive,
    Active,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(super) enum BaseRigLoad {
    #[default]
    Pending,
    Ready,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::animation) enum PoseOwnership {
    Authored,
    Physics,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum PoseVisibility {
    Visible,
    Frozen,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum InertialState {
    Settled,
    Blending,
}

impl PoseSource {
    pub(super) fn for_empty_plan(empty: bool) -> Self {
        if empty { Self::Bind } else { Self::Clips }
    }
}

impl OrdinaryLocomotion {
    pub(super) fn from_candidate(active: bool) -> Self {
        if active { Self::Active } else { Self::Inactive }
    }
}
