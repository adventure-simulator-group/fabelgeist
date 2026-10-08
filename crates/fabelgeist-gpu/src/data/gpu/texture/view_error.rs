//! Failures selecting a format view of an existing whole texture.

use super::TextureFormat;

/// Failures from the three whole-texture `view_with_format` methods.
///
/// Native variants retain the original request, even when storage binding
/// requires its linear counterpart at the WebGPU descriptor boundary.
#[derive(Debug)]
pub enum TextureViewError {
    /// A different-format request has no native texture to view.
    Uninitialized,
    /// A same-format request has no cached view to share.
    MissingView,
    /// Native rejection of a two-dimensional texture view.
    Texture2d {
        requested: TextureFormat,
        cause: Box<wgpu::Error>,
    },
    /// Native rejection of a three-dimensional texture view.
    Texture3d {
        requested: TextureFormat,
        cause: Box<wgpu::Error>,
    },
    /// Native rejection of a cube texture view.
    TextureCube {
        requested: TextureFormat,
        cause: Box<wgpu::Error>,
    },
}

// Native validation admission keeps operation selection and cause ownership
// together; view methods retain the descriptor and polling policy.
#[cfg(not(target_arch = "wasm32"))]
impl TextureViewError {
    pub(super) fn from_texture_2d_view(requested: TextureFormat, cause: wgpu::Error) -> Self {
        Self::Texture2d {
            requested,
            cause: Box::new(cause),
        }
    }

    pub(super) fn from_texture_3d_view(requested: TextureFormat, cause: wgpu::Error) -> Self {
        Self::Texture3d {
            requested,
            cause: Box::new(cause),
        }
    }

    pub(super) fn from_texture_cube_view(requested: TextureFormat, cause: wgpu::Error) -> Self {
        Self::TextureCube {
            requested,
            cause: Box::new(cause),
        }
    }
}

impl std::fmt::Display for TextureViewError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Uninitialized => formatter.write_str("Texture is not initialized"),
            Self::MissingView => formatter.write_str("Texture has no view"),
            Self::Texture2d { requested, cause } => write!(
                formatter,
                "WGPU Texture2d view_with_format Error (requested {requested:?}): {cause}"
            ),
            Self::Texture3d { requested, cause } => write!(
                formatter,
                "WGPU Texture3d view_with_format Error (requested {requested:?}): {cause}"
            ),
            Self::TextureCube { requested, cause } => write!(
                formatter,
                "WGPU TextureCube view_with_format Error (requested {requested:?}): {cause}"
            ),
        }
    }
}

impl std::error::Error for TextureViewError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Texture2d { cause, .. }
            | Self::Texture3d { cause, .. }
            | Self::TextureCube { cause, .. } => Some(cause.as_ref()),
            Self::Uninitialized | Self::MissingView => None,
        }
    }
}

/// The concrete result of selecting a whole-texture format view.
///
/// A generic caller admits the error with `?` or a standard error conversion;
/// the public producer itself retains this bespoke error type.
///
/// ```compile_fail
/// use fabelgeist_gpu::prelude::{Texture2d, TextureFormat};
/// use fabelgeist_gpu::globals::WgpuContext;
/// use std::sync::Arc;
///
/// fn erase_at_producer(
///     texture: &Texture2d,
///     context: &WgpuContext,
/// ) -> anyhow::Result<Arc<wgpu::TextureView>> {
///     texture.view_with_format(context, TextureFormat::R32Float)
/// }
/// ```
pub type TextureViewResult<T> = std::result::Result<T, TextureViewError>;
