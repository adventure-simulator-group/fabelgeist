//! Failures selecting a format view of an existing whole texture.

use super::TextureFormat;
use crate::globals::ValidationScopeError;
use derive_more::{Display, Error};

/// The concrete result of selecting a whole-texture format view.
///
/// A generic caller admits the error with `?` or a standard error conversion;
/// the public producer itself retains this bespoke error type.
pub type TextureViewResult<T> = std::result::Result<T, TextureViewError>;

/// The whole-texture format-view operation being observed.
#[derive(Clone, Copy, Debug)]
pub enum TextureViewOperation {
    Texture2d,
    Texture3d,
    TextureCube,
}

/// Missing texture resources and native whole-texture format-view failures.
///
/// Observation failures retain the original request, even when storage binding
/// requires its linear counterpart at the WebGPU descriptor boundary.
#[derive(Debug, Display, Error)]
pub enum TextureViewError {
    /// The operation requires a native texture, but none is present.
    #[display("Texture is not initialized")]
    Uninitialized,
    /// The operation requires a cached view, but none is present.
    #[display("Texture has no view")]
    MissingView,
    /// The request and the complete failure of its validation observation.
    #[display(
        "WGPU {operation:?} view validation failed (requested {requested:?}): {cause}"
    )]
    Observation {
        requested: TextureFormat,
        operation: TextureViewOperation,
        #[error(source)]
        cause: ValidationScopeError,
    },
}
