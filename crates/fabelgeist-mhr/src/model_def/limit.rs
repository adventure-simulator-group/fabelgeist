//! Admitted minmax intervals and the separate solver-enforcement weight.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ParameterBounds {
    pub(super) minimum: f32,
    pub(super) maximum: f32,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParameterBoundsError {
    NotNumber,
    Unordered,
}
impl TryFrom<(f32, f32)> for ParameterBounds {
    type Error = ParameterBoundsError;
    fn try_from((minimum, maximum): (f32, f32)) -> Result<Self, ParameterBoundsError> {
        if minimum.is_nan() || maximum.is_nan() {
            return Err(ParameterBoundsError::NotNumber);
        }
        if minimum > maximum {
            return Err(ParameterBoundsError::Unordered);
        }
        Ok(Self { minimum, maximum })
    }
}
impl std::fmt::Display for ParameterBoundsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::NotNumber => "a parameter bound is NaN",
            Self::Unordered => "minimum parameter bound exceeds maximum",
        })
    }
}
impl std::error::Error for ParameterBoundsError {}
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SolverLimitWeight(f32);
impl From<f32> for SolverLimitWeight {
    fn from(weight: f32) -> Self {
        Self(weight)
    }
}
