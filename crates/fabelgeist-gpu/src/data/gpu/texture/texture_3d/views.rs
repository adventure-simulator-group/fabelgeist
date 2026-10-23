use std::sync::Arc;

use super::Texture3d;
use crate::{
    data::gpu::texture::{TextureFormat, TextureViewError, TextureViewResult},
    globals::WgpuContext,
};

impl Texture3d {
    pub fn view_with_format(
        &self,
        context: &WgpuContext,
        format: TextureFormat,
    ) -> TextureViewResult<Arc<wgpu::TextureView>> {
        Ok(self.select_view(context, format)?.view)
    }

    pub(super) fn select_view(
        &self,
        context: &WgpuContext,
        format: TextureFormat,
    ) -> TextureViewResult<super::super::SelectedTextureView> {
        if format == self.format {
            let view = self.view.clone().ok_or(TextureViewError::MissingView)?;
            return Ok(super::super::SelectedTextureView::new(
                view,
                self.effective_format(format).into(),
                wgpu::TextureViewDimension::D3,
                self.device.clone().ok_or(TextureViewError::Uninitialized)?,
            ));
        }
        let texture = self
            .texture
            .as_ref()
            .ok_or(TextureViewError::Uninitialized)?;

        self.ensure_device(context)?;
        let mut requested_format = format;

        // The default view inherits storage usage, which excludes sRGB formats.
        if self.usage.contains(wgpu::TextureUsages::STORAGE_BINDING) && format.is_srgb() {
            requested_format = format.linear_counterpart();
        }

        #[cfg(not(target_arch = "wasm32"))]
        let error_scope = context.blocking_validation.then(|| {
            context
                .device
                .push_error_scope(wgpu::ErrorFilter::Validation)
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            format: Some(requested_format.into()),
            ..Default::default()
        });

        #[cfg(not(target_arch = "wasm32"))]
        if let Some(error_scope) = error_scope {
            let _ = context.device.poll(wgpu::PollType::wait_indefinitely());
            if let Some(cause) = pollster::block_on(error_scope.pop()) {
                return Err(TextureViewError::from_texture_3d_view(format, cause));
            }
        }

        Ok(super::super::SelectedTextureView::new(
            Arc::new(view),
            requested_format.into(),
            wgpu::TextureViewDimension::D3,
            self.device.clone().ok_or(TextureViewError::Uninitialized)?,
        ))
    }
    pub fn format(&self) -> TextureFormat {
        self.format
    }
    pub fn usage(&self) -> wgpu::TextureUsages {
        self.usage
    }
    pub fn extent(&self) -> wgpu::Extent3d {
        wgpu::Extent3d {
            width: self.size.0,
            height: self.size.1,
            depth_or_array_layers: self.size.2,
        }
    }
    pub fn native_texture(&self) -> Option<&Arc<wgpu::Texture>> {
        self.texture.as_ref()
    }
    pub fn cached_view(&self) -> Option<&Arc<wgpu::TextureView>> {
        self.view.as_ref()
    }

    /// Remove the cached view without admitting a replacement or changing metadata.
    pub fn clear_cached_view(&mut self) {
        self.view = None;
    }
    /// Detach the allocation port; a previously cached view remains usable.
    pub fn clear_native_texture(&mut self) {
        self.texture = None;
    }

    pub(super) fn effective_format(&self, format: TextureFormat) -> TextureFormat {
        if self.usage.contains(wgpu::TextureUsages::STORAGE_BINDING) && format.is_srgb() {
            format.linear_counterpart()
        } else {
            format
        }
    }

    pub fn ensure_device(&self, context: &WgpuContext) -> TextureViewResult<()> {
        if self.device.as_deref() != Some(context.device.as_ref()) {
            return Err(TextureViewError::WrongDevice);
        }
        Ok(())
    }
}
