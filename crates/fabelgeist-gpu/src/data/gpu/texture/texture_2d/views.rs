use std::sync::Arc;

use super::Texture2d;
use crate::{
    data::gpu::texture::{TextureFormat, TextureViewError, TextureViewResult},
    globals::WgpuContext,
};

impl Texture2d {
    pub fn view_with_format(
        &self,
        _context: &WgpuContext,
        format: TextureFormat,
    ) -> TextureViewResult<Arc<wgpu::TextureView>> {
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
        let error_scope = _context.blocking_validation.then(|| {
            _context
                .device
                .push_error_scope(wgpu::ErrorFilter::Validation)
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            format: Some(requested_format.into()),
            ..Default::default()
        });

        #[cfg(not(target_arch = "wasm32"))]
        if let Some(error_scope) = error_scope {
            let _ = _context.device.poll(wgpu::PollType::wait_indefinitely());
            if let Some(cause) = pollster::block_on(error_scope.pop()) {
                return Err(TextureViewError::from_texture_2d_view(format, cause));
            }
        }

        Ok(Arc::new(view))
    }
}
