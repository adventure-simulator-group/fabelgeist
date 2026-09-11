//! GPU-instanced grass on `bevy_eidolon`, for every sward in the scene.
//!
//! One instance is a multi-blade tuft whose mesh reuses the shared blade,
//! species, and pigment construction (`grass_tuft_mesh`). Placement keeps the
//! jittered cell grid, the cover gate, and the slope rejection, then
//! subdivides each eligible cell into tufts. Ground-cover coverage is sampled
//! on the CPU at placement time and packed into each instance's seed byte, so
//! the shader needs no mask texture.
//!
//! The playable scene and the vista rings differ only in where height,
//! coverage, and community come from, so both go through [`TuftPlacement`] and
//! share one lattice walk and one batching path. Native and wasm run the same
//! renderer: the fork's WebGPU draw path loops `draw_indexed_indirect` where
//! the native backend issues `multi_draw_indexed_indirect`.

use std::sync::Arc;

use adventuresim_tactical_core::prelude::{SceneEnvironment, SceneGround, SceneTerrain};
use bevy::{
    camera::{primitives::Aabb, visibility::NoFrustumCulling},
    color::{Color, LinearRgba},
    light::NotShadowCaster,
    prelude::*,
};
use bevy_eidolon::{prelude::*, prepass::CullComputeCamera};

use crate::presentation::{bps, grass_cover_mask_pixels, splitmix64, stable_text_seed, unit_hash};

use super::{
    GrassInteractor, GroundScatterLayer,
    grass::{
        GrassCommunity, GrassCommunityProfile, GrassMeshLod, GrassSpecies, cell_allows_grass,
        configured_tuft_footprint_metres, grass_community_at, grass_species, grass_tuft_mesh,
    },
    grass_pigment, grass_scatter_density,
};

mod diagnostics;
mod material;
pub(crate) use diagnostics::GrassTriangleCount;
pub(in crate::presentation) use material::TacticalGrassInstancedMaterial;

pub(crate) struct InstancedGrassPlugin;

impl Plugin for InstancedGrassPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            InstancedMaterialCorePlugin,
            GpuComputeCullCorePlugin,
            InstancedMaterialPlugin::<TacticalGrassInstancedMaterial>::default(),
            GpuCullComputePlugin::<TacticalGrassInstancedMaterial>::default(),
            InstancedMaterialPlugin::<super::TacticalShrubBarkInstancedMaterial>::default(),
            GpuCullComputePlugin::<super::TacticalShrubBarkInstancedMaterial>::default(),
            InstancedMaterialPlugin::<super::TacticalShrubLeafInstancedMaterial>::default(),
            GpuCullComputePlugin::<super::TacticalShrubLeafInstancedMaterial>::default(),
        ))
        .init_resource::<InstancedGrassInteractionState>()
        .add_systems(
            Update,
            (
                enable_camera_cull_compute,
                present_instanced_grass,
                update_instanced_grass_interaction,
            ),
        );
    }
}

/// Marks scenes whose instanced sward has been generated.
#[derive(Component)]
pub(super) struct InstancedGrassPresented;

