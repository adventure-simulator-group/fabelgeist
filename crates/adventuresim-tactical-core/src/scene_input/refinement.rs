//! Authoritative microrelief is prepared before owned property support.
use super::{
    GeneratedObstacle, SceneInputError, SceneInputResult, SceneValidationError,
    detail_noise::DetailNoise,
    detail_obstacles::{TerrainDetailObstacles, TerrainRockInfluence},
    rock_recipe::wrapped_angle_difference,
};
use crate::{
    city_layout::grounding::SupportElevation,
    scene::{GroundCover, GroundSubstrate, SceneGround, SceneTerrain},
    scene_coordinates::ScenePlanPoint,
    terrain_streams as streams,
};
use adventuresim_world_schema::UnitBasisPoints;
use bevy::math::Vec2;
use fabelgeist_determinism::Seed;
use road::road_surface_relief;

mod road;

pub(super) const AUTHORITATIVE_DETAIL_SPACING_METRES: f32 = 0.5;
const DETAIL_RELIEF_MINIMUM_METRES: f32 = -0.075;
const DETAIL_RELIEF_MAXIMUM_METRES: f32 = 0.105;

#[derive(Clone, Copy)]
struct TerrainShapeSample {
    downhill: Vec2,
    slope: f32,
    concavity: f32,
}

/// Builds the one high-resolution surface used by height queries, IK,
/// rendering, and the server heightfield collider. The original grid is kept
/// implicitly as `SceneTerrain`'s coarse render LOD.
pub(super) fn refine_authoritative_terrain(
    seed: Seed,
    terrain: &SceneTerrain,
    ground: &SceneGround,
    obstacles: &[GeneratedObstacle],
    obstacle_spacing: f32,
    moisture_bps: u16,
) -> SceneInputResult<SceneTerrain> {
    let influences = TerrainDetailObstacles::from_obstacles(terrain, obstacles, obstacle_spacing)?;
    let detail_seed = streams::DETAIL.seed(seed, &[]);
    let noise = DetailNoise::new(detail_seed);
    let mut terrain = terrain
        .refined(AUTHORITATIVE_DETAIL_SPACING_METRES, |point, base_height| {
            SupportElevation::from_metres(
                base_height.metres()
                    + authoritative_surface_relief(
                        detail_seed,
                        point,
                        terrain,
                        ground,
                        moisture_bps,
                        &influences,
                        &noise,
                    ),
            )
        })
        .ok_or(SceneInputError::Validation(
            SceneValidationError::DetailTerrain,
        ))?;
    terrain.constrain_max_grade(crate::scene::grade::TerrainGradeLimit::PLAYABLE)?;
    Ok(terrain)
}

