//! Procedural reflection cards and a soft diffuse surround, independent of albedo.
use super::*;
use bevy::{
    asset::RenderAssetUsages,
    render::render_resource::{
        Extent3d, TextureDimension, TextureFormat, TextureViewDescriptor, TextureViewDimension,
    },
};
const FACE_SIZE: u32 = 32;
pub(super) fn studio_map(world: &mut World) -> EnvironmentMapLight {
    let mut images = world.resource_mut::<Assets<Image>>();
    let mut light = EnvironmentMapLight::hemispherical_gradient(
        &mut images,
        Color::srgb(0.3, 0.34, 0.4),
        Color::srgb(0.15, 0.15, 0.15),
        Color::srgb(0.04, 0.035, 0.03),
    );
    let mut bytes = Vec::new();
    for face in 0..6 {
        for level in 0..=FACE_SIZE.ilog2() {
            let side = FACE_SIZE >> level;
            let blur = level as f32 / FACE_SIZE.ilog2() as f32;
            for y in 0..side {
                for x in 0..side {
                    let u = (x as f32 + 0.5) / side as f32;
                    let v = (y as f32 + 0.5) / side as f32;
                    let card = if face == 0 || face == 4 {
                        (-((u - 0.4) / (0.08 + blur * 0.4)).powi(2)).exp() * (1.0 - blur * 0.8)
                    } else {
                        0.0
                    };
                    let radiance = 0.035 + card * (0.65 + 0.35 * v);
                    let c = (radiance * 255.0).round() as u8;
                    bytes.extend_from_slice(&[c, c, c, 255]);
                }
            }
        }
    }
    let mut image = Image::new_uninit(
        Extent3d {
            width: FACE_SIZE,
            height: FACE_SIZE,
            depth_or_array_layers: 6,
        },
        TextureDimension::D2,
        TextureFormat::Rgba8Unorm,
        RenderAssetUsages::default(),
    );
    image.data = Some(bytes);
    image.texture_descriptor.mip_level_count = FACE_SIZE.ilog2() + 1;
    image.texture_view_descriptor = Some(TextureViewDescriptor {
        dimension: Some(TextureViewDimension::Cube),
        ..default()
    });
    image.sampler =
        bevy::image::ImageSampler::Descriptor(bevy::image::ImageSamplerDescriptor::linear());
    light.specular_map = images.add(image);
    light.intensity = 0.0;
    light
}
