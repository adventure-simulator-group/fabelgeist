//! Sprite-card grass: mesh chunks (no instancing, see `mesh_chunks.rs`) of
//! textured plant cards. Each plant is one triangle (the tightest apex-down
//! triangle fitted over its sprite by `tools/pack_foliage_atlas.py`) or,
//! behind a knob, one tight quad. The mesh stores almost nothing per plant:
//! the root position (identical on every vertex of the card), the facing
//! normal, and uv = (corner index, per-plant hash). Everything else happens
//! in the custom material's vertex shader (`card_vertex` in
//! `custom_bindings.wgsl`): a value noise over the world root picks the
//! family (grass, clover, dandelion), hashes pick the sprite and its size,
//! and the sprite table gives the corner's offset and atlas UV. Nothing is
//! uploaded after a chunk is baked, and a placement map, when one exists,
//! slots in as one texture fetch where the noise is.
//!
//! The atlas (`assets/textures/foliage/atlas.png` + `atlas.json`, embedded)
//! is decoded at startup into three images for the mip knob: mip 0 only, a
//! plain box-filtered chain, and a coverage-preserving chain where every
//! level's alpha is scaled per sprite cell until the fraction of texels at
//! or above the cutoff matches mip 0 (the NVTT alpha-test coverage rule).
//! The anisotropy knob rewrites the sampler of all three.
//!
//! Two modes run through all of this, one `CardSet` each: `Cards` with the
//! plant atlas, and `CardsCurved` with the tinted Kenney atlas
//! (`tools/pack_curved_atlas.py`) whose fragment shader bends the sprite
//! inside its card. They share this file on purpose -- placement, meshes,
//! chunk streaming and material are the same code, so what the two
//! benchmarks differ by is the fragment's curve and the atlas it samples.
//! The curved cards carry one extra vertex attribute (uv_b: the per-plant
//! hash and the card's yaw), because its fragment needs a per-plant number
//! and the depth prepass has no normal to take one from.

use std::collections::{HashMap, HashSet};

use bevy::asset::RenderAssetUsages;
use bevy::camera::primitives::Aabb;
use bevy::camera::visibility::{NoAutoAabb, VisibilityRange};
use bevy::image::{
    CompressedImageFormats, ImageAddressMode, ImageFilterMode, ImageSampler,
    ImageSamplerDescriptor, ImageType,
};
use bevy::light::NotShadowCaster;
use bevy::mesh::{Indices, MeshTag, PrimitiveTopology};
use bevy::prelude::*;

use serde::Deserialize;

use super::{BASE_SEED, GrassEntity, MIN_SLOPE_NORMAL_Y, splitmix64};
use crate::custom_material::{
    AffectorBuffer, CUSTOM_FLAG_LIT, CustomGlobals, CustomMaterial, KIND_CARD, KIND_CARD_CURVED,
    ObjectParams, ObjectParamsBuffer, Sprite, SpriteTable, SpriteTableBuffer,
};
use crate::scene::{PATCH_RADIUS, hash01, terrain_height, terrain_normal};
use crate::settings::{AtlasMips, BenchSettings, CardShape, FoliageAlpha, InstancingMode};
use crate::shading::SunSky;

const ATLAS_PNG: &[u8] = include_bytes!("../../assets/textures/foliage/atlas.png");
const ATLAS_JSON: &str = include_str!("../../assets/textures/foliage/atlas.json");
const CURVED_PNG: &[u8] = include_bytes!("../../assets/textures/foliage/curved_atlas.png");
const CURVED_JSON: &str = include_str!("../../assets/textures/foliage/curved_atlas.json");

/// Chunk edge in metres, the mesh-chunk size.
pub const CHUNK_SIZE: f32 = 8.0;
/// Plants per square metre at density 1.
const CARDS_PER_M2: f32 = 24.0;
/// Tallest card (sprite height x scale 2 x jitter) plus bend: the cull pad.
const CARD_REACH: f32 = 2.4;
/// Alpha at or above this counts as covered for the mip coverage rule; the
/// material's cutoff is the same 0.5.
const COVERAGE_CUTOFF: u8 = 128;
const MAX_BAKES_PER_FRAME: usize = 4;