fn authoritative_surface_relief(
    seed: Seed,
    point: ScenePlanPoint,
    terrain: &SceneTerrain,
    ground: &SceneGround,
    moisture_bps: u16,
    influences: &TerrainDetailObstacles,
    noise: &DetailNoise,
) -> f32 {
    let surface = ground.ground_at(point);
    if surface.is_some_and(|surface| surface.substrate == GroundSubstrate::Water) {
        return 0.0;
    }
    if surface.is_some_and(|surface| surface.substrate == GroundSubstrate::Road) {
        return road_surface_relief(point, ground, noise);
    }

    let broad = signed_ground_noise(noise.fields.broad, point.metres() / 3.2, noise) * 0.024;
    let fine = signed_ground_noise(noise.fields.fine, point.metres() / 0.92, noise) * 0.009;
    let clod_strength = match surface.map(|surface| surface.substrate) {
        Some(GroundSubstrate::Stone) => 0.0,
        Some(GroundSubstrate::Gravel) => 0.25,
        Some(GroundSubstrate::Mud | GroundSubstrate::Soil) => 1.0,
        Some(GroundSubstrate::Road | GroundSubstrate::Water) => unreachable!(),
        None => 1.0,
    };
    let clods = terrain_clod_relief(point, noise) * clod_strength;
    let shape = terrain_shape_sample(terrain, point, AUTHORITATIVE_DETAIL_SPACING_METRES * 3.0);
    let process_relief = shape.map_or(0.0, |shape| {
        drainage_relief(point, shape, moisture_bps, noise)
            + soil_creep_relief(point, shape, noise)
            + shape.concavity.clamp(-0.018, 0.024)
    });
    let roots = tree_root_relief(seed, point, &influences.trees);
    let rock_contact = boulder_ground_relief(seed, point, shape, &influences.rocks, noise);
    let rock_strata = match surface.map(|surface| surface.substrate) {
        Some(GroundSubstrate::Stone) => rocky_substrate_relief(seed, point, shape, 1.0, noise),
        Some(GroundSubstrate::Gravel) => rocky_substrate_relief(seed, point, shape, 0.62, noise),
        _ => 0.0,
    };
    let substrate_strength = match surface.map(|surface| surface.substrate) {
        Some(GroundSubstrate::Stone) => 0.54,
        Some(GroundSubstrate::Gravel) => 0.68,
        Some(GroundSubstrate::Mud) => 0.78,
        Some(GroundSubstrate::Soil) => 1.0,
        Some(GroundSubstrate::Road | GroundSubstrate::Water) => unreachable!(),
        None => 0.62,
    };
    let cover_strength = match surface.map(|surface| surface.cover) {
        Some(GroundCover::Reeds) => 0.35,
        Some(GroundCover::LooseStone) => 0.78,
        _ => 1.0,
    };
    ((broad + fine + clods + process_relief) * substrate_strength * cover_strength
        + roots
        + rock_contact
        + rock_strata)
        .clamp(DETAIL_RELIEF_MINIMUM_METRES, DETAIL_RELIEF_MAXIMUM_METRES)
}

fn terrain_shape_sample(
    terrain: &SceneTerrain,
    point: ScenePlanPoint,
    radius: f32,
) -> Option<TerrainShapeSample> {
    let centre = terrain.height_at(point)?;
    let east = terrain.height_at(ScenePlanPoint::from_metres(
        point.metres() + Vec2::X * radius,
    )?)?;
    let west = terrain.height_at(ScenePlanPoint::from_metres(
        point.metres() - Vec2::X * radius,
    )?)?;
    let north = terrain.height_at(ScenePlanPoint::from_metres(
        point.metres() + Vec2::Y * radius,
    )?)?;
    let south = terrain.height_at(ScenePlanPoint::from_metres(
        point.metres() - Vec2::Y * radius,
    )?)?;
    let gradient = Vec2::new(
        east.metres() - west.metres(),
        north.metres() - south.metres(),
    ) / (radius * 2.0);
    let slope = gradient.length();
    let downhill = if slope > 0.000_1 {
        -gradient / slope
    } else {
        Vec2::X
    };
    Some(TerrainShapeSample {
        downhill,
        slope,
        concavity: ((east.metres() + west.metres() + north.metres() + south.metres()) * 0.25
            - centre.metres())
            * 0.18,
    })
}

fn signed_ground_noise(seed: Seed, point: Vec2, noise: &DetailNoise) -> f32 {
    noise.sample(seed, point) * 2.0 - 1.0
}

fn terrain_clod_relief(point: ScenePlanPoint, noise: &DetailNoise) -> f32 {
    let point = point.metres();
    let field = noise.sample(noise.fields.clod, point / 0.58);
    detail_smoothstep(0.69, 0.91, field) * 0.022 - 0.003
}

