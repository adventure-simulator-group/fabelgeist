//! Fresh render acknowledgements at both ends of a frozen sampling window.

use std::sync::Mutex;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "viewer", derive(clap::ValueEnum))]
#[cfg_attr(feature = "prepared", derive(serde::Serialize))]
#[cfg_attr(feature = "prepared", serde(rename_all = "snake_case"))]
pub enum ValidationPolicy {
    #[default]
    EveryFrame,
    SamplingEndpoints,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "prepared", derive(serde::Serialize))]
#[cfg_attr(feature = "prepared", serde(rename_all = "snake_case"))]
pub enum ValidationPhase {
    #[default]
    Warmup,
    StartRequested,
    StartValidated,
    Sampling,
    FinalRequested,
    Complete,
}

pub enum WindowStart {
    Waiting,
    Opened,
    Active,
}

/// Diagnostic counts exclude warmup; this does not prove intermediate coverage.
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "prepared", derive(serde::Serialize))]
pub struct ValidationEvidence {
    pub policy: ValidationPolicy,
    pub phase: ValidationPhase,
    pub successful_full_checks: usize,
    pub skipped_structural_checks: usize,
    pub sampled_render_checks: usize,
}

pub struct ValidationWindow {
    state: Mutex<ValidationEvidence>,
}

/// Latched before drawing. Cleanup cannot acknowledge a newer main-world request.
#[derive(Clone, Copy)]
pub(super) struct RenderCheck {
    phase: ValidationPhase,
    pub full: bool,
}

impl ValidationWindow {
    pub(super) fn new(policy: ValidationPolicy) -> Self {
        Self {
            state: Mutex::new(ValidationEvidence {
                policy,
                phase: ValidationPhase::Warmup,
                successful_full_checks: 0,
                skipped_structural_checks: 0,
                sampled_render_checks: 0,
            }),
        }
    }

    pub fn begin_sampling(&self) -> WindowStart {
        let Ok(mut state) = self.state.lock() else {
            return WindowStart::Waiting;
        };
        match state.phase {
            ValidationPhase::Warmup => {
                state.phase = ValidationPhase::StartRequested;
                WindowStart::Waiting
            }
            ValidationPhase::StartValidated => {
                state.phase = ValidationPhase::Sampling;
                WindowStart::Opened
            }
            ValidationPhase::Sampling if state.sampled_render_checks == 0 => WindowStart::Waiting,
            ValidationPhase::Sampling
            | ValidationPhase::FinalRequested
            | ValidationPhase::Complete => WindowStart::Active,
            ValidationPhase::StartRequested => WindowStart::Waiting,
        }
    }

    pub fn finish_sampling(&self) -> bool {
        let Ok(mut state) = self.state.lock() else {
            return false;
        };
        if state.phase == ValidationPhase::Sampling && state.sampled_render_checks > 0 {
            state.phase = ValidationPhase::FinalRequested;
        }
        state.phase == ValidationPhase::Complete
    }

    pub fn evidence(&self) -> Option<ValidationEvidence> {
        self.state.lock().ok().map(|state| *state)
    }

    /// Serialize producer bookkeeping with the transition that freezes inputs.
    pub(super) fn with_mutation(&self, mutation: impl FnOnce(bool)) {
        match self.state.lock() {
            Ok(state) => mutation(state.phase != ValidationPhase::Warmup),
            Err(_) => mutation(true),
        }
    }

    pub(super) fn skip_structure(&self) -> bool {
        self.evidence().is_some_and(|state| {
            state.policy == ValidationPolicy::SamplingEndpoints
                && state.phase == ValidationPhase::Sampling
        })
    }

    pub(super) fn render_check(&self) -> Option<RenderCheck> {
        self.evidence().map(|state| RenderCheck {
            phase: state.phase,
            full: state.policy == ValidationPolicy::EveryFrame
                || state.phase != ValidationPhase::Sampling,
        })
    }

    pub(super) fn acknowledge(&self, check: RenderCheck, valid: bool) -> bool {
        let Ok(mut state) = self.state.lock() else {
            return false;
        };
        if state.phase != check.phase {
            return false;
        }
        if check.phase == ValidationPhase::Sampling {
            state.sampled_render_checks += 1;
        }
        if !check.full {
            state.skipped_structural_checks += 1;
            return valid;
        }
        if !valid {
            return false;
        }
        if matches!(
            check.phase,
            ValidationPhase::StartRequested
                | ValidationPhase::Sampling
                | ValidationPhase::FinalRequested
        ) {
            state.successful_full_checks += 1;
        }
        state.phase = match state.phase {
            ValidationPhase::StartRequested => ValidationPhase::StartValidated,
            ValidationPhase::FinalRequested => ValidationPhase::Complete,
            phase => phase,
        };
        true
    }
}

impl RenderCheck {
    pub(super) fn phase(self) -> ValidationPhase {
        self.phase
    }
}
