//! Creation and validation of alternate format views for existing textures.
use crate::data::{Texture2d, Texture3d, TextureCube, TextureFormat};
use crate::globals::WgpuContext;
use std::sync::Arc;

impl Texture2d {
    pub fn view_with_format(
        &self,
        _context: &WgpuContext,
        format: TextureFormat,
    ) -> std::result::Result<Arc<wgpu::TextureView>, crate::data::TextureViewError> {
        if format == self.format {
            return self
                .view
                .clone()
                .ok_or(crate::data::TextureViewError::MissingView {
                    dimension: wgpu::TextureViewDimension::D2,
                });
        }
        let texture = self
            .texture
            .as_ref()
            .ok_or(crate::data::TextureViewError::Uninitialized)?;

        let mut requested_format = format;

        // WebGPU Limitation: If a texture has STORAGE_BINDING, it cannot have an sRGB view.
        if self.usage.contains(wgpu::TextureUsages::STORAGE_BINDING) && format.is_srgb() {
            // Fallback to linear counterpart to avoid validation error.
            requested_format = format.linear_counterpart();
            // Optional: log or return a warning if we had a way to do so without noise.
        }

        #[cfg(not(target_arch = "wasm32"))]
        let error_scope = _context
            .device
            .push_error_scope(wgpu::ErrorFilter::Validation);

        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            format: Some(requested_format.into()),
            ..Default::default()
        });

        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = _context.device.poll(wgpu::PollType::wait_indefinitely());
            if let Some(err) = pollster::block_on(error_scope.pop()) {
                return Err(crate::data::TextureViewError::Device {
                    dimension: wgpu::TextureViewDimension::D2,
                    requested: format,
                    cause: err,
                });
            }
        }

        Ok(Arc::new(view))
    }
}

impl Texture3d {
    pub fn view_with_format(
        &self,
        _context: &WgpuContext,
        format: TextureFormat,
    ) -> std::result::Result<Arc<wgpu::TextureView>, crate::data::TextureViewError> {
        if format == self.format {
            return self
                .view
                .clone()
                .ok_or(crate::data::TextureViewError::MissingView {
                    dimension: wgpu::TextureViewDimension::D3,
                });
        }
        let texture = self
            .texture
            .as_ref()
            .ok_or(crate::data::TextureViewError::Uninitialized)?;

        let mut requested_format = format;

        // WebGPU Limitation: If a texture has STORAGE_BINDING, it cannot have an sRGB view.
        if self.usage.contains(wgpu::TextureUsages::STORAGE_BINDING) && format.is_srgb() {
            // Fallback to linear counterpart to avoid validation error.
            requested_format = format.linear_counterpart();
        }

        #[cfg(not(target_arch = "wasm32"))]
        let error_scope = _context
            .device
            .push_error_scope(wgpu::ErrorFilter::Validation);

        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            format: Some(requested_format.into()),
            ..Default::default()
        });

        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = _context.device.poll(wgpu::PollType::wait_indefinitely());
            if let Some(err) = pollster::block_on(error_scope.pop()) {
                return Err(crate::data::TextureViewError::Device {
                    dimension: wgpu::TextureViewDimension::D3,
                    requested: format,
                    cause: err,
                });
            }
        }

        Ok(Arc::new(view))
    }
}

impl TextureCube {
    pub fn view_with_format(
        &self,
        _context: &WgpuContext,
        format: TextureFormat,
    ) -> std::result::Result<Arc<wgpu::TextureView>, crate::data::TextureViewError> {
        if format == self.format {
            return self
                .view
                .clone()
                .ok_or(crate::data::TextureViewError::MissingView {
                    dimension: wgpu::TextureViewDimension::Cube,
                });
        }
        let texture = self
            .texture
            .as_ref()
            .ok_or(crate::data::TextureViewError::Uninitialized)?;

        let mut requested_format = format;
        if self.usage.contains(wgpu::TextureUsages::STORAGE_BINDING) && format.is_srgb() {
            requested_format = format.linear_counterpart();
        }

        #[cfg(not(target_arch = "wasm32"))]
        let error_scope = _context
            .device
            .push_error_scope(wgpu::ErrorFilter::Validation);

        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("TextureCube View"),
            dimension: Some(wgpu::TextureViewDimension::Cube),
            array_layer_count: Some(6),
            format: Some(requested_format.into()),
            ..Default::default()
        });

        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = _context.device.poll(wgpu::PollType::wait_indefinitely());
            if let Some(err) = pollster::block_on(error_scope.pop()) {
                return Err(crate::data::TextureViewError::Device {
                    dimension: wgpu::TextureViewDimension::Cube,
                    requested: format,
                    cause: err,
                });
            }
        }

        Ok(Arc::new(view))
    }
}
