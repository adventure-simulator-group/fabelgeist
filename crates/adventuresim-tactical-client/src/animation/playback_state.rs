//! Carried playback choices, separate from local selection predicates.

use bevy::gltf::Gltf;
use bevy::prelude::*;

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

#[derive(Debug, Clone, Default)]
pub(super) enum BaseRigLoad {
    #[default]
    Unrequested,
    Loading {
        source: Handle<Gltf>,
    },
    Ready {
        source: Handle<Gltf>,
        scene: Handle<WorldAsset>,
    },
    Failed {
        source: Handle<Gltf>,
    },
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

impl BaseRigLoad {
    pub(super) fn source(&self) -> Option<&Handle<Gltf>> {
        match self {
            Self::Unrequested => None,
            Self::Loading { source } | Self::Ready { source, .. } | Self::Failed { source } => {
                Some(source)
            }
        }
    }

    pub(super) fn scene(&self) -> Option<&Handle<WorldAsset>> {
        match self {
            Self::Ready { scene, .. } => Some(scene),
            _ => None,
        }
    }
}

impl OrdinaryLocomotion {
    pub(super) fn from_candidate(active: bool) -> Self {
        if active { Self::Active } else { Self::Inactive }
    }
}
