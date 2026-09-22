//! Simple, CPU-culled grass: one persistent instance buffer per tier,
//! refilled in place every frame with the tufts that survive a CPU distance
//! band + frustum test, one draw per tier.
//!
//! Main world: `spawn` keeps each tier's tufts in `GrassCulledSource`, sorted
//! by 8 m chunk with a per-chunk fitted AABB, and spawns one `Mesh3d` entity
//! per tier (never frustum culled, so bevy always extracts its mesh and lists
//! it visible). `cull_grass` walks the chunk index against the camera every
//! frame: a chunk wholly inside the tier's band and the frustum is copied as a
//! slice, a chunk wholly outside either is skipped, a straddling chunk is
//! tested tuft by tuft. Survivors land in `GrassCulledVisible`, extracted to
//! the render world by cloning (tens of thousands of 32-byte instances).
//!
//! Render world: `GrassCulledBuffers` holds one `RawBufferVec` per tier,
//! reserved once at the tier's full count and refilled with `write_buffer`
//! (the particles_and_trails pattern: never recreated). Each frame the tier
//! entity's render counterpart gets a `GrassChunkBuffer` over that buffer, so
//! `simple.rs`'s queue and draw commands draw it exactly like a chunk: same
//! pipeline, same tier uniform (fade band, dither) at group 2.

use std::collections::HashMap;

use bevy::camera::primitives::{Aabb, Frustum, Sphere};
use bevy::camera::visibility::{NoFrustumCulling, VisibilitySystems};
use bevy::ecs::query::QueryItem;
use bevy::light::NotShadowCaster;
use bevy::math::Vec3A;
use bevy::platform::time::Instant;
use bevy::prelude::*;
use bevy::render::extract_component::{ExtractComponent, ExtractComponentPlugin};
use bevy::render::extract_resource::{ExtractResource, ExtractResourcePlugin};
use bevy::render::render_resource::{BufferUsages, RawBufferVec};
use bevy::render::renderer::{RenderDevice, RenderQueue};
use bevy::render::sync_component::SyncComponent;
use bevy::render::{Render, RenderApp, RenderSystems};
use bevy::transform::TransformSystems;
use bevy_eidolon::components::InstanceData;

use super::simple::{CHUNK_SIZE, GrassChunkBuffer};
use super::{GrassEntity, TIER_COUNT, Tier, TierMesh, fitted_aabb};

/// Per-tuft frustum test: a sphere of this radius (plus half the tier
/// footprint) around the tuft's mid height, so a root just outside a frustum
/// plane keeps its blades (up to ~1.3 m tall, leaning) on screen.
const TUFT_FRUSTUM_MARGIN_M: f32 = 1.0;
const TUFT_MID_HEIGHT_M: f32 = 0.65;
/// Stats log cadence, seconds.
const LOG_INTERVAL_SECS: f32 = 3.0;

// ---------------------------------------------------------------------------
// Main world
// ---------------------------------------------------------------------------

/// Marks the one entity per tier; extracted so the render world can find the
/// tier's render entity and hang the instance buffer on it.
#[derive(Component, Clone, Copy)]
pub struct CulledTier {
    pub tier: u32,
}

impl SyncComponent for CulledTier {
    type Target = Self;
}

impl ExtractComponent for CulledTier {
    type QueryData = &'static CulledTier;
    type QueryFilter = ();
    type Out = Self;

    fn extract_component(item: QueryItem<'_, '_, Self::QueryData>) -> Option<Self> {
        Some(*item)
    }
}

/// One 8 m chunk of a tier: a range into the tier's sorted instances and its
/// fitted world bounds.
struct CulledChunk {
    start: usize,
    end: usize,
    aabb: Aabb,
}

