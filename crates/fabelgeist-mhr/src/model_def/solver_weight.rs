//! Solver-enforcement weights and their model-text admission boundary.

use std::num::ParseFloatError;

use super::ModelParameterName;

type Result<T> = std::result::Result<T, SolverLimitWeightAdmissionError>;

/// How strongly a solver should enforce a parameter limit.
///
/// Native construction admits every `f32` bit pattern, including signed zero,
/// negative values, infinities and NaN payloads. Clamping ignores this weight.
#[derive(Clone, Copy, PartialEq, derive_more::Debug, derive_more::From)]
#[debug("{_0:?}")]
pub struct SolverLimitWeight(f32);

/// A malformed present solver-weight token in a model definition.
///
/// The admitted parameter identity remains typed. Source text reflects the
/// parser's existing whitespace trimming and comment removal; the native float
/// parse failure is available through `std::error::Error::source`.
#[derive(Debug, derive_more::Display)]
#[display(
    "invalid solver limit weight {rejected_token:?} for parameter {parameter:?} in: {source_line}"
)]
pub struct SolverLimitWeightAdmissionError {
    parameter: ModelParameterName,
    source_line: String,
    rejected_token: String,
    cause: ParseFloatError,
}

impl SolverLimitWeight {
    /// Weight used when the model definition omits its trailing solver weight.
    pub const DEFAULT: Self = Self(1.0);

    pub(super) fn parse_model_token(
        token: &str,
        source_line: &str,
        parameter: &ModelParameterName,
    ) -> Result<Self> {
        if token.is_empty() {
            return Ok(Self::DEFAULT);
        }
        token.parse::<f32>().map(Self::from).map_err(|cause| {
            SolverLimitWeightAdmissionError::new(parameter, source_line, token, cause)
        })
    }
}

/// Projects a weight only at a native numerical or presentation boundary.
impl From<SolverLimitWeight> for f32 {
    fn from(weight: SolverLimitWeight) -> Self {
        weight.0
    }
}

impl SolverLimitWeightAdmissionError {
    pub fn parameter(&self) -> &ModelParameterName {
        &self.parameter
    }

    /// Rejected model-definition line after comment removal and trimming.
    pub fn line(&self) -> &str {
        &self.source_line
    }

    pub fn rejected_token(&self) -> &str {
        &self.rejected_token
    }

    fn new(
        parameter: &ModelParameterName,
        source_line: &str,
        rejected_token: &str,
        cause: ParseFloatError,
    ) -> Self {
        Self {
            parameter: parameter.clone(),
            source_line: source_line.to_owned(),
            rejected_token: rejected_token.to_owned(),
            cause,
        }
    }
}

impl std::error::Error for SolverLimitWeightAdmissionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.cause)
    }
}