#[derive(Deserialize)]
struct AtlasShape {
    pos: Vec<[f32; 2]>,
    uv: Vec<[f32; 2]>,
}

#[derive(Deserialize)]
struct AtlasSprite {
    name: String,
    family: usize,
    /// x, y, w, h in atlas pixels.
    cell: [u32; 4],
    tri: AtlasShape,
    quad: AtlasShape,
    area_ratio: f32,
    /// World height of this plant; the curved atlas ramps it with the sprite
    /// order, the plant atlas leaves it to the family.
    #[serde(default)]
    height_m: Option<f32>,
}

#[derive(Deserialize)]
struct AtlasFamily {
    name: String,
    height_m: f32,
    sprites: Vec<usize>,
}

#[derive(Deserialize)]
struct AtlasJson {
    /// Atlas size in pixels.
    atlas: [u32; 2],
    /// Uniform cell size in pixels, when the atlas is a grid (the curved
    /// one): its fragment shader finds its own cell from the atlas uv.
    #[serde(default)]
    cell: Option<[u32; 2]>,
    families: Vec<AtlasFamily>,
    sprites: Vec<AtlasSprite>,
}

/// Everything one card mode owns. `CardGrass` holds one per mode, so the
/// two share every line of the streaming and baking below.
#[derive(Default)]
struct CardSet {
    material: Option<Handle<CustomMaterial>>,
    tag: u32,
    /// Mip 0 only, plain chain, coverage-preserving chain.
    images: Option<[Handle<Image>; 3]>,
    /// This atlas's sprite table; `None` means the shared buffer.
    sprites: Option<SpriteTable>,
    /// Uniform grid cell in atlas uv (zero when the atlas is shelf-packed).
    cell_uv: Vec2,
    applied_mips: Option<AtlasMips>,
    applied_anisotropy: Option<u16>,
    applied_alpha: Option<FoliageAlpha>,
    applied_scale: Option<(f32, CardShape, f32, f32)>,
    /// `None` marks an empty chunk. Keyed by shape via `cache_key`.
    cache: HashMap<(i32, i32), Option<(Handle<Mesh>, usize, Aabb)>>,
    cache_key: Option<(u32, CardShape)>,
    resident_cards: usize,
    last_log: f32,
}

#[derive(Resource, Default)]
struct CardGrass {
    /// Indexed by `curved as usize`: the plant atlas, then the curved one.
    sets: [CardSet; 2],
}

#[derive(Component)]
struct CardChunk {
    key: (i32, i32),
    cards: usize,
}

pub struct GrassCardsPlugin;

impl Plugin for GrassCardsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CardGrass>()
            .add_systems(Startup, load_atlas)
            .add_systems(Update, stream_chunks.after(super::respawn_grass));
    }
}

fn sampler(anisotropy: u16) -> ImageSampler {
    ImageSampler::Descriptor(ImageSamplerDescriptor {
        label: Some("foliage_atlas".into()),
        address_mode_u: ImageAddressMode::ClampToEdge,
        address_mode_v: ImageAddressMode::ClampToEdge,
        mag_filter: ImageFilterMode::Linear,
        min_filter: ImageFilterMode::Linear,
        mipmap_filter: ImageFilterMode::Linear,
        anisotropy_clamp: anisotropy.max(1),
        ..default()
    })
}

/// 2x2 box filter of straight RGBA8 (the atlas is alpha-bled, so averaging
/// colour without alpha weighting pulls no black in).
fn downsample(src: &[u8], w: u32, h: u32) -> (Vec<u8>, u32, u32) {
    let (dw, dh) = ((w / 2).max(1), (h / 2).max(1));
    let mut out = vec![0u8; (dw * dh * 4) as usize];
    for y in 0..dh {
        for x in 0..dw {
            for c in 0..4 {
                let mut sum = 0u32;
                let mut n = 0u32;
                for (dy, dx) in [(0, 0), (0, 1), (1, 0), (1, 1)] {
                    let sx = (x * 2 + dx).min(w - 1);
                    let sy = (y * 2 + dy).min(h - 1);
                    sum += src[((sy * w + sx) * 4 + c) as usize] as u32;
                    n += 1;
                }
                out[((y * dw + x) * 4 + c) as usize] = (sum / n) as u8;
            }
        }
    }
    (out, dw, dh)
}