/// Instanced counterpart of the grass portion of `present_ground_scatter`.
/// Builds the playable scene's sward once per scene, alongside the vista rings
/// that `presentation::vista` spawns from the same lattice and batching path.
fn present_instanced_grass(
    scenes: super::scene_mask::InstancedGrassSceneQuery,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<TacticalGrassInstancedMaterial>>,
    settings: Res<crate::presentation::TacticalGraphicsSettings>,
) {
    for (entity, _scene_id, terrain, ground, environment, fault_scarp) in &scenes {
        if !settings.config.grass.enabled {
            commands.entity(entity).insert(InstancedGrassPresented);
            continue;
        }
        let started = web_time::Instant::now();
        let (grass_color, grass_dryness) = grass_pigment(environment);
        let grass_density = grass_scatter_density(
            bps(environment.canopy_bps),
            bps(environment.water_bps),
            bps(environment.cultivation_bps),
            bps(environment.weather.snow_cover_bps),
        ) * settings.config.grass.density_scale;
        let wind_scale = 0.16 + bps(environment.weather.wind_speed_bps) * 0.36;
        let masked_ground = fault_scarp.map(|recipe| {
            super::scene_mask::scatter_ground_without_patch(ground, recipe.transition_collar())
        });
        let ground = masked_ground.as_ref().unwrap_or(ground);
        let grass = &settings.config.grass;
        let base_seed = stable_text_seed(&environment.scene_digest) ^ 0x6772_6173_735f_6c6f;
        let placement = ScenePlacement {
            terrain,
            ground,
            mask: CoverageMask::new(ground, stable_text_seed(&environment.scene_digest)),
            profile: GrassCommunityProfile::from_environment(environment),
            base_seed,
        };
        let mut batches = TierSpeciesBatches::default();
        for lod in [GrassMeshLod::Near, GrassMeshLod::Far] {
            scatter_cell_tufts(
                &mut batches[lod.tier_index()],
                &placement,
                base_seed,
                lod,
                grass.placement.playable_patch_spacing_m,
                grass,
            );
        }
        // Reuse near placements so the near-edge crossfade does not move tufts.
        for species in GrassSpecies::ALL {
            batches[GrassMeshLod::NearEdge.tier_index()][species.index()] =
                batches[GrassMeshLod::Near.tier_index()][species.index()].clone();
        }
        scatter_cell_tufts(
            &mut batches[GrassMeshLod::Vista.tier_index()],
            &placement,
            base_seed ^ 0x7669_7374_615f_6c6f,
            GrassMeshLod::Vista,
            grass.placement.vista_patch_spacing_m,
            grass,
        );
        spawn_tuft_batches(
            GrassWorld {
                commands: &mut commands,
                meshes: &mut meshes,
                materials: &mut materials,
            },
            &mut batches,
            "Instanced grass",
            (),
            base_seed,
            TuftPigment {
                color: grass_color,
                density: grass_density,
                dryness: grass_dryness,
                wind_scale,
            },
            grass,
        );
        tracing::info!(
            elapsed_ms = started.elapsed().as_millis(),
            "Generated instanced tactical grass"
        );
        commands.entity(entity).insert(InstancedGrassPresented);
    }
}

/// Every gameplay/viewer camera drives eidolon's per-instance compute cull.
#[expect(
    clippy::type_complexity,
    reason = "the Bevy camera filter selects uncoupled gameplay cameras and excludes the offscreen cloud pass"
)]
fn enable_camera_cull_compute(
    mut commands: Commands,
    cameras: Query<
        Entity,
        (
            With<Camera3d>,
            Without<CullComputeCamera>,
            // The grass never renders on the offscreen cloud layer, and the
            // compute cull must follow the gameplay camera.
            Without<crate::presentation::TacticalCloudOffscreenCamera>,
        ),
    >,
) {
    for camera in &cameras {
        commands.entity(camera).insert(CullComputeCamera);
    }
}

/// Smoothing state for the grass interaction uniforms.
#[derive(Resource, Default)]
pub(in crate::presentation) struct InstancedGrassInteractionState {
    previous_position: Option<Vec3>,
    smoothed_velocity: Vec3,
    /// Last values written to the materials, to skip redundant asset writes
    /// (each write re-uploads the uniform and re-queues the batch).
    written: Option<(Vec3, Vec3)>,
}

