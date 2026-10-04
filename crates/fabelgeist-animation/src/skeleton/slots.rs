//! Skin-buffer slots are a separate authority from ordered skeleton joints.
use fabelgeist_rig::RigJointOrdinal;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SkinJointOrdinal(usize);
impl From<usize> for SkinJointOrdinal {
    fn from(index: usize) -> Self {
        Self(index)
    }
}
impl From<RigJointOrdinal> for SkinJointOrdinal {
    fn from(index: RigJointOrdinal) -> Self {
        Self(usize::from(index))
    }
}
impl From<SkinJointOrdinal> for usize {
    fn from(index: SkinJointOrdinal) -> Self {
        index.0
    }
}
impl From<SkinJointOrdinal> for u32 {
    fn from(index: SkinJointOrdinal) -> Self {
        index.0 as u32
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SkinJointCount(usize);
impl From<usize> for SkinJointCount {
    fn from(count: usize) -> Self {
        Self(count)
    }
}
impl SkinJointOrdinal {
    pub fn required_count(self) -> SkinJointCount {
        SkinJointCount(self.0 + 1)
    }
}
impl From<SkinJointCount> for usize {
    fn from(count: SkinJointCount) -> Self {
        count.0
    }
}
