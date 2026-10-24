use crate::{
    data::ComputePassError,
    data::gpu::texture::{Image, TextureFormat, TextureViewError, TextureViewResult},
    data::vector::{Vec2, Vec4},
    globals::WgpuContext,
};
use anyhow::{Result, anyhow};
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct TextureView {
    pub(super) view: Option<Arc<wgpu::TextureView>>,
    pub(super) texture: Option<Arc<wgpu::Texture>>,
    pub(super) size: (u32, u32),
    pub(super) format: TextureFormat,
    pub(super) dimension: wgpu::TextureViewDimension,
    pub(super) layer: Option<u32>,
    pub(super) device: Option<Arc<wgpu::Device>>,
}

impl PartialEq for TextureView {
    fn eq(&self, other: &Self) -> bool {
        (match (&self.view, &other.view) {
            (Some(a), Some(b)) => Arc::ptr_eq(a, b),
            (None, None) => true,
            _ => false,
        }) && (match (&self.texture, &other.texture) {
            (Some(a), Some(b)) => Arc::ptr_eq(a, b),
            (None, None) => true,
            _ => false,
        }) && self.size == other.size
            && self.device == other.device
            && self.format == other.format
            && self.dimension == other.dimension
            && self.layer == other.layer
    }
}

impl Default for TextureView {
    fn default() -> Self {
        Self {
            device: None,
            view: None,
            texture: None,
            size: (0, 0),
            format: TextureFormat::Rgba8Unorm,
            dimension: wgpu::TextureViewDimension::D2,
            layer: None,
        }
    }
}
impl TextureView {
    pub fn format(&self) -> TextureFormat {
        self.format
    }
    pub fn dimension(&self) -> wgpu::TextureViewDimension {
        self.dimension
    }
    pub fn native_texture(&self) -> Option<&Arc<wgpu::Texture>> {
        self.texture.as_ref()
    }
    pub fn cached_view(&self) -> Option<&Arc<wgpu::TextureView>> {
        self.view.as_ref()
    }
    pub fn extent(&self) -> wgpu::Extent3d {
        wgpu::Extent3d {
            width: self.size.0,
            height: self.size.1,
            depth_or_array_layers: 1,
        }
    }
    pub(super) fn selected_view(&self, context: &WgpuContext) -> super::SelectedTextureViewResult {
        let view = self.view.clone().ok_or(TextureViewError::MissingView)?;
        self.ensure_device(context)?;
        Ok(super::SelectedTextureView::new(
            view,
            self.format.into(),
            self.dimension,
            self.device
                .clone()
                .ok_or(super::TextureViewError::Uninitialized)?,
        ))
    }

    pub fn ensure_device(&self, context: &WgpuContext) -> TextureViewResult<()> {
        if self.device.as_deref() != Some(context.device.as_ref()) {
            return Err(TextureViewError::WrongDevice);
        }
        Ok(())
    }

    pub fn size(&self) -> Vec2 {
        Vec2::new(self.size.0 as f32, self.size.1 as f32)
    }

    pub fn clear(
        context: &WgpuContext,
        target: TextureView,
        color: Option<Vec4>,
    ) -> Result<TextureView> {
        target.clear_raw(
            context,
            color.unwrap_or_else(|| Vec4::new(0.0, 0.0, 0.0, 1.0)),
        )?;
        Ok(target)
    }

