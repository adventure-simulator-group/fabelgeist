//! Texture resource selection retains the existing sRGB counterpart policy.
use super::{BindingKind, ComputePassError};
use crate::data::{
    PassParameter, Texture2d, Texture3d, TextureBinding, TextureCube, TextureFormat, TextureView,
    TextureViewError,
};
use crate::globals::WgpuContext;
use std::sync::Arc;

enum TextureParameter<'a> {
    D2(&'a Texture2d),
    D3(&'a Texture3d),
    Cube(&'a TextureCube),
    View(&'a TextureView),
}
impl<'a> TextureParameter<'a> {
    fn new(value: &'a PassParameter) -> Option<Self> {
        match value {
            PassParameter::Texture2d(t) => Some(Self::D2(t)),
            PassParameter::Texture3d(t) => Some(Self::D3(t)),
            PassParameter::TextureCube(t) => Some(Self::Cube(t)),
            PassParameter::TextureView(t) => Some(Self::View(t)),
            _ => None,
        }
    }
    fn format(&self) -> TextureFormat {
        match self {
            Self::D2(t) => t.format,
            Self::D3(t) => t.format,
            Self::Cube(t) => t.format,
            Self::View(t) => t.format,
        }
    }
    fn actual_format(&self) -> Option<wgpu::TextureFormat> {
        match self {
            Self::D2(t) => &t.texture,
            Self::D3(t) => &t.texture,
            Self::Cube(t) => &t.texture,
            Self::View(t) => &t.texture,
        }
        .as_ref()
        .map(|t: &Arc<wgpu::Texture>| -> wgpu::TextureFormat { t.format() })
    }
    fn dimension(&self) -> wgpu::TextureViewDimension {
        match self {
            Self::D2(_) => wgpu::TextureViewDimension::D2,
            Self::D3(_) => wgpu::TextureViewDimension::D3,
            Self::Cube(_) => wgpu::TextureViewDimension::Cube,
            Self::View(t) => t.dimension,
        }
    }
    fn view(
        &self,
        context: &WgpuContext,
        binding: &TextureBinding,
    ) -> Result<Option<Arc<wgpu::TextureView>>, TextureViewError> {
        let counterpart = self.format().srgb_counterpart();
        if let Some(expected) = binding.format
            && wgpu::TextureFormat::from(self.format()) != expected
            && wgpu::TextureFormat::from(counterpart) == expected
        {
            match self {
                Self::D2(t) => return t.view_with_format(context, counterpart).map(Some),
                Self::D3(t) => return t.view_with_format(context, counterpart).map(Some),
                Self::Cube(t) => return t.view_with_format(context, counterpart).map(Some),
                Self::View(_) => {}
            }
        }
        Ok(match self {
            Self::D2(t) => t.view.clone(),
            Self::D3(t) => t.view.clone(),
            Self::Cube(t) => t.view.clone(),
            Self::View(t) => t.view.clone(),
        })
    }
}
impl TextureBinding {
    pub(super) fn resolve(
        &self,
        context: &WgpuContext,
        value: &PassParameter,
    ) -> Result<Arc<wgpu::TextureView>, ComputePassError> {
        let texture = TextureParameter::new(value).ok_or_else(|| -> ComputePassError {
            ComputePassError::WrongParameter {
                name: self.name.clone(),
                expected: BindingKind::Texture,
            }
        })?;
        // Preserve admission order: view creation precedes format, dimension,
        // and missing-view checks.
        let view =
            texture
                .view(context, self)
                .map_err(|cause: TextureViewError| -> ComputePassError {
                    ComputePassError::View {
                        name: self.name.clone(),
                        cause,
                    }
                })?;
        if let Some(expected) = self.format
            && let Some(actual) = texture.actual_format()
            && actual != expected
            && wgpu::TextureFormat::from(texture.format().srgb_counterpart()) != expected
        {
            return Err(ComputePassError::TextureFormat {
                name: self.name.clone(),
                expected,
                actual,
            });
        }
        let actual = texture.dimension();
        if actual != self.dimension {
            return Err(ComputePassError::TextureDimension {
                name: self.name.clone(),
                expected: self.dimension,
                actual,
            });
        }
        view.ok_or_else(|| -> ComputePassError {
            ComputePassError::MissingTextureView(self.name.clone())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{BindingIndex, TextureViewError};
    use std::error::Error;

    #[tokio::test]
    async fn texture_admission_preserves_dimension_view_order_and_view_causes() {
        let context = WgpuContext::new().await.unwrap();
        let binding = TextureBinding {
            name: "image".into(),
            binding: BindingIndex::from(2),
            format: None,
            dimension: wgpu::TextureViewDimension::D2,
        };
        assert!(matches!(
            binding.resolve(&context, &4u32.into()),
            Err(ComputePassError::WrongParameter {
                expected: BindingKind::Texture,
                ..
            })
        ));
        assert!(matches!(
            binding.resolve(&context, &Texture3d::default().into()),
            Err(ComputePassError::TextureDimension {
                expected: wgpu::TextureViewDimension::D2,
                actual: wgpu::TextureViewDimension::D3,
                ..
            })
        ));
        assert!(matches!(
            binding.resolve(&context, &Texture2d::default().into()),
            Err(ComputePassError::MissingTextureView(_))
        ));
        let texture = Texture2d {
            format: TextureFormat::Rgba8Unorm,
            ..Default::default()
        };
        let alias = TextureBinding {
            format: Some(texture.format.srgb_counterpart().into()),
            ..binding
        };
        let error = alias.resolve(&context, &texture.into()).unwrap_err();
        assert!(
            matches!(&error, ComputePassError::View { name, cause: TextureViewError::Uninitialized } if *name == alias.name)
        );
        assert!(matches!(
            error.source().unwrap().downcast_ref::<TextureViewError>(),
            Some(TextureViewError::Uninitialized)
        ));
        for error in [
            Texture2d::default()
                .view_with_format(&context, Texture2d::default().format)
                .unwrap_err(),
            Texture3d::default()
                .view_with_format(&context, Texture3d::default().format)
                .unwrap_err(),
            TextureCube::default()
                .view_with_format(&context, TextureCube::default().format)
                .unwrap_err(),
        ] {
            assert!(matches!(error, TextureViewError::MissingView { .. }));
        }
    }
}
