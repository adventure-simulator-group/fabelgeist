//! The substep loop.
//!
//! Extended position-based dynamics, in the small-substeps form Macklin et al.
//! settled on: rather than iterating a constraint solve many times inside one
//! large step, take many small substeps and solve once in each. It converges
//! better for the same total work, and it is what makes a stiff fabric behave
//! at a frame rate a garment fit can be watched at.
//!
//! Per substep:
//!
//! 1. **predict** -- integrate gravity and damping, remember where each
//!    particle was, move it where its velocity wants to go.
//! 2. *(the caller's hook)* -- collisions go here, as positional constraints
//!    like any other. See `fabelgeist-physics`.
//! 3. **solve** -- one Gauss-Seidel sweep over the constraint colours.
//! 4. **finalize** -- read velocity back out of the total position change, so
//!    that every correction made in 2 and 3 shows up in the velocity for free.

use crate::{
    ConstraintSet, ConstraintSweepCount, DampingRate, GravityAcceleration, ParticleSpeedLimit,
    Particles, StepActivity, StepDuration, SubstepCount, SubstepDuration,
};
use fabelgeist_compute::kernel::{Kernel, KernelBatch, KernelCache};
use fabelgeist_gpu::prelude::{PassParameters, WgpuContext};
use std::sync::Arc;
mod error;
#[cfg(test)]
mod fixture;
mod hook;
mod kernel;
#[cfg(test)]
mod tests;
pub use error::{
    SolverBuildError, SolverDispatchError, SolverHookPhase, SolverStepError, SolverSubstepError,
};
pub use hook::{HookChain, HookChainError, NoSubstepHook, SubstepHook};
pub use kernel::SolverKernel;

/// How the substep loop is driven.
#[derive(Clone, Copy, Debug)]
pub struct SolverSettings {
    /// Substeps per call to [`Solver::step`]. More is stiffer and steadier;
    /// the cost is linear.
    pub substeps: SubstepCount,
    /// Constraint sweeps within each substep. XPBD wants one; more helps a
    /// badly conditioned set converge, at the price of the compliance meaning
    /// slightly less than it says.
    pub iterations: ConstraintSweepCount,
    pub gravity: GravityAcceleration,
    /// Exponential velocity drag, per second. Independent of the substep
    /// count, so changing `substeps` does not change how draggy the cloth is.
    pub damping: DampingRate,
    /// Ceiling on particle speed. It only ever binds when something has
    /// already gone wrong, and it is what turns a blown-up frame into a
    /// recoverable one rather than a garment flung off the screen.
    pub max_speed: ParticleSpeedLimit,
}

impl Default for SolverSettings {
    fn default() -> Self {
        Self {
            substeps: SubstepCount::DEFAULT,
            iterations: ConstraintSweepCount::ONE,
            // Metres per second squared, and the rest of the stack is in
            // metres, so a garment in centimetres has to be scaled on the way
            // in.
            gravity: GravityAcceleration::earth(),
            damping: DampingRate::DEFAULT,
            max_speed: ParticleSpeedLimit::DEFAULT,
        }
    }
}

/// The predict/finalize pair, and the loop that drives them.
pub struct Solver {
    predict: Arc<Kernel>,
    finalize: Arc<Kernel>,
    pub settings: SolverSettings,
}

impl Solver {
    pub fn new(context: &WgpuContext, settings: SolverSettings) -> Result<Self, SolverBuildError> {
        Self::with_cache(context, &KernelCache::new(), settings)
    }

    pub fn with_cache(
        context: &WgpuContext,
        cache: &KernelCache,
        settings: SolverSettings,
    ) -> Result<Self, SolverBuildError> {
        Ok(Self {
            predict: SolverKernel::Predict.load(context, cache)?,
            finalize: SolverKernel::Finalize.load(context, cache)?,
            settings,
        })
    }