fn drainage_relief(
    point: ScenePlanPoint,
    shape: TerrainShapeSample,
    moisture_bps: u16,
    noise: &DetailNoise,
) -> f32 {
    let point = point.metres();
    let slope_weight = detail_smoothstep(0.012, 0.16, shape.slope);
    if slope_weight <= 0.0 {
        return 0.0;
    }
    let normal = Vec2::new(-shape.downhill.y, shape.downhill.x);
    let warp = signed_ground_noise(noise.fields.rill_warp, point / 5.5, noise) * 0.85;
    let spacing = 2.6 + noise.sample(noise.fields.rill_spacing, point / 11.0) * 1.4;
    let distance = periodic_distance(point.dot(normal) + warp, spacing);
    let channel = 1.0 - detail_smoothstep(0.08, 0.34, distance);
    let shoulder =
        detail_smoothstep(0.18, 0.42, distance) * (1.0 - detail_smoothstep(0.42, 0.72, distance));
    let moisture = UnitBasisPoints::saturating(moisture_bps).as_unit_f32();
    (-channel * (0.026 + moisture * 0.012) + shoulder * 0.009) * slope_weight
}

fn soil_creep_relief(point: ScenePlanPoint, shape: TerrainShapeSample, noise: &DetailNoise) -> f32 {
    let point = point.metres();
    let slope_weight = detail_smoothstep(0.035, 0.22, shape.slope);
    let warp = signed_ground_noise(noise.fields.creep_warp, point / 7.0, noise) * 0.55;
    let distance = periodic_distance(point.dot(shape.downhill) + warp, 3.1);
    (1.0 - detail_smoothstep(0.12, 0.52, distance)) * 0.019 * slope_weight
}

fn rocky_substrate_relief(
    seed: Seed,
    point: ScenePlanPoint,
    shape: Option<TerrainShapeSample>,
    strength: f32,
    noise: &DetailNoise,
) -> f32 {
    let point = point.metres();
    let fallback_angle = streams::STRATA_DIRECTION
        .rng(seed, &[])
        .inclusive_unit_f32()
        * core::f32::consts::TAU;
    let downhill = shape.map(|shape| shape.downhill).unwrap_or(Vec2::new(
        libm::cosf(fallback_angle),
        libm::sinf(fallback_angle),
    ));
    let across = Vec2::new(-downhill.y, downhill.x);
    let slope_weight = shape
        .map(|shape| detail_smoothstep(0.018, 0.18, shape.slope))
        .unwrap_or(0.35);
    let contour = point.dot(downhill)
        + signed_ground_noise(noise.fields.strata_warp, point / 6.5, noise) * 0.72;
    let shelf = (1.0 - detail_smoothstep(0.08, 0.48, periodic_distance(contour, 2.15)))
        * (0.019 + slope_weight * 0.029);
    let fracture_a = periodic_distance(
        point.dot(across) + signed_ground_noise(noise.fields.fracture_a, point / 4.8, noise) * 0.4,
        3.7,
    );
    let diagonal = (across * 0.72 + downhill * 0.69).normalize_or_zero();
    let fracture_b = periodic_distance(
        point.dot(diagonal)
            + signed_ground_noise(noise.fields.fracture_b, point / 5.6, noise) * 0.34,
        5.3,
    );
    let crack = (1.0 - detail_smoothstep(0.035, 0.17, fracture_a))
        .max((1.0 - detail_smoothstep(0.035, 0.15, fracture_b)) * 0.72);
    (shelf - crack * 0.019) * strength
}

