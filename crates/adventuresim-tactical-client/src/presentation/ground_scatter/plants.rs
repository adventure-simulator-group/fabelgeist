//! Habitat-filtered, instanced botanical meshes shared with Plant Studio.
mod streams;
use super::{GroundScatterLayer, scatter_ground_without_patch};
use crate::presentation::{bps, stable_text_seed};
use adventuresim_core::strategic_time::{DAYS_PER_YEAR, MINUTES_PER_DAY};
use adventuresim_plant_generator::{
    PlantSpecies,
    habitat::{PlantGround, PlantHabitat},
};
use adventuresim_tactical_core::prelude::{
    GroundCover, GroundSubstrate, GroundSurface, SceneEnvironment, SceneGround, SceneId,
    SceneTerrain, TerrainLandformRecipe,
};
use bevy::prelude::*;
mod lod;
pub(crate) use lod::PlantLodInstance;
use lod::SpecimenCache;

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
#[derive(Component)]
struct PlantsPresented;
/// Actual production roots retained for capture framing and placement diagnostics.
#[derive(Component)]
pub(crate) struct PlantCaptureAnchors(pub Vec<PlantCaptureAnchor>);
#[derive(Clone, Copy)]
pub(crate) struct PlantCaptureAnchor {
    pub root: Vec3,
    pub species: PlantSpecies,
}
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
        cache.prepare(&mut meshes, &mut materials);
        let masked = landform.map(|l| scatter_ground_without_patch(ground, l.transition_collar()));
        let ground = masked.as_ref().unwrap_or(ground);
        let seed =
            stable_text_seed(&id.0) ^ stable_text_seed(&environment.scene_digest) ^ PLANT_SEED;
        let sites = placements(terrain, ground, environment, seed);
        let mut anchors = Vec::new();
        for site in sites {
            let transform = Transform::from_translation(site.root)
                .with_rotation(Quat::from_rotation_y(
                    streams::YAW.rng(site.hash, &[]).inclusive_unit_f32() * std::f32::consts::TAU,
                ))
                .with_scale(Vec3::splat(
                    0.85 + streams::SCALE.rng(site.hash, &[]).inclusive_unit_f32() * 0.3,
                ));
            cache.spawn(&mut commands, site.species, transform);
            anchors.push(PlantCaptureAnchor {
                root: site.root,
                species: site.species,
            });
        }
        commands
            .entity(entity)
            .insert((PlantsPresented, PlantCaptureAnchors(anchors)));
    }
}

#[derive(Debug, PartialEq)]
struct PlantSite {
    root: Vec3,
    species: PlantSpecies,
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
            let hash = streams::SITE.seed(seed, &[x as u64, z as u64]).to_u64();
            if streams::PRESENCE.rng(hash, &[]).inclusive_unit_f32() > OCCUPANCY {
                continue;
            }
            let world = Vec2::new(
                -terrain.width() * 0.5
                    + (x as f32
                        + 0.15
                        + streams::JITTER_X.rng(hash, &[]).inclusive_unit_f32() * 0.7)
                        * SITE_SPACING_METRES,
                -terrain.depth() * 0.5
                    + (z as f32
                        + 0.15
                        + streams::JITTER_Z.rng(hash, &[]).inclusive_unit_f32() * 0.7)
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
            let community = streams::COMMUNITY
                .seed(seed, &[(x / 3) as u64, (z / 3) as u64])
                .to_u64();
            let weights = PlantSpecies::ALL.map(|species| {
                (species.habitat_weight(habitat)
                    * f32::from(adventuresim_world_schema::BASIS_POINTS_PER_WHOLE))
                .round() as u64
            });
            let Ok(species) = streams::SPECIES
                .rng(community, &[])
                .weighted_index(&weights)
            else {
                continue;
            };
            candidates.push(PlantSite {
                root: Vec3::new(world.x, height - ROOT_EMBED_METRES, world.y),
                species: PlantSpecies::ALL[species],
                hash,
            });
        }
    }
    // Stable priority sampling spreads the bounded population over the entire
    // scene; truncating row order would leave one populated corner.
    candidates.sort_by_key(|site| (site.hash, site.root.x.to_bits(), site.root.z.to_bits()));
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