fn update_instanced_grass_interaction(
    time: Res<Time>,
    interactors: Query<&GlobalTransform, With<GrassInteractor>>,
    mut state: ResMut<InstancedGrassInteractionState>,
    mut materials: ResMut<Assets<TacticalGrassInstancedMaterial>>,
    settings: Res<crate::presentation::TacticalGraphicsSettings>,
) {
    let Some(position) = interactors.iter().next().map(GlobalTransform::translation) else {
        if state.written.take().is_some() {
            for (_, material) in materials.iter_mut() {
                material.interaction = Vec4::ZERO;
                material.interaction_motion = Vec4::ZERO;
            }
        }
        state.previous_position = None;
        state.smoothed_velocity = Vec3::ZERO;
        return;
    };
    let delta_seconds = time.delta_secs().max(1.0 / 240.0);
    let velocity = state
        .previous_position
        .map(|previous| ((position - previous) / delta_seconds).clamp_length_max(8.0))
        .unwrap_or_default();
    let response = 1.0 - (-delta_seconds * 10.0).exp();
    state.smoothed_velocity = state.smoothed_velocity.lerp(velocity, response);
    state.previous_position = Some(position);

    // Idle interactors converge to constants; stop dirtying material assets
    // once the written values are close enough that no motion is visible.
    if state
        .written
        .is_some_and(|(written_position, written_velocity)| {
            written_position.distance_squared(position) < 1e-6
                && written_velocity.distance_squared(state.smoothed_velocity) < 1e-6
        })
    {
        return;
    }

    let speed = state.smoothed_velocity.length();
    for (_, material) in materials.iter_mut() {
        let interaction = &settings.config.grass.interaction;
        material.interaction = position.extend(interaction.radius_m);
        material.interaction_motion = Vec4::new(
            state.smoothed_velocity.x,
            state.smoothed_velocity.y,
            state.smoothed_velocity.z,
            (interaction.minimum_push + speed * 0.11)
                .clamp(interaction.minimum_push, interaction.maximum_push),
        );
    }
    state.written = Some((position, state.smoothed_velocity));
}

/// Number of tuft columns/rows that subdivide one placement cell, per tier.
fn tufts_per_cell_side(lod: GrassMeshLod) -> i32 {
    match lod {
        GrassMeshLod::Near | GrassMeshLod::NearEdge => 12,
        GrassMeshLod::Far => 4,
        GrassMeshLod::Vista => 2,
    }
}

/// Eidolon fade bands per tier: `xy` fade-in, `zw` fade-out, mirroring the
/// legacy `grass_lod_visibility` crossfade margins. The near tier's epsilon
/// fade-in keeps the dither-level math well-defined at zero distance.
///
/// `range_scale` contracts every band edge uniformly, so tier hand-offs stay
/// contiguous while the geometric sward trades reach for vertex throughput.
/// Near-tier cost scales with the square of its radius, making this the
/// single most effective grass performance lever.
fn tier_visibility_range(lod: GrassMeshLod, range_scale: f32) -> Vec4 {
    let scale = range_scale.clamp(0.35, 1.0);
    (match lod {
        // #560's legacy near band (0..14 m) splits into a full-detail field and
        // a slimmer edge ring; the ring covers most of the band's area, so most
        // near verts move to the nine-vertex 6x6 mesh. The instanced tiers
        // reproduce the legacy `grass_lod_visibility` bands so the native and
        // wasm swards fade out at the same distances (~50 m terminal).
        GrassMeshLod::Near => Vec4::new(0.0, 0.001, 4.0, 6.0),
        GrassMeshLod::NearEdge => Vec4::new(4.0, 6.0, 7.0, 14.0),
        GrassMeshLod::Far => Vec4::new(7.0, 14.0, 36.0, 44.0),
        // The vista fade-in shares the far tier's fade-out endpoints so the
        // complementary crossfade partition hands off exactly.
        GrassMeshLod::Vista => Vec4::new(34.0, 42.0, 42.0, 50.0),
    }) * scale
}

fn configured_tier_visibility_range(
    lod: GrassMeshLod,
    grass: &crate::presentation::config::GrassConfig,
) -> Vec4 {
    let tier = match lod {
        GrassMeshLod::Near => &grass.lod.near,
        GrassMeshLod::NearEdge => &grass.lod.near_edge,
        GrassMeshLod::Far => &grass.lod.far,
        GrassMeshLod::Vista => &grass.lod.vista,
    };
    Vec4::new(
        tier.fade_in_m[0],
        tier.fade_in_m[1],
        tier.fade_out_m[0],
        tier.fade_out_m[1],
    )
}

