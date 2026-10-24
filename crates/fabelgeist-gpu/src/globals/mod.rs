#[cfg(target_arch = "wasm32")]
pub use callback_progress::await_browser_callback;
pub use callback_progress::{CallbackProgressError, CallbackResult, poll_callback};
#[cfg(not(target_arch = "wasm32"))]
pub use validation_scope::ValidationScope;
pub use validation_scope::{ValidationProgressError, ValidationScopeError, ValidationScopeResult};
pub use wgpu_context::*;

mod callback_progress;
mod validation_scope;
mod wgpu_context;
