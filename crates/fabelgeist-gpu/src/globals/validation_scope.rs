//! Native validation observation keeps polling and scope failures together.

#[cfg(not(target_arch = "wasm32"))]
use super::WgpuContext;
use derive_more::{Display, Error};
#[cfg(not(target_arch = "wasm32"))]
use std::sync::Arc;

pub type ValidationScopeResult<T> = Result<T, ValidationScopeError>;

/// The actual device and its same-thread native validation guard.
#[cfg(not(target_arch = "wasm32"))]
#[must_use]
pub struct ValidationScope {
    device: Arc<wgpu::Device>,
    guard: wgpu::ErrorScopeGuard,
}

/// Native rejection or incomplete progress while observing validation.
#[derive(Debug, Display, Error)]
pub enum ValidationScopeError {
    #[display("{_0}")]
    Native(#[error(source)] wgpu::Error),
    #[display("Native validation progress failed: {_0}")]
    Progress(#[error(source)] wgpu::PollError),
    /// Polling failed, and the same scope also captured a native rejection.
    #[display("Native validation progress failed: {poll}; captured validation error: {native}")]
    ProgressWithNative {
        #[error(source)]
        poll: wgpu::PollError,
        native: wgpu::Error,
    },
}

#[cfg(not(target_arch = "wasm32"))]
impl ValidationScope {
    /// Return no guard when the context leaves validation errors uncaptured.
    pub fn new(context: &WgpuContext) -> Option<Self> {
        let guard = context
            .validation_observation
            .start_scope(&context.device)?;
        Some(Self {
            device: context.device.clone(),
            guard,
        })
    }

    /// Attempt the existing native wait, then always pop and read the scope.
    ///
    /// The SDK receives no explicit timeout. Its backend may still fail the
    /// wait. Neither success nor failure establishes global device health or
    /// rolls back commands already recorded or submitted by the caller.
    pub fn finish(self) -> ValidationScopeResult<()> {
        let progress = self.device.poll(wgpu::PollType::wait_indefinitely());
        let native = pollster::block_on(self.guard.pop());
        match (progress, native) {
            (Ok(_), None) => Ok(()),
            (Ok(_), Some(native)) => Err(ValidationScopeError::Native(native)),
            (Err(poll), None) => Err(ValidationScopeError::Progress(poll)),
            (Err(poll), Some(native)) => {
                Err(ValidationScopeError::ProgressWithNative { poll, native })
            }
        }
    }
}