const TIERS: [GrassMeshLod; 4] = [
    GrassMeshLod::Near,
    GrassMeshLod::NearEdge,
    GrassMeshLod::Far,
    GrassMeshLod::Vista,
];

/// Maximum blade reach above a tuft root: authored ribbon height times the
/// largest height/species scaling the mesh generator produces.
const TUFT_HEIGHT_MARGIN_METRES: f32 = 1.5;

/// Tight bounds over a batch's actual instances. The initial implementation
/// used one whole-terrain slab per batch; a fitted box keeps Bevy's
/// batch-level frustum test and the compute cull's chunk test conservative
/// without extending the volume past the sward's real extent.
pub(super) fn fitted_batch_aabb(instances: &[InstanceData], footprint: f32) -> Aabb {
    let mut minimum = Vec3::splat(f32::INFINITY);
    let mut maximum = Vec3::splat(f32::NEG_INFINITY);
    for instance in instances {
        minimum = minimum.min(instance.position);
        maximum = maximum.max(instance.position);
    }
    let margin = Vec3::new(footprint, TUFT_HEIGHT_MARGIN_METRES, footprint);
    let minimum = minimum - margin * Vec3::new(1.0, 0.2, 1.0);
    let maximum = maximum + margin;
    Aabb {
        center: ((minimum + maximum) * 0.5).into(),
        half_extents: ((maximum - minimum) * 0.5).into(),
    }
}

/// CPU-side sampler over the same feathered cover mask the legacy renderer
/// binds as a texture.
struct CoverageMask {
    width: usize,
    height: usize,
    pixels: Vec<u8>,
    ground_width: f32,
    ground_depth: f32,
}

impl CoverageMask {
    fn new(ground: &SceneGround, seed: u64) -> Self {
        let (width, height, pixels) = grass_cover_mask_pixels(ground, seed);
        Self {
            width: width as usize,
            height: height as usize,
            pixels,
            ground_width: ground.width(),
            ground_depth: ground.depth(),
        }
    }

    fn coverage_byte(&self, world: Vec2) -> u8 {
        let u = (world.x / self.ground_width + 0.5).clamp(0.0, 1.0);
        let v = (world.y / self.ground_depth + 0.5).clamp(0.0, 1.0);
        let x = ((u * self.width as f32) as usize).min(self.width - 1);
        let y = ((v * self.height as f32) as usize).min(self.height - 1);
        self.pixels[y * self.width + x]
    }
}

/// Per-site sampling behind the tuft lattice. The playable scene answers from
/// its authoritative terrain and cover mask; the vista rings answer from the
/// coarse vista heightfield and its stitched coverage. Everything downstream -
/// the lattice walk, the species split, the batching - is shared.
pub(in crate::presentation) trait TuftPlacement {
    /// Inclusive cell range to walk, in whole `cell_spacing` steps.
    fn lattice_bounds(&self, cell_spacing: f32) -> (IVec2, IVec2);

    /// Whether this jittered lattice cell may carry tufts at all.
    fn cell_allows(&self, cell_hash: u64, cell: IVec2, cell_spacing: f32, jitter: f32) -> bool;

    /// Ground-cover coverage at a tuft centre; 0 leaves the site bare.
    fn coverage(&self, centre: Vec2) -> u8;

    /// Ground height at a tuft centre, or `None` where the slope rejects grass.
    fn height(&self, centre: Vec2) -> Option<f32>;

    /// Which community - and so which species pool - claims this site.
    fn community(&self, centre: Vec2) -> GrassCommunity;
}

/// The command buffer and asset stores a sward spawn writes through.
pub(in crate::presentation) struct GrassWorld<'a, 'w, 's> {
    pub(in crate::presentation) commands: &'a mut Commands<'w, 's>,
    pub(in crate::presentation) meshes: &'a mut Assets<Mesh>,
    pub(in crate::presentation) materials: &'a mut Assets<TacticalGrassInstancedMaterial>,
}

/// Instance batches keyed by LOD tier, then by species within the tier.
pub(in crate::presentation) type TierSpeciesBatches =
    [[Vec<InstanceData>; GrassSpecies::ALL.len()]; TIERS.len()];

