//! The scalp's baked textures, copied on the GPU from the buffers they were
//! baked in into the images the scalp is drawn with.
//!
//! The bake leaves each texture's mip chain in one buffer, every level's
//! rows padded as a copy into a texture moves them. Its image is made with
//! no data and copied into once the renderer has made its texture, so no
//! texel comes back to the CPU.
//!
//! Copies use Bevy's configured device error handling. This render system does
//! not observe a synchronous native validation result.
use super::*;
use adventuresim_character_creator::hair::{ScalpTextureChain, ScalpTextureLayout};
use bevy::image::{ImageAddressMode, ImageFilterMode, ImageSampler, ImageSamplerDescriptor};
use bevy::render::render_asset::RenderAssets;
use bevy::render::render_resource::{
    CommandEncoderDescriptor, Extent3d, Origin3d, TexelCopyBufferInfo, TexelCopyBufferLayout,
    TexelCopyTextureInfo, TextureAspect, TextureDimension, TextureFormat,
};
use bevy::render::renderer::{RenderDevice, RenderQueue};
use bevy::render::texture::GpuImage;
use bevy::render::{Render, RenderApp, RenderSystems};
use std::sync::{Mutex, PoisonError};

/// The most samples the scalp's texture is filtered with along the line
/// of sight: hardware anisotropic filtering's usual ceiling.
const SCALP_ANISOTROPY: u16 = 16;

/// Scalp textures waiting for the renderer to make their images, shared by
/// the main world, which adds them, and the render world, which copies them.
#[derive(Resource, Clone, Default)]
pub(crate) struct ScalpUploads(Arc<Mutex<Vec<ScalpUpload>>>);

struct ScalpUpload {
    /// Held until copied into, so the renderer makes its texture.
    image: Handle<Image>,
    chain: ScalpTextureChain,
}

/// Copies the scalp's baked textures into their images on the GPU.
pub(super) struct ScalpUploadPlugin;

impl ScalpUploads {
    /// An image the baked mip `chain` is copied into on the GPU, sampled
    /// anisotropically: its strands lie along the scalp, which the eye
    /// meets at every angle round the head.
    pub(super) fn image(
        &self,
        chain: &ScalpTextureChain,
        images: &mut Assets<Image>,
    ) -> Handle<Image> {
        let image = images.add(scalp_image(chain.layout()));
        self.0
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push(ScalpUpload {
                image: image.clone(),
                chain: chain.clone(),
            });
        image
    }
}

impl Plugin for ScalpUploadPlugin {
    fn build(&self, app: &mut App) {
        let uploads = ScalpUploads::default();
        app.insert_resource(uploads.clone());
        if let Some(render) = app.get_sub_app_mut(RenderApp) {
            render.insert_resource(uploads).add_systems(
                Render,
                upload
                    .in_set(RenderSystems::PrepareResources)
                    .after(RenderSystems::PrepareAssets),
            );
        }
    }
}

/// An empty image the size of the scalp's textures with their whole mip
/// chain, filtered linearly and anisotropically.
fn scalp_image(layout: &ScalpTextureLayout) -> Image {
    let side = layout.base().side_texels();
    let mut image = Image::new_uninit(
        Extent3d {
            width: side,
            height: side,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        TextureFormat::Rgba8Unorm,
        RenderAssetUsages::RENDER_WORLD,
    );
    image.texture_descriptor.mip_level_count = layout.levels().len() as u32;
    image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        address_mode_u: ImageAddressMode::ClampToEdge,
        address_mode_v: ImageAddressMode::ClampToEdge,
        mag_filter: ImageFilterMode::Linear,
        min_filter: ImageFilterMode::Linear,
        mipmap_filter: ImageFilterMode::Linear,
        anisotropy_clamp: SCALP_ANISOTROPY,
        ..default()
    });
    image
}

/// Copy every scalp texture whose image the renderer has made, each mip
/// level from its place in the baked chain.
fn upload(
    uploads: Res<ScalpUploads>,
    images: Res<RenderAssets<GpuImage>>,
    device: Res<RenderDevice>,
    queue: Res<RenderQueue>,
) {
    let mut pending = uploads.0.lock().unwrap_or_else(PoisonError::into_inner);
    if pending.is_empty() {
        return;
    }
    let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
        label: Some("hair scalp upload"),
    });
    let mut copied = false;
    pending.retain(|upload| {
        let Some(image) = images.get(&upload.image) else {
            return true;
        };
        let layout = upload.chain.layout();
        for (mip, level) in layout.levels().iter().enumerate() {
            encoder.copy_buffer_to_texture(
                TexelCopyBufferInfo {
                    buffer: &upload.chain.buffer().buffer,
                    layout: TexelCopyBufferLayout {
                        offset: u64::from(level.offset()),
                        bytes_per_row: Some(u64::from(layout.row_bytes(level)) as u32),
                        rows_per_image: Some(level.side_texels()),
                    },
                },
                TexelCopyTextureInfo {
                    texture: &image.texture,
                    mip_level: mip as u32,
                    origin: Origin3d::ZERO,
                    aspect: TextureAspect::All,
                },
                Extent3d {
                    width: level.side_texels(),
                    height: level.side_texels(),
                    depth_or_array_layers: 1,
                },
            );
        }
        copied = true;
        false
    });
    if copied {
        queue.submit([encoder.finish()]);
    }
}
