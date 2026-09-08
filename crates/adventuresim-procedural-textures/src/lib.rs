//! Deterministic procedural texture generation shared by every renderer.
//!
//! Recipes live here rather than in a scene or material implementation so
//! each surface family can be reviewed and iterated independently.

#![cfg_attr(test, allow(clippy::chunks_exact_to_as_chunks))]

mod bake;
mod beech_leaf;
mod blackthorn_leaf;
pub mod building;
mod catalogue;
pub use bake::{BakedMap, BakedRecipe, MapChannel, PixelEncoding};
mod clay_roof_tile;
mod crenellation_mask;
mod dressed_stone;
mod dry_white_oak_leaf;
mod ironwork;
mod lead_sheet;
mod normal;
mod palette;
mod parameters;
pub use parameters::{
    BakeResolution, ControlBounds, ControlPath, LeafSpecies, ParameterError, TextureParameters,
};
mod plank_floor;
pub use palette::{MasonryColors, SrgbColor};
mod rock;
mod slate_roof;
mod timber_shingle;
mod window_glass;

pub use catalogue::{
    PROCEDURAL_TEXTURE_CATALOGUE, TextureFamily, TextureOutput, TextureRecipeDescriptor,
    TextureRecipeId, TextureRecipeStatus,
};
use rock::generate_rock_textures;
pub use rock::{ROCK_HEIGHT_RANGE_METRES, ROCK_TEXTURE_SIZE, ROCK_TILE_METRES};

use bevy::{
    asset::{Assets, RenderAssetUsages},
    image::{Image, ImageAddressMode, ImageSampler, ImageSamplerDescriptor},
    math::{FloatExt, Vec2, Vec3},
    prelude::{Handle, IVec2, Resource},
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};
use fabelgeist_determinism::inclusive_unit_f32;

fn unit_hash(value: u64) -> f32 {
    inclusive_unit_f32(value)
}

const TEXTURE_SIZE: u32 = 256;
const OAK_BARK_TEXTURE_SIZE: u32 = 1024;
const OAK_BARK_AO_SIZE: u32 = 512;
const OAK_BARK_AO_DIRECTIONS: [(i32, i32); 4] = [(1, 0), (0, 1), (-1, 0), (0, -1)];
const OAK_BARK_AO_STEPS: [i32; 4] = [1, 4, 12, 32];
pub const FOREST_SOIL_TEXTURE_SIZE: u32 = 1024;
const FOREST_SOIL_AO_SIZE: u32 = 512;
const FOREST_SOIL_AO_DIRECTIONS: [(i32, i32); 4] = [(1, 0), (0, 1), (-1, 0), (0, -1)];
const FOREST_SOIL_AO_STEPS: [i32; 4] = [1, 4, 12, 32];
pub const FOREST_SOIL_TILE_METRES: f32 = 2.0;
pub const FOREST_SOIL_HEIGHT_RANGE_METRES: f32 = 0.028;
pub const FOREST_LITTER_TILE_METRES: f32 = 4.0;
pub const FOREST_LITTER_HEIGHT_RANGE_METRES: f32 = 0.016;
const TERRAIN_BLOOD_MASK_SIZE: u32 = 1024;

#[derive(Clone, Debug)]
pub struct LeafTextureSet {
    pub opacity: Handle<Image>,
    pub front_albedo: Handle<Image>,
    pub back_albedo: Handle<Image>,
    pub front_normal: Handle<Image>,
    pub back_normal: Handle<Image>,
    pub height: Handle<Image>,
    pub arm: Handle<Image>,
}

#[derive(Clone, Debug)]
pub struct SurfaceTextureSet {
    pub albedo: Handle<Image>,
    /// OpenGL tangent-space normal: +X follows U, +Y opposes image-row V.
    /// Custom projections must transform both components into their world basis.
    pub normal_gl: Handle<Image>,
    pub height: Handle<Image>,
    pub arm: Handle<Image>,
}

#[derive(Clone, Debug)]
pub struct GlassTextureSet {
    /// RGB attenuation/transmittance tint. Alpha is deliberately opaque: this
    /// is not an alpha-blend fallback texture.
    pub transmittance: Handle<Image>,
    /// Optical surface distortion encoded as an OpenGL tangent-space normal.
    pub optical_normal_gl: Handle<Image>,
    /// R is normalized per-texel thickness variation and G is perceptual
    /// roughness. The current Bevy `StandardMaterial` samples G through its
    /// metallic/roughness binding but cannot consume R as thickness; runtime
    /// thickness is the contract's nominal 3.2 mm scalar. R remains evidence
    /// and future custom-shader data rather than a claim about current output.
    pub thickness_roughness: Handle<Image>,
}

#[derive(Clone, Debug)]
pub struct BarkTextureSet {
    pub height_ao: Handle<Image>,
}

#[derive(Clone, Debug)]
pub struct GroundTextureSet {
    pub height_ao: Handle<Image>,
    pub litter_surface: Handle<Image>,
    pub litter_normal: Handle<Image>,
}

