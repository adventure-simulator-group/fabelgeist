use bevy::prelude::*;

use crate::{
    BarkTextureSet, GlassTextureSet, GroundTextureSet, LeafTextureSet, MapChannel,
    ProceduralTextureAssets, SurfaceTextureSet, TextureRecipeId, empty_terrain_blood_mask,
};

impl ProceduralTextureAssets {
    /// Reserve stable image handles for committed maps without requesting any containers.
    pub fn reserve(
        residency: &mut super::ProceduralTextureResidency,
        images: &mut Assets<Image>,
    ) -> Self {
        use TextureRecipeId::*;
        Self {
            oak_leaf: LeafTextureSet::load(residency, images, WhiteOakLeaf),
            dry_oak_leaf: LeafTextureSet::load(residency, images, DryWhiteOakLeaf),
            hazel_leaf: LeafTextureSet::load(residency, images, HazelLeaf),
            blackthorn_leaf: LeafTextureSet::load(residency, images, BlackthornLeaf),
            hawthorn_leaf: LeafTextureSet::load(residency, images, HawthornLeaf),
            beech_leaf: LeafTextureSet::load(residency, images, BeechLeaf),
            oak_bark: BarkTextureSet {
                height_ao: load_map(residency, images, OakBark, MapChannel::HeightAo),
            },
            forest_soil: GroundTextureSet {
                height_ao: load_map(residency, images, ForestSoil, MapChannel::HeightAo),
                litter_surface: load_map(
                    residency,
                    images,
                    ForestLitter,
                    MapChannel::LitterSurface,
                ),
                litter_normal: load_map(residency, images, ForestLitter, MapChannel::Normal),
            },
            rock: SurfaceTextureSet::load(residency, images, Rock),
            lime_plaster: SurfaceTextureSet::load(residency, images, LimePlaster),
            hewn_oak: SurfaceTextureSet::load(residency, images, HewnOak),
            wattle_and_daub: SurfaceTextureSet::load(residency, images, WattleAndDaub),
            handmade_brick: SurfaceTextureSet::load(residency, images, HandmadeBrick),
            rubble_masonry: SurfaceTextureSet::load(residency, images, RubbleMasonry),
            dressed_stone: SurfaceTextureSet::load(residency, images, DressedStone),
            clay_roof_tile: SurfaceTextureSet::load(residency, images, ClayRoofTile),
            slate_roof: SurfaceTextureSet::load(residency, images, SlateRoof),
            timber_shingle: SurfaceTextureSet::load(residency, images, TimberShingle),
            plank_floor: SurfaceTextureSet::load(residency, images, PlankFloor),
            lead_sheet: SurfaceTextureSet::load(residency, images, LeadSheet),
            ironwork: SurfaceTextureSet::load(residency, images, Ironwork),
            window_glass: GlassTextureSet {
                transmittance: load_map(residency, images, WindowGlass, MapChannel::Transmittance),
                optical_normal_gl: load_map(
                    residency,
                    images,
                    WindowGlass,
                    MapChannel::OpticalNormal,
                ),
                thickness_roughness: load_map(
                    residency,
                    images,
                    WindowGlass,
                    MapChannel::ThicknessRoughness,
                ),
            },
            crenellation_mask: load_map(residency, images, CrenellationMask, MapChannel::Opacity),
            // Blood is transient presentation state, not a shared baked texture.
            terrain_blood_mask: images.add(empty_terrain_blood_mask()),
        }
    }
}

impl LeafTextureSet {
    fn load(
        residency: &mut super::ProceduralTextureResidency,
        images: &mut Assets<Image>,
        recipe: TextureRecipeId,
    ) -> Self {
        Self {
            opacity: load_map(residency, images, recipe, MapChannel::Opacity),
            front_albedo: load_map(residency, images, recipe, MapChannel::FrontAlbedo),
            back_albedo: load_map(residency, images, recipe, MapChannel::BackAlbedo),
            front_normal: load_map(residency, images, recipe, MapChannel::FrontNormal),
            back_normal: load_map(residency, images, recipe, MapChannel::BackNormal),
            height: load_map(residency, images, recipe, MapChannel::Height),
            arm: load_map(residency, images, recipe, MapChannel::Arm),
        }
    }
}

impl SurfaceTextureSet {
    fn load(
        residency: &mut super::ProceduralTextureResidency,
        images: &mut Assets<Image>,
        recipe: TextureRecipeId,
    ) -> Self {
        Self {
            albedo: load_map(residency, images, recipe, MapChannel::Albedo),
            normal_gl: load_map(residency, images, recipe, MapChannel::Normal),
            height: load_map(residency, images, recipe, MapChannel::Height),
            arm: load_map(residency, images, recipe, MapChannel::Arm),
        }
    }
}

fn load_map(
    residency: &mut super::ProceduralTextureResidency,
    images: &mut Assets<Image>,
    recipe: TextureRecipeId,
    channel: MapChannel,
) -> Handle<Image> {
    residency.destination(recipe, channel, images)
}
