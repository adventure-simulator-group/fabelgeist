//! Failures selecting a format view of an existing whole texture.

use super::TextureFormat;
use crate::globals::ValidationProgressError;
#[cfg(not(target_arch = "wasm32"))]
use crate::globals::ValidationScopeError;
use derive_more::Display;

/// The concrete result of selecting a whole-texture format view.
///
/// A generic caller admits the error with `?` or a standard error conversion;
/// the public producer itself retains this bespoke error type.
pub type TextureViewResult<T> = std::result::Result<T, TextureViewError>;

/// The view operation whose validation observation could not finish.
#[derive(Clone, Copy, Debug)]
pub enum TextureViewOperation {
    Texture2d,
    Texture3d,
    TextureCube,
}

/// Missing texture resources and native whole-texture format-view failures.
///
/// Native variants retain the original request, even when storage binding
/// requires its linear counterpart at the WebGPU descriptor boundary.
#[derive(Debug, Display)]
pub enum TextureViewError {
    /// The operation requires a native texture, but none is present.
    #[display("Texture is not initialized")]
    Uninitialized,
    /// The operation requires a cached view, but none is present.
    #[display("Texture has no view")]
    MissingView,
    /// Failed progress retains the request and any captured native rejection.
    #[display(
        "WGPU {operation:?} view validation observation Error (requested {requested:?}): {cause}"
    )]
    Progress {
        requested: TextureFormat,
        operation: TextureViewOperation,
        cause: Box<ValidationProgressError>,
    },
    /// Native rejection of a two-dimensional texture view.
    #[display("WGPU Texture2d view_with_format Error (requested {requested:?}): {cause}")]
    Texture2d {
        requested: TextureFormat,
        cause: Box<wgpu::Error>,
    },
    /// Native rejection of a three-dimensional texture view.
    #[display("WGPU Texture3d view_with_format Error (requested {requested:?}): {cause}")]
    Texture3d {
        requested: TextureFormat,
        cause: Box<wgpu::Error>,
    },
    /// Native rejection of a cube texture view.
    #[display("WGPU TextureCube view_with_format Error (requested {requested:?}): {cause}")]
    TextureCube {
        requested: TextureFormat,
        cause: Box<wgpu::Error>,
    },
}

// Native-only admission preserves the original operation-specific SDK cause.
#[cfg(not(target_arch = "wasm32"))]
impl TextureViewError {
    pub(super) fn from_observation(
        requested: TextureFormat,
        operation: TextureViewOperation,
        cause: ValidationScopeError,
    ) -> Self {
        match cause {
            ValidationScopeError::Progress(cause) => Self::Progress {
                requested,
                operation,
                cause: Box::new(cause),
            },
            ValidationScopeError::Native(cause) => {
                let cause = Box::new(cause);
                match operation {
                    TextureViewOperation::Texture2d => Self::Texture2d { requested, cause },
                    TextureViewOperation::Texture3d => Self::Texture3d { requested, cause },
                    TextureViewOperation::TextureCube => Self::TextureCube { requested, cause },
                }
            }
        }
    }
}

// Keep the concrete error behind each box as the dynamic source for downcasts.
// Deriving Error would expose the Box itself; the progress box also has a
// different concrete type from the three native-error boxes.
impl std::error::Error for TextureViewError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Texture2d { cause, .. }
            | Self::Texture3d { cause, .. }
            | Self::TextureCube { cause, .. } => Some(cause.as_ref()),
            Self::Progress { cause, .. } => Some(cause.as_ref()),
            Self::Uninitialized | Self::MissingView => None,
        }
    }
}
