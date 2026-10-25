//! Native upload of an admitted card atlas, retaining its variant population.

use crate::cards::{AtlasError, AtlasMip, AtlasResult, AtlasVariantCount, CardAtlas};
use crate::gpu::error::HairGpuError;

pub(super) struct UploadedAtlas {
    pub view: wgpu::TextureView,
    pub variants: AtlasVariantCount,
}

impl UploadedAtlas {
    /// Copy the admitted mip chain through a device and its paired queue.
    /// Native creation/write errors follow the device's configured handling;
    /// this result checks image admission and device limits, not global health.
    pub(super) fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        atlas: &CardAtlas,
    ) -> AtlasResult<Self> {
        let size = atlas.base().native_extent();
        let maximum = device.limits().max_texture_dimension_2d;
        if size.width > maximum || size.height > maximum {
            return Err(AtlasError::NativeExtent {
                width: size.width,
                height: size.height,
                maximum,
            });
        }
        let format = CardAtlas::FORMAT
            .to_wgpu()
            .ok_or(AtlasError::NativeFormat {
                format: CardAtlas::FORMAT,
            })?;
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("hair card atlas"),
            size,
            mip_level_count: atlas.mip_count(),
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        for (level, mip) in atlas.levels().enumerate() {
            queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &texture,
                    // The admitted chain has at most twelve levels.
                    mip_level: level as u32,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                mip.pixel_bytes(),
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(mip.bytes_per_row()),
                    rows_per_image: Some(mip.height_texels()),
                },
                mip.native_extent(),
            );
        }
        Ok(Self {
            view: texture.create_view(&Default::default()),
            variants: atlas.variants(),
        })
    }
}

impl AtlasMip {
    fn native_extent(&self) -> wgpu::Extent3d {
        wgpu::Extent3d {
            width: self.width_texels(),
            height: self.height_texels(),
            depth_or_array_layers: 1,
        }
    }
}

impl From<AtlasError> for HairGpuError {
    fn from(cause: AtlasError) -> Self {
        Self::Backend(cause.into())
    }
}
