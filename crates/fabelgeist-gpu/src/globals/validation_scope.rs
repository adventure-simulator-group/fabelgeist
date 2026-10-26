//! Native validation observation keeps polling and scope failures together.

#[cfg(not(target_arch = "wasm32"))]
use super::WgpuContext;
use derive_more::{Display, Error};
#[cfg(not(target_arch = "wasm32"))]
use std::sync::Arc;
use std::{error::Error, fmt};

pub type ValidationScopeResult<T> = Result<T, ValidationScopeError>;

/// A failed wait and any native validation error captured by the same scope.
#[derive(Debug)]
pub struct ValidationProgressError {
    pub poll: wgpu::PollError,
    pub native: Option<wgpu::Error>,
}

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
    #[display("{_0}")]
    Progress(#[error(source)] ValidationProgressError),
}

impl fmt::Display for ValidationProgressError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "Native validation progress failed: {}",
            self.poll
        )?;
        if let Some(native) = &self.native {
            write!(formatter, "; captured validation error: {native}")?;
        }
        Ok(())
    }
}

impl Error for ValidationProgressError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match &self.native {
            Some(native) => Some(native),
            None => Some(&self.poll),
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl ValidationScope {
    /// Return no guard when the context leaves validation errors uncaptured.
    pub fn new(context: &WgpuContext) -> Option<Self> {
        let guard = context.validation_observation.start_scope(&context.device)?;
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
            (Err(poll), native) => Err(ValidationScopeError::Progress(ValidationProgressError {
                poll,
                native,
            })),
        }
    }
}

impl ValidationScopeError {
    /// Admit the actual cause at an existing generic error boundary.
    pub fn into_anyhow(self) -> anyhow::Error {
        match self {
            Self::Native(cause) => anyhow::Error::new(cause),
            Self::Progress(cause) => anyhow::Error::new(cause),
        }
    }
}
