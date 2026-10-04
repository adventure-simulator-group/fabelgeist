//! Ordered skeleton storage retains joint coordinates through its consumers.
use super::Joint;
use fabelgeist_rig::{RigJointCount, RigJointOrdinal, RigJointOrdinals};
use serde::{Deserialize, Serialize};

/// Ordered joints require skeleton coordinates, independently of skin slots.
///
/// ```compile_fail
/// use fabelgeist_animation::skeleton::{SkeletonJoints, SkinJointOrdinal};
/// let joints = SkeletonJoints::default();
/// let _ = joints[SkinJointOrdinal::from(0_usize)];
/// ```
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SkeletonJoints(Vec<Joint>);
impl From<Vec<Joint>> for SkeletonJoints {
    fn from(joints: Vec<Joint>) -> Self {
        Self(joints)
    }
}
impl SkeletonJoints {
    pub fn count(&self) -> RigJointCount {
        RigJointCount::from(self.0.len())
    }
    pub fn ordinals(&self) -> RigJointOrdinals {
        self.count().ordinals()
    }
    pub fn iter(&self) -> std::slice::Iter<'_, Joint> {
        self.0.iter()
    }
    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, Joint> {
        self.0.iter_mut()
    }
    pub fn push(&mut self, joint: Joint) {
        self.0.push(joint);
    }
}
impl std::ops::Index<RigJointOrdinal> for SkeletonJoints {
    type Output = Joint;
    fn index(&self, index: RigJointOrdinal) -> &Joint {
        &self.0[usize::from(index)]
    }
}
impl std::ops::IndexMut<RigJointOrdinal> for SkeletonJoints {
    fn index_mut(&mut self, index: RigJointOrdinal) -> &mut Joint {
        &mut self.0[usize::from(index)]
    }
}
impl<'a> IntoIterator for &'a SkeletonJoints {
    type Item = &'a Joint;
    type IntoIter = std::slice::Iter<'a, Joint>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}