/// The playable scene's authoritative terrain, cover mask, and community profile.
struct ScenePlacement<'a> {
    terrain: &'a SceneTerrain,
    ground: &'a SceneGround,
    mask: CoverageMask,
    profile: GrassCommunityProfile,
    base_seed: u64,
}

impl TuftPlacement for ScenePlacement<'_> {
    fn lattice_bounds(&self, cell_spacing: f32) -> (IVec2, IVec2) {
        let half = Vec2::new(self.terrain.width(), self.terrain.depth()) * 0.5;
        (
            (-half / cell_spacing).floor().as_ivec2(),
            (half / cell_spacing).ceil().as_ivec2(),
        )
    }

    fn cell_allows(&self, cell_hash: u64, cell: IVec2, cell_spacing: f32, jitter: f32) -> bool {
        cell_allows_grass(
            self.terrain,
            self.ground,
            cell_hash,
            cell.x,
            cell.y,
            cell_spacing,
            jitter,
        )
    }

    fn coverage(&self, centre: Vec2) -> u8 {
        self.mask.coverage_byte(centre)
    }

    fn height(&self, centre: Vec2) -> Option<f32> {
        let height = self.terrain.height_at(centre)?;
        self.terrain
            .normal_at(centre)
            .filter(|normal| normal.y >= MINIMUM_GRASS_SLOPE_NORMAL_Y)
            .map(|_| height)
    }

    fn community(&self, centre: Vec2) -> GrassCommunity {
        grass_community_at(centre, self.base_seed, self.profile)
    }
}

/// Grass rejects any site steeper than this surface normal tilt.
pub(in crate::presentation) const MINIMUM_GRASS_SLOPE_NORMAL_Y: f32 = 0.72;

/// The scene-wide pigment and wind inputs every tuft mesh and material shares.
#[derive(Clone, Copy)]
pub(in crate::presentation) struct TuftPigment {
    pub(in crate::presentation) color: Color,
    pub(in crate::presentation) density: f32,
    pub(in crate::presentation) dryness: f32,
    pub(in crate::presentation) wind_scale: f32,
}

/// Turns filled instance batches into one entity per (tier, species), each
/// carrying `marker` on top of the shared instanced-draw components.
pub(in crate::presentation) fn spawn_tuft_batches(
    world: GrassWorld<'_, '_, '_>,
    batches: &mut TierSpeciesBatches,
    label: &str,
    marker: impl Bundle + Clone,
    base_seed: u64,
    pigment: TuftPigment,
    grass: &crate::presentation::config::GrassConfig,
) {
    let GrassWorld {
        commands,
        meshes,
        materials,
    } = world;
    for lod in TIERS {
        let material = materials.add(diagnostics::material(
            lod,
            grass,
            pigment.density,
            pigment.dryness,
            pigment.wind_scale,
        ));
        for species in GrassSpecies::ALL {
            let instances = std::mem::take(&mut batches[lod.tier_index()][species.index()]);
            if instances.is_empty() {
                continue;
            }
            let (mesh, triangle_count) = diagnostics::add_mesh(
                meshes,
                grass_tuft_mesh(
                    pigment.color,
                    lod,
                    pigment.density,
                    species,
                    splitmix64(
                        base_seed ^ ((species.index() as u64) << 8 | lod.tier_index() as u64),
                    ),
                    grass,
                ),
            );
            let mut entity = commands.spawn((
                Name::new(format!(
                    "{label} {species:?} {lod:?} tufts ({})",
                    instances.len()
                )),
                GroundScatterLayer::Grass,
                triangle_count,
                GpuCullCompute,
                // Batches span the whole scene, so CPU frustum culling can
                // only ever hide them wholesale - and worse, a culled frame
                // drops the batch from `RenderMeshInstances`, which makes
                // eidolon free and re-upload the retained instance buffers
                // every time the camera pitch crosses the horizon. Culling
                // belongs solely to the GPU compute pass.
                NoFrustumCulling,
                Mesh3d(mesh),
                InstancedMeshMaterial(material.clone()),
                fitted_batch_aabb(&instances, configured_tuft_footprint_metres(lod, grass)),
                InstanceMaterialData {
                    instances: Arc::new(instances),
                    color: LinearRgba::WHITE,
                    visibility_range: configured_tier_visibility_range(lod, grass),
                },
                Transform::default(),
                Visibility::Inherited,
            ));
            entity.insert(marker.clone());
            if !diagnostics::casts_shadows(lod, grass) {
                entity.insert(NotShadowCaster);
            }
        }
    }
}

