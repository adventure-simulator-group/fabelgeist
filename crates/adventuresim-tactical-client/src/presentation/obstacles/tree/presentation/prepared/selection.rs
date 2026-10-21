//! One checked fixed representation for process-isolated production parity.
use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, clap::ValueEnum, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum TreeCacheRepresentation {
    Cambered,
    Flat,
    Lod1,
    Lod2,
    Lod3,
    Lod4,
}

impl TreeCacheRepresentation {
    pub(crate) fn lod(self) -> u8 {
        match self {
            Self::Cambered | Self::Flat => 0,
            Self::Lod1 => 1,
            Self::Lod2 => 2,
            Self::Lod3 => 3,
            Self::Lod4 => 4,
        }
    }
    pub(crate) fn leaf(self) -> Option<TreeLeafRepresentation> {
        match self {
            Self::Cambered => Some(TreeLeafRepresentation::TexturedMesh),
            Self::Flat => Some(TreeLeafRepresentation::AlphaCard),
            _ => None,
        }
    }
    pub(crate) fn stream_mask(self) -> u8 {
        let lod = self.lod();
        (1 << (lod + 1)) | u8::from(lod < 4)
    }
    pub(crate) fn leaf_mask(self) -> u8 {
        match self {
            Self::Cambered => 1,
            Self::Flat => 2,
            _ => 0,
        }
    }
    pub(crate) fn matches(self, actual: &TreeLodRenderOverride) -> bool {
        actual.lod == Some(self.lod())
            && actual.leaf == self.leaf()
            && actual.projected_scale.is_none()
    }
    pub(crate) fn owns(self, role: InstalledTreePartRole) -> bool {
        match role {
            InstalledTreePartRole::DetailedTrunk | InstalledTreePartRole::MiddleTrunk => {
                self.lod() < 4
            }
            InstalledTreePartRole::Wood { .. } | InstalledTreePartRole::Buds { .. } => {
                self.lod() == 0
            }
            InstalledTreePartRole::CamberedLeaves { .. } => self == Self::Cambered,
            InstalledTreePartRole::FlatLeaves { .. } => self == Self::Flat,
            InstalledTreePartRole::AggregateWood { lod } => lod.index() as u8 + 1 == self.lod(),
            InstalledTreePartRole::CanopyCard { lod } => lod == self.lod(),
        }
    }
    pub(crate) fn visibility(self, role: InstalledTreePartRole) -> Visibility {
        let hidden = match role {
            InstalledTreePartRole::DetailedTrunk => self.lod() != 0,
            InstalledTreePartRole::MiddleTrunk => self.lod() == 0,
            _ => false,
        };
        if hidden {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        }
    }
}

impl InstalledTreePartRole {
    pub(crate) fn render_lod(self) -> Option<u8> {
        match self {
            Self::DetailedTrunk | Self::MiddleTrunk => None,
            Self::CanopyCard { lod } => Some(lod),
            Self::AggregateWood { lod } => Some(lod.index() as u8 + 1),
            _ => Some(0),
        }
    }
}
