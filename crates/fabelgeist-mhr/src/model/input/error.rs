//! Stable evaluation failures with nominal rejected dimensions.
use super::{
    ExpressionCoefficientCount, IdentityCoefficientCount, ModelBatchSize, PoseParameterCount,
};
use crate::model::{NUM_FACE_EXPRESSION_BLEND_SHAPES, NUM_IDENTITY_BLEND_SHAPES};

/// Host-side shape validation failures for an MHR evaluation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MhrEvaluationError {
    IdentityColumns(IdentityCoefficientCount),
    IdentityRows {
        actual: ModelBatchSize,
        batch: ModelBatchSize,
    },
    PoseColumns {
        actual: PoseParameterCount,
        expected: PoseParameterCount,
    },
    ExpressionColumns(ExpressionCoefficientCount),
    ExpressionRows {
        actual: ModelBatchSize,
        batch: ModelBatchSize,
    },
}
impl std::fmt::Display for MhrEvaluationError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IdentityColumns(actual) => write!(
                formatter,
                "identity coefficients have {actual} columns, expected {NUM_IDENTITY_BLEND_SHAPES}"
            ),
            Self::IdentityRows { actual, batch } => write!(
                formatter,
                "identity coefficients have {actual} rows, expected {batch} or 1"
            ),
            Self::PoseColumns { actual, expected } => write!(
                formatter,
                "model parameters have {actual} columns, expected {expected}"
            ),
            Self::ExpressionColumns(actual) => write!(
                formatter,
                "expression coefficients have {actual} columns, expected {NUM_FACE_EXPRESSION_BLEND_SHAPES}"
            ),
            Self::ExpressionRows { actual, batch } => write!(
                formatter,
                "expression coefficients have {actual} rows, expected {batch}"
            ),
        }
    }
}
impl std::error::Error for MhrEvaluationError {}