    pub async fn to_image(context: WgpuContext, self_view: TextureView) -> Result<Image> {
        let (width, height) = self_view.size;
        if width == 0 || height == 0 {
            return Ok(Image::default());
        }
        let rgba = self_view.read_to_rgba8(&context).await?;
        Image::from_pixels(rgba, width, height)
    }
    pub fn clear_raw(&self, context: &WgpuContext, color: Vec4) -> Result<()> {
        let view = self.view.as_ref().ok_or(TextureViewError::MissingView)?;
        self.ensure_device(context)?;
        let mut encoder = context
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("TextureView Clear Encoder"),
            });
        {
            if self.format.is_depth() {
                let depth_clear_value = if color.x == 0.0 && color.y == 0.0 && color.z == 0.0 {
                    1.0
                } else {
                    color.x
                };
                let _rp = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("TextureView Depth Clear Pass"),
                    color_attachments: &[],
                    depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                        view,
                        depth_ops: Some(wgpu::Operations {
                            load: wgpu::LoadOp::Clear(depth_clear_value),
                            store: wgpu::StoreOp::Store,
                        }),
                        stencil_ops: None,
                    }),
                    timestamp_writes: None,
                    occlusion_query_set: None,
                    multiview_mask: None,
                });
            } else {
                let mut clear_color = color;
                if self.format.is_uint() || self.format.is_sint() {
                    clear_color = Vec4::new(
                        color.x * 255.0,
                        color.y * 255.0,
                        color.z * 255.0,
                        color.w * 255.0,
                    );
                }

                let _rp = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("TextureView Clear Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view,
                        depth_slice: None,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color {
                                r: clear_color.x as f64,
                                g: clear_color.y as f64,
                                b: clear_color.z as f64,
                                a: clear_color.w as f64,
                            }),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                    multiview_mask: None,
                });
            }
        }
        context
            .submit(encoder)
            .map_err(|cause| ComputePassError::Native(Arc::new(cause)))?;
        Ok(())
    }
    pub async fn read<T: bytemuck::AnyBitPattern>(&self, context: &WgpuContext) -> Result<Vec<T>> {
        if self.size.0 == 0 || self.size.1 == 0 {
            return Ok(Vec::new());
        }
        let texture = self
            .texture
            .as_ref()
            .ok_or(TextureViewError::Uninitialized)?;
        self.ensure_device(context)?;
        super::readback::TextureReadSource {
            texture,
            origin: wgpu::Origin3d {
                x: 0,
                y: 0,
                z: self.layer.unwrap_or(0),
            },
            layout: super::transfer::TextureTransferLayout::new(self.extent(), self.format)?,
            kind: super::readback::TextureReadKind::View,
        }
        .read(context)
        .await
    }
    pub async fn read_to_rgba8(&self, context: &WgpuContext) -> Result<Vec<u8>> {
        let (width, height) = self.size;
        if width == 0 || height == 0 {
            return Ok(Vec::new());
        }

        let raw_data = self.read::<u8>(context).await?;
        let linear_to_srgb = fabelgeist_color::transfer::linear_to_srgb_u8;

        let rgba_data = match self.format {
            TextureFormat::Rgba8UnormSrgb => raw_data,
            TextureFormat::Bgra8UnormSrgb => raw_data
                .as_chunks::<4>()
                .0
                .iter()
                .flat_map(|bgra| [bgra[2], bgra[1], bgra[0], bgra[3]])
                .collect(),
            TextureFormat::Rgba8Unorm => raw_data
                .as_chunks::<4>()
                .0
                .iter()
                .flat_map(|rgba| {
                    [
                        linear_to_srgb(rgba[0] as f32 / 255.0),
                        linear_to_srgb(rgba[1] as f32 / 255.0),
                        linear_to_srgb(rgba[2] as f32 / 255.0),
                        rgba[3],
                    ]
                })
                .collect(),
            _ => {
                let pixel_size = self.format.pixel_size();
                if pixel_size == 4 {
                    raw_data
                } else {
                    return Err(anyhow!(
                        "Unsupported texture view conversion to rgba8 for format: {:?}",
                        self.format
                    ));
                }
            }
        };

        Ok(rgba_data)
    }
}

impl From<&crate::data::gpu::texture::Texture2d> for TextureView {
    fn from(t: &crate::data::gpu::texture::Texture2d) -> Self {
        TextureView {
            device: t.device.clone(),
            view: t.view.clone(),
            texture: t.texture.clone(),
            size: t.size,
            format: t.format,
            dimension: wgpu::TextureViewDimension::D2,
            layer: None,
        }
    }
}

impl From<crate::data::gpu::texture::Texture2d> for TextureView {
    fn from(t: crate::data::gpu::texture::Texture2d) -> Self {
        TextureView::from(&t)
    }
}
