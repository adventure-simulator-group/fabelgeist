//! Native main-color phase and storage-buffer identities for one observation.
use bevy::core_pipeline::core_3d::{AlphaMask3d, Opaque3d};
use serde::Serialize;
use std::any::TypeId;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum ColorPhase {
    Opaque,
    AlphaMask,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum SnapshotKind {
    // Native GPU array bound, not a source Entity or input-buffer ordinal.
    MeshOutput { logical_instances: usize },
    IndexedArguments,
    NonIndexedArguments,
    IndexedBatchSets,
    NonIndexedBatchSets,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
pub(super) struct SnapshotKey {
    pub phase: ColorPhase,
    pub kind: SnapshotKind,
}

impl ColorPhase {
    pub(super) fn type_id(self) -> TypeId {
        match self {
            Self::Opaque => TypeId::of::<Opaque3d>(),
            Self::AlphaMask => TypeId::of::<AlphaMask3d>(),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
pub(super) struct RenderFrameId(u64);

impl RenderFrameId {
    pub(super) fn next(self) -> anyhow::Result<Self> {
        self.0
            .checked_add(1)
            .map(Self)
            .ok_or_else(|| anyhow::anyhow!("GPU observation frame overflow"))
    }
}
