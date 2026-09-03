use crate::{
    data::gpu::texture::{Image, TextureFormat},
    data::vector::{Vec2, Vec4},
    globals::WgpuContext,
};
use anyhow::{Result, anyhow};
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct TextureView {
    pub view: Option<Arc<wgpu::TextureView>>,
    pub texture: Option<Arc<wgpu::Texture>>,
    pub size: (u32, u32),
    pub format: TextureFormat,
    pub dimension: wgpu::TextureViewDimension,
    pub layer: Option<u32>,
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
            && self.format == other.format
            && self.dimension == other.dimension
            && self.layer == other.layer
    }
}

impl Default for TextureView {
    fn default() -> Self {
        Self {
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
        let view = self
            .view
            .as_ref()
            .ok_or_else(|| anyhow!("TextureView has no view"))?;
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
        context.queue.submit(Some(encoder.finish()));
        Ok(())
    }
    pub async fn read<T: bytemuck::AnyBitPattern>(&self, context: &WgpuContext) -> Result<Vec<T>> {
        let (width, height) = self.size;
        if width == 0 || height == 0 {
            return Ok(Vec::new());
        }

        let texture = self
            .texture
            .as_ref()
            .ok_or_else(|| anyhow!("TextureView has no underlying texture to read from"))?;

        let pixel_size = self.format.pixel_size();
        let bytes_per_row = width * pixel_size;
        let padded_bytes_per_row = (bytes_per_row + 255) & !255;
        let buffer_size = (padded_bytes_per_row * height) as u64;

        let staging_buffer = context.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("TextureView Staging Buffer"),
            size: buffer_size,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let mut encoder = context
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("TextureView Read Encoder"),
            });

        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture,
                mip_level: 0,
                origin: wgpu::Origin3d {
                    x: 0,
                    y: 0,
                    z: self.layer.unwrap_or(0),
                },
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &staging_buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(padded_bytes_per_row),
                    rows_per_image: Some(height),
                },
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );

        context.queue.submit(Some(encoder.finish()));

        #[allow(unused_mut)]
        let (tx, mut rx) = futures_channel::oneshot::channel();
        {
            let slice = staging_buffer.slice(..);
            slice.map_async(wgpu::MapMode::Read, move |res| {
                let _ = tx.send(res);
            });
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            loop {
                match rx.try_recv() {
                    Ok(Some(res)) => {
                        res.map_err(|e| anyhow!("GPU Mapping error: {:?}", e))?;
                        break;
                    }
                    Ok(None) => {
                        let _ = context.device.poll(wgpu::PollType::Poll);
                        fabelgeist_timer::sleep(std::time::Duration::from_millis(1)).await;
                    }
                    Err(_) => return Err(anyhow!("Mapping channel closed")),
                }
            }
        }

        #[cfg(target_arch = "wasm32")]
        rx.await
            .map_err(|_| anyhow!("Mapping channel closed"))?
            .map_err(|_| anyhow!("GPU Mapping error"))?;

        let slice = staging_buffer.slice(..);
        let data = slice.get_mapped_range()?;

        let mut result = Vec::with_capacity((width * height) as usize * pixel_size as usize);
        if padded_bytes_per_row == bytes_per_row {
            result.extend_from_slice(&data);
        } else {
            for row in 0..height {
                let start = (row * padded_bytes_per_row) as usize;
                let end = start + bytes_per_row as usize;
                result.extend_from_slice(&data[start..end]);
            }
        }

        drop(data);
        staging_buffer.unmap();

        Ok(bytemuck::cast_slice::<u8, T>(&result).to_vec())
    }
    pub async fn read_to_rgba8(&self, context: &WgpuContext) -> Result<Vec<u8>> {
        let (width, height) = self.size;
        if width == 0 || height == 0 {
            return Ok(Vec::new());
        }

        let raw_data = self.read::<u8>(context).await?;

        let linear_to_srgb = |f: f32| -> u8 {
            let srgb = if f <= 0.0031308 {
                f * 12.92
            } else {
                1.055 * f.powf(1.0 / 2.4) - 0.055
            };
            (srgb.clamp(0.0, 1.0) * 255.0) as u8
        };

        let rgba_data = match self.format {
            TextureFormat::Rgba8UnormSrgb => raw_data,
            TextureFormat::Bgra8UnormSrgb => raw_data
                .chunks_exact(4)
                .flat_map(|bgra| [bgra[2], bgra[1], bgra[0], bgra[3]])
                .collect(),
            TextureFormat::Rgba8Unorm => raw_data
                .chunks_exact(4)
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
