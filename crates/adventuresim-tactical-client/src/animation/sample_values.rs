//! Values carried by resolved clip plans before the numerical sampler boundary.

/// Existing coalescing resolution for samples from the same clip/layer.
const SAMPLE_COHERENCE_SECONDS: f32 = 0.0001;

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize)]
#[serde(transparent)]
pub(super) struct ClipDuration(f32);

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize)]
#[serde(transparent)]
pub(super) struct ClipTime(f32);

/// Cycles, deliberately unwrapped until the consuming periodic operation.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize)]
#[serde(transparent)]
pub(super) struct CyclePhase(f32);

/// A blend contribution, distinct from sample time or cycle phase. Admission
/// retains the source factor; the sampler still rejects non-finite factors.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize)]
#[serde(transparent)]
pub(super) struct BlendWeight(f32);

impl ClipDuration {
    pub(super) fn from_seconds(seconds: f32) -> Self {
        Self(seconds)
    }
    pub(super) fn seconds(self) -> f32 {
        self.0
    }
    pub(super) fn at_phase(self, phase: CyclePhase) -> ClipTime {
        ClipTime::from_seconds(self.seconds() * phase.cycles())
    }
}

impl ClipTime {
    pub(super) fn from_seconds(seconds: f32) -> Self {
        Self(seconds)
    }
    pub(super) fn seconds(self) -> f32 {
        self.0
    }
    pub(super) fn same_sample(self, other: Self) -> bool {
        (self.0 - other.0).abs() < SAMPLE_COHERENCE_SECONDS
    }
}

impl CyclePhase {
    pub(super) fn from_cycles(cycles: f32) -> Self {
        Self(cycles)
    }
    pub(super) fn cycles(self) -> f32 {
        self.0
    }
    pub(super) fn wrapped(self) -> f32 {
        self.0.rem_euclid(1.0)
    }
}

impl BlendWeight {
    pub(super) const ZERO: Self = Self(0.0);
    pub(super) const ONE: Self = Self(1.0);
    pub(super) fn from_factor(factor: f32) -> Self {
        Self(factor)
    }
    pub(super) fn factor(self) -> f32 {
        self.0
    }
    /// There is no authored minimum contribution: every positive factor counts.
    pub(super) fn is_absent(self) -> bool {
        self.0 <= 0.0
    }
}

impl std::ops::AddAssign for BlendWeight {
    fn add_assign(&mut self, other: Self) {
        self.0 += other.0;
    }
}
