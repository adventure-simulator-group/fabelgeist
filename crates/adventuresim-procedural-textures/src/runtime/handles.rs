use bevy::prelude::*;

use crate::{
    BarkTextureSet, GlassTextureSet, GroundTextureSet, LeafTextureSet, MapChannel,
    ProceduralTextureAssets, SurfaceTextureSet, TextureRecipeId, empty_terrain_blood_mask,
};

impl ProceduralTextureAssets {
    /// Request committed maps; missing or corrupt assets fail loading, never trigger baking.
    pub fn load(server: &AssetServer, images: &mut Assets<Image>) -> Self {
        use TextureRecipeId::*;
        Self {
            oak_leaf: LeafTextureSet::load(server, WhiteOakLeaf),
            dry_oak_leaf: LeafTextureSet::load(server, DryWhiteOakLeaf),
            hazel_leaf: LeafTextureSet::load(server, HazelLeaf),
            blackthorn_leaf: LeafTextureSet::load(server, BlackthornLeaf),
            hawthorn_leaf: LeafTextureSet::load(server, HawthornLeaf),
            beech_leaf: LeafTextureSet::load(server, BeechLeaf),
            oak_bark: BarkTextureSet {
                height_ao: load_map(server, OakBark, MapChannel::HeightAo),
            },
            forest_soil: GroundTextureSet {
                height_ao: load_map(server, ForestSoil, MapChannel::HeightAo),
                litter_surface: load_map(server, ForestLitter, MapChannel::LitterSurface),
                litter_normal: load_map(server, ForestLitter, MapChannel::Normal),
            },
            rock: SurfaceTextureSet::load(server, Rock),
            lime_plaster: SurfaceTextureSet::load(server, LimePlaster),
            hewn_oak: SurfaceTextureSet::load(server, HewnOak),
            wattle_and_daub: SurfaceTextureSet::load(server, WattleAndDaub),
            handmade_brick: SurfaceTextureSet::load(server, HandmadeBrick),
            rubble_masonry: SurfaceTextureSet::load(server, RubbleMasonry),
            dressed_stone: SurfaceTextureSet::load(server, DressedStone),
            clay_roof_tile: SurfaceTextureSet::load(server, ClayRoofTile),
            slate_roof: SurfaceTextureSet::load(server, SlateRoof),
            timber_shingle: SurfaceTextureSet::load(server, TimberShingle),
            plank_floor: SurfaceTextureSet::load(server, PlankFloor),
            lead_sheet: SurfaceTextureSet::load(server, LeadSheet),
            ironwork: SurfaceTextureSet::load(server, Ironwork),
            window_glass: GlassTextureSet {
                transmittance: load_map(server, WindowGlass, MapChannel::Transmittance),
                optical_normal_gl: load_map(server, WindowGlass, MapChannel::OpticalNormal),
                thickness_roughness: load_map(server, WindowGlass, MapChannel::ThicknessRoughness),
            },
            crenellation_mask: load_map(server, CrenellationMask, MapChannel::Opacity),
            // Blood is transient presentation state, not a shared baked texture.
            terrain_blood_mask: images.add(empty_terrain_blood_mask()),
        }
    }
}

impl LeafTextureSet {
    fn load(server: &AssetServer, recipe: TextureRecipeId) -> Self {
        Self {
            opacity: load_map(server, recipe, MapChannel::Opacity),
            front_albedo: load_map(server, recipe, MapChannel::FrontAlbedo),
            back_albedo: load_map(server, recipe, MapChannel::BackAlbedo),
            front_normal: load_map(server, recipe, MapChannel::FrontNormal),
            back_normal: load_map(server, recipe, MapChannel::BackNormal),
            height: load_map(server, recipe, MapChannel::Height),
            arm: load_map(server, recipe, MapChannel::Arm),
        }
    }
}

impl SurfaceTextureSet {
    fn load(server: &AssetServer, recipe: TextureRecipeId) -> Self {
        Self {
            albedo: load_map(server, recipe, MapChannel::Albedo),
            normal_gl: load_map(server, recipe, MapChannel::Normal),
            height: load_map(server, recipe, MapChannel::Height),
            arm: load_map(server, recipe, MapChannel::Arm),
        }
    }
}

fn load_map(server: &AssetServer, recipe: TextureRecipeId, channel: MapChannel) -> Handle<Image> {
    server.load(format!(
        "{}#{}",
        recipe.runtime_asset_path(),
        channel.slug()
    ))
}