struct CulledTierSource {
    name: &'static str,
    /// The tier's tufts, sorted by chunk so a chunk is one contiguous slice.
    instances: Vec<InstanceData>,
    chunks: Vec<CulledChunk>,
    /// The tier's geometric band `[fade_in.x, fade_out.w)`: what the shader
    /// would collapse anyway.
    near: f32,
    far: f32,
    footprint: f32,
}

/// The placement the per-frame cull reads. Absent tiers (dropped by the
/// range, or empty) are `None`.
#[derive(Resource, Default)]
pub struct GrassCulledSource {
    tiers: [Option<CulledTierSource>; TIER_COUNT],
}

/// This frame's survivors per tier; rewritten by `cull_grass` and cloned into
/// the render world every frame.
#[derive(Resource, Clone, Default, ExtractResource)]
pub struct GrassCulledVisible {
    pub tiers: [Vec<InstanceData>; TIER_COUNT],
    /// Each tier's full tuft count: the one-time GPU buffer reservation.
    pub capacity: [u32; TIER_COUNT],
}

/// What the cull cost last frame, for the log and the stats CSV.
#[derive(Resource, Clone, Copy, Debug, Default)]
pub struct GrassCullStats {
    pub last_ms: f32,
    pub survivors: [u32; TIER_COUNT],
    /// Chunk AABB tests (band + frustum) this frame.
    pub chunk_tests: u32,
    /// Per-tuft tests this frame (the straddling chunks' contents).
    pub tuft_tests: u32,
}

pub struct GrassCulledPlugin;

impl Plugin for GrassCulledPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ExtractComponentPlugin::<CulledTier>::default(),
            ExtractResourcePlugin::<GrassCulledVisible>::default(),
        ))
        .add_systems(
            PostUpdate,
            cull_grass
                .after(TransformSystems::Propagate)
                .after(VisibilitySystems::CheckVisibility),
        );
        app.sub_app_mut(RenderApp)
            .init_resource::<GrassCulledBuffers>()
            .add_systems(
                Render,
                // Before the shared queue so a tier is queued with the length
                // its buffer was given this frame (the queue skips length 0).
                upload_culled_tiers
                    .in_set(RenderSystems::Queue)
                    .before(RenderSystems::QueueMeshes),
            );
    }
}

/// Sorts each tier's tufts by chunk, builds the chunk index and spawns one
/// entity per non-empty tier. Replaces the cull's resources wholesale, so a
/// respawn (mod.rs has just despawned the previous tier entities) starts clean.
pub fn spawn(
    commands: &mut Commands,
    tiers: &[Tier; TIER_COUNT],
    tier_meshes: &[TierMesh],
    batches: [Vec<InstanceData>; TIER_COUNT],
) {
    let mut source = GrassCulledSource::default();
    let mut visible = GrassCulledVisible::default();
    for (tier_index, ((tier, mesh), instances)) in
        tiers.iter().zip(tier_meshes).zip(batches).enumerate()
    {
        if instances.is_empty() {
            continue;
        }
        let total = instances.len();
        let mut buckets: HashMap<(i32, i32), Vec<InstanceData>> = HashMap::new();
        for instance in instances {
            let key = (
                (instance.position.x / CHUNK_SIZE).floor() as i32,
                (instance.position.z / CHUNK_SIZE).floor() as i32,
            );
            buckets.entry(key).or_default().push(instance);
        }
        // Deterministic chunk order; neighbouring chunks land next to each
        // other in memory, which is what the whole-slice copies walk.
        let mut keys: Vec<(i32, i32)> = buckets.keys().copied().collect();
        keys.sort_unstable();
        let mut sorted = Vec::with_capacity(total);
        let mut chunks = Vec::with_capacity(keys.len());
        for key in keys {
            let bucket = buckets.remove(&key).unwrap_or_default();
            let aabb = fitted_aabb(&bucket, tier.footprint());
            let start = sorted.len();
            sorted.extend(bucket);
            chunks.push(CulledChunk {
                start,
                end: sorted.len(),
                aabb,
            });
        }
        info!(
            "grass culled {}: {} tufts in {} chunks of {CHUNK_SIZE} m, band {:.1}..{:.1} m",
            tier.name,
            total,
            chunks.len(),
            tier.fade_in[0],
            tier.fade_out[1],
        );
        visible.capacity[tier_index] = total as u32;
        visible.tiers[tier_index] = Vec::with_capacity(total);
        source.tiers[tier_index] = Some(CulledTierSource {
            name: tier.name,
            instances: sorted,
            chunks,
            near: tier.fade_in[0],
            far: tier.fade_out[1],
            footprint: tier.footprint(),
        });
        commands.spawn((
            Name::new(format!("grass culled {} ({total} tufts)", tier.name)),
            GrassEntity,
            Mesh3d(mesh.handle.clone()),
            Transform::IDENTITY,
            Visibility::Inherited,
            // Always in the view's visible list: the CPU cull below decides
            // what the one draw covers.
            NoFrustumCulling,
            NotShadowCaster,
            CulledTier {
                tier: tier_index as u32,
            },
        ));
    }
    commands.insert_resource(source);
    commands.insert_resource(visible);
    commands.insert_resource(GrassCullStats::default());
}

