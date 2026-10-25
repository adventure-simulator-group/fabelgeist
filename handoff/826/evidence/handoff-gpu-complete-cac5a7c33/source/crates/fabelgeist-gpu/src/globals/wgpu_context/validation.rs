use super::WgpuContext;

impl WgpuContext {
    /// Observe immediate Validation errors under this context's native policy.
    ///
    /// The closure and its scope stay on the current thread. Reading the
    /// native scope does not poll the device or wait for submitted work.
    /// Browser builds and native contexts with `blocking_validation` disabled
    /// retain their device's configured error handling.
    pub(crate) fn observe_validation<T>(
        &self,
        operation: impl FnOnce() -> T,
    ) -> Result<T, wgpu::Error> {
        #[cfg(not(target_arch = "wasm32"))]
        let scope = self
            .blocking_validation
            .then(|| self.device.push_error_scope(wgpu::ErrorFilter::Validation));
        let value = operation();
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(cause) = scope.and_then(|scope| pollster::block_on(scope.pop())) {
            return Err(cause);
        }
        Ok(value)
    }
}