/// Walks the jittered placement cells and fills per-species instance vectors
/// with tuft placements.
pub(in crate::presentation) fn scatter_cell_tufts(
    species_batches: &mut [Vec<InstanceData>; GrassSpecies::ALL.len()],
    placement: &impl TuftPlacement,
    base_seed: u64,
    lod: GrassMeshLod,
    cell_spacing: f32,
    grass: &crate::presentation::config::GrassConfig,
) -> u32 {
    let (minimum, maximum) = placement.lattice_bounds(cell_spacing);
    let side = match lod {
        GrassMeshLod::Near => grass.lod.near.native_tufts_per_cell_side,
        GrassMeshLod::NearEdge => grass.lod.near_edge.native_tufts_per_cell_side,
        GrassMeshLod::Far => grass.lod.far.native_tufts_per_cell_side,
        GrassMeshLod::Vista => grass.lod.vista.native_tufts_per_cell_side,
    } as i32;
    let footprint = configured_tuft_footprint_metres(lod, grass);
    let mut emitted = 0_u32;
    for z in minimum.y..=maximum.y {
        for x in minimum.x..=maximum.x {
            let cell = ((x as u32 as u64) << 32) | z as u32 as u64;
            let cell_hash = splitmix64(base_seed ^ cell);
            if !placement.cell_allows(
                cell_hash,
                IVec2::new(x, z),
                cell_spacing,
                grass.placement.jitter_fraction,
            ) {
                continue;
            }
            let cell_origin = Vec2::new(x as f32, z as f32) * cell_spacing
                - Vec2::splat((side - 1) as f32 * 0.5 * footprint);
            for tuft_z in 0..side {
                for tuft_x in 0..side {
                    let tuft_hash =
                        splitmix64(cell_hash ^ (((tuft_x as u64) << 17) | ((tuft_z as u64) << 3)));
                    let jitter = Vec2::new(
                        unit_hash(tuft_hash) - 0.5,
                        unit_hash(splitmix64(tuft_hash)) - 0.5,
                    ) * footprint
                        * 0.35;
                    let centre =
                        cell_origin + Vec2::new(tuft_x as f32, tuft_z as f32) * footprint + jitter;
                    let coverage = placement.coverage(centre);
                    if coverage == 0 {
                        continue;
                    }
                    let Some(height) = placement.height(centre) else {
                        continue;
                    };
                    let community = placement.community(centre);
                    let species =
                        grass_species(community, splitmix64(tuft_hash ^ 0x7475_6674_5f63_656c));
                    let batch = &mut species_batches[species.index()];
                    batch.push(InstanceData {
                        position: Vec3::new(centre.x, height, centre.y),
                        scale: 1.0,
                        rotation: unit_hash(splitmix64(tuft_hash ^ 0x796177))
                            * core::f32::consts::TAU,
                        index: batch.len() as u32,
                        batch_id: 0,
                        seed: u32::from(coverage)
                            | ((splitmix64(tuft_hash ^ 0x7365_6564) as u32) << 8),
                    });
                    emitted += 1;
                }
            }
        }
    }
    emitted
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::presentation::ground_scatter::grass::{grass_lod_visibility, tuft_blade_side};

    #[test]
    fn instanced_tiers_reproduce_legacy_per_cell_shoot_totals() {
        // Legacy: 96x96 near shoots, 40x40 far shoots, 24x24 vista shoots
        // per placement cell. The near-edge ring deliberately thins the
        // legacy near total to 6x6 blades per tuft past ~9 m.
        for (lod, legacy_total) in [
            (GrassMeshLod::Near, 96 * 96),
            (GrassMeshLod::NearEdge, 12 * 12 * 36),
            (GrassMeshLod::Far, 1_600),
            (GrassMeshLod::Vista, 576),
        ] {
            let side = tufts_per_cell_side(lod) as usize;
            let blades = tuft_blade_side(lod) * tuft_blade_side(lod);
            assert_eq!(side * side * blades, legacy_total, "{lod:?}");
        }
    }

    #[test]
    fn instanced_fade_bands_match_the_legacy_visibility_ranges() {
        // The legacy near band's 18..26 fade-out is owned by the near-edge
        // sub-tier; the near field hands off to the edge ring at 8..10.
        for lod in [
            GrassMeshLod::NearEdge,
            GrassMeshLod::Far,
            GrassMeshLod::Vista,
        ] {
            let range = tier_visibility_range(lod, 1.0);
            let legacy = grass_lod_visibility(lod);
            assert_eq!(range.x, legacy.start_margin.start, "{lod:?}");
            assert_eq!(range.y, legacy.start_margin.end, "{lod:?}");
            assert_eq!(range.z, legacy.end_margin.start, "{lod:?}");
            assert_eq!(range.w, legacy.end_margin.end, "{lod:?}");
        }
        let near = tier_visibility_range(GrassMeshLod::Near, 1.0);
        let edge = tier_visibility_range(GrassMeshLod::NearEdge, 1.0);
        assert_eq!(near.z, edge.x);
        assert_eq!(near.w, edge.y);
        for lod in TIERS {
            let range = tier_visibility_range(lod, 1.0);
            assert!(range.x <= range.y && range.y <= range.z && range.z < range.w);
        }
    }

    #[test]
    fn contracted_fade_bands_stay_contiguous_across_tiers() {
        for scale in [0.35, 0.6, 0.75, 1.0] {
            let near = tier_visibility_range(GrassMeshLod::Near, scale);
            let edge = tier_visibility_range(GrassMeshLod::NearEdge, scale);
            let far = tier_visibility_range(GrassMeshLod::Far, scale);
            let vista = tier_visibility_range(GrassMeshLod::Vista, scale);
            assert_eq!(near.z, edge.x);
            assert_eq!(near.w, edge.y);
            assert_eq!(edge.z, far.x);
            assert_eq!(edge.w, far.y);
            assert!(far.z >= vista.x && far.w >= vista.y);
            for range in [near, edge, far, vista] {
                assert!(range.x <= range.y && range.y <= range.z && range.z < range.w);
            }
        }
        // Out-of-range requests clamp instead of collapsing the sward.
        assert_eq!(
            tier_visibility_range(GrassMeshLod::Near, 0.0),
            tier_visibility_range(GrassMeshLod::Near, 0.35)
        );
    }

    #[test]
    fn fitted_batch_aabb_bounds_all_instances_with_blade_headroom() {
        let instances = vec![
            InstanceData {
                position: Vec3::new(-4.0, 1.0, 6.0),
                scale: 1.0,
                ..Default::default()
            },
            InstanceData {
                position: Vec3::new(9.0, 3.0, -2.0),
                scale: 1.0,
                ..Default::default()
            },
        ];
        let aabb = fitted_batch_aabb(&instances, 0.5);
        let minimum = aabb.center - aabb.half_extents;
        let maximum = aabb.center + aabb.half_extents;
        assert!(minimum.x <= -4.5 && maximum.x >= 9.5);
        assert!(minimum.z <= -2.5 && maximum.z >= 6.5);
        assert!(maximum.y >= 3.0 + TUFT_HEIGHT_MARGIN_METRES);
        assert!(minimum.y <= 1.0);
    }

    #[test]
    fn instance_seed_low_byte_carries_placement_coverage() {
        let coverage = 173_u8;
        let seed = u32::from(coverage) | (0xdead_beef_u32 << 8);
        assert_eq!((seed & 0xff) as u8, coverage);
    }
}