/// Fraction of a cell's texels at or above the cutoff after scaling alpha.
fn coverage(level: &[u8], w: u32, cell: [u32; 4], scale: f32) -> f32 {
    let [cx, cy, cw, ch] = cell;
    let mut covered = 0u32;
    for y in cy..cy + ch {
        for x in cx..cx + cw {
            let a = level[((y * w + x) * 4 + 3) as usize] as f32 * scale;
            if a >= COVERAGE_CUTOFF as f32 {
                covered += 1;
            }
        }
    }
    covered as f32 / (cw * ch).max(1) as f32
}

/// Builds the mip chain. With `preserve`, each level's alpha inside every
/// sprite cell is scaled (binary search on the scale) so its coverage at
/// the cutoff matches mip 0's for that cell.
fn mip_chain(base: &[u8], w: u32, h: u32, cells: &[[u32; 4]], preserve: bool) -> (Vec<u8>, u32) {
    let targets: Vec<f32> = cells.iter().map(|c| coverage(base, w, *c, 1.0)).collect();
    let mut data = base.to_vec();
    let mut levels = 1;
    let (mut cur, mut cw, mut ch) = (base.to_vec(), w, h);
    while cw > 1 || ch > 1 {
        let (mut next, nw, nh) = downsample(&cur, cw, ch);
        let shift = levels;
        if preserve {
            for (cell, target) in cells.iter().zip(&targets) {
                let scaled = [
                    cell[0] >> shift,
                    cell[1] >> shift,
                    (cell[2] >> shift).max(1),
                    (cell[3] >> shift).max(1),
                ];
                if scaled[0] + scaled[2] > nw
                    || scaled[1] + scaled[3] > nh
                    || scaled[2] * scaled[3] < 4
                {
                    continue;
                }
                let (mut lo, mut hi) = (1.0f32, 8.0f32);
                for _ in 0..10 {
                    let mid = 0.5 * (lo + hi);
                    if coverage(&next, nw, scaled, mid) < *target {
                        lo = mid;
                    } else {
                        hi = mid;
                    }
                }
                let scale = 0.5 * (lo + hi);
                for y in scaled[1]..scaled[1] + scaled[3] {
                    for x in scaled[0]..scaled[0] + scaled[2] {
                        let i = ((y * nw + x) * 4 + 3) as usize;
                        next[i] = (next[i] as f32 * scale).min(255.0) as u8;
                    }
                }
            }
        }
        data.extend_from_slice(&next);
        levels += 1;
        cur = next;
        cw = nw;
        ch = nh;
    }
    (data, levels)
}