#[derive(Resource, Clone, Debug)]
pub struct ProceduralTextureAssets {
    pub oak_leaf: LeafTextureSet,
    pub dry_oak_leaf: LeafTextureSet,
    pub hazel_leaf: LeafTextureSet,
    pub blackthorn_leaf: LeafTextureSet,
    pub hawthorn_leaf: LeafTextureSet,
    pub beech_leaf: LeafTextureSet,
    pub oak_bark: BarkTextureSet,
    pub forest_soil: GroundTextureSet,
    pub rock: SurfaceTextureSet,
    pub lime_plaster: SurfaceTextureSet,
    pub hewn_oak: SurfaceTextureSet,
    pub wattle_and_daub: SurfaceTextureSet,
    pub handmade_brick: SurfaceTextureSet,
    pub rubble_masonry: SurfaceTextureSet,
    pub dressed_stone: SurfaceTextureSet,
    pub clay_roof_tile: SurfaceTextureSet,
    pub slate_roof: SurfaceTextureSet,
    pub timber_shingle: SurfaceTextureSet,
    pub plank_floor: SurfaceTextureSet,
    pub lead_sheet: SurfaceTextureSet,
    pub ironwork: SurfaceTextureSet,
    pub window_glass: GlassTextureSet,
    /// Render-only shell-LOD crown silhouette; not close geometry or collision.
    pub crenellation_mask: Handle<Image>,
    pub terrain_blood_mask: Handle<Image>,
}

#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
pub struct LeafRecipe {
    pub blade: [u8; 3],
    pub vein: [u8; 3],
    pub back_blade: [u8; 3],
    pub roughness: u8,
}

impl LeafRecipe {
    const WHITE_OAK: Self = Self {
        blade: [76, 111, 48],
        vein: [139, 157, 76],
        back_blade: [91, 116, 65],
        roughness: 219,
    };

    const DRY_WHITE_OAK: Self = Self {
        blade: [126, 91, 49],
        vein: [103, 73, 40],
        back_blade: [116, 94, 65],
        roughness: 236,
    };

    const HAZEL: Self = Self {
        blade: [66, 112, 48],
        vein: [129, 154, 75],
        back_blade: [82, 119, 61],
        roughness: 216,
    };

    const BLACKTHORN: Self = Self {
        blade: [61, 103, 42],
        vein: [117, 139, 66],
        back_blade: [77, 111, 56],
        roughness: 220,
    };

    const HAWTHORN: Self = Self {
        blade: [72, 113, 44],
        vein: [132, 151, 68],
        back_blade: [84, 119, 58],
        roughness: 217,
    };

    const BEECH: Self = Self {
        blade: [67, 108, 46],
        vein: [126, 147, 72],
        back_blade: [81, 117, 59],
        roughness: 214,
    };
}

pub fn generate_procedural_textures(
    params: &crate::TextureParameters,
    images: &mut Assets<Image>,
) -> ProceduralTextureAssets {
    ProceduralTextureAssets {
        oak_leaf: generate_leaf_textures(params, images, crate::LeafSpecies::WhiteOak),
        dry_oak_leaf: generate_leaf_textures(params, images, crate::LeafSpecies::DryWhiteOak),
        hazel_leaf: generate_leaf_textures(params, images, crate::LeafSpecies::Hazel),
        blackthorn_leaf: generate_leaf_textures(params, images, crate::LeafSpecies::Blackthorn),
        hawthorn_leaf: generate_leaf_textures(params, images, crate::LeafSpecies::Hawthorn),
        beech_leaf: generate_leaf_textures(params, images, crate::LeafSpecies::Beech),
        oak_bark: generate_oak_bark_texture(params, images),
        forest_soil: generate_forest_soil_texture(params, images),
        rock: generate_rock_textures(params, images),
        lime_plaster: generate_lime_plaster_textures(params, images),
        hewn_oak: generate_hewn_oak_textures(params, images),
        wattle_and_daub: generate_wattle_and_daub_textures(params, images),
        handmade_brick: generate_handmade_brick_textures(params, images),
        rubble_masonry: generate_rubble_masonry_textures(params, images),
        dressed_stone: generate_dressed_stone_textures(params, images),
        clay_roof_tile: generate_clay_roof_tile_textures(params, images),
        slate_roof: generate_slate_roof_textures(params, images),
        timber_shingle: generate_timber_shingle_textures(params, images),
        plank_floor: generate_plank_floor_textures(params, images),
        lead_sheet: generate_lead_sheet_textures(params, images),
        ironwork: generate_ironwork_textures(params, images),
        window_glass: generate_window_glass_textures(params, images),
        crenellation_mask: generate_crenellation_mask(params, images),
        terrain_blood_mask: images.add(empty_terrain_blood_mask()),
    }
}

