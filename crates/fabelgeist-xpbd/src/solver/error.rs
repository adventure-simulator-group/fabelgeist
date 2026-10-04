//! Driver failures retain shader, constraint, hook phase and substep identity.
use super::SolverKernel;
use crate::{ConstraintDispatchError, SubstepIndex};
use fabelgeist_compute::kernel::{KernelCacheError, KernelDispatchError};

#[derive(Debug)]
pub struct SolverBuildError {
    pub kernel: SolverKernel,
    pub source: Box<KernelCacheError>,
}
impl std::fmt::Display for SolverBuildError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.source.fmt(formatter)
    }
}
impl std::error::Error for SolverBuildError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(self.source.as_ref())
    }
}

#[derive(Debug)]
pub struct SolverDispatchError {
    pub kernel: SolverKernel,
    pub source: Box<KernelDispatchError>,
}
impl std::fmt::Display for SolverDispatchError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.source.fmt(formatter)
    }
}
impl std::error::Error for SolverDispatchError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(self.source.as_ref())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SolverHookPhase {
    BeforeSolve,
    AfterSolve,
}
/// Hook causes retain a concrete error carrier.
///
/// ```compile_fail
/// use fabelgeist_xpbd::SolverSubstepError;
/// let _: Option<SolverSubstepError<String>> = None;
/// ```
#[derive(Debug)]
pub enum SolverSubstepError<E: std::error::Error> {
    Dispatch(SolverDispatchError),
    Constraint(ConstraintDispatchError),
    Hook { phase: SolverHookPhase, source: E },
}
impl<E: std::error::Error> SolverSubstepError<E> {
    pub(super) fn before_hook(source: E) -> Self {
        Self::Hook {
            phase: SolverHookPhase::BeforeSolve,
            source,
        }
    }
    pub(super) fn after_hook(source: E) -> Self {
        Self::Hook {
            phase: SolverHookPhase::AfterSolve,
            source,
        }
    }
}
impl<E: std::error::Error> std::fmt::Display for SolverSubstepError<E> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Dispatch(source) => source.fmt(formatter),
            Self::Constraint(source) => source.fmt(formatter),
            Self::Hook { source, .. } => std::fmt::Display::fmt(source, formatter),
        }
    }
}
impl<E: std::error::Error + 'static> std::error::Error for SolverSubstepError<E> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Dispatch(source) => Some(source),
            Self::Constraint(source) => Some(source),
            Self::Hook { source, .. } => Some(source),
        }
    }
}

#[derive(Debug)]
pub struct SolverStepError<E: std::error::Error> {
    pub substep: SubstepIndex,
    pub source: SolverSubstepError<E>,
}
impl<E: std::error::Error> SolverStepError<E> {
    pub(super) fn for_substep(
        substep: SubstepIndex,
        result: Result<(), SolverSubstepError<E>>,
    ) -> Result<(), Self> {
        match result {
            Ok(()) => Ok(()),
            Err(source) => Err(Self { substep, source }),
        }
    }
}
impl<E: std::error::Error> std::fmt::Display for SolverStepError<E> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.source.fmt(formatter)
    }
}
impl<E: std::error::Error + 'static> std::error::Error for SolverStepError<E> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.source)
    }
}
