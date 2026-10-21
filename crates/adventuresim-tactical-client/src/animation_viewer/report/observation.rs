//! Report evidence keeps applicability, absence and checked quantities distinct.

use adventuresim_tactical_core::animation::{
    AnimationValueError, CyclePhase, HeightDistance, UnitInterval,
};
use fabelgeist_geometry::{GeometryError, SignedLength};
use serde::Serialize;

use super::BoneRole;

pub(in crate::animation_viewer) type ReportResult<T> = Result<T, ReportError>;

#[derive(Debug, Clone, Copy, PartialEq)]
pub(in crate::animation_viewer) struct QualityPercent(UnitInterval);

/// The finite native normalized-vector dot, including -1 and rounding above 1.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[serde(transparent)]
pub(in crate::animation_viewer) struct KneeHemisphereDot(f32);

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub(in crate::animation_viewer) struct SignedLengthRange {
    minimum: SignedLength,
    maximum: SignedLength,
}

#[derive(Debug, Clone, Copy)]
pub(in crate::animation_viewer) struct HeightSample {
    phase: CyclePhase,
    height: SignedLength,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub(in crate::animation_viewer) struct HeightPeaks {
    pub(super) count: usize,
    pub(super) in_passing_windows: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub(in crate::animation_viewer) struct VisualHeightPeaks {
    maximum_count: usize,
    each_cycle_has_two_passing_peaks: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[serde(tag = "status", content = "value", rename_all = "snake_case")]
pub(in crate::animation_viewer) enum Observation<T> {
    NotApplicable,
    Unavailable(UnavailableReason),
    Measured(T),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(in crate::animation_viewer) enum UnavailableReason {
    NoFrames,
    IncompleteCapture,
    NoAdjacentFrames,
    NoStoppedFrame,
    NoCompletedCycle,
    InsufficientHeightSamples,
    MissingHeightNeighborhood,
    NoContactHeightSamples,
    NoPassingHeightSamples,
    NoContactSamples,
    NoCompletedHalfStep,
    NoMotionSamples,
    NoStableDiveAxis,
    DegenerateLegAxis,
    UndefinedKneeBend,
    MissingBodyResponse,
    MissingIkDiagnostics,
    MissingTerrainClearance {
        #[serde(serialize_with = "super::serialize_report_bone")]
        bone: BoneRole,
    },
    MissingBone {
        #[serde(serialize_with = "super::serialize_report_bone")]
        bone: BoneRole,
    },
}

#[derive(Debug, thiserror::Error)]
pub(in crate::animation_viewer) enum ReportError {
    #[error(transparent)]
    Animation(#[from] AnimationValueError),
    #[error(transparent)]
    Geometry(#[from] GeometryError),
    #[error("the knee hemisphere dot is not finite")]
    NonFiniteKneeHemisphere,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::animation_viewer) enum ContactSample {
    Contact,
    Flight,
}

impl VisualHeightPeaks {
    pub(super) fn from_cycles(cycles: &[HeightPeaks]) -> Option<Self> {
        let maximum_count = cycles.iter().map(|cycle| cycle.count).max()?;
        Some(Self {
            maximum_count,
            each_cycle_has_two_passing_peaks: cycles.iter().all(|cycle| cycle.count == 2 && cycle.in_passing_windows),
        })
    }
    pub(super) fn maximum_count(self) -> usize { self.maximum_count }
    pub(super) fn each_cycle_has_two_passing_peaks(self) -> bool { self.each_cycle_has_two_passing_peaks }
}

impl HeightSample {
    pub(super) fn new(phase: CyclePhase, height: SignedLength) -> Self {
        Self { phase: phase.wrapped(), height }
    }
    pub(super) fn phase(self) -> CyclePhase { self.phase }
    pub(super) fn height(self) -> SignedLength { self.height }
}

impl SignedLengthRange {
    pub(super) fn from_sample(sample: SignedLength) -> Self {
        Self { minimum: sample, maximum: sample }
    }
    pub(super) fn include(self, sample: SignedLength) -> Self {
        Self { minimum: self.minimum.min(sample), maximum: self.maximum.max(sample) }
    }
    pub(super) fn minimum(self) -> SignedLength { self.minimum }
    pub(super) fn maximum(self) -> SignedLength { self.maximum }
}

impl QualityPercent {
    pub(super) fn from_fraction(fraction: UnitInterval) -> Self {
        Self(fraction)
    }
    pub(super) fn percent(self) -> f32 {
        100.0 * self.0.fraction()
    }
}

impl Serialize for QualityPercent {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_f32(self.percent())
    }
}

impl KneeHemisphereDot {
    pub(super) fn from_native(value: f32) -> ReportResult<Self> {
        if !value.is_finite() {
            return Err(ReportError::NonFiniteKneeHemisphere);
        }
        Ok(Self(value))
    }
    pub(super) fn value(self) -> f32 {
        self.0
    }
    pub(super) fn min(self, other: Self) -> Self {
        Self(self.0.min(other.0))
    }
}

impl<T> Observation<T> {
    /// Required evidence passes only when a real measurement satisfies its gate.
    pub(super) fn measured(&self) -> Option<&T> {
        match self {
            Self::Measured(value) => Some(value),
            Self::NotApplicable | Self::Unavailable(_) => None,
        }
    }

    /// An optional scenario is exempt only through its explicit applicability.
    pub(super) fn when_applicable(&self, gate: impl FnOnce(&T) -> bool) -> bool {
        match self {
            Self::NotApplicable => true,
            Self::Unavailable(_) => false,
            Self::Measured(value) => gate(value),
        }
    }
}

impl UnavailableReason {
    pub(super) fn description(self) -> &'static str {
        match self {
            Self::NoFrames => "no captured frames",
            Self::IncompleteCapture => "capture scenarios or artifacts are incomplete",
            Self::NoAdjacentFrames => "no adjacent captured frames",
            Self::NoStoppedFrame => "no stopped pose was captured",
            Self::NoCompletedCycle => "no complete post-warmup cycle",
            Self::InsufficientHeightSamples => "fewer than three height samples",
            Self::MissingHeightNeighborhood => "a peak has no neighboring height samples",
            Self::NoContactHeightSamples => "no contact-window height samples",
            Self::NoPassingHeightSamples => "no passing-window height samples",
            Self::NoContactSamples => "no contact samples",
            Self::NoCompletedHalfStep => "no completed half-step",
            Self::NoMotionSamples => "no moving pose was captured",
            Self::NoStableDiveAxis => "no stable horizontal dive axis",
            Self::DegenerateLegAxis => "the hip-to-foot axis is undefined",
            Self::UndefinedKneeBend => "the knee bend direction is undefined",
            Self::MissingBodyResponse => "body response was not observed",
            Self::MissingIkDiagnostics => "procedural leg diagnostics were not observed",
            Self::MissingTerrainClearance { .. } => "terrain clearance was not observed",
            Self::MissingBone { .. } => "a required tracked bone is missing",
        }
    }
}
