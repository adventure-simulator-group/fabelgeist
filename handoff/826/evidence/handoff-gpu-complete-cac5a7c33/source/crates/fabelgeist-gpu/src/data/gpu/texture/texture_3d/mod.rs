#[cfg(not(target_arch = "wasm32"))]
use crate::globals::ValidationScope;
use std::sync::Arc;

use crate::data::gpu::texture::{TextureFormat, TextureViewError};
use anyhow::Result;

use crate::{data::vector::Vec3, globals::WgpuContext};

mod views;

#[derive(Clone, Debug)]
pub struct Texture3d {
    pub(super) texture: Option<Arc<wgpu::Texture>>,
    pub(super) view: Option<Arc<wgpu::TextureView>>,
    pub(super) size: (u32, u32, u32),
    pub(super) format: TextureFormat,
    pub(super) usage: wgpu::TextureUsages,
    pub(super) device: Option<Arc<wgpu::Device>>,
}

impl PartialEq for Texture3d {
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

impl Default for Texture3d {
    fn default() -> Self {
        Self {
            device: None,
            texture: None,
            view: None,
            size: (0, 0, 0),
            format: TextureFormat::default(),
            usage: wgpu::TextureUsages::empty(),
        }
    }
}
impl Texture3d {
    pub fn new(context: &WgpuContext, size: Vec3, format: TextureFormat) -> Result<Texture3d> {
        let _wgpu_format: wgpu::TextureFormat = format.into();

        if size.x.is_nan() || size.y.is_nan() || size.z.is_nan() {
            return Err(anyhow::anyhow!("Texture size contains NaN: {:?}", size));
        }

        let width = size.x.max(1.0) as u32;
        let height = size.y.max(1.0) as u32;
        let depth = size.z.max(1.0) as u32;

        let base_format = format.linear_counterpart();
        let wgpu_base_format: wgpu::TextureFormat = base_format.into();

        let mut view_formats = vec![wgpu_base_format];
        let counterpart = base_format.srgb_counterpart().into();
        if counterpart != wgpu_base_format {
            view_formats.push(counterpart);
        }

        let mut usage = wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::COPY_DST
            | wgpu::TextureUsages::COPY_SRC;

        if format.supports_storage() && !format.is_srgb() {
            usage |= wgpu::TextureUsages::STORAGE_BINDING;
        }

        let texture_desc = wgpu::TextureDescriptor {
            label: Some("Texture3d"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: depth,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D3,
            format: wgpu_base_format,
            usage,
            view_formats: &view_formats,
        };

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
                let message = format!("WGPU Texture3d Creation Error: {cause}");
                return Err(cause.context(message));
            }
        }

        let texture_value = Texture3d {
            device: Some(context.device.clone()),
            texture: Some(Arc::new(texture)),
            view: Some(Arc::new(view)),
            size: (width, height, depth),
            format,
            usage,
        };

        Ok(texture_value)
    }

    pub fn size(&self) -> Vec3 {
        Vec3::new(self.size.0 as f32, self.size.1 as f32, self.size.2 as f32)
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
            kind: super::readback::TextureReadKind::Volume,
        }
        .read(context)
        .await
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
}
