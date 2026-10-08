//! Diagnostic quantities retain their units; comparisons own their bound role.
use super::{SupportBound, SupportDiagnosticUnit};
use serde::Serialize;

/// Metres from the support solver's exact diagnostic controls.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(transparent)]
pub struct DiagnosticMetres(f64);
/// Square metres from exact clipped support footprints.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(transparent)]
pub struct DiagnosticArea(f64);
/// Cardinality of a required binding or an indexed mesh.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct DiagnosticCount(u64);
impl DiagnosticMetres {
    pub fn from_metres(value: f64) -> Option<Self> {
        value.is_finite().then_some(Self(value))
    }
    pub fn metres(self) -> f64 {
        self.0
    }
}
impl DiagnosticArea {
    pub fn from_square_metres(value: f64) -> Option<Self> {
        (value.is_finite() && value >= 0.0).then_some(Self(value))
    }
    pub fn square_metres(self) -> f64 {
        self.0
    }
}
impl DiagnosticCount {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }
    pub const fn count(self) -> u64 {
        self.0
    }
}

/// A minimum deficit and maximum excess have opposite subtraction directions.
/// Exact comparisons report the absolute discrepancy.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BoundViolation<T> {
    Minimum { actual: T, required: T },
    Maximum { actual: T, permitted: T },
    Exact { actual: T, required: T },
}
impl<T: Copy> BoundViolation<T> {
    fn controls(self) -> ViolationControls<T> {
        match self {
            Self::Minimum { actual, required } => ViolationControls {
                actual,
                limit: required,
                bound: SupportBound::Minimum,
            },
            Self::Maximum { actual, permitted } => ViolationControls {
                actual,
                limit: permitted,
                bound: SupportBound::Maximum,
            },
            Self::Exact { actual, required } => ViolationControls {
                actual,
                limit: required,
                bound: SupportBound::Exact,
            },
        }
    }
    fn with_bound(self, bound: SupportBound) -> Self {
        let ViolationControls { actual, limit, .. } = self.controls();
        match bound {
            SupportBound::Minimum => Self::Minimum {
                actual,
                required: limit,
            },
            SupportBound::Maximum => Self::Maximum {
                actual,
                permitted: limit,
            },
            SupportBound::Exact => Self::Exact {
                actual,
                required: limit,
            },
        }
    }
}