/// Decodes an embedded atlas: the three mip variants for the knob, and the
/// sprite table the vertex shader reads. Returns the images, the table and
/// the uniform cell size in atlas uv (zero when the atlas is not a grid).
fn decode_atlas(
    label: &str,
    json: &str,
    png: &[u8],
    images: &mut Assets<Image>,
    anisotropy: u16,
) -> Option<([Handle<Image>; 3], SpriteTable, Vec2)> {
    let atlas: AtlasJson = match serde_json::from_str(json) {
        Ok(atlas) => atlas,
        Err(err) => {
            error!("{label} atlas.json: {err}");
            return None;
        }
    };
    let base = match Image::from_buffer(
        png,
        ImageType::Extension("png"),
        CompressedImageFormats::NONE,
        true,
        sampler(anisotropy),
        RenderAssetUsages::RENDER_WORLD,
    ) {
        Ok(image) => image,
        Err(err) => {
            error!("{label} atlas.png: {err}");
            return None;
        }
    };
    let (w, h) = (
        base.texture_descriptor.size.width,
        base.texture_descriptor.size.height,
    );
    let pixels = base.data.clone()?;
    let cells: Vec<[u32; 4]> = atlas.sprites.iter().map(|s| s.cell).collect();
    let started = bevy::platform::time::Instant::now();
    let plain = mip_chain(&pixels, w, h, &cells, false);
    let preserving = mip_chain(&pixels, w, h, &cells, true);
    let make = |data: Vec<u8>, levels: u32| {
        let mut image = base.clone();
        image.data = Some(data);
        image.texture_descriptor.mip_level_count = levels;
        image
    };
    let handles = [
        images.add(base.clone()),
        images.add(make(plain.0, plain.1)),
        images.add(make(preserving.0, preserving.1)),
    ];
    info!(
        "{label} atlas {}x{} ({} sprites, {} mip levels) decoded and mipped in {} ms",
        w,
        h,
        atlas.sprites.len(),
        plain.1,
        started.elapsed().as_millis()
    );
    for family in &atlas.families {
        let members: Vec<String> = family
            .sprites
            .iter()
            .map(|i| {
                let s = &atlas.sprites[*i];
                format!("{}={:.2}m", s.name, s.height_m.unwrap_or(family.height_m))
            })
            .collect();
        info!("{label} family {}: {}", family.name, members.join(" "));
    }

    // Sprite table: families as contiguous runs, in family order.
    let mut table = SpriteTable::default();
    let mut sprite_count = 0usize;
    let mut starts = [0u32; 4];
    let mut counts = [0u32; 4];
    for (fi, family) in atlas.families.iter().enumerate().take(4) {
        starts[fi] = sprite_count as u32;
        for &si in &family.sprites {
            let s = &atlas.sprites[si];
            let corner = |shape: &AtlasShape, i: usize| {
                Vec4::new(
                    shape.pos[i][0],
                    shape.pos[i][1],
                    shape.uv[i][0],
                    shape.uv[i][1],
                )
            };
            if sprite_count >= table.sprites.len() {
                return None;
            }
            table.sprites[sprite_count] = Sprite {
                tri: [corner(&s.tri, 0), corner(&s.tri, 1), corner(&s.tri, 2)],
                quad: [
                    corner(&s.quad, 0),
                    corner(&s.quad, 1),
                    corner(&s.quad, 2),
                    corner(&s.quad, 3),
                ],
                info: Vec4::new(
                    s.height_m.unwrap_or(family.height_m),
                    s.family as f32,
                    s.area_ratio,
                    0.0,
                ),
            };
            sprite_count += 1;
        }
        counts[fi] = family.sprites.len() as u32;
    }
    table.family_start = UVec4::from_array(starts);
    table.family_count = UVec4::from_array(counts);
    let cell_uv = atlas.cell.map_or(Vec2::ZERO, |[cw, ch]| {
        Vec2::new(
            cw as f32 / atlas.atlas[0] as f32,
            ch as f32 / atlas.atlas[1] as f32,
        )
    });
    Some((handles, table, cell_uv))
}

/// Decodes both atlases at startup: the plant one into the shared sprite
/// table buffer, the curved one into its own.
fn load_atlas(
    mut grass: ResMut<CardGrass>,
    mut images: ResMut<Assets<Image>>,
    mut sprite_table: ResMut<SpriteTableBuffer>,
    settings: Res<BenchSettings>,
) {
    let aniso = settings.atlas_anisotropy;
    if let Some((handles, table, _)) =
        decode_atlas("foliage", ATLAS_JSON, ATLAS_PNG, &mut images, aniso)
    {
        sprite_table.data = table;
        grass.sets[0].images = Some(handles);
        grass.sets[0].applied_anisotropy = Some(aniso);
    }
    if let Some((handles, table, cell_uv)) =
        decode_atlas("curved", CURVED_JSON, CURVED_PNG, &mut images, aniso)
    {
        grass.sets[1].images = Some(handles);
        grass.sets[1].sprites = Some(table);
        grass.sets[1].cell_uv = cell_uv;
        grass.sets[1].applied_anisotropy = Some(aniso);
    }
}

