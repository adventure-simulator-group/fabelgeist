//! Identities and dense adapters for Momentum's joint-channel transform.

use crate::character::PARAMETERS_PER_JOINT;
use fabelgeist_rig::{RigJointOrdinal, RigJointTable};
use std::ops::{Index, IndexMut};

/// A joint channel is a matrix row, never a model-parameter column.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct JointParameterRow {
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
pub struct JointParameterTable<T>(RigJointTable<[T; PARAMETERS_PER_JOINT]>);

/// Values in insertion-ordered model-parameter columns.
#[derive(Debug, Clone)]
pub struct ModelParameterTable<T>(Vec<T>);

/// Row-major coefficients with row and column identity retained at every API.
#[derive(Debug, Default, Clone)]
pub struct ParameterMatrix {
    values: Vec<f32>,
    joints: usize,
    columns: usize,
}

/// Borrowed coefficients selected by a joint-channel row.
#[derive(Debug)]
pub struct ModelParameterRow<'a>(&'a [f32]);

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
    pub const TRANSLATIONS: [Self; 3] = [
        Self::TranslationX,
        Self::TranslationY,
        Self::TranslationZ,
    ];
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

    fn offset(self) -> usize {
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
    pub fn new(joint: RigJointOrdinal, channel: JointChannel) -> Self {
        Self { joint, channel }
    }

    /// Native row-major offset, used only by the dense matrix adapter.
    fn offset(self) -> usize {
        usize::from(self.joint) * PARAMETERS_PER_JOINT + self.channel.offset()
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

impl<T: Clone> JointParameterTable<T> {
    pub fn filled(joints: usize, value: T) -> Self {
        Self(
            (0..joints)
                .map(|_| std::array::from_fn(|_| value.clone()))
                .collect(),
        )
    }
}

impl<T> JointParameterTable<T> {
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.0.iter().flatten()
    }
}

impl JointParameterTable<f32> {
    /// Row-major upload to a native tensor, in the model's seven-channel order.
    pub fn tensor_values(&self) -> Vec<f32> {
        self.iter().copied().collect()
    }
}

impl<T> Index<JointParameterRow> for JointParameterTable<T> {
    type Output = T;

    fn index(&self, row: JointParameterRow) -> &Self::Output {
        &self.0[row.joint][row.channel.offset()]
    }
}

impl<T> IndexMut<JointParameterRow> for JointParameterTable<T> {
    fn index_mut(&mut self, row: JointParameterRow) -> &mut Self::Output {
        &mut self.0[row.joint][row.channel.offset()]
    }
}

impl<T> ModelParameterTable<T> {
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
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
    pub fn filled(columns: usize, value: T) -> Self {
        Self(vec![value; columns])
    }

    pub fn resize(&mut self, columns: usize, value: T) {
        self.0.resize(columns, value);
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

impl ParameterMatrix {
    pub fn zeros(joints: usize, columns: usize) -> Self {
        Self {
            values: vec![0.0; joints * PARAMETERS_PER_JOINT * columns],
            joints,
            columns,
        }
    }

    pub fn rows(&self) -> impl Iterator<Item = JointParameterRow> + use<> {
        (0..self.joints).flat_map(|joint| {
            JointChannel::ALL
                .into_iter()
                .map(move |channel| JointParameterRow::new(joint.into(), channel))
        })
    }

    pub fn row(&self, row: JointParameterRow) -> ModelParameterRow<'_> {
        let start = row.offset() * self.columns;
        ModelParameterRow(&self.values[start..start + self.columns])
    }

    pub fn add(&mut self, row: JointParameterRow, column: ModelParameterColumn, value: f32) {
        self.values[row.offset() * self.columns + column.0] += value;
    }

    pub fn append_columns(&mut self, columns: usize) {
        let mut expanded = Self::zeros(self.joints, columns);
        for row in self.rows() {
            let start = row.offset() * columns;
            expanded.values[start..start + self.columns].copy_from_slice(self.row(row).as_slice());
        }
        *self = expanded;
    }

    /// Transpose the selected leading columns for native `parameters @ transform`.
    pub fn tensor_transpose(&self, columns: usize) -> Vec<f32> {
        let rows = self.joints * PARAMETERS_PER_JOINT;
        let mut transposed = vec![0.0; columns * rows];
        for row in self.rows() {
            for column in 0..columns {
                transposed[column * rows + row.offset()] =
                    self.row(row)[ModelParameterColumn(column)];
            }
        }
        transposed
    }
}

impl ModelParameterRow<'_> {
    pub fn as_slice(&self) -> &[f32] {
        self.0
    }
}

impl Index<ModelParameterColumn> for ModelParameterRow<'_> {
    type Output = f32;

    fn index(&self, column: ModelParameterColumn) -> &Self::Output {
        &self.0[column.0]
    }
}

impl<T> Default for JointParameterTable<T> {
    fn default() -> Self {
        Self(RigJointTable::default())
    }
}

impl<T> Default for ModelParameterTable<T> {
    fn default() -> Self {
        Self(Vec::new())
    }
}
