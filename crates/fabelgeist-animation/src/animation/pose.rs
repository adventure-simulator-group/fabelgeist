//! Local and accumulated model poses have separate coordinate authority.
use super::JointTransform;
use fabelgeist_rig::{RigJointCount, RigJointOrdinal};
use serde::{Deserialize, Serialize};

/// Local transforms must be accumulated before model-space consumers use them.
///
/// ```compile_fail
/// use fabelgeist_animation::{ModelPose, Skeleton, model_pose};
/// let skeleton = Skeleton::new(vec![]);
/// let _ = model_pose(&skeleton, &ModelPose::default());
/// ```
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct LocalPose(Vec<JointTransform>);
impl From<Vec<JointTransform>> for LocalPose {
    fn from(joints: Vec<JointTransform>) -> Self {
        Self(joints)
    }
}
impl FromIterator<JointTransform> for LocalPose {
    fn from_iter<T: IntoIterator<Item = JointTransform>>(iter: T) -> Self {
        Self(iter.into_iter().collect())
    }
}
impl LocalPose {
    pub fn count(&self) -> RigJointCount {
        RigJointCount::from(self.0.len())
    }
    pub fn identity(count: RigJointCount) -> Self {
        Self(vec![JointTransform::identity(); usize::from(count)])
    }
    pub fn iter(&self) -> std::slice::Iter<'_, JointTransform> {
        self.0.iter()
    }
    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, JointTransform> {
        self.0.iter_mut()
    }
    pub fn push(&mut self, joint: JointTransform) {
        self.0.push(joint);
    }
}
impl std::ops::Index<RigJointOrdinal> for LocalPose {
    type Output = JointTransform;
    fn index(&self, joint: RigJointOrdinal) -> &JointTransform {
        &self.0[usize::from(joint)]
    }
}
impl std::ops::IndexMut<RigJointOrdinal> for LocalPose {
    fn index_mut(&mut self, joint: RigJointOrdinal) -> &mut JointTransform {
        &mut self.0[usize::from(joint)]
    }
}
impl<'a> IntoIterator for &'a LocalPose {
    type Item = &'a JointTransform;
    type IntoIter = std::slice::Iter<'a, JointTransform>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}
impl IntoIterator for LocalPose {
    type Item = JointTransform;
    type IntoIter = std::vec::IntoIter<JointTransform>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ModelPose(Vec<JointTransform>);
impl From<Vec<JointTransform>> for ModelPose {
    fn from(joints: Vec<JointTransform>) -> Self {
        Self(joints)
    }
}
impl FromIterator<JointTransform> for ModelPose {
    fn from_iter<T: IntoIterator<Item = JointTransform>>(iter: T) -> Self {
        Self(iter.into_iter().collect())
    }
}
impl ModelPose {
    pub fn count(&self) -> RigJointCount {
        RigJointCount::from(self.0.len())
    }
    pub fn identity(count: RigJointCount) -> Self {
        Self(vec![JointTransform::identity(); usize::from(count)])
    }
    pub fn iter(&self) -> std::slice::Iter<'_, JointTransform> {
        self.0.iter()
    }
    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, JointTransform> {
        self.0.iter_mut()
    }
    pub fn push(&mut self, joint: JointTransform) {
        self.0.push(joint);
    }
}
impl std::ops::Index<RigJointOrdinal> for ModelPose {
    type Output = JointTransform;
    fn index(&self, joint: RigJointOrdinal) -> &JointTransform {
        &self.0[usize::from(joint)]
    }
}
impl std::ops::IndexMut<RigJointOrdinal> for ModelPose {
    fn index_mut(&mut self, joint: RigJointOrdinal) -> &mut JointTransform {
        &mut self.0[usize::from(joint)]
    }
}
impl<'a> IntoIterator for &'a ModelPose {
    type Item = &'a JointTransform;
    type IntoIter = std::slice::Iter<'a, JointTransform>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}
impl IntoIterator for ModelPose {
    type Item = JointTransform;
    type IntoIter = std::vec::IntoIter<JointTransform>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

/// The bind pose as authored, in local joint coordinates.
pub fn rest_pose(skeleton: &crate::skeleton::Skeleton) -> LocalPose {
    skeleton
        .joints
        .iter()
        .map(|joint| JointTransform::from_transform(&joint.local_transform))
        .collect()
}

/// Parents appearing later are treated as roots, preserving the authored policy.
pub fn model_pose(skeleton: &crate::skeleton::Skeleton, locals: &LocalPose) -> ModelPose {
    let mut model = ModelPose::default();
    for joint in locals.count().ordinals() {
        let local = locals[joint];
        let transform = match skeleton.joints[joint].parent_index {
            Some(parent) if parent < joint => model[parent].compose(local),
            _ => local,
        };
        model.push(transform);
    }
    model
}