/// Bakes one chunk: uniform random roots at `CARDS_PER_M2 x density`, slope-
/// and disc-gated like the other modes. Roots are chunk-local in xz
/// (relative to the chunk centre, which bevy's range test measures from) and
/// world-absolute in y. Every vertex of a card carries the same root; the
/// normal is the card's facing; uv = (corner, hash). Triangles need no
/// indices; quads get 6 per card. Curved cards get uv_b = (hash, yaw) on
/// top: the bend is a fragment, and the depth prepass drops the normal, so
/// the per-plant phase and the card's facing have to ride along in a
/// channel both passes keep.
fn bake_chunk(
    key: (i32, i32),
    density: f32,
    shape: CardShape,
    curved: bool,
) -> Option<(Mesh, Aabb)> {
    let origin = Vec2::new(key.0 as f32, key.1 as f32) * CHUNK_SIZE;
    let half = CHUNK_SIZE * 0.5;
    let chunk_id = ((key.0 as u32 as u64) << 32) | key.1 as u32 as u64;
    let mut state = splitmix64(BASE_SEED ^ 0x6361_7264 ^ chunk_id);
    let mut next = || {
        state = splitmix64(state);
        hash01(state, 0x7370_7269)
    };
    let corners = match shape {
        CardShape::Triangle => 3,
        CardShape::Quad => 4,
    };
    let candidates = (CHUNK_SIZE * CHUNK_SIZE * CARDS_PER_M2 * density).round() as usize;
    let mut positions = Vec::with_capacity(candidates * corners);
    let mut normals = Vec::with_capacity(candidates * corners);
    let mut uvs = Vec::with_capacity(candidates * corners);
    let mut uv_bs: Vec<[f32; 2]> = Vec::new();
    let mut indices = Vec::new();
    let (mut y_min, mut y_max) = (f32::MAX, f32::MIN);
    let mut cards = 0usize;

    for _ in 0..candidates {
        let local = Vec2::new(next(), next()) * CHUNK_SIZE;
        let world = origin + local;
        if world.length() > PATCH_RADIUS || terrain_normal(world.x, world.y).y < MIN_SLOPE_NORMAL_Y
        {
            continue;
        }
        let y = terrain_height(world.x, world.y);
        y_min = y_min.min(y);
        y_max = y_max.max(y);
        let root = [local.x - half, y, local.y - half];
        let yaw = next() * std::f32::consts::TAU;
        let normal = [-yaw.sin(), 0.0, yaw.cos()];
        let hash = next();
        let base = positions.len() as u32;
        for corner in 0..corners {
            positions.push(root);
            normals.push(normal);
            uvs.push([corner as f32, hash]);
            if curved {
                uv_bs.push([hash, yaw]);
            }
        }
        if shape == CardShape::Quad {
            indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
        }
        cards += 1;
    }

    if cards == 0 {
        return None;
    }
    let aabb = Aabb::from_min_max(
        Vec3::new(-half - CARD_REACH, y_min - 0.2, -half - CARD_REACH),
        Vec3::new(half + CARD_REACH, y_max + CARD_REACH, half + CARD_REACH),
    );
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    if curved {
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_1, uv_bs);
    }
    if shape == CardShape::Quad {
        mesh.insert_indices(Indices::U32(indices));
    }
    Some((mesh, aabb))
}

fn alpha_for(foliage: FoliageAlpha) -> (AlphaMode, bool) {
    match foliage {
        FoliageAlpha::AlphaToCoverage => (AlphaMode::AlphaToCoverage, false),
        FoliageAlpha::Blend => (AlphaMode::Blend, false),
        FoliageAlpha::Mask => (AlphaMode::Mask(0.5), false),
        FoliageAlpha::OpaqueDiscard => (AlphaMode::Opaque, true),
    }
}

