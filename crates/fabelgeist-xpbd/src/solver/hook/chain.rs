//! Ordered hook composition retains both participants' concrete error types.
use super::SubstepHook;
use crate::{Particles, SubstepDuration};
use fabelgeist_compute::kernel::KernelBatch;

/// Ordered borrowed participants with independent concrete failure types.
pub struct HookChain<'a, A: std::error::Error + 'static, B: std::error::Error + 'static> {
    first: &'a mut dyn SubstepHook<Error = A>,
    second: &'a mut dyn SubstepHook<Error = B>,
}
impl<'a, A: std::error::Error + 'static, B: std::error::Error + 'static> HookChain<'a, A, B> {
    pub fn new(
        first: &'a mut dyn SubstepHook<Error = A>,
        second: &'a mut dyn SubstepHook<Error = B>,
    ) -> Self {
        Self { first, second }
    }
}
impl<A: std::error::Error + 'static, B: std::error::Error + 'static> SubstepHook
    for HookChain<'_, A, B>
{
    type Error = HookChainError<A, B>;
    fn record(
        &mut self,
        batch: &mut KernelBatch,
        particles: &Particles,
        substep: SubstepDuration,
    ) -> Result<(), Self::Error> {
        self.first
            .record(batch, particles, substep)
            .map_err(HookChainError::First)?;
        self.second
            .record(batch, particles, substep)
            .map_err(HookChainError::Second)
    }
}

/// Both participants retain concrete error carriers.
///
/// ```compile_fail
/// use fabelgeist_xpbd::HookChainError;
/// let _: Option<HookChainError<u32, std::convert::Infallible>> = None;
/// ```
#[derive(Debug)]
pub enum HookChainError<A: std::error::Error, B: std::error::Error> {
    First(A),
    Second(B),
}
impl<A: std::error::Error, B: std::error::Error> std::fmt::Display for HookChainError<A, B> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::First(source) => std::fmt::Display::fmt(source, formatter),
            Self::Second(source) => std::fmt::Display::fmt(source, formatter),
        }
    }
}
impl<A: std::error::Error + 'static, B: std::error::Error + 'static> std::error::Error
    for HookChainError<A, B>
{
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::First(source) => Some(source),
            Self::Second(source) => Some(source),
        }
    }
}
