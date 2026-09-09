use super::*;
use adventuresim_procedural_textures::{
    BakedMap, BakedRecipe, MapChannel, PixelEncoding, TextureRecipeId,
};
use bevy::{
    asset::RenderAssetUsages,
    image::{ImageAddressMode, ImageSampler, ImageSamplerDescriptor},
    math::Affine2,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};

pub(super) fn image(map: &BakedMap, display: bool) -> Image {
    let mut bytes = map.bytes.clone();
    let mut format = map.encoding.texture_format();
    if display {
        bytes = map
            .bytes
            .chunks_exact(map.encoding.channels())
            .flat_map(|p| match map.encoding {
                PixelEncoding::R8 => [p[0], p[0], p[0], 255],
                PixelEncoding::Rg8 => [p[0], p[1], 0, 255],
                _ if map.channel == MapChannel::Opacity => [p[3], p[3], p[3], 255],
                _ => [p[0], p[1], p[2], 255],
            })
            .collect();
        format = TextureFormat::Rgba8UnormSrgb;
    }
    let mut image = Image::new_uninit(
        Extent3d {
            width: map.size,
            height: map.size,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        format,
        RenderAssetUsages::default(),
    );
    image.data = Some(bytes);
    image.texture_descriptor.mip_level_count = map.mip_levels;
    image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        address_mode_u: ImageAddressMode::Repeat,
        address_mode_v: ImageAddressMode::Repeat,
        anisotropy_clamp: 8,
        ..ImageSamplerDescriptor::linear()
    });
    image
}

pub(super) fn upload(
    world: &mut World,
    assets: &mut SceneAssets,
    map: &BakedMap,
    display: bool,
) -> Handle<Image> {
    let handle = world
        .resource_mut::<Assets<Image>>()
        .add(image(map, display));
    assets.images.push(handle.clone());
    handle
}

pub(super) fn material(
    world: &mut World,
    assets: &mut SceneAssets,
    bake: &BakedRecipe,
    document: &Document,
    channel: Option<MapChannel>,
) -> Handle<StandardMaterial> {
    let uv_transform = Affine2::from_scale_angle_translation(
        Vec2::splat(document.view.repeats),
        0.0,
        Vec2::from_array(document.view.offset),
    );
    let mut material = StandardMaterial {
        uv_transform,
        cull_mode: None,
        double_sided: true,
        perceptual_roughness: document.surface.roughness,
        metallic: if bake.map(MapChannel::Arm).is_some() {
            document.surface.metallic
        } else {
            0.0
        },
        ..default()
    };
    if let Some(map) = channel.and_then(|channel| bake.map(channel)) {
        material.base_color_texture = Some(upload(world, assets, map, true));
        material.unlit = true;
        return world
            .resource_mut::<Assets<StandardMaterial>>()
            .add(material);
    }
    let albedo = bake
        .map(MapChannel::Albedo)
        .or_else(|| bake.map(MapChannel::FrontAlbedo))
        .or_else(|| bake.map(MapChannel::Transmittance));
    if let Some(map) = albedo {
        material.base_color_texture = Some(upload(world, assets, map, false));
    }
    let normal = bake
        .map(MapChannel::Normal)
        .or_else(|| bake.map(MapChannel::FrontNormal))
        .or_else(|| bake.map(MapChannel::OpticalNormal));
    if document.view.displacement == 0.0
        && let Some(map) = normal
    {
        let mut map = decoded_normal(map);
        if document.surface.normal_strength != 1.0 {
            for pixel in map.bytes.as_chunks_mut::<4>().0 {
                let n = Vec3::new(
                    (pixel[0] as f32 / 127.5 - 1.0) * document.surface.normal_strength,
                    (pixel[1] as f32 / 127.5 - 1.0) * document.surface.normal_strength,
                    pixel[2] as f32 / 127.5 - 1.0,
                )
                .normalize_or_zero();
                for i in 0..3 {
                    pixel[i] = ((n[i] * 0.5 + 0.5) * 255.0).round() as u8;
                }
            }
        }
        material.normal_map_texture = Some(upload(world, assets, &map, false));
    }
    if let Some(map) = bake.map(MapChannel::Arm) {
        let mut map = map.clone();
        for pixel in map.bytes.as_chunks_mut::<4>().0 {
            pixel[0] =
                (255.0 - (255 - pixel[0]) as f32 * document.surface.ao_strength).round() as u8;
        }
        let arm = upload(world, assets, &map, false);
        material.metallic_roughness_texture = Some(arm.clone());
        material.occlusion_texture = Some(arm);
    }
    if let Some(map) = bake.map(MapChannel::Opacity) {
        material.alpha_mode = AlphaMode::Mask(0.5);
        if albedo.is_none() {
            let mut opacity = map.clone();
            if map.encoding.channels() < 4 {
                opacity.bytes = map
                    .bytes
                    .chunks_exact(map.encoding.channels())
                    .flat_map(|p| [200, 200, 200, p[0]])
                    .collect();
            }
            opacity.encoding = PixelEncoding::Srgb8;
            material.base_color_texture = Some(upload(world, assets, &opacity, false));
        }
    }
    if let Some(map) = bake.map(MapChannel::HeightAo) {
        packed_surface(world, assets, &mut material, bake, map, document);
    }
    if let Some(map) = bake.map(MapChannel::LitterSurface) {
        super::terrain::litter(world, assets, &mut material, map, document);
    }
    if bake.recipe == TextureRecipeId::WindowGlass {
        glass_material(world, assets, bake, document, &mut material);
    }
    world
        .resource_mut::<Assets<StandardMaterial>>()
        .add(material)
}

