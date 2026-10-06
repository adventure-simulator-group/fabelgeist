//! Select the physical surface for vista meshes and scenery placement.
use super::*;

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
) -> Vec<Mesh> {
    let width = usize::from(lod.width);
    let depth = usize::from(lod.depth);
    if width < 2
        || depth < 2
        || width.checked_mul(depth).is_none_or(|samples| {
            lod.heights_metres.len() != samples || lod.environment.len() != samples
        })
        || !lod.spacing_metres.is_finite()
        || lod.spacing_metres <= 0.0
    {
        return Vec::new();
    }
    if let Some(terrain) = playable_terrain
        && terrain.property_surface().is_some()
    {
        return owned::vista_meshes(
            terrain,
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
    point: Vec2,
    hash: u64,
    lift: f32,
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
    Some(
        Transform::from_xyz(point.x, hit.elevation.metres() + lift, point.y).with_rotation(
            Quat::from_rotation_arc(Vec3::Y, *hit.normal)
                * Quat::from_rotation_y(
                    streams::ROCK_YAW.rng(hash.into(), &[]).inclusive_unit_f32()
                        * core::f32::consts::TAU,
                ),
        ),
    )
}

pub(super) fn tree_root_height(
    terrain: &SceneTerrain,
    lod: &VistaLod,
    coarser: Option<&VistaLod>,
    world: Vec2,
) -> Option<f32> {
    if terrain.property_surface().is_some() {
        return terrain
            .surface_below(
                adventuresim_tactical_core::city_layout::grounding::SupportQuery::unbounded(
                    adventuresim_tactical_core::scene_coordinates::ScenePlanPoint::from_metres(
                        bevy::math::Vec2::new(world.x, world.y),
                    )?,
                ),
            )
            .map(|hit| hit.elevation.metres());
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
    point: Vec2,
) -> Option<adventuresim_tactical_core::city_layout::grounding::SurfaceHit> {
    if playable_terrain.property_surface().is_some() {
        return playable_terrain.surface_below(
            adventuresim_tactical_core::city_layout::grounding::SupportQuery::unbounded(
                adventuresim_tactical_core::scene_coordinates::ScenePlanPoint::from_metres(
                    bevy::math::Vec2::new(point.x, point.y),
                )?,
            ),
        );
    }
    let origin = Vec2::new(
        lod.origin_east_metres as f32,
        lod.origin_north_metres as f32,
    );
    let local = point - origin;
    let height = presented_vista_vertex_height(
        lod,
        coarser_lod,
        Some(playable_terrain),
        local,
        playable_half_extent,
    )?;
    let delta = VISTA_SCENERY_NORMAL_SAMPLE_DISTANCE_METRES;
    let at = |offset: Vec2| {
        presented_vista_vertex_height(
            lod,
            coarser_lod,
            Some(playable_terrain),
            local + offset,
            playable_half_extent,
        )
        .unwrap_or(height)
    };
    let tangent_x = Vec3::new(delta * 2.0, at(Vec2::X * delta) - at(-Vec2::X * delta), 0.0);
    let tangent_z = Vec3::new(0.0, at(Vec2::Y * delta) - at(-Vec2::Y * delta), delta * 2.0);
    let normal = tangent_z.cross(tangent_x);
    adventuresim_tactical_core::city_layout::grounding::SurfaceHit::from_geometry(height, normal)
}

#[cfg(test)]
mod tests;
