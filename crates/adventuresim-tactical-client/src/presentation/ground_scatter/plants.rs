//! Habitat-filtered, cell-batched botanical meshes shared with Plant Studio.
use super::{GroundScatterLayer, scatter_ground_without_patch};
use crate::presentation::{bps, stable_text_seed, unit_hash};
use adventuresim_core::strategic_time::{DAYS_PER_YEAR, MINUTES_PER_DAY};
use adventuresim_plant_generator::{
    PlantMesh, Tessellation,
    flower::FlowerSpecies,
    habitat::{PlantGround, PlantHabitat},
};
use adventuresim_tactical_core::prelude::{
    GroundCover, GroundSubstrate, GroundSurface, SceneEnvironment, SceneGround, SceneId,
    SceneTerrain, TerrainLandformRecipe,
};
use bevy::{camera::visibility::VisibilityRange, prelude::*};
use fabelgeist_determinism::splitmix64;
use std::collections::BTreeMap;

const CELL_METRES: f32 = 12.0;
const SITE_SPACING_METRES: f32 = 2.0;
const MAX_SPECIMENS: usize = 512;
const MIN_SLOPE_NORMAL_Y: f32 = 0.8;
const ROOT_EMBED_METRES: f32 = 0.002;
const OCCUPANCY: f32 = 0.32;
const PLANT_SEED: u64 = 0x504c_414e_5453;
#[cfg(test)]
mod tests;

pub(in crate::presentation) struct PlantPresentationPlugin;
impl Plugin for PlantPresentationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SpecimenCache>()
            .add_systems(Update, present);
    }
}
#[derive(Resource, Default)]
struct SpecimenCache {
    flowers: Vec<PlantMesh>,
    material: Option<Handle<StandardMaterial>>,
}
#[derive(Component)]
struct PlantsPresented;
/// Actual production roots retained for capture framing and placement diagnostics.
#[derive(Component)]
pub(crate) struct PlantCaptureAnchors(pub Vec<Vec3>);
type NewPlantScenes<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static SceneId,
        &'static SceneTerrain,
        &'static SceneGround,
        &'static SceneEnvironment,
        Option<&'static TerrainLandformRecipe>,
    ),
    Without<PlantsPresented>,
>;

fn present(
    mut commands: Commands,
    scenes: NewPlantScenes,
    mut cache: ResMut<SpecimenCache>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (entity, id, terrain, ground, environment, landform) in &scenes {
        if cache.flowers.is_empty() {
            cache.flowers = FlowerSpecies::ALL
                .iter()
                .map(|s| {
                    s.parameters()
                        .generate(PLANT_SEED, Tessellation::Field)
                        .expect("valid flower preset")
                })
                .collect();
        }
        let material = cache
            .material
            .get_or_insert_with(|| {
                materials.add(StandardMaterial {
                    perceptual_roughness: 0.72,
                    double_sided: true,
                    cull_mode: None,
                    ..default()
                })
            })
            .clone();
        let masked = landform.map(|l| scatter_ground_without_patch(ground, l.transition_collar()));
        let ground = masked.as_ref().unwrap_or(ground);
        let seed =
            stable_text_seed(&id.0) ^ stable_text_seed(&environment.scene_digest) ^ PLANT_SEED;
        let sites = placements(terrain, ground, environment, seed);
        let mut batches: BTreeMap<(i32, i32), PlantMesh> = BTreeMap::new();
        let mut anchors = Vec::new();
        for site in sites {
            let cell = (
                (site.root.x / CELL_METRES).floor() as i32,
                (site.root.z / CELL_METRES).floor() as i32,
            );
            let origin = Vec3::new(
                (cell.0 as f32 + 0.5) * CELL_METRES,
                0.0,
                (cell.1 as f32 + 0.5) * CELL_METRES,
            );
            batches.entry(cell).or_default().append(
                &cache.flowers[site.species],
                site.root - origin,
                Quat::from_rotation_y(unit_hash(site.hash) * std::f32::consts::TAU),
                0.85 + unit_hash(splitmix64(site.hash)) * 0.3,
            );
            anchors.push(site.root);
        }
        for ((x, z), mesh) in batches {
            commands.spawn((
                Name::new("Parametric flower community"),
                GroundScatterLayer::BotanicalPlants,
                Mesh3d(meshes.add(mesh.into_bevy())),
                MeshMaterial3d(material.clone()),
                Transform::from_xyz(
                    (x as f32 + 0.5) * CELL_METRES,
                    0.0,
                    (z as f32 + 0.5) * CELL_METRES,
                ),
                VisibilityRange {
                    start_margin: 0.0..0.0,
                    end_margin: 22.0..27.0,
                    use_aabb: true,
                },
            ));
        }
        commands
            .entity(entity)
            .insert((PlantsPresented, PlantCaptureAnchors(anchors)));
    }
}