fn packed_surface(
    world: &mut World,
    assets: &mut SceneAssets,
    material: &mut StandardMaterial,
    bake: &BakedRecipe,
    map: &BakedMap,
    document: &Document,
) {
    material.base_color = Color::srgb_from_array(document.surface.pigment);
    material.perceptual_roughness = 0.84 * document.surface.roughness;
    material.metallic = 0.0;
    let mut ao = map.clone();
    ao.encoding = PixelEncoding::Rgba8;
    ao.bytes = map
        .bytes
        .as_chunks::<4>()
        .0
        .iter()
        .flat_map(|p| {
            [
                (255.0 - (255 - p[2]) as f32 * document.surface.ao_strength) as u8,
                255,
                0,
                255,
            ]
        })
        .collect();
    material.occlusion_texture = Some(upload(world, assets, &ao, false));
    if document.view.displacement > 0.0 {
        return;
    }
    let mut normal = ao;
    normal.bytes.clear();
    let mut offset = 0;
    for level in 0..map.mip_levels {
        let side = (map.size >> level) as i32;
        let data = &map.bytes[offset..offset + (side * side * 4) as usize];
        let height = |x: i32, y: i32| {
            adventuresim_procedural_textures::decode_height_ao(
                &data[((y.rem_euclid(side) * side + x.rem_euclid(side)) * 4) as usize..],
            )
        };
        let strength = bake.height_range_metres / (2.0 * bake.tile_metres / side as f32)
            * document.surface.normal_strength;
        for y in 0..side {
            for x in 0..side {
                let n = Vec3::new(
                    -(height(x + 1, y) - height(x - 1, y)) * strength,
                    (height(x, y + 1) - height(x, y - 1)) * strength,
                    1.0,
                )
                .normalize();
                normal.bytes.extend_from_slice(&[
                    ((n.x * 0.5 + 0.5) * 255.0) as u8,
                    ((n.y * 0.5 + 0.5) * 255.0) as u8,
                    ((n.z * 0.5 + 0.5) * 255.0) as u8,
                    255,
                ]);
            }
        }
        offset += (side * side * 4) as usize;
    }
    material.normal_map_texture = Some(upload(world, assets, &normal, false));
}

fn decoded_normal(map: &BakedMap) -> BakedMap {
    let mut decoded = map.clone();
    if matches!(map.encoding, PixelEncoding::Rg8) {
        decoded.encoding = PixelEncoding::Rgba8;
        decoded.bytes = map
            .bytes
            .as_chunks::<2>()
            .0
            .iter()
            .flat_map(|p| {
                let x = p[0] as f32 / 127.5 - 1.0;
                let y = 1.0 - p[1] as f32 / 127.5;
                let z = (1.0 - x * x - y * y).max(0.0).sqrt();
                [
                    p[0],
                    255 - p[1],
                    ((z * 0.5 + 0.5) * 255.0).round() as u8,
                    255,
                ]
            })
            .collect();
    }
    decoded
}

fn glass_material(
    world: &mut World,
    assets: &mut SceneAssets,
    bake: &BakedRecipe,
    document: &Document,
    material: &mut StandardMaterial,
) {
    let glass = &document.texture.window_glass.material_contract;
    material.ior = glass.index_of_refraction;
    material.specular_transmission = glass.specular_transmission;
    material.diffuse_transmission = glass.diffuse_transmission;
    material.thickness = glass.nominal_thickness_metres;
    material.attenuation_color = Color::linear_rgb(
        glass.attenuation_color_linear[0],
        glass.attenuation_color_linear[1],
        glass.attenuation_color_linear[2],
    );
    material.attenuation_distance = glass.attenuation_distance_metres;
    material.metallic = 0.0;
    material.perceptual_roughness = glass.base_perceptual_roughness * document.surface.roughness;
    material.cull_mode = if glass.double_sided {
        None
    } else {
        Some(bevy::render::render_resource::Face::Back)
    };
    if let Some(map) = bake.map(MapChannel::ThicknessRoughness) {
        material.metallic_roughness_texture = Some(upload(world, assets, map, false));
    }
}
