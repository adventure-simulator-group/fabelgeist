//! Substep participants retain their concrete failure type and interval role.
use crate::{Particles, SubstepDuration};
use fabelgeist_compute::kernel::KernelBatch;
use std::convert::Infallible;

mod chain;
pub use chain::{HookChain, HookChainError};

/// A positional participant between prediction and the material solve.
///
/// Solver entry points borrow this domain interface, including participants
/// already selected behind a trait object. Their error carrier stays concrete.
///
/// ```compile_fail
/// use fabelgeist_xpbd::SubstepHook;
/// let mut count = 7u32;
/// let _: &mut dyn SubstepHook<Error = std::convert::Infallible> = &mut count;
/// ```
pub trait SubstepHook {
    type Error: std::error::Error + 'static;

    fn record(
        &mut self,
        batch: &mut KernelBatch,
        particles: &Particles,
        substep: SubstepDuration,
    ) -> Result<(), Self::Error>;

    fn after_solve(
        &mut self,
        _batch: &mut KernelBatch,
        _particles: &Particles,
        _substep: SubstepDuration,
    ) -> Result<(), Self::Error> {
        Ok(())
    }
}

/// Explicitly run the material solve without an additional participant.
#[derive(Default)]
pub struct NoSubstepHook;
impl SubstepHook for NoSubstepHook {
    type Error = Infallible;
    fn record(
        &mut self,
        _batch: &mut KernelBatch,
        _particles: &Particles,
        _substep: SubstepDuration,
    ) -> Result<(), Infallible> {
        Ok(())
    }
}

impl<F, E> SubstepHook for F
where
    F: FnMut(&mut KernelBatch, &Particles, SubstepDuration) -> Result<(), E>,
    E: std::error::Error + 'static,
{
    type Error = E;
    fn record(
        &mut self,
        batch: &mut KernelBatch,
        particles: &Particles,
        substep: SubstepDuration,
    ) -> Result<(), E> {
        self(batch, particles, substep)
    }
}
