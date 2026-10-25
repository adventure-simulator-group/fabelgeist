#[cfg(not(target_arch = "wasm32"))]
use crate::globals::ValidationScope;
use std::sync::Arc;

use anyhow::Result;

use crate::{
    data::ComputePassError,
    data::gpu::texture::{TextureFormat, TextureViewError},
    data::vector::Vec2,
    globals::WgpuContext,
};

pub use image::*;

mod image;
mod views;

#[derive(Clone, Debug)]
pub struct Texture2d {
    pub(super) texture: Option<Arc<wgpu::Texture>>,
    pub(super) view: Option<Arc<wgpu::TextureView>>,
    pub(super) size: (u32, u32),
    pub(super) format: TextureFormat,
    pub(super) usage: wgpu::TextureUsages,
    pub(super) device: Option<Arc<wgpu::Device>>,
}

impl PartialEq for Texture2d {
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

impl Default for Texture2d {
    fn default() -> Self {
        Self {
            device: None,
            texture: None,
            view: None,
            size: (0, 0),
            format: TextureFormat::default(),
            usage: wgpu::TextureUsages::empty(),
        }
    }
}
impl Texture2d {
    pub fn new(
        context: &WgpuContext,
        size: Option<Vec2>,
        format: Option<TextureFormat>,
    ) -> Result<Texture2d> {
        Self::create(
            context,
            size.unwrap_or_else(|| {
                Vec2::new(
                    super::DEFAULT_TEXTURE_EDGE_PIXELS,
                    super::DEFAULT_TEXTURE_EDGE_PIXELS,
                )
            }),
            format.unwrap_or_default(),
        )
    }
    pub fn create(context: &WgpuContext, size: Vec2, format: TextureFormat) -> Result<Texture2d> {
        let _wgpu_format: wgpu::TextureFormat = format.into();

        let limits = context.device.limits();
        let max_dim = limits.max_texture_dimension_2d;

        if size.x.is_nan() || size.y.is_nan() {
            return Err(anyhow::anyhow!("Texture size contains NaN: {:?}", size));
        }

        let width = size.x as u32;
        let height = size.y as u32;

        if width < 1 || height < 1 {
            return Err(anyhow::anyhow!(
                "Texture size must be at least 1x1. Got {}x{} (from {:?})",
                width,
                height,
                size
            ));
        }

        if width > max_dim || height > max_dim {
            return Err(anyhow::anyhow!(
                "Texture size {}x{} exceeds maximum supported dimension {} (from {:?})",
                width,
                height,
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
            label: Some("Texture2d"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu_base_format,
            usage,
            view_formats: &view_formats,
        };

        // Borrowed contexts avoid synchronous validation readback; see
        // WgpuContext::blocking_validation.
        #[cfg(not(target_arch = "wasm32"))]
        let error_scope = ValidationScope::new(context);

        let texture = context.device.create_texture(&texture_desc);

        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            format: Some(format.into()),
            ..Default::default()
        });

        #[cfg(not(target_arch = "wasm32"))]
        if let Some(error_scope) = error_scope {
            if let Err(cause) = error_scope.finish() {
                let cause = cause.into_anyhow();
                let message = format!("WGPU Texture2d Creation Error: {cause}");
                return Err(cause.context(message));
            }
        }

        let texture_value = Texture2d {
            device: Some(context.device.clone()),
            texture: Some(Arc::new(texture)),
            view: Some(Arc::new(view)),
            size: (width, height),
            format,
            usage,
        };

        Ok(texture_value)
    }

    pub fn size(&self) -> Vec2 {
        Vec2::new(self.size.0 as f32, self.size.1 as f32)
    }

    pub fn from_image(
        context: &WgpuContext,
        image: Image,
        format: Option<TextureFormat>,
    ) -> Result<Texture2d> {
        Self::create_from_image(
            context,
            image,
            format.unwrap_or(TextureFormat::Rgba8UnormSrgb),
        )
    }

    /// A texture holding this image's pixels.
    ///
    /// The pixels are stored as they arrived; `format` says how a shader will
    /// read them back, not what to turn them into on the way in. See
    /// [`Image::packed_for`], which is where that rule is written down and why.
    pub fn create_from_image(
        context: &WgpuContext,
        image: Image,
        format: TextureFormat,
    ) -> Result<Texture2d> {
        if image.width == 0 || image.height == 0 {
            return Ok(Texture2d::default());
        }

        let width = image.width;
        let height = image.height;

        let tex = Texture2d::create(context, Vec2::new(width as f32, height as f32), format)?;

        let converted_data = image.packed_for(format)?;

        super::transfer::TextureTransferLayout::new(
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            format,
        )?
        .write(
            context,
            tex.texture
                .as_ref()
                .ok_or(TextureViewError::Uninitialized)?,
            wgpu::Origin3d::ZERO,
            &converted_data,
        )?;

        Ok(tex)
    }

    pub fn from_color(
        context: &WgpuContext,
        size: Option<Vec2>,
        color: Option<crate::data::vector::Vec4>,
        format: Option<TextureFormat>,
    ) -> Result<Texture2d> {
        Self::create_from_color(
            context,
            size.unwrap_or_else(|| {
                Vec2::new(
                    super::DEFAULT_TEXTURE_EDGE_PIXELS,
                    super::DEFAULT_TEXTURE_EDGE_PIXELS,
                )
            }),
            color.unwrap_or_else(|| crate::data::vector::Vec4::new(0.0, 0.0, 0.0, 1.0)),
            format.unwrap_or(TextureFormat::Rgba8UnormSrgb),
        )
    }
    pub fn create_from_color(
        context: &WgpuContext,
        size: Vec2,
        color: crate::data::vector::Vec4,
        format: TextureFormat,
    ) -> Result<Texture2d> {
        let tex = Texture2d::create(context, size, format)?;
        tex.clear_raw(context, color)?;
        Ok(tex)
    }
    pub async fn read<T: bytemuck::AnyBitPattern>(&self, context: &WgpuContext) -> Result<Vec<T>> {
        let texture = self
            .texture
            .as_ref()
            .ok_or(TextureViewError::Uninitialized)?;
        self.ensure_device(context)?;
        super::readback::TextureReadSource {
            texture,
            origin: wgpu::Origin3d::ZERO,
            layout: super::transfer::TextureTransferLayout::new(self.extent(), self.format)?,
            kind: super::readback::TextureReadKind::Plane,
        }
        .read(context)
        .await
    }
    pub async fn read_to_rgba8(&self, context: &WgpuContext) -> Result<Vec<u8>> {
        // For now, we assume it's already in a readable format or we should convert it.
        // If it's Rgba8UnormSrgb, read() works directly.
        self.read::<u8>(context).await
    }
    pub async fn to_image(context: &WgpuContext, self_tex: Texture2d) -> Result<Image> {
        self_tex.read_image(context).await
    }

    pub fn write<T: bytemuck::NoUninit>(&self, context: &WgpuContext, data: &[T]) -> Result<()> {
        let texture = self
            .texture
            .as_ref()
            .ok_or(TextureViewError::Uninitialized)?;
        self.ensure_device(context)?;
        let bytes = bytemuck::cast_slice(data);

        super::transfer::TextureTransferLayout::new(self.extent(), self.format)?.write(
            context,
            texture,
            wgpu::Origin3d::ZERO,
            bytes,
        )?;

        Ok(())
    }

    pub fn to_view(self_tex: Texture2d) -> super::texture_view::TextureView {
        super::texture_view::TextureView::from(&self_tex)
    }
    pub fn as_view(&self) -> super::texture_view::TextureView {
        super::texture_view::TextureView::from(self)
    }

    pub fn add(
        context: &WgpuContext,
        self_tex: Texture2d,
        other: Texture2d,
        amount: Option<f64>,
    ) -> Result<Texture2d> {
        self_tex.add_raw(context, &other, amount.unwrap_or(1.0) as f32)
    }
    pub fn add_raw(
        &self,
        context: &WgpuContext,
        other: &Texture2d,
        amount: f32,
    ) -> Result<Texture2d> {
        use crate::data::gpu::compute::texture_ops::{TextureBinaryOp, TextureBinaryOpDefinition};
        use crate::data::gpu::resource::GpuResource;

        let def = TextureBinaryOpDefinition::new("a + b * amount");
        let output = Texture2d::create(context, self.size(), self.format)?;

        TextureBinaryOp::execute(
            context,
            &def,
            &GpuResource::Texture2d(self.clone()),
            &GpuResource::Texture2d(other.clone()),
            amount,
            &GpuResource::Texture2d(output.clone()),
        )?;

        Ok(output)
    }

    pub fn mix(
        context: &WgpuContext,
        self_tex: Texture2d,
        other: Texture2d,
        amount: Option<f64>,
    ) -> Result<Texture2d> {
        self_tex.mix_raw(context, &other, amount.unwrap_or(0.5) as f32)
    }
    pub fn mix_raw(
        &self,
        context: &WgpuContext,
        other: &Texture2d,
        amount: f32,
    ) -> Result<Texture2d> {
        use crate::data::gpu::compute::texture_ops::{TextureBinaryOp, TextureBinaryOpDefinition};
        use crate::data::gpu::resource::GpuResource;

        let def = TextureBinaryOpDefinition::new("mix(a, b, amount)");
        let output = Texture2d::create(context, self.size(), self.format)?;

        TextureBinaryOp::execute(
            context,
            &def,
            &GpuResource::Texture2d(self.clone()),
            &GpuResource::Texture2d(other.clone()),
            amount,
            &GpuResource::Texture2d(output.clone()),
        )?;

        Ok(output)
    }
    pub fn blit(&self, context: &WgpuContext, target: &Texture2d) -> Result<()> {
        let source_view = self.view.as_ref().ok_or(TextureViewError::MissingView)?;
        let target_view = target.view.as_ref().ok_or(TextureViewError::MissingView)?;
        self.ensure_device(context)?;
        target.ensure_device(context)?;
        let encoder = context
            .observe_validation(|| {
                let blitter = crate::globals::Blitter::new(
                    &context.device,
                    target.effective_format(target.format).into(),
                );
                let mut encoder =
                    context
                        .device
                        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                            label: Some("Texture2d Blit Encoder"),
                        });
                blitter.blit(&context.device, &mut encoder, source_view, target_view);
                encoder
            })
            .map_err(|cause| ComputePassError::Native(Arc::new(cause)))?;
        context
            .submit(encoder)
            .map_err(|cause| ComputePassError::Native(Arc::new(cause)))?;
        Ok(())
    }
    pub fn clear(
        context: &WgpuContext,
        target: Texture2d,
        color: Option<crate::data::vector::Vec4>,
    ) -> Result<Texture2d> {
        target.clear_raw(
            context,
            color.unwrap_or_else(|| crate::data::vector::Vec4::new(0.0, 0.0, 0.0, 1.0)),
        )?;
        Ok(target)
    }
    pub fn clear_raw(&self, context: &WgpuContext, color: crate::data::vector::Vec4) -> Result<()> {
        self.as_view().clear_raw(context, color)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A picture goes to the GPU and comes back the same picture.
    ///
    /// [`Image::packed_for`] and [`Texture2d::read_image`] are a pair, and the
    /// promise is only worth anything if they stay one. This walks a ramp of
    /// every eight-bit level through a real device in each eight-bit format.
    /// It used to come back with the darkest thirteen levels flattened onto
    /// zero and 133 of the 256 moved.
    #[tokio::test]
    async fn a_picture_survives_a_round_trip_through_the_gpu() -> Result<()> {
        let context = WgpuContext::new().await?;
        let pixels: Vec<u8> = (0..256u32)
            .flat_map(|level| [level as u8, level as u8, level as u8, 255])
            .collect();
        let image = Image::from_pixels(pixels, 256, 1)?;

        for format in [
            TextureFormat::Rgba8Unorm,
            TextureFormat::Rgba8UnormSrgb,
            TextureFormat::Bgra8Unorm,
            TextureFormat::Bgra8UnormSrgb,
        ] {
            let texture = Texture2d::create_from_image(&context, image.clone(), format)?;
            let read = texture.read_image(&context).await?;
            let out: Vec<u8> = read
                .data
                .as_chunks::<4>()
                .0
                .iter()
                .map(|pixel| pixel[0])
                .collect();
            let moved: Vec<usize> = (0..256)
                .filter(|&level| out[level] != level as u8)
                .collect();
            assert!(
                moved.is_empty(),
                "{format:?} moved {} of 256 levels; the darkest 32 came back as {:?}",
                moved.len(),
                &out[..32]
            );
        }
        Ok(())
    }
}
