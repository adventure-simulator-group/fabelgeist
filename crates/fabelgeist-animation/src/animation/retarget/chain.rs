//! Ordered joint chains and their continuous positions have separate identities.
use crate::animation::ModelPose;
use fabelgeist_math::vector::Vec4;
use fabelgeist_rig::RigJointOrdinal;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RigJointChain(Vec<RigJointOrdinal>);
impl From<Vec<RigJointOrdinal>> for RigJointChain {
    fn from(joints: Vec<RigJointOrdinal>) -> Self {
        Self(joints)
    }
}
impl FromIterator<RigJointOrdinal> for RigJointChain {
    fn from_iter<T: IntoIterator<Item = RigJointOrdinal>>(joints: T) -> Self {
        Self(joints.into_iter().collect())
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChainPresence {
    Empty,
    Populated,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChainJointCount(usize);
impl From<usize> for ChainJointCount {
    fn from(count: usize) -> Self {
        Self(count)
    }
}
impl std::fmt::Display for ChainJointCount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ChainPosition(f32);
impl ChainPosition {
    /// Admit an enumeration coordinate from the ordered chain's iterator.
    fn from_chain_link(link: usize, count: ChainJointCount) -> Self {
        let last = count.0.saturating_sub(1);
        Self(if last == 0 {
            1.0
        } else {
            link as f32 / last as f32
        })
    }
}
impl From<f32> for ChainPosition {
    fn from(position: f32) -> Self {
        Self(position)
    }
}

impl RigJointChain {
    pub fn count(&self) -> ChainJointCount {
        ChainJointCount(self.0.len())
    }
    pub fn presence(&self) -> ChainPresence {
        if self.0.is_empty() {
            ChainPresence::Empty
        } else {
            ChainPresence::Populated
        }
    }
    pub fn iter(&self) -> std::slice::Iter<'_, RigJointOrdinal> {
        self.0.iter()
    }
    pub fn positions(&self) -> ChainJointPositions {
        ChainJointPositions {
            joints: self.0.clone().into_iter().enumerate(),
            count: self.count(),
        }
    }
    pub fn rotation_in(&self, model: &ModelPose, position: ChainPosition) -> Vec4 {
        match self.0.len() {
            0 => Vec4::quat_identity(),
            1 => model[self.0[0]].rotation,
            length => {
                let scaled = position.0.clamp(0.0, 1.0) * (length - 1) as f32;
                let lower = (scaled.floor() as usize).min(length - 1);
                let upper = (lower + 1).min(length - 1);
                model[self.0[lower]]
                    .rotation
                    .slerp(model[self.0[upper]].rotation, scaled - lower as f32)
            }
        }
    }
    /// Empty chains retain the existing ordered-slot-zero fallback.
    pub fn nearest_joint(&self, position: ChainPosition) -> RigJointOrdinal {
        if self.0.is_empty() {
            return RigJointOrdinal::from(0_usize);
        }
        let scaled = position.0.clamp(0.0, 1.0) * (self.0.len() - 1) as f32;
        self.0[(scaled.round() as usize).min(self.0.len() - 1)]
    }
}
pub struct ChainJointPositions {
    joints: std::iter::Enumerate<std::vec::IntoIter<RigJointOrdinal>>,
    count: ChainJointCount,
}
impl Iterator for ChainJointPositions {
    type Item = (ChainPosition, RigJointOrdinal);
    fn next(&mut self) -> Option<(ChainPosition, RigJointOrdinal)> {
        let (link, joint) = self.joints.next()?;
        Some((ChainPosition::from_chain_link(link, self.count), joint))
    }
}
impl<'a> IntoIterator for &'a RigJointChain {
    type Item = &'a RigJointOrdinal;
    type IntoIter = std::slice::Iter<'a, RigJointOrdinal>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

/// Address in the prepared source-chain table, never a skeleton joint.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct SourceChainOrdinal(usize);
impl From<usize> for SourceChainOrdinal {
    fn from(index: usize) -> Self {
        Self(index)
    }
}
impl From<SourceChainOrdinal> for usize {
    fn from(index: SourceChainOrdinal) -> Self {
        index.0
    }
}
