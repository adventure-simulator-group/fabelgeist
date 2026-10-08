//! Host admission of identity, pose and expression tensor layouts.
use super::{NUM_FACE_EXPRESSION_BLEND_SHAPES, NUM_IDENTITY_BLEND_SHAPES};
mod error;
pub use error::MhrEvaluationError;

/// Rows in a model evaluation; zero rows retain their existing layout meaning.
///
/// ```compile_fail
/// use fabelgeist_mhr::{Mhr, PoseParameterCount};
/// fn rest_pose(model: &Mhr, columns: PoseParameterCount) {
///     model.zero_parameters(columns);
/// }
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ModelBatchSize(usize);
impl From<usize> for ModelBatchSize {
    fn from(rows: usize) -> Self {
        Self(rows)
    }
}
impl From<ModelBatchSize> for usize {
    fn from(batch: ModelBatchSize) -> Self {
        batch.0
    }
}
impl std::fmt::Display for ModelBatchSize {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(formatter)
    }
}
/// Number of identity coefficients in a tensor row.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IdentityCoefficientCount(usize);
impl From<usize> for IdentityCoefficientCount {
    fn from(columns: usize) -> Self {
        Self(columns)
    }
}
impl std::fmt::Display for IdentityCoefficientCount {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(formatter)
    }
}
/// Number of facial-expression coefficients in a tensor row.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ExpressionCoefficientCount(usize);
impl From<usize> for ExpressionCoefficientCount {
    fn from(columns: usize) -> Self {
        Self(columns)
    }
}
/// Pose/scale columns, excluding the identity columns appended to the transform.
///
/// ```compile_fail
/// use fabelgeist_mhr::{ModelParameterCount, PoseParameterCount};
/// fn pose_columns(total: ModelParameterCount) -> PoseParameterCount {
///     PoseParameterCount::from(total)
/// }
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PoseParameterCount(usize);
impl From<usize> for PoseParameterCount {
    fn from(columns: usize) -> Self {
        Self(columns)
    }
}
impl From<PoseParameterCount> for usize {
    fn from(columns: PoseParameterCount) -> Self {
        columns.0
    }
}
impl std::fmt::Display for PoseParameterCount {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(formatter)
    }
}
impl std::fmt::Display for ExpressionCoefficientCount {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(formatter)
    }
}
pub(super) enum IdentityRows {
    Exact,
    Broadcast,
}
pub(super) struct IdentityLayout {
    rows: ModelBatchSize,
    columns: IdentityCoefficientCount,
}
impl From<[usize; 2]> for IdentityLayout {
    fn from(dimensions: [usize; 2]) -> Self {
        Self {
            rows: ModelBatchSize::from(dimensions[0]),
            columns: IdentityCoefficientCount::from(dimensions[1]),
        }
    }
}
impl IdentityLayout {
    pub(super) fn admit(self, batch: ModelBatchSize) -> Result<IdentityRows, MhrEvaluationError> {
        if self.columns != IdentityCoefficientCount::from(NUM_IDENTITY_BLEND_SHAPES) {
            return Err(MhrEvaluationError::IdentityColumns(self.columns));
        }
        if self.rows == batch {
            Ok(IdentityRows::Exact)
        } else if self.rows == ModelBatchSize::from(1) {
            Ok(IdentityRows::Broadcast)
        } else {
            Err(MhrEvaluationError::IdentityRows {
                actual: self.rows,
                batch,
            })
        }
    }
}
pub(super) struct PoseLayout {
    pub(super) batch: ModelBatchSize,
    columns: PoseParameterCount,
}
impl From<[usize; 2]> for PoseLayout {
    fn from(dimensions: [usize; 2]) -> Self {
        Self {
            batch: ModelBatchSize::from(dimensions[0]),
            columns: PoseParameterCount::from(dimensions[1]),
        }
    }
}
impl PoseLayout {
    pub(super) fn admit(self, expected: PoseParameterCount) -> Result<(), MhrEvaluationError> {
        if self.columns != expected {
            return Err(MhrEvaluationError::PoseColumns {
                actual: self.columns,
                expected,
            });
        }
        Ok(())
    }
}
pub(super) struct ExpressionLayout {
    rows: ModelBatchSize,
    columns: ExpressionCoefficientCount,
}
impl From<[usize; 2]> for ExpressionLayout {
    fn from(dimensions: [usize; 2]) -> Self {
        Self {
            rows: ModelBatchSize::from(dimensions[0]),
            columns: ExpressionCoefficientCount::from(dimensions[1]),
        }
    }
}
impl ExpressionLayout {
    pub(super) fn admit(self, batch: ModelBatchSize) -> Result<(), MhrEvaluationError> {
        if self.columns != ExpressionCoefficientCount::from(NUM_FACE_EXPRESSION_BLEND_SHAPES) {
            return Err(MhrEvaluationError::ExpressionColumns(self.columns));
        }
        if self.rows != batch {
            return Err(MhrEvaluationError::ExpressionRows {
                actual: self.rows,
                batch,
            });
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
