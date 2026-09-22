//! Mesh-chunk grass: no instancing at all. Every 8 m chunk near the camera
//! is one baked mesh (tufts copied into it, world-space heights baked in),
//! drawn through the bench custom material like any other opaque object:
//! bevy's own frustum culling, `VisibilityRange` fade, shadows and prepasses,
//! zero render code. Wind and the affector array run in the custom
//! material's vertex shader behind the per-object grass flag.
//!
//! Budget: a blade is one triangle (root pair + tip, 3 vertices), a tuft is
//! 4x4 of them (48 vertices), and only chunks within the geometric range are
//! resident. The vertex format is still the game's fat 56-byte one.

use std::collections::{HashMap, HashSet};

use bevy::asset::RenderAssetUsages;
use bevy::camera::visibility::VisibilityRange;
use bevy::light::NotShadowCaster;
use bevy::mesh::{Indices, MeshTag, PrimitiveTopology, VertexAttributeValues};
use bevy::prelude::*;

use super::{
    BASE_SEED, BLADE_HEIGHT_M, BLADE_WIDTH_M, CELL_SPACING, JITTER_FRACTION, MIN_SLOPE_NORMAL_Y,
    GrassEntity, Tier, TierId, grass_color, splitmix64, unit_hash,
};
use crate::custom_material::{
    AffectorBuffer, CUSTOM_FLAG_LIT, CustomGlobals, CustomMaterial, KIND_GRASS, KIND_GRASS_MAP,
    ObjectParams, ObjectParamsBuffer, SpriteTableBuffer,
};
use crate::displacement::DisplacementMap;
use crate::scene::{PATCH_RADIUS, hash01, terrain_height, terrain_normal};
use crate::settings::{BenchSettings, InstancingMode};
use crate::shading::SunSky;

pub const CHUNK_SIZE: f32 = 8.0;
const VARIANTS: u64 = 4;
/// Chunk bakes per frame while streaming in.
const MAX_BAKES_PER_FRAME: usize = 4;

/// The mesh-mode tuft footprint and lattice. Its band is irrelevant (bevy's
/// `VisibilityRange` fades the chunk); the blades themselves come from
/// [`triangle_tuft_mesh`], not the ribbon builder.
const MESH_TIER: Tier = Tier {
    id: TierId::Near,
    name: "mesh",
    fade_in: [0.0, 0.001],
    fade_out: [1.0e5, 1.0e5 + 1.0],
    ribbon_rows: &[0.0, 0.45, 0.82],
    tufts_per_cell_side: 6,
    blades_per_tuft_side: 4,
    cell_spacing: CELL_SPACING,
    casts_shadows: true,
    width_compensation: 1.2,
    seed_heads: false,
};

struct TuftVariant {
    positions: Vec<Vec3>,
    normals: Vec<Vec3>,
    uvs: Vec<[f32; 2]>,
    roots: Vec<Vec2>,
    colors: Vec<[f32; 4]>,
    indices: Vec<u32>,
}

#[derive(Resource, Default)]
struct MeshGrass {
    /// One material (and object tag) per bend source: [analytic, map].
    materials: [Option<(Handle<CustomMaterial>, u32)>; 2],
    variants: Vec<TuftVariant>,
    /// `None` marks a chunk with no tufts (slope / disc edge): nothing to draw.
    cache: HashMap<(i32, i32), Option<(Handle<Mesh>, usize)>>,
    cache_density: u32,
    resident_vertices: usize,
    last_log: f32,
}

#[derive(Component)]
struct MeshChunk {
    key: (i32, i32),
    vertices: usize,
}

pub struct GrassMeshChunksPlugin;

impl Plugin for GrassMeshChunksPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MeshGrass>()
            .add_systems(Update, stream_chunks.after(super::respawn_grass));
    }
}