fn empty_terrain_blood_mask() -> Image {
    let pixels = vec![0; (TERRAIN_BLOOD_MASK_SIZE * TERRAIN_BLOOD_MASK_SIZE) as usize];
    let mut image = Image::new(
        Extent3d {
            width: TERRAIN_BLOOD_MASK_SIZE,
            height: TERRAIN_BLOOD_MASK_SIZE,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        pixels,
        TextureFormat::R8Unorm,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    );
    image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        address_mode_u: ImageAddressMode::ClampToEdge,
        address_mode_v: ImageAddressMode::ClampToEdge,
        ..ImageSamplerDescriptor::linear()
    });
    image
}

mod foliage;
pub use hewn_oak::grain::Knot;
pub use wattle_and_daub::CapsuleLayer;
mod ground;
mod handmade_brick;
mod hawthorn_leaf;
mod hazel_leaf;
mod hewn_oak;
mod image;
mod leaf_pixels;
mod lime_plaster;
mod rubble_masonry;
mod surface;
mod wattle_and_daub;

pub use clay_roof_tile::{
    CLAY_ROOF_TILE_HEIGHT_RANGE_METRES, CLAY_ROOF_TILE_TEXTURE_SIZE, CLAY_ROOF_TILE_TILE_METRES,
    generate_clay_roof_tile_textures,
};
pub use crenellation_mask::{
    CRENELLATION_ALPHA_CUTOFF, CRENELLATION_BREASTWORK_HEIGHT_RATIO,
    CRENELLATION_MASK_TEXTURE_SIZE, CRENELLATION_MERLON_DUTY_CYCLE, generate_crenellation_mask,
};
pub use dressed_stone::{
    DRESSED_STONE_COLORS, DRESSED_STONE_HEIGHT_RANGE_METRES, DRESSED_STONE_TEXTURE_SIZE,
    DRESSED_STONE_TILE_METRES, generate_dressed_stone_textures,
};
use foliage::*;
use ground::*;
pub use handmade_brick::{
    HANDMADE_BRICK_COLORS, HANDMADE_BRICK_HEIGHT_RANGE_METRES, HANDMADE_BRICK_TEXTURE_SIZE,
    HANDMADE_BRICK_TILE_METRES, generate_handmade_brick_textures,
};
pub use hewn_oak::{
    HEWN_OAK_COLORS, HEWN_OAK_HEIGHT_RANGE_METRES, HEWN_OAK_TEXTURE_SIZE, HEWN_OAK_TILE_METRES,
    HewnOakColors, generate_hewn_oak_textures,
};
use image::*;
pub use ironwork::{
    IRONWORK_HEIGHT_RANGE_METRES, IRONWORK_TEXTURE_SIZE, IRONWORK_TILE_METRES,
    generate_ironwork_textures,
};
pub use lead_sheet::{
    LEAD_SHEET_HEIGHT_RANGE_METRES, LEAD_SHEET_TEXTURE_SIZE, LEAD_SHEET_TILE_METRES,
    generate_lead_sheet_textures,
};
pub use lime_plaster::{
    LIME_PLASTER_HEIGHT_RANGE_METRES, LIME_PLASTER_REFERENCE_SRGB, LIME_PLASTER_TEXTURE_SIZE,
    LIME_PLASTER_TILE_METRES, generate_lime_plaster_textures,
};
pub use plank_floor::{
    PLANK_FLOOR_HEIGHT_RANGE_METRES, PLANK_FLOOR_TEXTURE_SIZE, PLANK_FLOOR_TILE_METRES,
    generate_plank_floor_textures,
};
pub use rubble_masonry::{
    RUBBLE_MASONRY_HEIGHT_RANGE_METRES, RUBBLE_MASONRY_TEXTURE_SIZE, RUBBLE_MASONRY_TILE_METRES,
    generate_rubble_masonry_textures,
};
pub use slate_roof::{
    SLATE_ROOF_HEIGHT_RANGE_METRES, SLATE_ROOF_TEXTURE_SIZE, SLATE_ROOF_TILE_METRES,
    generate_slate_roof_textures,
};
use surface::*;
pub use timber_shingle::{
    TIMBER_SHINGLE_HEIGHT_RANGE_METRES, TIMBER_SHINGLE_TEXTURE_SIZE, TIMBER_SHINGLE_TILE_METRES,
    generate_timber_shingle_textures,
};
pub use wattle_and_daub::{
    WATTLE_AND_DAUB_HEIGHT_RANGE_METRES, WATTLE_AND_DAUB_TEXTURE_SIZE, WATTLE_AND_DAUB_TILE_METRES,
    generate_wattle_and_daub_textures,
};
pub use window_glass::{
    WINDOW_GLASS_MATERIAL_CONTRACT, WINDOW_GLASS_NOMINAL_THICKNESS_METRES,
    WINDOW_GLASS_TEXTURE_SIZE, WINDOW_GLASS_THICKNESS_VARIATION_METRES, WINDOW_GLASS_TILE_METRES,
    WindowGlassMaterialContract, generate_window_glass_textures,
};

#[cfg(test)]
mod tests;

mod common_controls;
pub use common_controls::Parameters;