/// Nearest and farthest distance from `point` to any point of `aabb`.
fn aabb_distance_range(aabb: &Aabb, point: Vec3A) -> (f32, f32) {
    let delta = (point - aabb.center).abs();
    let nearest = (delta - aabb.half_extents).max(Vec3A::ZERO).length();
    let farthest = (delta + aabb.half_extents).length();
    (nearest, farthest)
}

/// The per-frame cull: chunk AABBs against the tier band and the camera
/// frustum, straddling chunks tuft by tuft.
#[expect(clippy::too_many_arguments, reason = "bevy system")]
fn cull_grass(
    mut commands: Commands,
    time: Res<Time>,
    source: Option<Res<GrassCulledSource>>,
    visible: Option<ResMut<GrassCulledVisible>>,
    stats: Option<ResMut<GrassCullStats>>,
    cameras: Query<(&GlobalTransform, &Frustum), With<Camera3d>>,
    tier_entities: Query<(), With<CulledTier>>,
    mut last_log: Local<Option<f32>>,
) {
    let (Some(source), Some(mut visible), Some(mut stats)) = (source, visible, stats) else {
        return;
    };
    if tier_entities.is_empty() {
        // The mode changed: mod.rs despawned the tier entities, so drop the
        // placement and stop culling (the render side draws nothing without
        // the entities).
        commands.remove_resource::<GrassCulledSource>();
        commands.remove_resource::<GrassCulledVisible>();
        commands.remove_resource::<GrassCullStats>();
        return;
    }
    let Some((camera_transform, frustum)) = cameras.iter().next() else {
        return;
    };
    let started = Instant::now();
    let camera = camera_transform.translation();
    let camera_a = Vec3A::from(camera);
    let mut chunk_tests = 0u32;
    let mut tuft_tests = 0u32;

    for (tier_index, tier) in source.tiers.iter().enumerate() {
        let out = &mut visible.tiers[tier_index];
        out.clear();
        let Some(tier) = tier else {
            stats.survivors[tier_index] = 0;
            continue;
        };
        let tuft_radius = TUFT_FRUSTUM_MARGIN_M + tier.footprint * 0.5;
        for chunk in &tier.chunks {
            chunk_tests += 1;
            let (nearest, farthest) = aabb_distance_range(&chunk.aabb, camera_a);
            if farthest < tier.near || nearest >= tier.far {
                continue;
            }
            if !frustum.intersects_obb_identity(&chunk.aabb) {
                continue;
            }
            let slice = &tier.instances[chunk.start..chunk.end];
            let band_full = nearest >= tier.near && farthest < tier.far;
            let frustum_full = frustum.contains_aabb_identity(&chunk.aabb);
            if band_full && frustum_full {
                out.extend_from_slice(slice);
                continue;
            }
            tuft_tests += slice.len() as u32;
            for instance in slice {
                if !band_full {
                    let distance = instance.position.distance(camera);
                    if distance < tier.near || distance >= tier.far {
                        continue;
                    }
                }
                if !frustum_full {
                    let sphere = Sphere {
                        center: Vec3A::from(instance.position + Vec3::Y * TUFT_MID_HEIGHT_M),
                        radius: tuft_radius,
                    };
                    if !frustum.intersects_sphere(&sphere, true) {
                        continue;
                    }
                }
                out.push(*instance);
            }
        }
        stats.survivors[tier_index] = out.len() as u32;
    }
    stats.last_ms = started.elapsed().as_secs_f32() * 1000.0;
    stats.chunk_tests = chunk_tests;
    stats.tuft_tests = tuft_tests;

    let now = time.elapsed_secs();
    if last_log.is_none_or(|last| now - last >= LOG_INTERVAL_SECS) {
        *last_log = Some(now);
        let names: Vec<String> = source
            .tiers
            .iter()
            .enumerate()
            .filter_map(|(index, tier)| {
                tier.as_ref()
                    .map(|tier| format!("{} {}", tier.name, stats.survivors[index]))
            })
            .collect();
        info!(
            "grass culled: tufts per tier [{}], tests {} chunks + {} tufts, {:.3} ms",
            names.join(", "),
            chunk_tests,
            tuft_tests,
            stats.last_ms,
        );
    }
}

