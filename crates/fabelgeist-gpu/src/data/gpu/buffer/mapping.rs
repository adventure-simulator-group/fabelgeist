use super::{BufferByteLength, BufferReadError, ReadResult};
use crate::globals::WgpuContext;
#[cfg(target_arch = "wasm32")]
use crate::globals::await_browser_callback;
#[cfg(not(target_arch = "wasm32"))]
use crate::globals::poll_callback;
use futures_channel::oneshot;
use std::sync::Arc;

/// Owns this operation's mapping request, including callback failure/cancel.
pub(super) struct MappingRequest {
    target: Arc<wgpu::Buffer>,
    extent: MappingExtent,
    #[cfg(not(target_arch = "wasm32"))]
    device: Arc<wgpu::Device>,
    completion: oneshot::Receiver<std::result::Result<(), wgpu::BufferAsyncError>>,
    release: ReleaseObligation,
}

/// Nonzero, SDK-aligned range covering a logical prefix within its allocation.
#[derive(Clone, Copy)]
pub(super) struct MappingExtent(BufferByteLength);

enum ReleaseObligation {
    Required,
    Complete,
}

impl MappingExtent {
    pub(super) fn for_prefix(
        logical: BufferByteLength,
        allocated: BufferByteLength,
    ) -> ReadResult<Option<Self>> {
        let bytes = u64::from(logical);
        if bytes == 0 {
            return Ok(None);
        }
        // Offset zero satisfies MAP_ALIGNMENT; mapping lengths and staging
        // copies both require COPY_BUFFER_ALIGNMENT.
        let aligned = bytes
            .div_ceil(wgpu::COPY_BUFFER_ALIGNMENT)
            .checked_mul(wgpu::COPY_BUFFER_ALIGNMENT)
            .map(BufferByteLength::from)
            .ok_or(BufferReadError::MappingExtentOverflow { requested: logical })?;
        if aligned > allocated {
            return Err(BufferReadError::ExtentExceedsAllocation {
                requested: aligned,
                available: allocated,
            });
        }
        Ok(Some(Self(aligned)))
    }

    pub(super) fn bytes(self) -> BufferByteLength {
        self.0
    }
}

impl MappingRequest {
    pub(super) fn request(
        context: &WgpuContext,
        target: Arc<wgpu::Buffer>,
        extent: MappingExtent,
    ) -> ReadResult<Self> {
        let (sender, receiver) = oneshot::channel();
        let request = context
            .observe_validation(|| {
                target.slice(..u64::from(extent.bytes())).map_async(
                    wgpu::MapMode::Read,
                    move |result| {
                        let _ = sender.send(result);
                    },
                );
                // A callback may already have run. Own cleanup before the
                // native scope is inspected or any result can return early.
                Self {
                    target,
                    extent,
                    #[cfg(not(target_arch = "wasm32"))]
                    device: context.device.clone(),
                    completion: receiver,
                    release: ReleaseObligation::Required,
                }
            })
            .map_err(BufferReadError::Native)?;
        Ok(request)
    }

    pub(super) fn mapped_range(&self) -> wgpu::BufferView {
        self.target
            .slice(..u64::from(self.extent.bytes()))
            .get_mapped_range()
    }

    pub(super) async fn complete(&mut self, context: &WgpuContext) -> ReadResult<()> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            context
                .device
                .poll(wgpu::PollType::wait_indefinitely())
                .map_err(BufferReadError::Poll)?;
            loop {
                match poll_callback(&context.device, &mut self.completion)? {
                    Some(result) => return result.map_err(BufferReadError::Mapping),
                    None => {
                        std::thread::yield_now();
                    }
                }
            }
        }
        #[cfg(target_arch = "wasm32")]
        {
            await_browser_callback(&context.device, &mut self.completion)
                .await
                .map_err(BufferReadError::from)?
                .map_err(BufferReadError::Mapping)
        }
    }

    /// Consume the request after its mapped view has been dropped.
    pub(super) fn release(mut self, context: &WgpuContext) -> ReadResult<()> {
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