    /// Record one full step: `settings.substeps` substeps of `delta / substeps`.
    pub fn record_step<E: std::error::Error + 'static>(
        &self,
        batch: &mut KernelBatch,
        particles: &Particles,
        constraints: &mut [&mut ConstraintSet],
        hook: &mut dyn SubstepHook<Error = E>,
        delta: StepDuration,
    ) -> Result<(), SolverStepError<E>> {
        let count = particles.count();
        if count == crate::ParticleCount::EMPTY
            || delta.activity() == StepActivity::Inactive
            || self.settings.substeps == SubstepCount::EMPTY
        {
            return Ok(());
        }
        let substep = delta.for_substeps(self.settings.substeps);

        for index in self.settings.substeps.sequence() {
            SolverStepError::for_substep(
                index,
                self.record_substep(batch, particles, constraints, hook, substep),
            )?;
        }
        Ok(())
    }

    /// One substep, into the given batch.
    pub fn record_substep<E: std::error::Error + 'static>(
        &self,
        batch: &mut KernelBatch,
        particles: &Particles,
        constraints: &mut [&mut ConstraintSet],
        hook: &mut dyn SubstepHook<Error = E>,
        substep: SubstepDuration,
    ) -> Result<(), SolverSubstepError<E>> {
        self.record_predict(batch, particles, substep)
            .map_err(SolverSubstepError::Dispatch)?;

        // Collisions are positional constraints too, and they belong before
        // the material ones: the cloth should be told where it may not be
        // before it is asked to hold its shape there.
        hook.record(batch, particles, substep)
            .map_err(SolverSubstepError::before_hook)?;

        for set in constraints.iter_mut() {
            set.record_clear(batch)
                .map_err(SolverSubstepError::Constraint)?;
        }
        for _ in self.settings.iterations.sequence() {
            for set in constraints.iter_mut() {
                set.record_solve(batch, particles, substep)
                    .map_err(SolverSubstepError::Constraint)?;
            }
        }

        hook.after_solve(batch, particles, substep)
            .map_err(SolverSubstepError::after_hook)?;
        self.record_finalize(batch, particles, substep)
            .map_err(SolverSubstepError::Dispatch)
    }

    /// Step with one submission **per substep** rather than one for the whole
    /// step.
    ///
    /// The total work is identical; what changes is how it reaches the device.
    /// A whole step is several hundred dispatches, and submitted together they
    /// occupy the GPU for as long as they take -- tens of milliseconds for a
    /// garment. Anything else sharing that device gets nothing in the
    /// meantime, and a compositor that cannot acquire a swapchain image in
    /// time drops the frame or gives up on the window.
    ///
    /// Submitting each substep separately gives the scheduler a dozen places
    /// to fit other work. Use this whenever the solver shares a device with a
    /// renderer; `step` is for a device the solver has to itself.
    pub fn step_interleaved<E: std::error::Error + 'static>(
        &self,
        context: &WgpuContext,
        particles: &Particles,
        constraints: &mut [&mut ConstraintSet],
        hook: &mut dyn SubstepHook<Error = E>,
        delta: StepDuration,
    ) -> Result<(), SolverStepError<E>> {
        let count = particles.count();
        if count == crate::ParticleCount::EMPTY
            || delta.activity() == StepActivity::Inactive
            || self.settings.substeps == SubstepCount::EMPTY
        {
            return Ok(());
        }
        let substep = delta.for_substeps(self.settings.substeps);

        for index in self.settings.substeps.sequence() {
            let mut batch = KernelBatch::labelled(context, ("xpbd substep").into());
            SolverStepError::for_substep(
                index,
                self.record_substep(&mut batch, particles, constraints, hook, substep),
            )?;
            batch.submit();
        }
        Ok(())
    }

    /// Step on a batch of its own and submit.
    pub fn step<E: std::error::Error + 'static>(
        &self,
        context: &WgpuContext,
        particles: &Particles,
        constraints: &mut [&mut ConstraintSet],
        hook: &mut dyn SubstepHook<Error = E>,
        delta: StepDuration,
    ) -> Result<(), SolverStepError<E>> {
        let mut batch = KernelBatch::labelled(context, ("xpbd step").into());
        self.record_step(&mut batch, particles, constraints, hook, delta)?;
        batch.submit();
        Ok(())
    }

    fn record_predict(
        &self,
        batch: &mut KernelBatch,
        particles: &Particles,
        substep: SubstepDuration,
    ) -> Result<(), SolverDispatchError> {
        let mut parameters = PassParameters::new();
        parameters.insert("positions".into(), (particles.positions.clone()).into());
        parameters.insert("previous".into(), (particles.previous.clone()).into());
        parameters.insert("velocities".into(), (particles.velocities.clone()).into());
        self.settings.gravity.bind(&mut parameters);
        substep.bind(&mut parameters);
        self.settings.damping.bind(&mut parameters);
        particles.count().bind(&mut parameters);
        self.settings.max_speed.bind(&mut parameters);
        SolverKernel::Predict.dispatch(
            batch,
            &self.predict,
            &parameters,
            particles.count().invocations(),
        )?;
        Ok(())
    }

    fn record_finalize(
        &self,
        batch: &mut KernelBatch,
        particles: &Particles,
        substep: SubstepDuration,
    ) -> Result<(), SolverDispatchError> {
        let mut parameters = PassParameters::new();
        parameters.insert("positions".into(), (particles.positions.clone()).into());
        parameters.insert("previous".into(), (particles.previous.clone()).into());
        parameters.insert("velocities".into(), (particles.velocities.clone()).into());
        substep.bind(&mut parameters);
        particles.count().bind(&mut parameters);
        parameters.insert("pad0".into(), (0u32).into());
        parameters.insert("pad1".into(), (0u32).into());
        SolverKernel::Finalize.dispatch(
            batch,
            &self.finalize,
            &parameters,
            particles.count().invocations(),
        )?;
        Ok(())
    }
}