/// One triangle per blade: root pair (uv.y = 0, x = 0 | 1) and a leaning tip
/// (uv = 0.5, 1). uv_b = the blade root in tuft space so the vertex shader
/// can widen the root pair; color.a = the per-blade hash the bend uses.
fn triangle_tuft_mesh(seed: u64) -> Mesh {
    let side = MESH_TIER.blades_per_tuft_side as usize;
    let centre = (side - 1) as f32 * 0.5;
    let spacing = MESH_TIER.footprint() / side as f32;
    let pigment = grass_color().to_linear();
    let mut positions = Vec::with_capacity(side * side * 3);
    let mut normals = Vec::with_capacity(side * side * 3);
    let mut uvs = Vec::with_capacity(side * side * 3);
    let mut roots = Vec::with_capacity(side * side * 3);
    let mut colors = Vec::with_capacity(side * side * 3);
    let mut indices = Vec::with_capacity(side * side * 3);
    for index in 0..side * side {
        let row = index / side;
        let column = index % side;
        let hash = splitmix64(index as u64 ^ seed ^ 0x8d12_6f4a_0bc3_7791);
        let jitter_x = (unit_hash(hash) - 0.5) * spacing * 0.6;
        let jitter_z = (unit_hash(splitmix64(hash)) - 0.5) * spacing * 0.6;
        let root = Vec3::new(
            (column as f32 - centre) * spacing + jitter_x,
            0.0,
            (row as f32 - centre) * spacing + jitter_z,
        );
        let facing = unit_hash(splitmix64(hash ^ 0x6661_6365)) * std::f32::consts::TAU;
        let side_dir = Vec3::new(facing.cos(), 0.0, facing.sin());
        let normal = Vec3::new(-facing.sin(), 0.0, facing.cos());
        let height = BLADE_HEIGHT_M * (0.55 + unit_hash(splitmix64(hash ^ 0x52a9_f131)) * 0.7);
        let half_width = BLADE_WIDTH_M * 0.5;
        let lean_angle = unit_hash(splitmix64(hash ^ 0x6c65_616e)) * std::f32::consts::TAU;
        let lean = Vec3::new(lean_angle.cos(), 0.0, lean_angle.sin()) * height * 0.12;
        let blade_hash = unit_hash(splitmix64(hash ^ 0x6861_7368));
        let shade = 0.8 + 0.4 * unit_hash(splitmix64(hash ^ 0x7368_6164));
        let color = [pigment.red * shade, pigment.green * shade, pigment.blue * shade, blade_hash];
        let base = positions.len() as u32;
        for (position, uv) in [
            (root - side_dir * half_width, [0.0, 0.0]),
            (root + side_dir * half_width, [1.0, 0.0]),
            (root + Vec3::Y * height + lean, [0.5, 1.0]),
        ] {
            positions.push(position.to_array());
            normals.push(normal.to_array());
            uvs.push(uv);
            roots.push([root.x, root.z]);
            colors.push(color);
        }
        indices.extend_from_slice(&[base, base + 1, base + 2]);
    }
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::RENDER_WORLD);
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_1, roots);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}

fn variant_from_mesh(mesh: &Mesh) -> TuftVariant {
    let positions = match mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
        Some(VertexAttributeValues::Float32x3(v)) => v.iter().map(|p| Vec3::from(*p)).collect(),
        _ => Vec::new(),
    };
    let normals = match mesh.attribute(Mesh::ATTRIBUTE_NORMAL) {
        Some(VertexAttributeValues::Float32x3(v)) => v.iter().map(|n| Vec3::from(*n)).collect(),
        _ => Vec::new(),
    };
    let uvs = match mesh.attribute(Mesh::ATTRIBUTE_UV_0) {
        Some(VertexAttributeValues::Float32x2(v)) => v.clone(),
        _ => Vec::new(),
    };
    let roots = match mesh.attribute(Mesh::ATTRIBUTE_UV_1) {
        Some(VertexAttributeValues::Float32x2(v)) => v.iter().map(|r| Vec2::from(*r)).collect(),
        _ => Vec::new(),
    };
    let colors = match mesh.attribute(Mesh::ATTRIBUTE_COLOR) {
        Some(VertexAttributeValues::Float32x4(v)) => v.clone(),
        _ => Vec::new(),
    };
    let indices = match mesh.indices() {
        Some(Indices::U32(v)) => v.clone(),
        Some(Indices::U16(v)) => v.iter().map(|i| *i as u32).collect(),
        None => Vec::new(),
    };
    TuftVariant {
        positions,
        normals,
        uvs,
        roots,
        colors,
        indices,
    }
}

