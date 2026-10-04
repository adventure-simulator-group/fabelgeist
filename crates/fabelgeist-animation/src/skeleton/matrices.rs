//! Matrix poses retain the coordinate authority of their collection.
use super::{SkinJointCount, SkinJointOrdinal};
use fabelgeist_math::matrix::Mat4;
use fabelgeist_rig::{RigJointCount, RigJointOrdinal};

#[derive(Clone, Debug, PartialEq)]
pub struct SkeletonModelMatrices(Vec<Mat4>);
impl From<Vec<Mat4>> for SkeletonModelMatrices {
    fn from(matrices: Vec<Mat4>) -> Self {
        Self(matrices)
    }
}
impl SkeletonModelMatrices {
    pub fn count(&self) -> RigJointCount {
        RigJointCount::from(self.0.len())
    }
    pub fn iter(&self) -> std::slice::Iter<'_, Mat4> {
        self.0.iter()
    }
}
impl std::ops::Index<RigJointOrdinal> for SkeletonModelMatrices {
    type Output = Mat4;
    fn index(&self, index: RigJointOrdinal) -> &Mat4 {
        &self.0[usize::from(index)]
    }
}
impl std::ops::IndexMut<RigJointOrdinal> for SkeletonModelMatrices {
    fn index_mut(&mut self, index: RigJointOrdinal) -> &mut Mat4 {
        &mut self.0[usize::from(index)]
    }
}

/// Matrices addressed by skin-buffer slots, independently of skeleton order.
///
/// ```compile_fail
/// use fabelgeist_animation::skeleton::{SkinJointMatrices, SkinJointCount};
/// use fabelgeist_rig::RigJointOrdinal;
/// let skin = SkinJointMatrices::from(SkinJointCount::default());
/// let _ = skin[RigJointOrdinal::from(0_usize)];
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct SkinJointMatrices(Vec<Mat4>);
impl SkinJointMatrices {
    pub fn count(&self) -> SkinJointCount {
        SkinJointCount::from(self.0.len())
    }
    pub fn iter(&self) -> std::slice::Iter<'_, Mat4> {
        self.0.iter()
    }
}
impl From<SkinJointCount> for SkinJointMatrices {
    fn from(count: SkinJointCount) -> Self {
        Self(vec![Mat4::identity(); usize::from(count)])
    }
}
impl std::ops::Index<SkinJointOrdinal> for SkinJointMatrices {
    type Output = Mat4;
    fn index(&self, index: SkinJointOrdinal) -> &Mat4 {
        &self.0[usize::from(index)]
    }
}
impl std::ops::IndexMut<SkinJointOrdinal> for SkinJointMatrices {
    fn index_mut(&mut self, index: SkinJointOrdinal) -> &mut Mat4 {
        &mut self.0[usize::from(index)]
    }
}
