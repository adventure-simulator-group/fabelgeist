//! Failures while selecting a view of an existing texture.
use crate::data::TextureFormat;

#[derive(Debug)]
pub enum TextureViewError {
    Uninitialized,
    MissingView {
        dimension: wgpu::TextureViewDimension,
    },
    Device {
        dimension: wgpu::TextureViewDimension,
        requested: TextureFormat,
        cause: wgpu::Error,
    },
}
impl std::fmt::Display for TextureViewError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Uninitialized => write!(formatter, "Texture is not initialized"),
            Self::MissingView { dimension } => {
                write!(formatter, "Texture {dimension:?} has no view")
            }
            Self::Device {
                dimension,
                requested,
                cause,
            } => {
                let texture = match dimension {
                    wgpu::TextureViewDimension::D2 => "Texture2d",
                    wgpu::TextureViewDimension::D3 => "Texture3d",
                    wgpu::TextureViewDimension::Cube => "TextureCube",
                    _ => "Texture",
                };
                write!(
                    formatter,
                    "WGPU {texture} view_with_format Error (requested {requested:?}): {cause}"
                )
            }
        }
    }
}
impl std::error::Error for TextureViewError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Device { cause, .. } => Some(cause),
            _ => None,
        }
    }
}
