#[cfg(not(target_arch = "wasm32"))]
use crate::globals::ValidationScope;
use std::sync::Arc;

use anyhow::Result;

use crate::{
    data::gpu::texture::{CubeFace, Image, TextureFormat, TextureView, TextureViewError},
    globals::WgpuContext,
};

mod blit;
mod image_data;
mod shader;
mod views;

#[derive(Clone, Debug)]
pub struct TextureCube {
    pub(super) texture: Option<Arc<wgpu::Texture>>,
    pub(super) view: Option<Arc<wgpu::TextureView>>,
    pub(super) size: u32,
    pub(super) format: TextureFormat,
    pub(super) usage: wgpu::TextureUsages,
    pub(super) device: Option<Arc<wgpu::Device>>,
}

impl PartialEq for TextureCube {
    fn eq(&self, other: &Self) -> bool {
        (match (&self.texture, &other.texture) {
            (Some(a), Some(b)) => Arc::ptr_eq(a, b),
            (None, None) => true,
            _ => false,
        }) && (match (&self.view, &other.view) {
            (Some(a), Some(b)) => Arc::ptr_eq(a, b),
            (None, None) => true,
            _ => false,
        }) && self.size == other.size
            && self.device == other.device
            && self.format == other.format
            && self.usage == other.usage
    }
}

impl Default for TextureCube {
    fn default() -> Self {
        Self {
            device: None,
            texture: None,
            view: None,
            size: 0,
            format: TextureFormat::default(),
            usage: wgpu::TextureUsages::empty(),
        }
    }
}
impl TextureCube {
    pub fn new(context: &WgpuContext, size: f32, format: TextureFormat) -> Result<TextureCube> {
        let _wgpu_format: wgpu::TextureFormat = format.into();

        if size.is_nan() {
            return Err(anyhow::anyhow!("Texture size contains NaN: {:?}", size));
        }

        let size = size as u32;

        if size < 1 {
            return Err(anyhow::anyhow!(
                "Texture size must be at least 1. Got {} (from {:?})",
                size,
                size
            ));
        }

        let limits = context.device.limits();
        let max_dim = limits.max_texture_dimension_2d;
        if size > max_dim {
            return Err(anyhow::anyhow!(
                "Texture size {} exceeds maximum supported dimension {} (from {:?})",
                size,
                max_dim,
                size
            ));
        }

        let mut usage = wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::COPY_DST
            | wgpu::TextureUsages::COPY_SRC;

        if format.supports_render_attachment() {
            usage |= wgpu::TextureUsages::RENDER_ATTACHMENT;
        }

        if format.supports_storage() && !format.is_srgb() {
            usage |= wgpu::TextureUsages::STORAGE_BINDING;
        }

        let base_format = format.linear_counterpart();
        let wgpu_base_format: wgpu::TextureFormat = base_format.into();

        let mut view_formats = vec![wgpu_base_format];
        let counterpart = base_format.srgb_counterpart().into();
        if counterpart != wgpu_base_format {
            view_formats.push(counterpart);
        }

        let texture_desc = wgpu::TextureDescriptor {
            label: Some("TextureCube"),
            size: wgpu::Extent3d {
                width: size,
                height: size,
                depth_or_array_layers: 6,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu_base_format,
            usage,
            view_formats: &view_formats,
        };

        #[cfg(not(target_arch = "wasm32"))]
        let error_scope = ValidationScope::new(context);

        let texture = context.device.create_texture(&texture_desc);

        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("TextureCube View"),
            dimension: Some(wgpu::TextureViewDimension::Cube),
            array_layer_count: Some(6),
            format: Some(format.into()),
            ..Default::default()
        });

        #[cfg(not(target_arch = "wasm32"))]
        if let Some(error_scope) = error_scope {
            if let Err(cause) = error_scope.finish() {
                let cause = cause.into_anyhow();
                let message = format!("WGPU TextureCube Creation Error: {cause}");
                return Err(cause.context(message));
            }
        }

        let texture_value = TextureCube {
            device: Some(context.device.clone()),
            texture: Some(Arc::new(texture)),
            view: Some(Arc::new(view)),
            size,
            format,
            usage,
        };

