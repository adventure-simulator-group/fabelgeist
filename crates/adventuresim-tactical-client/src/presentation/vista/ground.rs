//! Select canonical owned terrain or sampled natural terrain for a vista ring.
use super::*;

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
