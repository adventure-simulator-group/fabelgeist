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

use std::sync::Arc;

use fabelgeist_compute::prelude::*;
use fabelgeist_gpu::prelude::*;
use fabelgeist_math::Vec3;

use crate::constraint::ConstraintSet;
use crate::particles::{ParticleCount, Particles};
use crate::wgsl;

/// How the substep loop is driven.
#[derive(Clone, Copy, Debug)]
pub struct SolverSettings {
    /// Substeps per call to [`Solver::step`]. More is stiffer and steadier;
    /// the cost is linear.
    pub substeps: u32,
    /// Constraint sweeps within each substep. XPBD wants one; more helps a
    /// badly conditioned set converge, at the price of the compliance meaning
    /// slightly less than it says.
    pub iterations: u32,
    pub gravity: Vec3,
    /// Exponential velocity drag, per second. Independent of the substep
    /// count, so changing `substeps` does not change how draggy the cloth is.
    pub damping: f32,
    /// Ceiling on particle speed. It only ever binds when something has
    /// already gone wrong, and it is what turns a blown-up frame into a
    /// recoverable one rather than a garment flung off the screen.
    pub max_speed: f32,
}

impl Default for SolverSettings {
    fn default() -> Self {
        Self {
            substeps: 10,
            iterations: 1,
            // Metres per second squared, and the rest of the stack is in
            // metres, so a garment in centimetres has to be scaled on the way
            // in.
            gravity: Vec3::new(0.0, -9.81, 0.0),
            damping: 0.1,
            max_speed: 20.0,
        }
    }
}

/// Anything that wants to run between the prediction and the constraint solve
/// of every substep -- which is where collision response belongs.
///
/// It sees the substep length because a positional correction has to know it
/// to turn into the right velocity change, and friction depends on it.
pub trait SubstepHook {
    fn record(
        &mut self,
        batch: &mut KernelBatch,
        particles: &Particles,
        substep: f32,
    ) -> Result<()>;

    fn after_solve(
        &mut self,
        _batch: &mut KernelBatch,
        _particles: &Particles,
        _substep: f32,
    ) -> Result<()> {
        Ok(())
    }
}

impl SubstepHook for () {
    fn record(
        &mut self,
        _batch: &mut KernelBatch,
        _particles: &Particles,
        _substep: f32,
    ) -> Result<()> {
        Ok(())
    }
}

impl<F> SubstepHook for F
where
    F: FnMut(&mut KernelBatch, &Particles, f32) -> Result<()>,
{
    fn record(
        &mut self,
        batch: &mut KernelBatch,
        particles: &Particles,
        substep: f32,
    ) -> Result<()> {
        self(batch, particles, substep)
    }
}

/// The predict/finalize pair, and the loop that drives them.
pub struct Solver {
    predict: Arc<Kernel>,
    finalize: Arc<Kernel>,
    pub settings: SolverSettings,
}

impl Solver {
    pub fn new(context: &WgpuContext, settings: SolverSettings) -> Result<Self> {
        Self::with_cache(context, &KernelCache::new(), settings)
    }

    pub fn with_cache(
        context: &WgpuContext,
        cache: &KernelCache,
        settings: SolverSettings,
    ) -> Result<Self> {
        Ok(Self {
            predict: cache.get(context, wgsl::PREDICT)?,
            finalize: cache.get(context, wgsl::FINALIZE)?,
            settings,
        })
    }

    /// Record one full step: `settings.substeps` substeps of `delta / substeps`.
    pub fn record_step(
        &self,
        batch: &mut KernelBatch,
        particles: &Particles,
        constraints: &mut [&mut ConstraintSet],
        hook: &mut impl SubstepHook,
        delta: f32,
    ) -> Result<()> {
        let count = particles.count();
        if count == ParticleCount::EMPTY || delta <= 0.0 || self.settings.substeps == 0 {
            return Ok(());
        }
        let substep = delta / self.settings.substeps as f32;

        for _ in 0..self.settings.substeps {
            self.record_substep(batch, particles, constraints, hook, substep)?;
        }
        Ok(())
    }

    /// One substep, into the given batch.
    pub fn record_substep(
        &self,
        batch: &mut KernelBatch,
        particles: &Particles,
        constraints: &mut [&mut ConstraintSet],
        hook: &mut impl SubstepHook,
        substep: f32,
    ) -> Result<()> {
        self.record_predict(batch, particles, substep)?;

        // Collisions are positional constraints too, and they belong before
        // the material ones: the cloth should be told where it may not be
        // before it is asked to hold its shape there.
        hook.record(batch, particles, substep)?;

        for set in constraints.iter_mut() {
            set.record_clear(batch)?;
        }
        for _ in 0..self.settings.iterations.max(1) {
            for set in constraints.iter_mut() {
                set.record_solve(batch, particles, substep)?;
            }
        }

        hook.after_solve(batch, particles, substep)?;
        self.record_finalize(batch, particles, substep)
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
    pub fn step_interleaved(
        &self,
        context: &WgpuContext,
        particles: &Particles,
        constraints: &mut [&mut ConstraintSet],
        hook: &mut impl SubstepHook,
        delta: f32,
    ) -> Result<()> {
        let count = particles.count();
        if count == ParticleCount::EMPTY || delta <= 0.0 || self.settings.substeps == 0 {
            return Ok(());
        }
        let substep = delta / self.settings.substeps as f32;

        for _ in 0..self.settings.substeps {
            let mut batch = KernelBatch::labelled(context, "xpbd substep");
            self.record_substep(&mut batch, particles, constraints, hook, substep)?;
            batch.submit();
        }
        Ok(())
    }

    /// Step on a batch of its own and submit.
    pub fn step(
        &self,
        context: &WgpuContext,
        particles: &Particles,
        constraints: &mut [&mut ConstraintSet],
        hook: &mut impl SubstepHook,
        delta: f32,
    ) -> Result<()> {
        let mut batch = KernelBatch::labelled(context, "xpbd step");
        self.record_step(&mut batch, particles, constraints, hook, delta)?;
        batch.submit();
        Ok(())
    }

    fn record_predict(
        &self,
        batch: &mut KernelBatch,
        particles: &Particles,
        substep: f32,
    ) -> Result<()> {
        let mut parameters = PassParameters::new();
        parameters.insert("positions", particles.positions.clone());
        parameters.insert("previous", particles.previous.clone());
        parameters.insert("velocities", particles.velocities.clone());
        parameters.insert(
            "gravity",
            fabelgeist_math::Vec4::new(
                self.settings.gravity.x,
                self.settings.gravity.y,
                self.settings.gravity.z,
                0.0,
            ),
        );
        parameters.insert("substep", substep);
        parameters.insert("damping", self.settings.damping);
        parameters.insert("count", particles.count());
        parameters.insert("max_speed", self.settings.max_speed);
        batch.dispatch_items(&self.predict, &parameters, u32::from(particles.count()))?;
        Ok(())
    }

    fn record_finalize(
        &self,
        batch: &mut KernelBatch,
        particles: &Particles,
        substep: f32,
    ) -> Result<()> {
        let mut parameters = PassParameters::new();
        parameters.insert("positions", particles.positions.clone());
        parameters.insert("previous", particles.previous.clone());
        parameters.insert("velocities", particles.velocities.clone());
        parameters.insert("substep", substep);
        parameters.insert("count", particles.count());
        parameters.insert("pad0", 0u32);
        parameters.insert("pad1", 0u32);
        batch.dispatch_items(&self.finalize, &parameters, u32::from(particles.count()))?;
        Ok(())
    }
}