#[derive(Debug, PartialEq)]
struct PlantSite {
    root: Vec3,
    species: usize,
    hash: u64,
}

fn placements(
    terrain: &SceneTerrain,
    ground: &SceneGround,
    environment: &SceneEnvironment,
    seed: u64,
) -> Vec<PlantSite> {
    let count_x = (terrain.width() / SITE_SPACING_METRES).floor() as i32;
    let count_z = (terrain.depth() / SITE_SPACING_METRES).floor() as i32;
    let mut candidates = Vec::new();
    let cover =
        super::cover_mask::CoverageMask::new(ground, stable_text_seed(&environment.scene_digest));
    for z in 0..count_z {
        for x in 0..count_x {
            let cell = ((x as u64) << 32) | z as u64;
            let hash = splitmix64(seed ^ cell);
            if unit_hash(hash) > OCCUPANCY {
                continue;
            }
            let world = Vec2::new(
                -terrain.width() * 0.5
                    + (x as f32 + 0.15 + unit_hash(splitmix64(hash)) * 0.7) * SITE_SPACING_METRES,
                -terrain.depth() * 0.5
                    + (z as f32 + 0.15 + unit_hash(splitmix64(hash ^ PLANT_SEED)) * 0.7)
                        * SITE_SPACING_METRES,
            );
            let (Some(surface), Some(height), Some(normal)) = (
                ground.ground_at(world),
                terrain.height_at(world),
                terrain.normal_at(world),
            ) else {
                continue;
            };
            if normal.y < MIN_SLOPE_NORMAL_Y || !open_understory_site(&cover, world) {
                continue;
            }
            let habitat = habitat(surface, environment);
            // A shared macro-cell roll gives patches botanical coherence; roots
            // retain independent jitter so the planting lattice is not visible.
            let community = splitmix64(seed ^ (((x / 3) as u64) << 32) ^ (z / 3) as u64);
            let weights = FlowerSpecies::ALL.map(|species| habitat.flower_weight(species));
            let total: f32 = weights.iter().sum();
            let mut roll = unit_hash(community) * total;
            let Some(species) = weights.iter().position(|weight| {
                roll -= weight;
                roll < 0.0
            }) else {
                continue;
            };
            candidates.push(PlantSite {
                root: Vec3::new(world.x, height - ROOT_EMBED_METRES, world.y),
                species,
                hash,
            });
        }
    }
    // Stable priority sampling spreads the bounded population over the entire
    // scene; truncating row order would leave one populated corner.
    candidates.sort_by_key(|site| site.hash);
    candidates.truncate(MAX_SPECIMENS);
    candidates
}

/// Shoots occupy existing vegetation openings, never clear their own.
fn open_understory_site(cover: &super::cover_mask::CoverageMask, point: Vec2) -> bool {
    const MAX_GRASS_COVERAGE: u8 = 64;
    const SHOOT_CLEARANCE_METRES: f32 = 0.4;
    [Vec2::ZERO, Vec2::X, Vec2::NEG_X, Vec2::Y, Vec2::NEG_Y]
        .into_iter()
        .all(|offset| {
            cover.coverage_byte(point + offset * SHOOT_CLEARANCE_METRES) <= MAX_GRASS_COVERAGE
        })
}

fn habitat(surface: GroundSurface, e: &SceneEnvironment) -> PlantHabitat {
    let ground = if !matches!(
        surface.substrate,
        GroundSubstrate::Soil | GroundSubstrate::Mud
    ) {
        PlantGround::Unsuitable
    } else {
        match surface.cover {
            GroundCover::LeafLitter => PlantGround::WoodlandLitter,
            GroundCover::TallGrass => PlantGround::Grass,
            GroundCover::Bare => PlantGround::Soil,
            GroundCover::LooseStone | GroundCover::Reeds => PlantGround::Unsuitable,
        }
    };
    PlantHabitat {
        canopy: bps(e.canopy_bps),
        cultivation: bps(e.cultivation_bps),
        moisture: bps(e.weather.ground_moisture_bps),
        snow: bps(e.weather.snow_cover_bps),
        day_of_year: ((e.absolute_minute / MINUTES_PER_DAY) % DAYS_PER_YEAR + 1) as u16,
        ground,
    }
}
