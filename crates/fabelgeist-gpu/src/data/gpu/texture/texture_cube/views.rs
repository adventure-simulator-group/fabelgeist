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
        _ctx: &WgpuContext,
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
        if self.usage.contains(wgpu::TextureUsages::STORAGE_BINDING) && format.is_srgb() {
            requested_format = format.linear_counterpart();
        }

        #[cfg(not(target_arch = "wasm32"))]
        let error_scope = _ctx
            .blocking_validation
            .then(|| _ctx.device.push_error_scope(wgpu::ErrorFilter::Validation));

        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("TextureCube View"),
            dimension: Some(wgpu::TextureViewDimension::Cube),
            array_layer_count: Some(6),
            format: Some(requested_format.into()),
            ..Default::default()
        });

        #[cfg(not(target_arch = "wasm32"))]
        if let Some(error_scope) = error_scope {
            let _ = _ctx.device.poll(wgpu::PollType::wait_indefinitely());
            if let Some(cause) = pollster::block_on(error_scope.pop()) {
                return Err(TextureViewError::from_texture_cube_view(format, cause));
            }
        }

        Ok(Arc::new(view))
    }
    pub fn face(context: &WgpuContext, target: TextureCube, face: CubeFace) -> Result<TextureView> {
        target.face_view_with_format(Some(context), face, target.format)
    }
    pub fn face_view(&self, face: CubeFace) -> Result<TextureView> {
        self.face_view_with_format(None, face, self.format)
    }
    pub fn face_view_with_format(
        &self,
        // Only read to check the view natively; the web cannot block on it.
        #[cfg_attr(target_arch = "wasm32", allow(unused_variables))] context: Option<&WgpuContext>,
        face: CubeFace,
        format: TextureFormat,
    ) -> Result<TextureView> {
        let texture = self
            .texture
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Texture is not initialized"))?;

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
            view: Some(Arc::new(view)),
            texture: self.texture.clone(),
            size: (self.size, self.size),
            format: requested_format,
            dimension: wgpu::TextureViewDimension::D2,
            layer: Some(face.index()),
        })
    }
    pub fn view_2d_array(&self) -> Option<wgpu::TextureView> {
        self.texture.as_ref().map(|tex| {
            let mut requested_format = self.format;
            if self.usage.contains(wgpu::TextureUsages::STORAGE_BINDING) && self.format.is_srgb() {
                requested_format = self.format.linear_counterpart();
            }
            tex.create_view(&wgpu::TextureViewDescriptor {
                label: Some("TextureCube 2D Array View for Storage"),
                dimension: Some(wgpu::TextureViewDimension::D2Array),
                array_layer_count: Some(6),
                format: Some(requested_format.into()),
                ..Default::default()
            })
        })
    }
}
