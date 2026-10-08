//! Per-second exponential particle velocity drag.
use std::fmt;

/// Exponential velocity drag per second, independent of the substep count.
///
/// Native admission preserves every f32 word. Negative rates amplify velocity;
/// material and editor consumers own their narrower local validity policies.
/// Projection to f32 belongs only at numerical, format and UI boundaries.
///
/// Composite solver settings require explicit rate admission:
///
/// ```compile_fail
/// use fabelgeist_xpbd::SolverSettings;
/// let settings = SolverSettings { damping: 0.6, ..Default::default() };
/// ```
///
/// A rate also cannot be silently erased to a numerical operand:
///
/// ```compile_fail
/// use fabelgeist_xpbd::dynamics::DampingRate;
/// let native: f32 = DampingRate::per_second(0.6);
/// ```
#[derive(Clone, Copy, PartialEq, PartialOrd)]
pub struct DampingRate(f32);

impl DampingRate {
    /// Admit a native per-second rate without rewriting or restricting its bits.
    pub const fn per_second(rate: f32) -> Self {
        Self(rate)
    }

    /// Whether the rate is finite, for a consumer's local admission policy.
    pub fn is_finite(self) -> bool {
        self.0.is_finite()
    }
}

impl From<f32> for DampingRate {
    fn from(rate: f32) -> Self {
        Self::per_second(rate)
    }
}

/// Native projection for numerical, serialization, presentation and UI ports.
impl From<DampingRate> for f32 {
    fn from(rate: DampingRate) -> Self {
        rate.0
    }
}

impl fmt::Debug for DampingRate {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.0, formatter)
    }
}
