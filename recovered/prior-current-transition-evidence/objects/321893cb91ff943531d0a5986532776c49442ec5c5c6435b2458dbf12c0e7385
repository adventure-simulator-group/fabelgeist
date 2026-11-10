//! Identities and dense adapters for Momentum's joint-channel transform.

use crate::character::PARAMETERS_PER_JOINT;
use fabelgeist_numpy_storage::NpyDimension;
use fabelgeist_rig::{RigJointCount, RigJointOrdinal, RigJointTable};
use std::ops::{Index, IndexMut};

pub use matrix::ParameterLayoutError;
pub(crate) use matrix::ParameterMatrix;

mod matrix;

/// Private parser row key, resolved against the same skeleton as its tables.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct JointParameterRow {
    joint: RigJointOrdinal,
    channel: JointChannel,
}

/// Insertion-ordered column in one admitted parameter transform.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModelParameterColumn(usize);

/// Exact, case-sensitive model-parameter identity; duplicates retain order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelParameterName(String);

/// Exact identity of an authored parameter subset.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ParameterSetName(String);

/// Values in joint order, with the seven channels stored in authored order.
#[derive(Debug, Clone)]
pub struct JointParameterTable<T>(RigJointTable<JointChannels<T>>);

/// The authored channel roles own selection within one joint.
#[derive(Debug, Clone)]
pub struct JointChannels<T>([T; PARAMETERS_PER_JOINT]);

/// Values in insertion-ordered model-parameter columns.
#[derive(Debug, Clone)]
pub struct ModelParameterTable<T>(Vec<T>);

/// Borrowed coefficients selected by a joint-channel row.
#[derive(Debug, Clone, Copy)]
pub struct ModelParameterRow<'a> {
    pub(super) values: &'a [f32],
    pub(super) source: &'a super::ParameterTransform,
    pub(super) joint: RigJointOrdinal,
    pub(super) channel: JointChannel,
}

/// The authored seven-channel order of each Momentum joint.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JointChannel {
    TranslationX,
    TranslationY,
    TranslationZ,
    RotationX,
    RotationY,
    RotationZ,
    Scale,
}

impl JointChannel {
    pub const ALL: [Self; PARAMETERS_PER_JOINT] = [
        Self::TranslationX,
        Self::TranslationY,
        Self::TranslationZ,
        Self::RotationX,
        Self::RotationY,
        Self::RotationZ,
        Self::Scale,
    ];
    pub const TRANSLATIONS: [Self; 3] =
        [Self::TranslationX, Self::TranslationY, Self::TranslationZ];
    pub const ROTATIONS: [Self; 3] = [Self::RotationX, Self::RotationY, Self::RotationZ];
    pub const ROTATION_AND_SCALE: [Self; 4] = [
        Self::RotationX,
        Self::RotationY,
        Self::RotationZ,
        Self::Scale,
    ];

    pub fn from_model_token(token: &str) -> Option<Self> {
        match token {
            "tx" => Some(Self::TranslationX),
            "ty" => Some(Self::TranslationY),
            "tz" => Some(Self::TranslationZ),
            "rx" => Some(Self::RotationX),
            "ry" => Some(Self::RotationY),
            "rz" => Some(Self::RotationZ),
            "sc" => Some(Self::Scale),
            _ => None,
        }
    }

    /// Native seven-channel tensor position.
    pub(crate) fn native_index(self) -> usize {
        match self {
            Self::TranslationX => 0,
            Self::TranslationY => 1,
            Self::TranslationZ => 2,
            Self::RotationX => 3,
            Self::RotationY => 4,
            Self::RotationZ => 5,
            Self::Scale => 6,
        }
    }
}

impl JointParameterRow {
    pub(crate) fn new(joint: RigJointOrdinal, channel: JointChannel) -> Self {
        Self { joint, channel }
    }

    /// Native row-major offset, used only by the dense matrix adapter.
    fn offset(self) -> usize {
        usize::from(self.joint) * PARAMETERS_PER_JOINT + self.channel.native_index()
    }
}

impl ModelParameterColumn {
    /// Native parameter-vector/tensor indexing at a numerical boundary.
    pub fn native_index(self) -> usize {
        self.0
    }
}

impl From<&str> for ModelParameterName {
    fn from(name: &str) -> Self {
        Self(name.to_owned())
    }
}

impl From<String> for ModelParameterName {
    fn from(name: String) -> Self {
        Self(name)
    }
}