/// Keeps the chunks within the geometric range resident (a few bakes per
/// frame, the rest from the cache), despawns the ones that left the window,
/// and pushes the card knobs into the material and the atlas sampler.
fn stream_chunks(
    mut commands: Commands,
    time: Res<Time>,
    settings: Res<BenchSettings>,
    sun_sky: Option<Res<SunSky>>,
    mut grass: ResMut<CardGrass>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut customs: ResMut<Assets<CustomMaterial>>,
    mut objects: ResMut<ObjectParamsBuffer>,
    affector_buffer: Res<AffectorBuffer>,
    sprite_table: Res<SpriteTableBuffer>,
    cameras: Query<&GlobalTransform, With<Camera3d>>,
    existing: Query<(Entity, &CardChunk), With<GrassEntity>>,
) {
    let curved = match settings.instancing {
        InstancingMode::Cards => false,
        InstancingMode::CardsCurved => true,
        _ => return,
    };
    let set = &mut grass.sets[curved as usize];
    let (Some(sun_sky), Some(atlas)) = (sun_sky, set.images.clone()) else {
        return;
    };
    let Ok(camera) = cameras.single() else {
        return;
    };
    // The curve needs the whole cell to lean into, and the cell is a
    // rectangle: the shape knob is the plant atlas's business.
    let shape = if curved {
        CardShape::Quad
    } else {
        settings.card_shape
    };

    let atlas_for = |mips: AtlasMips| match mips {
        AtlasMips::Off => atlas[0].clone(),
        AtlasMips::Plain => atlas[1].clone(),
        AtlasMips::CoveragePreserving => atlas[2].clone(),
    };
    let material = match set.material.clone() {
        Some(handle) => handle,
        None => {
            let tag = objects.push(ObjectParams {
                base_color: Vec4::ONE,
                emissive: Vec4::ZERO,
                params: Vec4::new(
                    0.0,
                    1.0,
                    0.0,
                    if curved { KIND_CARD_CURVED } else { KIND_CARD },
                ),
            });
            let (alpha_mode, force_discard) = alpha_for(settings.foliage_alpha);
            let handle = customs.add(CustomMaterial {
                globals: CustomGlobals {
                    sun_dir: sun_sky.sun_dir.extend(0.0),
                    sun_color: sun_sky.sun_color,
                    sky_strength: sun_sky.sky_strength,
                    flags: CUSTOM_FLAG_LIT,
                    ..default()
                },
                base_texture: Some(atlas_for(settings.atlas_mips)),
                sky_cube: Some(sun_sky.sky_cube.clone()),
                objects: objects.data,
                affectors: affector_buffer.0.clone(),
                sprites: set
                    .sprites
                    .clone()
                    .unwrap_or_else(|| sprite_table.data.clone()),
                displacement: None,
                alpha_mode,
                cull_mode: None,
                force_discard,
            });
            set.material = Some(handle.clone());
            set.tag = tag;
            set.applied_mips = Some(settings.atlas_mips);
            set.applied_alpha = Some(settings.foliage_alpha);
            handle
        }
    };

    // Knobs into the material: alpha mode, atlas image, card scale / shape.
    // Each one touches the material asset, so bind groups follow.
    if set.applied_alpha != Some(settings.foliage_alpha) {
        set.applied_alpha = Some(settings.foliage_alpha);
        if let Some(mut m) = customs.get_mut(&material) {
            let (alpha_mode, force_discard) = alpha_for(settings.foliage_alpha);
            m.alpha_mode = alpha_mode;
            m.force_discard = force_discard;
        }
    }
    if set.applied_mips != Some(settings.atlas_mips) {
        set.applied_mips = Some(settings.atlas_mips);
        if let Some(mut m) = customs.get_mut(&material) {
            m.base_texture = Some(atlas_for(settings.atlas_mips));
        }
    }
    if set.applied_anisotropy != Some(settings.atlas_anisotropy) {
        set.applied_anisotropy = Some(settings.atlas_anisotropy);
        for handle in &atlas {
            if let Some(mut image) = images.get_mut(handle) {
                image.sampler = sampler(settings.atlas_anisotropy);
            }
        }
        if let Some(mut m) = customs.get_mut(&material) {
            m.base_texture = Some(atlas_for(settings.atlas_mips));
        }
    }
    let scale_key = (
        settings.card_scale,
        shape,
        settings.card_curve,
        settings.card_flutter,
    );
    if set.applied_scale != Some(scale_key) {
        set.applied_scale = Some(scale_key);
        if let Some(mut m) = customs.get_mut(&material) {
            // card.zw is the atlas's uniform cell in uv, which is how the
            // curved fragment finds the cell it is standing in.
            m.globals.card = Vec4::new(
                settings.card_scale,
                match shape {
                    CardShape::Triangle => 0.0,
                    CardShape::Quad => 1.0,
                },
                set.cell_uv.x,
                set.cell_uv.y,
            );
            m.globals.curve = Vec4::new(settings.card_curve, settings.card_flutter, 0.0, 0.0);
        }
    }

    let density = settings.grass_density.clamp(0.05, 1.0);
    let cache_key = (density.to_bits(), shape);
    if set.cache_key != Some(cache_key) {
        // The shape changes the mesh itself: drop the cache and the chunks.
        set.cache.clear();
        set.cache_key = Some(cache_key);
        for (entity, _) in &existing {
            commands.entity(entity).despawn();
        }
        set.resident_cards = 0;
        return;
    }
    let range = settings.grass_range.clamp(8.0, 72.0);
    let window = range + CHUNK_SIZE * 0.75;

    let cam = camera.translation();
    let reach = (PATCH_RADIUS / CHUNK_SIZE).ceil() as i32;
    let mut wanted: HashSet<(i32, i32)> = HashSet::new();
    for kz in -reach - 1..=reach {
        for kx in -reach - 1..=reach {
            let centre = Vec2::new(kx as f32 + 0.5, kz as f32 + 0.5) * CHUNK_SIZE;
            if centre.length() > PATCH_RADIUS + CHUNK_SIZE
                || centre.distance(Vec2::new(cam.x, cam.z)) > window
            {
                continue;
            }
            wanted.insert((kx, kz));
        }
    }

    let mut present: HashSet<(i32, i32)> = HashSet::new();
    for (entity, chunk) in &existing {
        if wanted.contains(&chunk.key) {
            present.insert(chunk.key);
        } else {
            set.resident_cards = set.resident_cards.saturating_sub(chunk.cards);
            commands.entity(entity).despawn();
        }
    }

    let mut bakes = 0;
    let mut keys: Vec<(i32, i32)> = wanted.difference(&present).copied().collect();
    keys.sort_by_key(|(kx, kz)| {
        let centre = Vec2::new(*kx as f32 + 0.5, *kz as f32 + 0.5) * CHUNK_SIZE;
        (centre.distance(Vec2::new(cam.x, cam.z)) * 100.0) as i64
    });
    for key in keys {
        let entry = match set.cache.get(&key) {
            Some(entry) => entry.clone(),
            None => {
                if bakes >= MAX_BAKES_PER_FRAME {
                    continue;
                }
                bakes += 1;
                let entry = bake_chunk(key, density, shape, curved).map(|(mesh, aabb)| {
                    let cards = mesh.count_vertices()
                        / match shape {
                            CardShape::Triangle => 3,
                            CardShape::Quad => 4,
                        };
                    (meshes.add(mesh), cards, aabb)
                });
                set.cache.insert(key, entry.clone());
                entry
            }
        };
        let Some((handle, cards, aabb)) = entry else {
            continue;
        };
        set.resident_cards += cards;
        let mut entity = commands.spawn((
            Name::new(format!("grass card chunk {},{}", key.0, key.1)),
            GrassEntity,
            CardChunk { key, cards },
            Mesh3d(handle),
            MeshMaterial3d(material.clone()),
            MeshTag(set.tag),
            Transform::from_xyz(
                (key.0 as f32 + 0.5) * CHUNK_SIZE,
                0.0,
                (key.1 as f32 + 0.5) * CHUNK_SIZE,
            ),
            Visibility::Inherited,
            // Positions are roots only; the pad covers the tallest card.
            aabb,
            NoAutoAabb,
        ));
        let pad = CHUNK_SIZE * core::f32::consts::FRAC_1_SQRT_2;
        // WebGL2 uses the CPU streaming window as a hard range boundary.
        // Bevy 0.19's dither-range uniform layout is smaller than its shader array.
        if !cfg!(feature = "downlevel") && std::env::var("BENCH_MESH_NO_RANGE").is_err() {
            entity.insert(VisibilityRange {
                start_margin: 0.0..0.0,
                end_margin: (range + pad - 3.0).max(1.0)..(range + pad),
                use_aabb: false,
            });
        }
        if !settings.grass_shadows {
            entity.insert(NotShadowCaster);
        }
    }

    let t = time.elapsed_secs();
    if t - set.last_log > 3.0 {
        set.last_log = t;
        info!(
            "grass: {} {} chunks resident, {} k cards ({}), {} cached",
            if curved { "curved cards" } else { "cards" },
            present.len(),
            set.resident_cards / 1000,
            shape.label(),
            set.cache.len()
        );
    }
}
