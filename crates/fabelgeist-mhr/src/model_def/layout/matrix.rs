//! Checked dense coefficient storage and native tensor transposition.

use super::{JointChannel, JointChannels, JointParameterRow, ModelParameterColumn};
use crate::character::PARAMETERS_PER_JOINT;
use derive_more::{Display, Error};
use fabelgeist_numpy_storage::{NpyDimension, NpyLayoutError, NpyTensorLayout};
use fabelgeist_rig::{RigJointCount, RigJointOrdinal};

pub(crate) type Result<T> = std::result::Result<T, ParameterLayoutError>;

/// Row-major coefficients; the entire native allocation extent is admitted.
#[derive(Debug, Clone)]
pub(crate) struct ParameterMatrix {
    values: Vec<f32>,
    joints: RigJointCount,
    layout: NpyTensorLayout<2>,
}

#[derive(Debug, Display, Error)]
pub enum ParameterLayoutError {
    #[display("joint-channel row count overflows for {joints} joints")]
    JointRows { joints: RigJointCount },
    #[display("parameter matrix layout is not representable: {source}")]
    Matrix { source: NpyLayoutError },
    #[display("parameter matrix allocation is not representable: {source}")]
    Allocation { source: std::alloc::LayoutError },
    #[display("model parameter column count overflows")]
    ColumnCount,
    #[display("requested {columns} parameter columns from a matrix with {available}")]
    ColumnPrefix {
        columns: NpyDimension,
        available: NpyDimension,
    },
    #[display("joint-channel row is outside the parameter matrix")]
    RowMembership,
    #[display("model parameter column is outside the parameter matrix")]
    ColumnMembership,
}

impl ParameterMatrix {
    pub(crate) fn zeros(joints: RigJointCount, columns: NpyDimension) -> Result<Self> {
        let rows = NpyDimension::from(joints.native_joint_count())
            .checked_mul(NpyDimension::from(PARAMETERS_PER_JOINT))
            .ok_or(ParameterLayoutError::JointRows { joints })?;
        let layout = NpyTensorLayout::try_from([rows, columns])
            .map_err(|source| ParameterLayoutError::Matrix { source })?;
        let elements = usize::from(layout.element_count());
        std::alloc::Layout::array::<f32>(elements)
            .map_err(|source| ParameterLayoutError::Allocation { source })?;
        Ok(Self {
            values: vec![0.0; elements],
            joints,
            layout,
        })
    }

    pub(crate) fn columns(&self) -> NpyDimension {
        self.layout.dimensions()[1]
    }

    /// An external candidate is checked against both the actual joint bound and
    /// the retained buffer. A zero-column matrix still checks joint membership.
    pub(crate) fn row(&self, row: JointParameterRow) -> Option<&[f32]> {
        if usize::from(row.joint) >= self.joints.native_joint_count() {
            return None;
        }
        let columns = usize::from(self.columns());
        let start = row.offset().checked_mul(columns)?;
        self.values.get(start..start.checked_add(columns)?)
    }

    /// Each yielded block comes from the admitted matrix's own ordered extent.
    pub(crate) fn joints(&self) -> impl Iterator<Item = JointChannels<&[f32]>> {
        self.joints.ordinals().map(move |joint| {
            let columns = usize::from(self.columns());
            JointChannels::from_fn(|channel| {
                let row = JointParameterRow::new(joint, channel);
                let start = row.offset() * columns;
                &self.values[start..start + columns]
            })
        })
    }

    pub(crate) fn add(
        &mut self,
        row: JointParameterRow,
        column: ModelParameterColumn,
        value: f32,
    ) -> Result<()> {
        let columns = usize::from(self.columns());
        let selected = self.row(row).ok_or(ParameterLayoutError::RowMembership)?;
        let length = selected.len();
        let start = row.offset() * columns;
        let entry = self
            .values
            .get_mut(start..start + length)
            .and_then(|values| values.get_mut(column.0))
            .ok_or(ParameterLayoutError::ColumnMembership)?;
        *entry += value;
        Ok(())
    }

    pub(crate) fn append_columns(&mut self, columns: NpyDimension) -> Result<()> {
        let previous = self.columns();
        if usize::from(columns) < usize::from(previous) {
            return Err(ParameterLayoutError::ColumnPrefix {
                columns: previous,
                available: columns,
            });
        }
        let mut expanded = Self::zeros(self.joints, columns)?;
        for (joint, rows) in self.joints().enumerate() {
            for channel in JointChannel::ALL {
                let row = JointParameterRow::new(RigJointOrdinal::from(joint), channel);
                let start = row.offset() * usize::from(columns);
                expanded.values[start..start + usize::from(previous)]
                    .copy_from_slice(rows[channel]);
            }
        }
        *self = expanded;
        Ok(())
    }

    /// Transpose the selected leading columns for native `parameters @ transform`.
    pub(crate) fn tensor_transpose(&self, columns: NpyDimension) -> Result<Vec<f32>> {
        if usize::from(columns) > usize::from(self.columns()) {
            return Err(ParameterLayoutError::ColumnPrefix {
                columns,
                available: self.columns(),
            });
        }
        let mut transposed = Self::zeros(self.joints, columns)?.values;
        let rows = usize::from(self.layout.dimensions()[0]);
        for (joint, channels) in self.joints().enumerate() {
            for channel in JointChannel::ALL {
                let row = JointParameterRow::new(RigJointOrdinal::from(joint), channel);
                for column in 0..usize::from(columns) {
                    transposed[column * rows + row.offset()] = channels[channel][column];
                }
            }
        }
        Ok(transposed)
    }
}
