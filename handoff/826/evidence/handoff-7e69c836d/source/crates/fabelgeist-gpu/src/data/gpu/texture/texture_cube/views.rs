use std::sync::Arc;

use anyhow::Result;

use super::TextureCube;
use crate::{
    data::gpu::texture::{
        CubeFace, TextureFormat, TextureView, TextureViewError, TextureViewResult,
    },
    globals::WgpuContext,
};

impl TextureCube {
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
                wgpu::TextureViewDimension::Cube,
                self.device.clone().ok_or(TextureViewError::Uninitialized)?,
            ));
        }
        let texture = self
            .texture
            .as_ref()
            .ok_or(TextureViewError::Uninitialized)?;

        self.ensure_device(context)?;
        let mut requested_format = format;
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
            label: Some("TextureCube View"),
            dimension: Some(wgpu::TextureViewDimension::Cube),
            array_layer_count: Some(6),
            format: Some(requested_format.into()),
            ..Default::default()
        });

        #[cfg(not(target_arch = "wasm32"))]
        if let Some(error_scope) = error_scope {
            let _ = context.device.poll(wgpu::PollType::wait_indefinitely());
            if let Some(cause) = pollster::block_on(error_scope.pop()) {
                return Err(TextureViewError::from_texture_cube_view(format, cause));
            }
        }

        Ok(super::super::SelectedTextureView::new(
            Arc::new(view),
            requested_format.into(),
            wgpu::TextureViewDimension::Cube,
            self.device.clone().ok_or(TextureViewError::Uninitialized)?,
        ))
    }
    pub fn face(context: &WgpuContext, target: TextureCube, face: CubeFace) -> Result<TextureView> {
        target.face_view_with_format(Some(context), face, target.format)
    }
    pub fn face_view(&self, face: CubeFace) -> Result<TextureView> {
        self.face_view_with_format(None, face, self.format)
    }
    pub fn face_view_with_format(
        &self,
        // A supplied context proves device identity on every target.
        context: Option<&WgpuContext>,
        face: CubeFace,
        format: TextureFormat,
    ) -> Result<TextureView> {
        let texture = self
            .texture
            .as_ref()
            .ok_or(TextureViewError::Uninitialized)?;

        if let Some(context) = context {
            self.ensure_device(context)?;
        }
        let mut requested_format = format;
        if self.usage.contains(wgpu::TextureUsages::STORAGE_BINDING) && format.is_srgb() {
            requested_format = format.linear_counterpart();
        }

        #[cfg(not(target_arch = "wasm32"))]
        let error_scope = context
            .filter(|context| context.blocking_validation)
            .map(|context| {
                context
                    .device
                    .push_error_scope(wgpu::ErrorFilter::Validation)
            });

        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some(&format!("TextureCube Face View ({:?})", face)),
            dimension: Some(wgpu::TextureViewDimension::D2),
            format: Some(requested_format.into()),
            base_array_layer: face.index(),
            array_layer_count: Some(1),
            ..Default::default()
        });

        #[cfg(not(target_arch = "wasm32"))]
        if let (Some(c), Some(scope)) = (context, error_scope) {
            let _ = c.device.poll(wgpu::PollType::wait_indefinitely());
            if let Some(err) = pollster::block_on(scope.pop()) {
                let message = format!("WGPU TextureCube Face View Error: {err}");
                return Err(anyhow::Error::new(err).context(message));
            }
        }

        Ok(TextureView {
            device: self.device.clone(),
            view: Some(Arc::new(view)),
            texture: self.texture.clone(),
            size: (self.size, self.size),
            format: requested_format,
            dimension: wgpu::TextureViewDimension::D2,
            layer: Some(face.index()),
        })
    }
    pub fn view_2d_array(&self, context: &WgpuContext) -> TextureViewResult<wgpu::TextureView> {
        let texture = self
            .texture
            .as_ref()
            .ok_or(TextureViewError::Uninitialized)?;
        self.ensure_device(context)?;
        let requested_format = self.effective_format(self.format);
        #[cfg(not(target_arch = "wasm32"))]
        let scope = context.blocking_validation.then(|| {
            context
                .device
                .push_error_scope(wgpu::ErrorFilter::Validation)
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("TextureCube 2D Array View for Storage"),
            dimension: Some(wgpu::TextureViewDimension::D2Array),
            array_layer_count: Some(6),
            format: Some(requested_format.into()),
            ..Default::default()
        });
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(scope) = scope {
            let _ = context.device.poll(wgpu::PollType::wait_indefinitely());
            if let Some(cause) = pollster::block_on(scope.pop()) {
                return Err(TextureViewError::CubeArray {
                    requested: self.format,
                    cause: Box::new(cause),
                });
            }
        }
        Ok(view)
    }
    pub fn format(&self) -> TextureFormat {
        self.format
    }
    pub fn usage(&self) -> wgpu::TextureUsages {
        self.usage
    }
    pub fn extent(&self) -> wgpu::Extent3d {
        wgpu::Extent3d {
            width: self.size,
            height: self.size,
            depth_or_array_layers: 6,
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
