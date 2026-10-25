use super::WgpuContext;

impl WgpuContext {
    /// Finish and submit an encoder created by this context's device.
    ///
    /// The encoder must be created by this context's device, and the queue
    /// must belong to that same logical `wgpu::Device`. Sharing a backend or
    /// adapter is insufficient.
    /// With native synchronous validation enabled, capture Validation errors
    /// from finalization before submitting, then capture submission errors
    /// separately. Preserve the SDK cause without waiting for GPU completion.
    /// Other error classes follow the device's configured error handling.
    ///
    /// Borrowed contexts and browser builds use device error handling instead
    /// of observing these scopes. On those paths, `Ok` provides no synchronous
    /// validation verdict. No path promises GPU completion or later device
    /// health; use a separate completion operation when results are needed.
    pub fn submit(
        &self,
        encoder: wgpu::CommandEncoder,
    ) -> std::result::Result<wgpu::SubmissionIndex, wgpu::Error> {
        #[cfg(not(target_arch = "wasm32"))]
        let scope = self
            .blocking_validation
            .then(|| self.device.push_error_scope(wgpu::ErrorFilter::Validation));
        let commands = encoder.finish();
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(cause) = scope.and_then(|scope| pollster::block_on(scope.pop())) {
            return Err(cause);
        }

        #[cfg(not(target_arch = "wasm32"))]
        let scope = self
            .blocking_validation
            .then(|| self.device.push_error_scope(wgpu::ErrorFilter::Validation));
        let index = self.queue.submit([commands]);
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(cause) = scope.and_then(|scope| pollster::block_on(scope.pop())) {
            return Err(cause);
        }
        Ok(index)
    }
}
