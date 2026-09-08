//! One-recipe baking with owned pixel data, usable without a renderer or filesystem.

mod wire;

use bevy::{prelude::*, render::render_resource::TextureFormat};
use serde::{Deserialize, Serialize};

use crate::{LeafSpecies, SurfaceTextureSet, TextureParameters, TextureRecipeId};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum MapChannel {
    Albedo,
    Normal,
    Height,
    Arm,
    Opacity,
    FrontAlbedo,
    BackAlbedo,
    FrontNormal,
    BackNormal,
    HeightAo,
    LitterSurface,
    Transmittance,
    OpticalNormal,
    ThicknessRoughness,
}

impl MapChannel {
    pub const fn slug(self) -> &'static str {
        match self {
            Self::Albedo => "albedo",
            Self::Normal => "normal",
            Self::Height => "height",
            Self::Arm => "arm",
            Self::Opacity => "opacity",
            Self::FrontAlbedo => "front-albedo",
            Self::BackAlbedo => "back-albedo",
            Self::FrontNormal => "front-normal",
            Self::BackNormal => "back-normal",
            Self::HeightAo => "height-ao",
            Self::LitterSurface => "surface",
            Self::Transmittance => "transmittance",
            Self::OpticalNormal => "optical-normal",
            Self::ThicknessRoughness => "thickness-roughness",
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum PixelEncoding {
    R8,
    Rg8,
    Rgba8,
    Srgb8,
}

impl PixelEncoding {
    pub const fn channels(self) -> usize {
        match self {
            Self::R8 => 1,
            Self::Rg8 => 2,
            Self::Rgba8 | Self::Srgb8 => 4,
        }
    }

    pub const fn texture_format(self) -> TextureFormat {
        match self {
            Self::R8 => TextureFormat::R8Unorm,
            Self::Rg8 => TextureFormat::Rg8Unorm,
            Self::Rgba8 => TextureFormat::Rgba8Unorm,
            Self::Srgb8 => TextureFormat::Rgba8UnormSrgb,
        }
    }
}

#[derive(Clone)]
pub struct BakedMap {
    pub channel: MapChannel,
    pub size: u32,
    pub mip_levels: u32,
    pub encoding: PixelEncoding,
    /// Complete production mip payload, not only the base image.
    pub bytes: Vec<u8>,
}

#[derive(Clone)]
pub struct BakedRecipe {
    pub recipe: TextureRecipeId,
    pub tile_metres: f32,
    pub height_range_metres: f32,
    pub maps: Vec<BakedMap>,
}

impl BakedRecipe {
    pub fn generate(recipe: TextureRecipeId, params: &TextureParameters) -> Self {
        let mut images = Assets::<Image>::default();
        let handles = generate_maps(recipe, params, &mut images);
        let maps = handles
            .into_iter()
            .map(|(channel, handle)| {
                let mut image = images.get_mut(&handle).expect("generated image exists");
                let encoding = match image.texture_descriptor.format {
                    TextureFormat::R8Unorm => PixelEncoding::R8,
                    TextureFormat::Rg8Unorm => PixelEncoding::Rg8,
                    TextureFormat::Rgba8Unorm => PixelEncoding::Rgba8,
                    TextureFormat::Rgba8UnormSrgb => PixelEncoding::Srgb8,
                    format => panic!("unsupported procedural encoding: {format:?}"),
                };
                BakedMap {
                    channel,
                    size: image.width(),
                    mip_levels: image.texture_descriptor.mip_level_count,
                    encoding,
                    bytes: image.data.take().expect("CPU recipe produces pixels"),
                }
            })
            .collect();
        let (tile_metres, height_range_metres) = physical_scale(recipe, params);
        Self {
            recipe,
            tile_metres,
            height_range_metres,
            maps,
        }
    }

    pub fn map(&self, channel: MapChannel) -> Option<&BakedMap> {
        self.maps.iter().find(|map| map.channel == channel)
    }
}

fn surface_maps(surface: SurfaceTextureSet) -> Vec<(MapChannel, Handle<Image>)> {
    vec![
        (MapChannel::Albedo, surface.albedo),
        (MapChannel::Normal, surface.normal_gl),
        (MapChannel::Height, surface.height),
        (MapChannel::Arm, surface.arm),
    ]
}

fn generate_maps(
    recipe: TextureRecipeId,
    params: &TextureParameters,
    images: &mut Assets<Image>,
) -> Vec<(MapChannel, Handle<Image>)> {
    use TextureRecipeId::*;
    let leaf_species = match recipe {
        WhiteOakLeaf => Some(LeafSpecies::WhiteOak),
        DryWhiteOakLeaf => Some(LeafSpecies::DryWhiteOak),
        HazelLeaf => Some(LeafSpecies::Hazel),
        BlackthornLeaf => Some(LeafSpecies::Blackthorn),
        HawthornLeaf => Some(LeafSpecies::Hawthorn),
        BeechLeaf => Some(LeafSpecies::Beech),
        _ => None,
    };
    if let Some(species) = leaf_species {
        let leaf = crate::foliage::generate_leaf_textures(params, images, species);
        return vec![
            (MapChannel::Opacity, leaf.opacity),
            (MapChannel::FrontAlbedo, leaf.front_albedo),
            (MapChannel::BackAlbedo, leaf.back_albedo),
            (MapChannel::FrontNormal, leaf.front_normal),
            (MapChannel::BackNormal, leaf.back_normal),
            (MapChannel::Height, leaf.height),
            (MapChannel::Arm, leaf.arm),
        ];
    }
    let surface = match recipe {
        Rock => crate::rock::generate_rock_textures(params, images),
        LimePlaster => crate::generate_lime_plaster_textures(params, images),
        HewnOak => crate::generate_hewn_oak_textures(params, images),
        WattleAndDaub => crate::generate_wattle_and_daub_textures(params, images),
        HandmadeBrick => crate::generate_handmade_brick_textures(params, images),
        RubbleMasonry => crate::generate_rubble_masonry_textures(params, images),
        DressedStone => crate::generate_dressed_stone_textures(params, images),
        ClayRoofTile => crate::generate_clay_roof_tile_textures(params, images),
        SlateRoof => crate::generate_slate_roof_textures(params, images),
        TimberShingle => crate::generate_timber_shingle_textures(params, images),
        PlankFloor => crate::generate_plank_floor_textures(params, images),
        LeadSheet => crate::generate_lead_sheet_textures(params, images),
        Ironwork => crate::generate_ironwork_textures(params, images),
        _ => return special_maps(recipe, params, images),
    };
    surface_maps(surface)
}

fn special_maps(
    recipe: TextureRecipeId,
    params: &TextureParameters,
    images: &mut Assets<Image>,
) -> Vec<(MapChannel, Handle<Image>)> {
    use TextureRecipeId::*;
    match recipe {
        OakBark => vec![(
            MapChannel::HeightAo,
            crate::surface::generate_oak_bark_texture(params, images).height_ao,
        )],
        ForestSoil => vec![(
            MapChannel::HeightAo,
            images.add(crate::ground::generate_soil_height_ao(params)),
        )],
        ForestLitter => {
            let (surface, normal) = crate::ground::generate_forest_litter_textures(params);
            vec![
                (MapChannel::LitterSurface, images.add(surface)),
                (MapChannel::Normal, images.add(normal)),
            ]
        }
        WindowGlass => {
            let glass = crate::generate_window_glass_textures(params, images);
            vec![
                (MapChannel::Transmittance, glass.transmittance),
                (MapChannel::OpticalNormal, glass.optical_normal_gl),
                (MapChannel::ThicknessRoughness, glass.thickness_roughness),
            ]
        }
        CrenellationMask => vec![(
            MapChannel::Opacity,
            crate::generate_crenellation_mask(params, images),
        )],
        _ => unreachable!("surface and leaf recipes are dispatched before specialized maps"),
    }
}

fn physical_scale(recipe: TextureRecipeId, p: &TextureParameters) -> (f32, f32) {
    use TextureRecipeId::*;
    match recipe {
        Rock => (p.rock.tile_metres, p.rock.height_range_metres),
        LimePlaster => (
            p.lime_plaster.tile_metres,
            p.lime_plaster.height_range_metres,
        ),
        HewnOak => (p.hewn_oak.tile_metres, p.hewn_oak.height_range_metres),
        WattleAndDaub => (
            p.wattle_and_daub.tile_metres,
            p.wattle_and_daub.height_range_metres,
        ),
        HandmadeBrick => (
            p.handmade_brick.tile_metres,
            p.handmade_brick.height_range_metres,
        ),
        RubbleMasonry => (
            p.rubble_masonry.tile_metres,
            p.rubble_masonry.height_range_metres,
        ),
        DressedStone => (
            p.dressed_stone.tile_metres,
            p.dressed_stone.height_range_metres,
        ),
        ClayRoofTile => (
            p.clay_roof_tile.tile_metres,
            p.clay_roof_tile.height_range_metres,
        ),
        SlateRoof => (p.slate_roof.tile_metres, p.slate_roof.height_range_metres),
        TimberShingle => (
            p.timber_shingle.tile_metres,
            p.timber_shingle.height_range_metres,
        ),
        PlankFloor => (p.plank_floor.tile_metres, p.plank_floor.height_range_metres),
        LeadSheet => (p.lead_sheet.tile_metres, p.lead_sheet.height_range_metres),
        Ironwork => (p.ironwork.tile_metres, p.ironwork.height_range_metres),
        OakBark => (
            p.surface.oak_bark_tile_metres,
            p.surface.oak_bark_height_range_metres,
        ),
        ForestSoil => (
            p.common.forest_soil_tile_metres,
            p.common.forest_soil_height_range_metres,
        ),
        ForestLitter => (
            p.common.forest_litter_tile_metres,
            p.common.forest_litter_height_range_metres,
        ),
        WindowGlass => (
            p.window_glass.tile_metres,
            p.window_glass.thickness_variation_metres,
        ),
        // Atlas recipes have no periodic world-space surface contract.
        _ => (1.0, 0.0),
    }
}
