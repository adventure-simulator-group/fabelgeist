use super::{TextureViewError, TextureViewResult};
use crate::data::{PassParameter, TextureBinding};
use crate::globals::WgpuContext;
use std::sync::Arc;

pub(crate) type SelectedTextureViewResult = TextureViewResult<SelectedTextureView>;

/// Facts about the selected view, rather than its underlying allocation.
pub(crate) struct SelectedTextureView {
    pub(crate) view: Arc<wgpu::TextureView>,
    format: wgpu::TextureFormat,
    dimension: wgpu::TextureViewDimension,
    device: Arc<wgpu::Device>,
}

impl SelectedTextureView {
    pub(super) fn new(
        view: Arc<wgpu::TextureView>,
        format: wgpu::TextureFormat,
        dimension: wgpu::TextureViewDimension,
        device: Arc<wgpu::Device>,
    ) -> Self {
        Self {
            view,
            format,
            dimension,
            device,
        }
    }

    fn validate(&self, context: &WgpuContext, binding: &TextureBinding) -> TextureViewResult<()> {
        if self.device.as_ref() != context.device.as_ref() {
            return Err(TextureViewError::WrongDevice);
        }
        if let Some(expected) = binding.format
            && expected != self.format
        {
            return Err(TextureViewError::FormatMismatch {
                expected,
                actual: self.format,
            });
        }
        if binding.dimension != self.dimension {
            return Err(TextureViewError::DimensionMismatch {
                expected: binding.dimension,
                actual: self.dimension,
            });
        }
        Ok(())
    }
}

impl PassParameter {
    pub(crate) fn texture_binding(
        &self,
        context: &WgpuContext,
        binding: &TextureBinding,
    ) -> SelectedTextureViewResult {
        let selected = match self {
            Self::Texture2d(texture) => {
                let requested =
                    if binding.format == Some(texture.format().srgb_counterpart().into()) {
                        texture.format().srgb_counterpart()
                    } else {
                        texture.format()
                    };
                texture.select_view(context, requested)?
            }
            Self::Texture3d(texture) => {
                let requested =
                    if binding.format == Some(texture.format().srgb_counterpart().into()) {
                        texture.format().srgb_counterpart()
                    } else {
                        texture.format()
                    };
                texture.select_view(context, requested)?
            }
            Self::TextureCube(texture) => {
                let requested =
                    if binding.format == Some(texture.format().srgb_counterpart().into()) {
                        texture.format().srgb_counterpart()
                    } else {
                        texture.format()
                    };
                texture.select_view(context, requested)?
            }
            Self::TextureView(texture) => texture.selected_view(context)?,
            _ => return Err(TextureViewError::NotTexture),
        };
        selected.validate(context, binding)?;
        Ok(selected)
    }
}
