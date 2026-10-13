//! Checked inclusive intervals for model parameters.

use super::ModelParameterName;

/// Inclusive parameter endpoints, admitted without changing their float bits.
///
/// Equal endpoints and infinities are valid. NaN endpoints and a minimum greater
/// than the maximum are rejected before the interval can reach clamping.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ParameterBounds {
    minimum: f32,
    maximum: f32,
}

/// A rejected `minmax` interval, with its parameter and model-definition line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParameterBoundsAdmissionError {
    parameter: ModelParameterName,
    line: String,
    cause: ParameterBoundsError,
}

/// Why a native interval cannot become checked parameter bounds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParameterBoundsError {
    /// At least one endpoint is NaN.
    NaNEndpoint,
    /// The minimum is greater than the maximum.
    Reversed,
}

impl TryFrom<(f32, f32)> for ParameterBounds {
    type Error = ParameterBoundsError;

    fn try_from((minimum, maximum): (f32, f32)) -> Result<Self, Self::Error> {
        if minimum.is_nan() || maximum.is_nan() {
            return Err(ParameterBoundsError::NaNEndpoint);
        }
        if minimum > maximum {
            return Err(ParameterBoundsError::Reversed);
        }
        Ok(Self { minimum, maximum })
    }
}

/// Native endpoints for standard numeric and presentation APIs.
impl From<ParameterBounds> for (f32, f32) {
    fn from(bounds: ParameterBounds) -> Self {
        (bounds.minimum, bounds.maximum)
    }
}

impl std::fmt::Display for ParameterBoundsError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::NaNEndpoint => "a parameter bound is NaN",
            Self::Reversed => "minimum parameter bound exceeds maximum",
        })
    }
}

impl std::error::Error for ParameterBoundsError {}

impl ParameterBoundsAdmissionError {
    pub(super) fn new(
        parameter: ModelParameterName,
        line: &str,
        cause: ParameterBoundsError,
    ) -> Self {
        Self {
            parameter,
            line: line.to_owned(),
            cause,
        }
    }

    pub fn parameter(&self) -> &ModelParameterName {
        &self.parameter
    }

    /// Rejected model-definition line after comment removal and trimming.
    pub fn line(&self) -> &str {
        &self.line
    }

    pub fn cause(&self) -> ParameterBoundsError {
        self.cause
    }
}

impl std::fmt::Display for ParameterBoundsAdmissionError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "invalid minmax bounds for parameter {} in: {}",
            self.parameter, self.line
        )
    }
}

impl std::error::Error for ParameterBoundsAdmissionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.cause)
    }
}
