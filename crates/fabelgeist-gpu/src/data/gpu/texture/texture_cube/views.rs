//! Whole-texture format views preserve cached identity and native failure causes.

use super::TextureCube;
#[cfg(not(target_arch = "wasm32"))]
use crate::{data::gpu::texture::TextureViewOperation, globals::ValidationScope};
use crate::{
    data::gpu::texture::{TextureFormat, TextureViewError, TextureViewResult},
    globals::WgpuContext,
};
use std::sync::Arc;

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
        let error_scope = ValidationScope::new(_ctx);

        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("TextureCube View"),
            dimension: Some(wgpu::TextureViewDimension::Cube),
            array_layer_count: Some(6),
            format: Some(requested_format.into()),
            ..Default::default()
        });

        #[cfg(not(target_arch = "wasm32"))]
        if let Some(error_scope) = error_scope {
            error_scope.finish().map_err(|cause| {
                TextureViewError::from_observation(format, TextureViewOperation::TextureCube, cause)
            })?;
        }

        Ok(Arc::new(view))
    }
}
