//! Select how native validation errors are observed.

/// Whether creation may synchronously observe a native validation scope.
///
/// Borrowed devices and browser contexts leave errors to the provider's
/// uncaptured-error handler. Ownership and presentation remain separate from
/// this observation choice.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ValidationObservation {
    BlockingScopes,
    Uncaptured,
}

impl ValidationObservation {
    /// Begin a native validation scope only when synchronous observation is
    /// allowed.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn start_scope(self, device: &wgpu::Device) -> Option<wgpu::ErrorScopeGuard> {
        match self {
            Self::BlockingScopes => Some(device.push_error_scope(wgpu::ErrorFilter::Validation)),
            Self::Uncaptured => None,
        }
    }
}