/// Bakes one chunk: the game's jittered lattice, thinned by density, each
/// tuft a rotated, scaled copy of a variant with the terrain height in its
/// vertices. xz relative to the chunk centre (the entity's translation, which
/// bevy's range test measures from), world y.
fn bake_chunk(key: (i32, i32), density: f32, variants: &[TuftVariant]) -> Mesh {
    let origin = Vec2::new(key.0 as f32, key.1 as f32) * CHUNK_SIZE;
    let centre_offset = Vec2::splat(CHUNK_SIZE * 0.5);
    let spacing = MESH_TIER.cell_spacing;
    let side = MESH_TIER.tufts_per_cell_side as i32;
    let footprint = MESH_TIER.footprint();
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut roots = Vec::new();
    let mut colors = Vec::new();
    let mut indices = Vec::new();

    let cell_min = ((origin - Vec2::splat(spacing)) / spacing).floor();
    let cell_max = ((origin + Vec2::splat(CHUNK_SIZE + spacing)) / spacing).ceil();
    for z in cell_min.y as i32..=cell_max.y as i32 {
        for x in cell_min.x as i32..=cell_max.x as i32 {
            let cell = ((x as u32 as u64) << 32) | z as u32 as u64;
            let cell_hash = splitmix64(BASE_SEED ^ 0x6d65_7368 ^ cell);
            let gate = Vec2::new(
                (x as f32 + (hash01(cell_hash, 0x39bd_7f21) - 0.5) * JITTER_FRACTION) * spacing,
                (z as f32 + (hash01(cell_hash, 0xe651_34aa) - 0.5) * JITTER_FRACTION) * spacing,
            );
            if gate.length() > PATCH_RADIUS || terrain_normal(gate.x, gate.y).y < MIN_SLOPE_NORMAL_Y {
                continue;
            }
            let cell_origin = Vec2::new(x as f32, z as f32) * spacing
                - Vec2::splat((side - 1) as f32 * 0.5 * footprint);
            for tuft_z in 0..side {
                for tuft_x in 0..side {
                    let tuft_hash =
                        splitmix64(cell_hash ^ (((tuft_x as u64) << 17) | ((tuft_z as u64) << 3)));
                    if density < 1.0 && hash01(tuft_hash, 0x6465_6e73) >= density {
                        continue;
                    }
                    let jitter = Vec2::new(hash01(tuft_hash, 1) - 0.5, hash01(tuft_hash, 2) - 0.5)
                        * footprint
                        * 0.35;
                    let centre =
                        cell_origin + Vec2::new(tuft_x as f32, tuft_z as f32) * footprint + jitter;
                    let local = centre - origin;
                    if local.x < 0.0 || local.x >= CHUNK_SIZE || local.y < 0.0 || local.y >= CHUNK_SIZE {
                        continue;
                    }
                    if centre.length() > PATCH_RADIUS
                        || terrain_normal(centre.x, centre.y).y < MIN_SLOPE_NORMAL_Y
                    {
                        continue;
                    }
                    let variant = &variants[(tuft_hash % VARIANTS) as usize];
                    let yaw = hash01(tuft_hash, 0x0079_6177) * std::f32::consts::TAU;
                    let scale = 0.85 + hash01(tuft_hash, 0x7363) * 0.3;
                    let rotation = Quat::from_rotation_y(yaw);
                    let local = local - centre_offset;
                    let offset = Vec3::new(local.x, terrain_height(centre.x, centre.y), local.y);
                    let base = positions.len() as u32;
                    for (i, p) in variant.positions.iter().enumerate() {
                        positions.push((rotation * (*p * scale) + offset).to_array());
                        normals.push((rotation * variant.normals[i]).to_array());
                        uvs.push(variant.uvs[i]);
                        let root = variant.roots[i] * scale;
                        let root = rotation * Vec3::new(root.x, 0.0, root.y);
                        roots.push([root.x + offset.x, root.z + offset.z]);
                        colors.push(variant.colors[i]);
                    }
                    indices.extend(variant.indices.iter().map(|i| base + i));
                }
            }
        }
    }

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::RENDER_WORLD);
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_1, roots);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}

