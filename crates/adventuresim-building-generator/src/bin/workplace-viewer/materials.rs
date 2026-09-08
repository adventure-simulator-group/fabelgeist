//! GPU review uses the production texture recipes with their physical metre scale.
use adventuresim_building_generator::{BUILDING_DETAIL_UV_METRES_PER_UNIT, BuildingLodMaterial};
use adventuresim_procedural_textures::{
    BakedRecipe, MapChannel, TextureParameters, TextureRecipeId,
};
use bevy::math::Affine2;
use bevy::{
    asset::RenderAssetUsages,
    image::{ImageAddressMode, ImageSampler, ImageSamplerDescriptor},
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension},
};

#[derive(Resource, Default)]
struct ReviewMaterialCache(std::collections::HashMap<TextureRecipeId, Handle<StandardMaterial>>);

pub(super) fn material(
    world: &mut World,
    material: BuildingLodMaterial,
) -> Handle<StandardMaterial> {
    let recipe = match material {
        BuildingLodMaterial::Timber | BuildingLodMaterial::InteriorTimber => {
            TextureRecipeId::HewnOak
        }
        BuildingLodMaterial::Floor => TextureRecipeId::PlankFloor,
        BuildingLodMaterial::Roof(_) => TextureRecipeId::ClayRoofTile,
        BuildingLodMaterial::Iron => TextureRecipeId::Ironwork,
        _ => TextureRecipeId::RubbleMasonry,
    };
    world.init_resource::<ReviewMaterialCache>();
    if let Some(handle) = world.resource::<ReviewMaterialCache>().0.get(&recipe) {
        return handle.clone();
    }
    let baked = BakedRecipe::generate(recipe, &TextureParameters::default());
    let mut map = |channel| {
        let map = baked.map(channel).unwrap();
        let mut image = Image::new(
            Extent3d {
                width: map.size,
                height: map.size,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            map.bytes[..map.size as usize * map.size as usize * map.encoding.channels()].to_vec(),
            map.encoding.texture_format(),
            RenderAssetUsages::RENDER_WORLD,
        );
        image.texture_descriptor.mip_level_count = map.mip_levels;
        image.data = Some(map.bytes.clone());
        image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
            address_mode_u: ImageAddressMode::Repeat,
            address_mode_v: ImageAddressMode::Repeat,
            ..default()
        });
        world.resource_mut::<Assets<Image>>().add(image)
    };
    let albedo = map(MapChannel::Albedo);
    let normal = map(MapChannel::Normal);
    let handle = world
        .resource_mut::<Assets<StandardMaterial>>()
        .add(StandardMaterial {
            base_color_texture: Some(albedo),
            normal_map_texture: Some(normal),
            uv_transform: Affine2::from_scale(Vec2::splat(
                BUILDING_DETAIL_UV_METRES_PER_UNIT / baked.tile_metres,
            )),
            perceptual_roughness: 0.88,
            metallic: if recipe == TextureRecipeId::Ironwork {
                0.7
            } else {
                0.0
            },
            ..default()
        });
    world
        .resource_mut::<ReviewMaterialCache>()
        .0
        .insert(recipe, handle.clone());
    handle
}
