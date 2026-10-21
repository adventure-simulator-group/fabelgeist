//! The two production reduced crown-wood policies.
use super::{TreeBranchSegment, WoodyBranchMeshQuality, procedural_woody_crown_mesh};
use bevy::prelude::Mesh;

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum AggregateTreeLod {
    Near,
    Middle,
}

impl AggregateTreeLod {
    pub(in crate::presentation) const ALL: [Self; 2] = [Self::Near, Self::Middle];

    pub(in crate::presentation) fn from_render_lod(lod: u8) -> Option<Self> {
        match lod {
            1 => Some(Self::Near),
            2 => Some(Self::Middle),
            _ => None,
        }
    }

    pub(in crate::presentation) fn index(self) -> usize {
        match self {
            Self::Near => 0,
            Self::Middle => 1,
        }
    }

    pub(in crate::presentation) fn mesh(self, branches: &[TreeBranchSegment]) -> Mesh {
        let (depth, quality) = match self {
            Self::Near => (2, WoodyBranchMeshQuality::AggregateLod1),
            Self::Middle => (1, WoodyBranchMeshQuality::AggregateLod2),
        };
        procedural_woody_crown_mesh(branches, depth, quality)
    }
}
