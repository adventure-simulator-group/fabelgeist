//! Whole-texture and cube-face views share typed validation failures.

use super::TextureCube;
#[cfg(not(target_arch = "wasm32"))]
use crate::{data::gpu::texture::TextureViewOperation, globals::ValidationScope};
use crate::{
    data::gpu::texture::{
        CubeFace, TextureFormat, TextureView, TextureViewError, TextureViewResult,
    },
    globals::WgpuContext,
};
use std::sync::Arc;
use wgpu::{
    TextureUsages, TextureView as WgpuTextureView, TextureViewDescriptor, TextureViewDimension,
};

impl TextureCube {
    pub fn view_with_format(
        &self,
        _ctx: &WgpuContext,
        format: TextureFormat,
    ) -> TextureViewResult<Arc<WgpuTextureView>> {
        if format == self.format {
            return self.view.clone().ok_or(TextureViewError::MissingView);
        }
        let texture = self
            .texture
            .as_ref()
            .ok_or(TextureViewError::Uninitialized)?;

        let mut requested_format = format;
        if self.usage.contains(TextureUsages::STORAGE_BINDING) && format.is_srgb() {
            requested_format = format.linear_counterpart();
        }

        #[cfg(not(target_arch = "wasm32"))]
        let error_scope = ValidationScope::new(_ctx);

        let view = texture.create_view(&TextureViewDescriptor {
            label: Some("TextureCube View"),
            dimension: Some(TextureViewDimension::Cube),
            array_layer_count: Some(6),
            format: Some(requested_format.into()),
            ..Default::default()
        });

        #[cfg(not(target_arch = "wasm32"))]
        if let Some(error_scope) = error_scope {
            error_scope
                .finish()
                .map_err(|cause| TextureViewError::Observation {
                    requested: format,
                    operation: TextureViewOperation::TextureCube,
                    cause,
                })?;
        }

        Ok(Arc::new(view))
    }

    pub fn face(
        context: &WgpuContext,
        target: TextureCube,
        face: CubeFace,
    ) -> TextureViewResult<TextureView> {
        target.face_view_with_format(Some(context), face, target.format)
    }
    /// Select a face without synchronously observing native validation.
    pub fn face_view(&self, face: CubeFace) -> TextureViewResult<TextureView> {
        self.face_view_with_format(None, face, self.format)
    }
    /// Select a face and format, optionally observing validation on native targets.
    ///
    /// A context enables observation only under `BlockingScopes`. Without one,
    /// or under `Uncaptured`, validation reporting stays with the device provider.
    /// Observed failures retain the face and original requested format.
    pub fn face_view_with_format(
        &self,
        _context: Option<&WgpuContext>,
        face: CubeFace,
        format: TextureFormat,
    ) -> TextureViewResult<TextureView> {
        let texture = self
            .texture
            .as_ref()
            .ok_or(TextureViewError::Uninitialized)?;

        let mut requested_format = format;
        if self.usage.contains(TextureUsages::STORAGE_BINDING) && format.is_srgb() {
            requested_format = format.linear_counterpart();
        }

        #[cfg(not(target_arch = "wasm32"))]
        let error_scope = _context.and_then(ValidationScope::new);

        let view = texture.create_view(&TextureViewDescriptor {
            label: Some(&format!("TextureCube Face View ({:?})", face)),
            dimension: Some(TextureViewDimension::D2),
            format: Some(requested_format.into()),
            base_array_layer: face.index(),
            array_layer_count: Some(1),
            ..Default::default()
        });

        #[cfg(not(target_arch = "wasm32"))]
        if let Some(error_scope) = error_scope {
            error_scope
                .finish()
                .map_err(|cause| TextureViewError::Observation {
                    requested: format,
                    operation: TextureViewOperation::TextureCubeFace(face),
                    cause,
                })?;
        }

        Ok(TextureView {
            view: Some(Arc::new(view)),
            texture: self.texture.clone(),
            size: (self.size, self.size),
            format: requested_format,
            dimension: TextureViewDimension::D2,
            layer: Some(face.index()),
        })
    }
}
