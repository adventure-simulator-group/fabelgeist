//! Whole-texture format views preserve cached identity and native failure causes.

use super::Texture2d;
#[cfg(not(target_arch = "wasm32"))]
use crate::{data::gpu::texture::TextureViewOperation, globals::ValidationScope};
use crate::{
    data::gpu::texture::{TextureFormat, TextureViewError, TextureViewResult},
    globals::WgpuContext,
};
use std::sync::Arc;
use wgpu::TextureView;

impl Texture2d {
    pub fn view_with_format(
        &self,
        _context: &WgpuContext,
        format: TextureFormat,
    ) -> TextureViewResult<Arc<TextureView>> {
        if format == self.format {
            return self.view.clone().ok_or(TextureViewError::MissingView);
        }
        let texture = self
            .texture
            .as_ref()
            .ok_or(TextureViewError::Uninitialized)?;

        let mut requested_format = format;

        // The default view inherits storage usage, which excludes sRGB formats.
        if self.usage.contains(wgpu::TextureUsages::STORAGE_BINDING) && format.is_srgb() {
            requested_format = format.linear_counterpart();
        }

        #[cfg(not(target_arch = "wasm32"))]
        let error_scope = ValidationScope::new(_context);

        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            format: Some(requested_format.into()),
            ..Default::default()
        });

        #[cfg(not(target_arch = "wasm32"))]
        if let Some(error_scope) = error_scope {
            error_scope.finish().map_err(|cause| {
                TextureViewError::from_observation(format, TextureViewOperation::Texture2d, cause)
            })?;
        }

        Ok(Arc::new(view))
    }
}