fn boulder_ground_relief(
    seed: Seed,
    point: ScenePlanPoint,
    shape: Option<TerrainShapeSample>,
    rocks: &[TerrainRockInfluence],
    noise: &DetailNoise,
) -> f32 {
    let point = point.metres();
    let mut relief = 0.0;
    for rock in rocks {
        let offset = point - rock.centre.metres();
        let distance = offset.length();
        let radius = rock.radius.metres().max(0.35);
        if distance > radius * 5.0 {
            continue;
        }
        let rock_seed = streams::ROCK_INFLUENCE.seed(
            seed,
            &[
                u64::from(rock.centre.metres().x.to_bits()),
                u64::from(rock.centre.metres().y.to_bits()),
            ],
        );
        let fallback_angle = streams::ROCK_DOWNHILL
            .rng(rock_seed, &[])
            .inclusive_unit_f32()
            * core::f32::consts::TAU;
        let downhill = shape.map(|shape| shape.downhill).unwrap_or(Vec2::new(
            libm::cosf(fallback_angle),
            libm::sinf(fallback_angle),
        ));
        let across_axis = Vec2::new(-downhill.y, downhill.x);
        let socket = (1.0 - detail_smoothstep(radius * 0.48, radius * 1.08, distance)) * -0.042;
        let apron = detail_smoothstep(radius * 0.72, radius * 1.04, distance)
            * (1.0 - detail_smoothstep(radius * 1.04, radius * 1.72, distance))
            * 0.033;
        let downstream = offset.dot(downhill);
        let across = offset.dot(across_axis).abs();
        let tail_length =
            radius * (3.2 + streams::ROCK_TAIL.rng(rock_seed, &[]).inclusive_unit_f32() * 1.1);
        let longitudinal = detail_smoothstep(radius * 0.45, radius * 0.95, downstream)
            * (1.0 - detail_smoothstep(tail_length * 0.62, tail_length, downstream));
        let tail_width = radius * 0.42 + downstream.max(0.0) * 0.24;
        let lateral = 1.0 - detail_smoothstep(tail_width * 0.42, tail_width, across);
        let granular = 0.72
            + noise.sample(
                streams::ROCK_DEBRIS.seed(rock_seed, &[]),
                Vec2::new(downstream / 1.7, across / 0.8),
            ) * 0.28;
        relief += socket + apron + longitudinal * lateral * granular * 0.034;
    }
    relief.clamp(-0.055, 0.07)
}

fn tree_root_relief(seed: Seed, point: ScenePlanPoint, tree_positions: &[ScenePlanPoint]) -> f32 {
    let point = point.metres();
    let mut relief = 0.0;
    for &tree in tree_positions {
        let tree = tree.metres();
        let offset = point - tree;
        let radius = offset.length();
        if radius > 8.0 {
            continue;
        }
        let tree_seed = streams::TREE_ROOTS.seed(
            seed,
            &[u64::from(tree.x.to_bits()), u64::from(tree.y.to_bits())],
        );
        let mound = libm::expf(-(radius / 1.35).powi(2)) * 0.045;
        let basin = detail_smoothstep(0.9, 1.8, radius)
            * (1.0 - detail_smoothstep(5.2, 7.7, radius))
            * -0.012;
        let angle = libm::atan2f(offset.y, offset.x);
        let mut ridges = 0.0_f32;
        for root in 0..7_u64 {
            let mut random = streams::ROOT_SHAPE.rng(tree_seed, &[root]);
            let origin = random.inclusive_unit_f32() * core::f32::consts::TAU;
            let phase = random.inclusive_unit_f32() * core::f32::consts::TAU;
            let length = 4.8 + random.inclusive_unit_f32() * 2.9;
            if radius > length || radius < 0.28 {
                continue;
            }
            let curved_angle = origin + libm::sinf(radius * 0.9 + phase) * 0.11;
            let angular_distance = wrapped_angle_difference(angle, curved_angle).abs() * radius;
            let width = 0.16 + radius * 0.045;
            let ridge = 1.0 - detail_smoothstep(width * 0.3, width, angular_distance);
            let taper = 1.0 - detail_smoothstep(length * 0.58, length, radius);
            ridges = ridges.max(ridge * taper * (0.082 - radius * 0.007).max(0.025));
        }
        relief += mound + basin + ridges;
    }
    relief.clamp(-0.02, 0.115)
}

fn periodic_distance(value: f32, period: f32) -> f32 {
    let wrapped = value.rem_euclid(period);
    wrapped.min(period - wrapped)
}

fn detail_smoothstep(edge0: f32, edge1: f32, value: f32) -> f32 {
    let t = ((value - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}
