use super::BufferReadError;
use crate::globals::WgpuContext;
use futures_channel::oneshot;
use std::sync::Arc;

/// Owns this operation's mapping request, including callback failure/cancel.
pub(super) struct MappingRequest {
    target: Arc<wgpu::Buffer>,
    #[cfg(not(target_arch = "wasm32"))]
    device: Arc<wgpu::Device>,
    completion: oneshot::Receiver<Result<(), wgpu::BufferAsyncError>>,
    release: ReleaseObligation,
}

enum ReleaseObligation {
    Required,
    Complete,
}

impl MappingRequest {
    pub(super) fn request(
        context: &WgpuContext,
        target: Arc<wgpu::Buffer>,
    ) -> Result<Self, BufferReadError> {
        let (sender, receiver) = oneshot::channel();
        let request = context
            .observe_validation(|| {
                target
                    .slice(..)
                    .map_async(wgpu::MapMode::Read, move |result| {
                        let _ = sender.send(result);
                    });
                // A callback may already have run. Own cleanup before the
                // native scope is inspected or any result can return early.
                Self {
                    target,
                    #[cfg(not(target_arch = "wasm32"))]
                    device: context.device.clone(),
                    completion: receiver,
                    release: ReleaseObligation::Required,
                }
            })
            .map_err(BufferReadError::Native)?;
        Ok(request)
    }

    pub(super) fn buffer(&self) -> &wgpu::Buffer {
        &self.target
    }

    pub(super) async fn complete(&mut self, context: &WgpuContext) -> Result<(), BufferReadError> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            context
                .device
                .poll(wgpu::PollType::wait_indefinitely())
                .map_err(BufferReadError::Poll)?;
            loop {
                match self
                    .completion
                    .try_recv()
                    .map_err(BufferReadError::Channel)?
                {
                    Some(result) => return result.map_err(BufferReadError::Mapping),
                    None => {
                        // Drive progress until delivery. Callback absence
                        // does not identify the kind of source buffer.
                        context
                            .device
                            .poll(wgpu::PollType::Poll)
                            .map_err(BufferReadError::Poll)?;
                        std::thread::yield_now();
                    }
                }
            }
        }
        #[cfg(target_arch = "wasm32")]
        {
            let _ = context;
            (&mut self.completion)
                .await
                .map_err(BufferReadError::Channel)?
                .map_err(BufferReadError::Mapping)
        }
    }

    /// Consume the request after its mapped view has been dropped.
    pub(super) fn release(mut self, context: &WgpuContext) -> Result<(), BufferReadError> {
        self.release = ReleaseObligation::Complete;
        context
            .observe_validation(|| self.target.unmap())
            .map_err(BufferReadError::Native)
    }
}

impl Drop for MappingRequest {
    fn drop(&mut self) {
        if matches!(self.release, ReleaseObligation::Required) {
            // A failed map can leave the SDK wrapper marked requested while
            // native state is Idle. Reset it without replacing the primary
            // failure with the resulting NotMapped validation error. This
            // cleanup-only scope also applies to borrowed native contexts;
            // dropping it pops immediately, without polling or waiting.
            #[cfg(not(target_arch = "wasm32"))]
            let _cleanup = self.device.push_error_scope(wgpu::ErrorFilter::Validation);
            self.target.unmap();
        }
    }
}
