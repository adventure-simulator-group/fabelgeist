//! A reference grid behind transmitting materials makes distortion and tint legible.
use super::*;

pub(super) fn spawn(world: &mut World, assets: &mut SceneAssets, x: f32, distance: f32) {
    use bevy::{
        asset::RenderAssetUsages,
        render::render_resource::{Extent3d, TextureDimension, TextureFormat},
    };
    // Maintain background coverage when orbiting a pane with a distant backdrop.
    // One repeating quad avoids creating hundreds of checkerboard entities.
    let span = 2.1 + distance * 2.0;
    let mesh = world
        .resource_mut::<Assets<Mesh>>()
        .add(Rectangle::new(span, span));
    assets.meshes.push(mesh.clone());
    let light = [173, 166, 148, 255];
    let dark = [26, 46, 59, 255];
    let mut checker = Image::new(
        Extent3d {
            width: 2,
            height: 2,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        [light, dark, dark, light].concat(),
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    );
    checker.sampler = bevy::image::ImageSampler::Descriptor(bevy::image::ImageSamplerDescriptor {
        address_mode_u: bevy::image::ImageAddressMode::Repeat,
        address_mode_v: bevy::image::ImageAddressMode::Repeat,
        ..bevy::image::ImageSamplerDescriptor::nearest()
    });
    let image = world.resource_mut::<Assets<Image>>().add(checker);
    assets.images.push(image.clone());
    let material = world
        .resource_mut::<Assets<StandardMaterial>>()
        .add(StandardMaterial {
            base_color_texture: Some(image),
            uv_transform: bevy::math::Affine2::from_scale(Vec2::splat(span / 0.6)),
            unlit: true,
            cull_mode: None,
            ..default()
        });
    assets.materials.push(material.clone());
    assets.entities.push(
        world
            .spawn((
                Mesh3d(mesh),
                MeshMaterial3d(material),
                Transform::from_xyz(x, 0.0, -distance),
            ))
            .id(),
    );
}

pub(super) fn frame(world: &mut World, assets: &mut SceneAssets, x: f32) {
    let material = world
        .resource_mut::<Assets<StandardMaterial>>()
        .add(StandardMaterial {
            base_color: Color::srgb(0.12, 0.09, 0.06),
            perceptual_roughness: 0.8,
            ..default()
        });
    assets.materials.push(material.clone());
    for (half, position) in [
        (Vec3::new(0.82, 0.04, 0.035), Vec3::new(x, 0.78, 0.0)),
        (Vec3::new(0.82, 0.04, 0.035), Vec3::new(x, -0.78, 0.0)),
        (Vec3::new(0.04, 0.74, 0.035), Vec3::new(x - 0.78, 0.0, 0.0)),
        (Vec3::new(0.04, 0.74, 0.035), Vec3::new(x + 0.78, 0.0, 0.0)),
    ] {
        let mesh = world
            .resource_mut::<Assets<Mesh>>()
            .add(Cuboid::from_size(half * 2.0));
        assets.meshes.push(mesh.clone());
        assets.entities.push(
            world
                .spawn((
                    Mesh3d(mesh),
                    MeshMaterial3d(material.clone()),
                    Transform::from_translation(position),
                ))
                .id(),
        );
    }
}
