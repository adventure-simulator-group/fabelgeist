//! Whether this context may drain device work to read validation scopes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ValidationReadback {
    Blocking,
    Deferred,
}
impl ValidationReadback {
    /// Only contexts permitted to drain device work capture blocking scopes.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn scope(self, device: &wgpu::Device) -> Option<wgpu::ErrorScopeGuard> {
        match self {
            Self::Blocking => Some(device.push_error_scope(wgpu::ErrorFilter::Validation)),
            Self::Deferred => None,
        }
    }
}
