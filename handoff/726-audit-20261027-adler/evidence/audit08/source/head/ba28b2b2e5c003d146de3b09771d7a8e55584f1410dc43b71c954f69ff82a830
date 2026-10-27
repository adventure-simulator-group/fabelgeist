//! Select the physical surface for vista meshes and scenery placement.
use super::*;
use adventuresim_building_generator::spatial_geometry::{Position, PositiveLength, SignedLength};
use adventuresim_tactical_core::scene_coordinates::Scene;
use adventuresim_tactical_core::{
    city_layout::grounding::{SupportElevation, SupportQuery, SurfaceHit},
    scene_coordinates::ScenePlanPoint,
};
use fabelgeist_determinism::Seed;

const MINIMUM_VISTA_ROCK_SLOPE_NORMAL_Y: f32 = 0.72;
const VISTA_SCENERY_NORMAL_SAMPLE_DISTANCE_METRES: f32 = 2.0;

pub(super) fn vista_lod_meshes_with_morph(
    lod: &VistaLod,
    inner_half_extent: Vec2,
    coarser_lod: Option<&VistaLod>,
    playable_terrain: Option<&SceneTerrain>,
    playable_environment: Option<&SceneEnvironment>,
    weather: WeatherSnapshot,
    transition_collar: Option<TerrainTransitionCollar>,
) -> VistaMeshResult<Vec<Mesh>> {
    if let Some(surface) = playable_terrain.and_then(SceneTerrain::property_surface) {
        natural::validate_source(lod)?;
        if let Some(coarser) = coarser_lod {
            natural::validate_source(coarser)?;
        }
        return owned::vista_meshes(
            surface,
            lod,
            inner_half_extent,
            coarser_lod,
            playable_environment,
            weather,
            transition_collar,
        );
    }
    natural::sampled_vista_lod_meshes_with_morph(
        lod,
        inner_half_extent,
        coarser_lod,
        playable_terrain.filter(|terrain| {
            inner_half_extent == Vec2::new(terrain.width(), terrain.depth()) * 0.5
        }),
        playable_environment,
        weather,
    )
}

pub(super) fn vista_scatter_transform(
    lod: &VistaLod,
    coarser_lod: Option<&VistaLod>,
    playable_terrain: &SceneTerrain,
    playable_half_extent: Vec2,
    point: ScenePlanPoint,
    hash: Seed,
    lift: SignedLength,
) -> Option<Transform> {
    let hit = scenery_surface(
        lod,
        coarser_lod,
        playable_terrain,
        playable_half_extent,
        point,
    )?;
    if hit.normal.y < MINIMUM_VISTA_ROCK_SLOPE_NORMAL_Y {
        return None;
    }
    let position = Position::<Scene>::from_metres(Vec3::new(
        point.metres().x,
        hit.elevation.metres() + lift.metres(),
        point.metres().y,
    ))
    .ok()?;
    Some(
        Transform::from_translation(position.metres()).with_rotation(
            Quat::from_rotation_arc(Vec3::Y, *hit.normal)
                * Quat::from_rotation_y(
                    streams::ROCK_YAW.rng(hash, &[]).inclusive_unit_f32() * core::f32::consts::TAU,
                ),
        ),
    )
}

pub(super) fn tree_root_height(
    terrain: &SceneTerrain,
    lod: &VistaLod,
    coarser: Option<&VistaLod>,
    world: ScenePlanPoint,
) -> Option<SupportElevation> {
    if terrain.property_surface().is_some() {
        return terrain
            .surface_below(SupportQuery::unbounded(world))
            .map(|hit| hit.elevation);
    }
    presented_height_at(lod, world, coarser)
}

/// Owned scenery uses the same highest exterior support as the rendered ground.
/// Sampled scenes retain their stitched heightfield and finite-difference normal.
fn scenery_surface(
    lod: &VistaLod,
    coarser_lod: Option<&VistaLod>,
    playable_terrain: &SceneTerrain,
    playable_half_extent: Vec2,
    point: ScenePlanPoint,
) -> Option<SurfaceHit> {
    if playable_terrain.property_surface().is_some() {
        return playable_terrain.surface_below(SupportQuery::unbounded(point));
    }
    let local = lod.local_offset(point).ok()?;
    let geometry = VistaVertexGeometry::sample(
        lod,
        coarser_lod,
        Some(playable_terrain),
        local,
        playable_half_extent,
        PositiveLength::from_metres(VISTA_SCENERY_NORMAL_SAMPLE_DISTANCE_METRES).ok()?,
    )?;
    let tangent_x = geometry.east_tangent().metres();
    let tangent_z = geometry.north_tangent().metres();
    let normal = tangent_z.cross(tangent_x);
    Some(SurfaceHit::new(
        geometry.elevation(),
        Dir3::new(normal).ok()?,
    ))
}

#[cfg(test)]
mod tests;
