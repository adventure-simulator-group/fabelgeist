use super::*;
use adventuresim_heraldry::bake::{TextureKind, mips};
use bevy::{
    asset::RenderAssetUsages,
    image::{ImageAddressMode, ImageSampler, ImageSamplerDescriptor},
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};
fn upload(
    world: &mut World,
    assets: &mut SceneAssets,
    bytes: &[u8],
    size: u32,
    kind: TextureKind,
) -> Handle<Image> {
    let levels = mips(bytes, size, kind);
    let mut image = Image::new_uninit(
        Extent3d {
            width: size,
            height: size,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        if kind == TextureKind::Color {
            TextureFormat::Rgba8UnormSrgb
        } else {
            TextureFormat::Rgba8Unorm
        },
        RenderAssetUsages::default(),
    );
    image.texture_descriptor.mip_level_count = levels.len() as u32;
    image.data = Some(levels.into_iter().flat_map(|m| m.rgba).collect());
    image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        address_mode_u: ImageAddressMode::ClampToEdge,
        address_mode_v: ImageAddressMode::ClampToEdge,
        anisotropy_clamp: 8,
        ..ImageSamplerDescriptor::linear()
    });
    let handle = world.resource_mut::<Assets<Image>>().add(image);
    assets.images.push(handle.clone());
    handle
}
pub(super) fn material(
    world: &mut World,
    assets: &mut SceneAssets,
    b: &Baked,
    channel: Channel,
) -> Handle<StandardMaterial> {
    let material = if channel == Channel::Physical {
        let albedo = upload(world, assets, &b.albedo, b.size, TextureKind::Color);
        let normal = upload(world, assets, &b.normal, b.size, TextureKind::Normal);
        let orm = upload(world, assets, &b.orm, b.size, TextureKind::Linear);
        let coat = upload(world, assets, &b.coat, b.size, TextureKind::Linear);
        StandardMaterial {
            base_color_texture: Some(albedo),
            normal_map_texture: Some(normal),
            metallic_roughness_texture: Some(orm.clone()),
            occlusion_texture: Some(orm),
            metallic: 1.0,
            perceptual_roughness: 1.0,
            clearcoat: 1.0,
            clearcoat_perceptual_roughness: 1.0,
            clearcoat_texture: Some(coat.clone()),
            clearcoat_roughness_texture: Some(coat),
            ..default()
        }
    } else {
        let pixels = channel_pixels(b, channel);
        let texture = upload(world, assets, &pixels, b.size, TextureKind::Color);
        StandardMaterial {
            base_color_texture: Some(texture),
            unlit: true,
            ..default()
        }
    };
    world
        .resource_mut::<Assets<StandardMaterial>>()
        .add(material)
}
fn channel_pixels(b: &Baked, channel: Channel) -> Vec<u8> {
    match channel {
        Channel::Flat => b.flat.clone(),
        Channel::BaseColor | Channel::Physical => b.albedo.clone(),
        Channel::Normal => b.normal.clone(),
        Channel::Coating => b
            .coat
            .as_chunks::<4>()
            .0
            .iter()
            .flat_map(|p| [p[0], p[0], p[0], 255])
            .collect(),
        Channel::Roughness | Channel::Metallic => b
            .orm
            .as_chunks::<4>()
            .0
            .iter()
            .flat_map(|p| {
                let v = p[if channel == Channel::Roughness { 1 } else { 2 }];
                [v, v, v, 255]
            })
            .collect(),
        Channel::Height => {
            let low = b.height.iter().copied().fold(f32::INFINITY, f32::min);
            let high = b.height.iter().copied().fold(f32::NEG_INFINITY, f32::max);
            b.height
                .iter()
                .map(|v| ((v - low) / (high - low).max(f32::EPSILON) * 255.0) as u8)
                .flat_map(|v| [v, v, v, 255])
                .collect()
        }
    }
}