// ---------------------------------------------------------------------------
// Render world
// ---------------------------------------------------------------------------

/// The persistent per-tier instance buffers. Reserved once at the tier's full
/// count (a respawn at a higher density is the only later growth) and refilled
/// in place by `write_buffer`.
#[derive(Resource)]
struct GrassCulledBuffers {
    tiers: Vec<RawBufferVec<InstanceData>>,
}

impl Default for GrassCulledBuffers {
    fn default() -> Self {
        Self {
            tiers: (0..TIER_COUNT)
                .map(|_| {
                    let mut buffer = RawBufferVec::new(BufferUsages::VERTEX);
                    buffer.set_label(Some("grass_culled_tier_instances"));
                    buffer
                })
                .collect(),
        }
    }
}

/// Refills each present tier's buffer with this frame's survivors and points
/// the tier's render entity at it through `GrassChunkBuffer`, which is what
/// `simple.rs` queues and draws.
fn upload_culled_tiers(
    mut commands: Commands,
    visible: Option<Res<GrassCulledVisible>>,
    mut buffers: ResMut<GrassCulledBuffers>,
    mut tiers: Query<(Entity, &CulledTier, Option<&mut GrassChunkBuffer>)>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
) {
    let Some(visible) = visible else {
        return;
    };
    for (entity, culled, existing) in &mut tiers {
        let tier = culled.tier as usize;
        let Some(buffer) = buffers.tiers.get_mut(tier) else {
            continue;
        };
        // No-op once the capacity is there.
        buffer.reserve(visible.capacity[tier] as usize, &render_device);
        if visible.is_changed() || existing.is_none() {
            buffer.clear();
            buffer.values_mut().extend_from_slice(&visible.tiers[tier]);
            // Returns early on an empty tier, keeping the buffer.
            buffer.write_buffer(&render_device, &render_queue);
        }
        let length = visible.tiers[tier].len() as u32;
        let Some(gpu_buffer) = buffer.buffer() else {
            continue;
        };
        match existing {
            Some(mut chunk) => {
                chunk.length = length;
                if chunk.buffer.id() != gpu_buffer.id() {
                    chunk.buffer = gpu_buffer.clone();
                }
            }
            None => {
                commands.entity(entity).insert(GrassChunkBuffer {
                    buffer: gpu_buffer.clone(),
                    length,
                    tier: culled.tier,
                });
            }
        }
    }
}