impl From<&str> for ParameterSetName {
    fn from(name: &str) -> Self {
        Self(name.to_owned())
    }
}

impl<T> JointChannels<T> {
    pub(crate) fn from_fn(mut value: impl FnMut(JointChannel) -> T) -> Self {
        Self(JointChannel::ALL.map(&mut value))
    }
}

impl<T> Index<JointChannel> for JointChannels<T> {
    type Output = T;

    fn index(&self, channel: JointChannel) -> &T {
        &self.0[channel.native_index()]
    }
}

impl<T> IndexMut<JointChannel> for JointChannels<T> {
    fn index_mut(&mut self, channel: JointChannel) -> &mut T {
        &mut self.0[channel.native_index()]
    }
}

impl<T: Clone> JointParameterTable<T> {
    pub fn filled(joints: RigJointCount, value: T) -> Self {
        Self(
            joints
                .ordinals()
                .map(|_| JointChannels::from_fn(|_| value.clone()))
                .collect(),
        )
    }
}

impl<T> JointParameterTable<T> {
    pub(crate) fn iter(&self) -> impl Iterator<Item = &T> {
        self.0.iter().flat_map(|channels| channels.0.iter())
    }

    pub(crate) fn rows(&self) -> impl Iterator<Item = &JointChannels<T>> {
        self.0.iter()
    }

    pub fn row(&self, joint: RigJointOrdinal) -> Option<&JointChannels<T>> {
        self.0.get(joint)
    }

    pub(crate) fn get_mut(&mut self, row: JointParameterRow) -> Option<&mut T> {
        self.0
            .get_mut(row.joint)
            .map(|channels| &mut channels[row.channel])
    }
}

impl JointParameterTable<f32> {
    /// Row-major upload to a native tensor, in the model's seven-channel order.
    pub(crate) fn tensor_values(&self) -> Vec<f32> {
        self.iter().copied().collect()
    }
}

impl<T> ModelParameterTable<T> {
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Array-axis extent at the table's actual storage owner.
    pub fn column_count(&self) -> NpyDimension {
        NpyDimension::from(self.0.len())
    }

    pub fn push(&mut self, value: T) -> ModelParameterColumn {
        let column = ModelParameterColumn(self.len());
        self.0.push(value);
        column
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.0.iter()
    }
}

impl<T: Clone> ModelParameterTable<T> {
    pub fn filled(columns: NpyDimension, value: T) -> Self {
        Self(vec![value; usize::from(columns)])
    }

    pub fn resize(&mut self, columns: NpyDimension, value: T) {
        self.0.resize(usize::from(columns), value);
    }
}

impl ModelParameterTable<f32> {
    /// Consumes model-column coefficients for a native tensor upload.
    pub fn into_tensor_values(self) -> Vec<f32> {
        self.0
    }
}

impl ModelParameterTable<ModelParameterName> {
    /// Duplicate names resolve to their first introduced column.
    pub fn column(&self, name: &ModelParameterName) -> Option<ModelParameterColumn> {
        self.0
            .iter()
            .position(|candidate| candidate == name)
            .map(ModelParameterColumn)
    }
}

impl<T> Index<ModelParameterColumn> for ModelParameterTable<T> {
    type Output = T;

    fn index(&self, column: ModelParameterColumn) -> &Self::Output {
        &self.0[column.0]
    }
}

impl<T> IndexMut<ModelParameterColumn> for ModelParameterTable<T> {
    fn index_mut(&mut self, column: ModelParameterColumn) -> &mut Self::Output {
        &mut self.0[column.0]
    }
}

impl ModelParameterRow<'_> {
    /// The source rig label at this row's actual ordered slot.
    pub fn joint_name(&self) -> &fabelgeist_rig::RigJointName {
        &self.source.source_joints[self.joint]
    }

    pub fn channel(&self) -> JointChannel {
        self.channel
    }

    pub fn get(&self, column: ModelParameterColumn) -> Option<&f32> {
        self.values.get(column.0)
    }

    pub fn as_slice(&self) -> &[f32] {
        self.values
    }
}

impl Index<ModelParameterColumn> for ModelParameterRow<'_> {
    type Output = f32;

    fn index(&self, column: ModelParameterColumn) -> &Self::Output {
        &self.values[column.0]
    }
}

impl<T> Default for ModelParameterTable<T> {
    fn default() -> Self {
        Self(Vec::new())
    }
}
