//! Interactive fit stepping and queue backpressure.
use super::Fit;
use fabelgeist_shell::ShellStepError;
use fabelgeist_xpbd::StepDuration;

impl Fit {
    /// Step the fit.
    ///
    /// Interleaved: one submission per substep. The solver is running on the
    /// application's own device, alongside the compositor presenting the
    /// window it is drawn in, and a whole step submitted at once occupies the
    /// GPU long enough that the compositor cannot get a swapchain image.
    ///
    /// Asynchronous because the host-side contact projection between substeps
    /// reads positions back from the GPU.
    pub async fn step(&mut self, delta: StepDuration) -> Result<(), ShellStepError> {
        self.cloth
            .step_interleaved(&self.context, &self.solver, &mut self.collisions, delta)
            .await?;
        self.frames += 1;
        Ok(())
    }

    /// Step, then wait for the GPU to finish it.
    ///
    /// What an interactive loop must call. A step is tens of milliseconds of
    /// GPU work and nothing throttles submission, so a loop that steps on a
    /// timer submits faster than the device drains and the queue grows without
    /// bound -- for a minute or so, until the driver loses the device and
    /// takes the window with it. Waiting also hands the compositor sharing
    /// this device a clear gap between steps.
    #[cfg_attr(not(test), allow(dead_code))]
    pub async fn step_and_wait(&mut self, delta: StepDuration) -> Result<(), ShellStepError> {
        self.step(delta).await?;
        self.context.submitted_work_done().await;
        Ok(())
    }
}
