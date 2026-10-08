//! Solver-enforcement weight admission and its native representation ports.

use std::fmt;
use std::num::ParseFloatError;

/// How strongly a solver should enforce a parameter limit.
///
/// Every native `f32` value is admitted, including negative values, NaN,
/// infinities and signed zero. Clamping does not use this weight. Construct it
/// explicitly with `SolverLimitWeight::from(value)` at a native numeric boundary.
///
/// Public limit metadata retains the weight's role:
///
/// ```compile_fail
/// use fabelgeist_mhr::model_def::ParameterLimit;
/// let _ = ParameterLimit { parameter: 0, min: -1.0, max: 1.0, weight: 0.5 };
/// ```
///
/// Reading the field also retains that role until explicit native inspection:
///
/// ```compile_fail
/// use fabelgeist_mhr::model_def::SolverLimitWeight;
/// let weight = SolverLimitWeight::from(0.5);
/// let native: f32 = weight;
/// ```
#[derive(Clone, Copy, PartialEq)]
pub struct SolverLimitWeight(f32);

impl SolverLimitWeight {
    /// Weight used when a minmax declaration omits its solver weight.
    pub const DEFAULT: Self = Self(1.0);

    pub(super) fn from_model_token(
        token: &str,
        source_line: &str,
        parameter_name: &str,
    ) -> Result<Self, SolverLimitWeightAdmissionError> {
        if token.is_empty() {
            return Ok(Self::DEFAULT);
        }
        token
            .parse::<f32>()
            .map(Self::from)
            .map_err(|cause| SolverLimitWeightAdmissionError {
                source_line: source_line.to_owned(),
                parameter_name: parameter_name.to_owned(),
                rejected_token: token.to_owned(),
                cause,
            })
    }
}

impl From<f32> for SolverLimitWeight {
    fn from(weight: f32) -> Self {
        Self(weight)
    }
}

/// Projects the weight for native inspection or external numeric representation.
impl From<SolverLimitWeight> for f32 {
    fn from(weight: SolverLimitWeight) -> Self {
        weight.0
    }
}

// Public ParameterLimit diagnostics retain the native float spelling.
impl fmt::Debug for SolverLimitWeight {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.0, formatter)
    }
}

/// A present minmax solver-weight token could not be parsed as a native float.
///
/// Context retains the parameter, rejected trailing text and source line after
/// the model parser's existing comment stripping and whitespace trimming.
/// The native parse error remains available through [`std::error::Error::source`].
#[derive(Debug)]
pub struct SolverLimitWeightAdmissionError {
    pub source_line: String,
    pub parameter_name: String,
    pub rejected_token: String,
    cause: ParseFloatError,
}

impl fmt::Display for SolverLimitWeightAdmissionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "invalid solver limit weight {:?} for parameter {:?} in: {}",
            self.rejected_token, self.parameter_name, self.source_line,
        )
    }
}

impl std::error::Error for SolverLimitWeightAdmissionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.cause)
    }
}
