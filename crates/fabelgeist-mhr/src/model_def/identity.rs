//! Exact model/rig/set identities and distinct matrix coordinates.
use fabelgeist_rig::RigJointOrdinal;
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ModelParameterName(String);
impl From<&str> for ModelParameterName {
    fn from(name: &str) -> Self {
        Self(name.to_owned())
    }
}
impl std::fmt::Display for ModelParameterName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}
impl ModelParameterName {
    pub(super) fn blend_shape(index: BlendShapeOrdinal) -> Self {
        Self(format!("blend_{}", index.0))
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ParameterSetName(String);
impl From<&str> for ParameterSetName {
    fn from(name: &str) -> Self {
        Self(name.to_owned())
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModelParameterIndex(pub(super) usize);
impl From<ModelParameterIndex> for usize {
    fn from(index: ModelParameterIndex) -> Self {
        index.0
    }
}
/// Number of named columns in a model-definition transform or pose input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModelParameterCount(usize);
impl From<usize> for ModelParameterCount {
    fn from(count: usize) -> Self {
        Self(count)
    }
}
impl From<ModelParameterCount> for usize {
    fn from(count: ModelParameterCount) -> Self {
        count.0
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct JointParameterRow(pub(super) usize);
impl From<usize> for JointParameterRow {
    fn from(row: usize) -> Self {
        Self(row)
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlendShapeParameterCount(pub(super) usize);
impl From<usize> for BlendShapeParameterCount {
    fn from(count: usize) -> Self {
        Self(count)
    }
}
#[derive(Clone, Copy)]
pub(super) struct BlendShapeOrdinal(pub(super) usize);
/// The seven Momentum channels, in serialized rig order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JointParameterChannel {
    TranslationX,
    TranslationY,
    TranslationZ,
    RotationX,
    RotationY,
    RotationZ,
    Scale,
}
impl JointParameterChannel {
    pub const TRANSLATIONS: [Self; 3] =
        [Self::TranslationX, Self::TranslationY, Self::TranslationZ];
    pub const ROTATIONS_AND_SCALE: [Self; 4] = [
        Self::RotationX,
        Self::RotationY,
        Self::RotationZ,
        Self::Scale,
    ];
    pub fn row(self, joint: RigJointOrdinal) -> JointParameterRow {
        let channel = match self {
            Self::TranslationX => 0,
            Self::TranslationY => 1,
            Self::TranslationZ => 2,
            Self::RotationX => 3,
            Self::RotationY => 4,
            Self::RotationZ => 5,
            Self::Scale => 6,
        };
        JointParameterRow(usize::from(joint) * crate::character::PARAMETERS_PER_JOINT + channel)
    }
}
