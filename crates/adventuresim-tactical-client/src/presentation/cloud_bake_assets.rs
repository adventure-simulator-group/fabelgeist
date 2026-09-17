//! Offline cloud-bake format shared by fixed art-demo environments.

use bevy::{
    asset::RenderAssetUsages,
    image::{ImageAddressMode, ImageSampler, ImageSamplerDescriptor},
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};

pub(super) const CLOUD_BAKE_AZIMUTH_SEGMENTS: u32 = 1_024;
pub(super) const CLOUD_BAKE_ELEVATION_SEGMENTS: u32 = 256;
pub(super) const CLOUD_BAKE_TEXTURE_WIDTH: u32 = CLOUD_BAKE_AZIMUTH_SEGMENTS + 1;
pub(super) const CLOUD_BAKE_TEXTURE_HEIGHT: u32 = CLOUD_BAKE_ELEVATION_SEGMENTS + 1;
pub(super) const CLOUD_BAKE_CHANNELS: usize = 4;

/// A fixed art-demo environment can provide its initial optical bake directly.
/// Production scenes never insert this resource and retain the animated bake
/// pipeline.
#[derive(Resource)]
pub(crate) struct PrebakedCloudEnvironment {
    pub(crate) rgba8: &'static [u8],
}

pub(super) fn initial_image(
    prebaked: Option<&PrebakedCloudEnvironment>,
    procedural: impl FnOnce() -> Image,
) -> Image {
    prebaked.map_or_else(procedural, |asset| image_from_rgba8(asset.rgba8))
}

pub(super) fn image_from_rgba8(pixels: &[u8]) -> Image {
    assert_eq!(
        pixels.len(),
        CLOUD_BAKE_TEXTURE_WIDTH as usize
            * CLOUD_BAKE_TEXTURE_HEIGHT as usize
            * CLOUD_BAKE_CHANNELS,
        "prebaked cloud dimensions must match the tactical cloud shell"
    );
    let mut image = Image::new(
        Extent3d {
            width: CLOUD_BAKE_TEXTURE_WIDTH,
            height: CLOUD_BAKE_TEXTURE_HEIGHT,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        pixels.to_vec(),
        TextureFormat::Rgba8Unorm,
        RenderAssetUsages::RENDER_WORLD,
    );
    image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        address_mode_u: ImageAddressMode::ClampToEdge,
        address_mode_v: ImageAddressMode::ClampToEdge,
        ..ImageSamplerDescriptor::linear()
    });
    image
}