        Ok(texture_value)
    }

    /// A cube map holding these six images' pixels.
    ///
    /// Each face is stored as it arrived, on the same terms as a 2D texture:
    /// see [`Image::packed_for`].
    pub fn create_from_images(
        context: &WgpuContext,
        images: [Image; 6],
        format: TextureFormat,
    ) -> Result<TextureCube> {
        let size = images[0].width;
        for (i, img) in images.iter().enumerate() {
            if img.width != size || img.height != size {
                return Err(anyhow::anyhow!(
                    "Cube map images must be square and identical in size. Face 0: {}x{}, Face {}: {}x{}",
                    size,
                    size,
                    i,
                    img.width,
                    img.height
                ));
            }
        }

        let tex = TextureCube::new(context, size as f32, format)?;

        for (i, image) in images.into_iter().enumerate() {
            let converted_data = image.packed_for(format)?;

            super::transfer::TextureTransferLayout::new(
                wgpu::Extent3d {
                    width: size,
                    height: size,
                    depth_or_array_layers: 1,
                },
                format,
            )?
            .write(
                context,
                tex.texture
                    .as_ref()
                    .ok_or(TextureViewError::Uninitialized)?,
                wgpu::Origin3d {
                    x: 0,
                    y: 0,
                    z: i as u32,
                },
                &converted_data,
            )?;
        }

        Ok(tex)
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "the cubemap boundary names its six independently optional faces"
    )]
    pub fn from_textures(
        context: &WgpuContext,
        right: Option<TextureView>,
        left: Option<TextureView>,
        top: Option<TextureView>,
        bottom: Option<TextureView>,
        front: Option<TextureView>,
        back: Option<TextureView>,
        format: Option<TextureFormat>,
        size: Option<f64>,
    ) -> Result<TextureCube> {
        let size_val = size
            .map(|s| s as f32)
            .or_else(|| right.as_ref().map(|t| t.size.0 as f32))
            .or_else(|| left.as_ref().map(|t| t.size.0 as f32))
            .or_else(|| top.as_ref().map(|t| t.size.0 as f32))
            .or_else(|| bottom.as_ref().map(|t| t.size.0 as f32))
            .or_else(|| front.as_ref().map(|t| t.size.0 as f32))
            .or_else(|| back.as_ref().map(|t| t.size.0 as f32))
            .unwrap_or(super::DEFAULT_TEXTURE_EDGE_PIXELS);

        let format_val = format
            .or_else(|| right.as_ref().map(|t| t.format))
            .unwrap_or(TextureFormat::Rgba8UnormSrgb);

        let cube = TextureCube::new(context, size_val, format_val)?;

        let faces = [
            (CubeFace::PositiveX, right),
            (CubeFace::NegativeX, left),
            (CubeFace::PositiveY, top),
            (CubeFace::NegativeY, bottom),
            (CubeFace::PositiveZ, front),
            (CubeFace::NegativeZ, back),
        ];

        for (face, tex_opt) in faces {
            if let Some(tex) = tex_opt {
                cube.set_side_texture(context, face, &tex)?;
            }
        }

        Ok(cube)
    }

    pub fn set_side(
        context: &WgpuContext,
        target: TextureCube,
        face: CubeFace,
        texture: TextureView,
    ) -> Result<TextureCube> {
        target.set_side_texture(context, face, &texture)?;
        Ok(target)
    }

    pub fn set_side_image(
        &self,
        context: &WgpuContext,
        face: CubeFace,
        image: &Image,
    ) -> Result<()> {
        let texture = self
            .texture
            .as_ref()
            .ok_or(TextureViewError::Uninitialized)?;
        self.ensure_device(context)?;

        if image.width != self.size || image.height != self.size {
            return Err(anyhow::anyhow!(
                "Image size ({}x{}) must match cubemap face size ({}x{})",
                image.width,
                image.height,
                self.size,
                self.size
            ));
        }

        let converted_data = image_data::convert_face_pixels(image, self.format)?;

        super::transfer::TextureTransferLayout::new(
            wgpu::Extent3d {
                width: self.size,
                height: self.size,
                depth_or_array_layers: 1,
            },
            self.format,
        )?
        .write(
            context,
            texture,
            wgpu::Origin3d {
                x: 0,
                y: 0,
                z: face.index(),
            },
            &converted_data,
        )?;

        Ok(())
    }
    pub fn size(&self) -> f32 {
        self.size as f32
    }
}
