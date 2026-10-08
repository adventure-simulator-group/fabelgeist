//! Cardinality of solver substeps, including locally disabled schedules.
use std::fmt;

/// A number of solver substeps or substeps covered by a host projection.
///
/// Every `u32` is admitted. Zero, minimum-one and range policies belong to the
/// driver or settings consumer; construction never applies those policies.
///
/// Native scalar construction is explicit:
/// ```
/// use fabelgeist_xpbd::{SolverSettings, SubstepCount};
/// let settings = SolverSettings { substeps: SubstepCount::from(3), ..Default::default() };
/// assert_eq!(u32::from(settings.substeps), 3);
/// ```
/// Primitive counts cannot enter solver settings directly:
/// ```compile_fail
/// use fabelgeist_xpbd::SolverSettings;
/// let settings = SolverSettings { substeps: 3u32, ..Default::default() };
/// ```
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct SubstepCount(u32);

impl SubstepCount {
    /// Admit a native count, including zero and the full `u32` range.
    ///
    /// This const construction port also permits typed settings bounds.
    pub const fn from_native(value: u32) -> Self {
        Self(value)
    }

    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// Apply the minimum-one policy where a consumer explicitly requests it.
    pub const fn at_least_one(self) -> Self {
        if self.is_empty() { Self(1) } else { self }
    }
}

impl From<u32> for SubstepCount {
    fn from(value: u32) -> Self {
        Self::from_native(value)
    }
}

impl From<SubstepCount> for u32 {
    fn from(value: SubstepCount) -> Self {
        value.0
    }
}

// Settings diagnostics retain the original native scalar spelling.
impl fmt::Debug for SubstepCount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}