/// A comparison cannot mix lengths, areas or cardinalities.
/// ```compile_fail
/// use adventuresim_tactical_core::city_layout::grounding::{BoundViolation, DiagnosticMetres, DiagnosticCount};
/// let comparison = BoundViolation::Maximum {
///     actual: DiagnosticMetres::from_metres(2.0).unwrap(),
///     permitted: DiagnosticCount::new(1),
/// };
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SupportViolation {
    Metres(BoundViolation<DiagnosticMetres>),
    SquareMetres(BoundViolation<DiagnosticArea>),
    Count(BoundViolation<DiagnosticCount>),
    /// A numerical kernel could not produce a finite measurement.
    Unmeasurable {
        unit: SupportDiagnosticUnit,
        bound: SupportBound,
    },
}
impl SupportViolation {
    /// Native diagnostic adapter for finite solver controls. This port chooses
    /// units from the preventing constraint; values never re-enter geometry.
    pub(super) fn maximum(unit: SupportDiagnosticUnit, actual: f64, permitted: f64) -> Self {
        match unit {
            SupportDiagnosticUnit::Metres => match (
                DiagnosticMetres::from_metres(actual),
                DiagnosticMetres::from_metres(permitted),
            ) {
                (Some(actual), Some(permitted)) => {
                    Self::Metres(BoundViolation::Maximum { actual, permitted })
                }
                _ => Self::Unmeasurable {
                    unit,
                    bound: SupportBound::Maximum,
                },
            },
            SupportDiagnosticUnit::SquareMetres => match (
                DiagnosticArea::from_square_metres(actual),
                DiagnosticArea::from_square_metres(permitted),
            ) {
                (Some(actual), Some(permitted)) => {
                    Self::SquareMetres(BoundViolation::Maximum { actual, permitted })
                }
                _ => Self::Unmeasurable {
                    unit,
                    bound: SupportBound::Maximum,
                },
            },
            SupportDiagnosticUnit::Count
                if actual.is_finite()
                    && actual >= 0.0
                    && permitted.is_finite()
                    && permitted >= 0.0 =>
            {
                Self::Count(BoundViolation::Maximum {
                    actual: DiagnosticCount(actual as u64),
                    permitted: DiagnosticCount(permitted as u64),
                })
            }
            _ => Self::Unmeasurable {
                unit,
                bound: SupportBound::Maximum,
            },
        }
    }
    pub(super) fn with_bound(self, bound: SupportBound) -> Self {
        match self {
            Self::Metres(v) => Self::Metres(v.with_bound(bound)),
            Self::SquareMetres(v) => Self::SquareMetres(v.with_bound(bound)),
            Self::Count(v) => Self::Count(v.with_bound(bound)),
            Self::Unmeasurable { unit, .. } => Self::Unmeasurable { unit, bound },
        }
    }
    pub fn unit(self) -> SupportDiagnosticUnit {
        match self {
            Self::Metres(_) => SupportDiagnosticUnit::Metres,
            Self::SquareMetres(_) => SupportDiagnosticUnit::SquareMetres,
            Self::Count(_) => SupportDiagnosticUnit::Count,
            Self::Unmeasurable { unit, .. } => unit,
        }
    }
    /// Presentation/test numerical port. Unmeasurable controls remain explicit
    /// in the structured error; infinity here conveys them only to formatting.
    pub fn actual_value(self) -> f64 {
        self.values().actual
    }
    pub fn limit_value(self) -> f64 {
        self.values().limit
    }
    pub fn discrepancy_value(self) -> f64 {
        let ViolationControls {
            actual,
            limit,
            bound,
        } = self.values();
        match bound {
            SupportBound::Minimum => (limit - actual).max(0.0),
            SupportBound::Maximum => (actual - limit).max(0.0),
            SupportBound::Exact => (actual - limit).abs(),
        }
    }
    fn values(self) -> ViolationControls<f64> {
        match self {
            Self::Metres(v) => {
                let c = v.controls();
                ViolationControls {
                    actual: c.actual.0,
                    limit: c.limit.0,
                    bound: c.bound,
                }
            }
            Self::SquareMetres(v) => {
                let c = v.controls();
                ViolationControls {
                    actual: c.actual.0,
                    limit: c.limit.0,
                    bound: c.bound,
                }
            }
            Self::Count(v) => {
                let c = v.controls();
                ViolationControls {
                    actual: c.actual.0 as f64,
                    limit: c.limit.0 as f64,
                    bound: c.bound,
                }
            }
            Self::Unmeasurable { bound, .. } => ViolationControls {
                actual: f64::INFINITY,
                limit: 0.0,
                bound,
            },
        }
    }
}

struct ViolationControls<T> {
    actual: T,
    limit: T,
    bound: SupportBound,
}

/// A nonnegative finite rise/run observation. f64 retains the geographic
/// triangle normal's precision rather than rounding to an engineering f32 bound.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(transparent)]
pub struct GeographicGrade(f64);
impl GeographicGrade {
    pub fn from_ratio(value: f64) -> Option<Self> {
        (value.is_finite() && value >= 0.0).then_some(Self(value))
    }
    pub fn ratio(self) -> f64 {
        self.0
    }
}

impl core::fmt::Display for DiagnosticArea {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.0.fmt(f)
    }
}
impl core::fmt::Display for DiagnosticCount {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.0.fmt(f)
    }
}
impl core::fmt::Display for GeographicGrade {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.0.fmt(f)
    }
}