/// Keeps the chunks within the geometric range of the camera resident:
/// bakes (or reuses from the cache) the missing ones, a few per frame, and
/// despawns the ones that left the window. Everything it spawns is a
/// `GrassEntity`, so a settings change tears it down like the other modes.
fn stream_chunks(
    mut commands: Commands,
    time: Res<Time>,
    settings: Res<BenchSettings>,
    sun_sky: Option<Res<SunSky>>,
    mut grass: ResMut<MeshGrass>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut customs: ResMut<Assets<CustomMaterial>>,
    mut objects: ResMut<ObjectParamsBuffer>,
    affector_buffer: Res<AffectorBuffer>,
    sprite_table: Res<SpriteTableBuffer>,
    map: Option<Res<DisplacementMap>>,
    cameras: Query<&GlobalTransform, With<Camera3d>>,
    existing: Query<(Entity, &MeshChunk), With<GrassEntity>>,
) {
    let use_map = match settings.instancing {
        InstancingMode::MeshChunks => false,
        InstancingMode::MeshChunksMap => true,
        _ => return,
    };
    let Some(sun_sky) = sun_sky else {
        return;
    };
    let Ok(camera) = cameras.single() else {
        return;
    };
    let map = match (use_map, map) {
        (true, Some(map)) => Some(map.clone()),
        (true, None) => return,
        (false, _) => None,
    };

    let slot = use_map as usize;
    let (material, tag) = match grass.materials[slot].clone() {
        Some(entry) => entry,
        None => {
            let tag = objects.push(ObjectParams {
                base_color: Vec4::ONE,
                emissive: Vec4::ZERO,
                params: Vec4::new(0.0, 1.0, 0.0, if use_map { KIND_GRASS_MAP } else { KIND_GRASS }),
            });
            let handle = customs.add(CustomMaterial {
                globals: CustomGlobals {
                    sun_dir: sun_sky.sun_dir.extend(0.0),
                    sun_color: sun_sky.sun_color,
                    sky_strength: sun_sky.sky_strength,
                    flags: CUSTOM_FLAG_LIT,
                    map: map
                        .as_ref()
                        .map(|m| m.mapping.with_w(settings.trample_scorch))
                        .unwrap_or_default(),
                    ..default()
                },
                base_texture: None,
                sky_cube: Some(sun_sky.sky_cube.clone()),
                objects: objects.handle.clone(),
                affectors: affector_buffer.0.clone(),
                sprites: sprite_table.handle.clone(),
                displacement: map.as_ref().map(|m| m.image.clone()),
                alpha_mode: AlphaMode::Opaque,
                cull_mode: None,
                force_discard: false,
            });
            grass.materials[slot] = Some((handle.clone(), tag));
            (handle, tag)
        }
    };
    // The resolution knob recreates the map image: follow it.
    if let Some(map) = &map
        && customs.get(&material).is_some_and(|m| m.displacement.as_ref() != Some(&map.image))
        && let Some(mut m) = customs.get_mut(&material)
    {
        m.displacement = Some(map.image.clone());
        m.globals.map = map.mapping.with_w(settings.trample_scorch);
    }
    if grass.variants.is_empty() {
        grass.variants = (0..VARIANTS)
            .map(|v| variant_from_mesh(&triangle_tuft_mesh(splitmix64(BASE_SEED ^ 0x7661_7269 ^ v))))
            .collect();
        let v = &grass.variants[0];
        info!(
            "grass mesh: tuft variant {} verts / {} tris, {} tufts per cell at density 1",
            v.positions.len(),
            v.indices.len() / 3,
            MESH_TIER.tufts_per_cell_side * MESH_TIER.tufts_per_cell_side
        );
    }
    let density = settings.grass_density.clamp(0.05, 1.0);
    if grass.cache_density != density.to_bits() {
        grass.cache.clear();
        grass.cache_density = density.to_bits();
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
            grass.resident_vertices = grass.resident_vertices.saturating_sub(chunk.vertices);
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
        let entry = match grass.cache.get(&key) {
            Some(entry) => entry.clone(),
            None => {
                if bakes >= MAX_BAKES_PER_FRAME {
                    continue;
                }
                bakes += 1;
                let mesh = bake_chunk(key, density, &grass.variants);
                let vertices = mesh.count_vertices();
                let entry = (vertices > 0).then(|| (meshes.add(mesh), vertices));
                grass.cache.insert(key, entry.clone());
                entry
            }
        };
        let Some((handle, vertices)) = entry else {
            continue;
        };
        grass.resident_vertices += vertices;
        let mut entity = commands.spawn((
            Name::new(format!("grass mesh chunk {},{}", key.0, key.1)),
            GrassEntity,
            MeshChunk { key, vertices },
            Mesh3d(handle),
            MeshMaterial3d(material.clone()),
            MeshTag(tag),
            Transform::from_xyz(
                (key.0 as f32 + 0.5) * CHUNK_SIZE,
                0.0,
                (key.1 as f32 + 0.5) * CHUNK_SIZE,
            ),
            Visibility::Inherited,
        ));
        // Measured from the chunk centre (`use_aabb: true` culled every chunk
        // in bevy 0.19), padded by the half diagonal so no tuft inside the
        // range is cut; the whole chunk dithers out together.
        let pad = CHUNK_SIZE * core::f32::consts::FRAC_1_SQRT_2;
        if std::env::var("BENCH_MESH_NO_RANGE").is_err() {
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
    if t - grass.last_log > 3.0 {
        grass.last_log = t;
        info!(
            "grass mesh: {} chunks resident, {} k vertices, {} cached",
            present.len(),
            grass.resident_vertices / 1000,
            grass.cache.len()
        );
    }
}
