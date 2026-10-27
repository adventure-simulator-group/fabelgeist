//! Landform fields extend the actual scene support at its clamped edge.
use super::{TerrainRecipeError, TerrainRecipeResult};
use crate::{
    city_layout::grounding::SupportElevation, scene::SceneTerrain,
    scene_coordinates::ScenePlanPoint,
};
use bevy::math::Vec2;

pub(crate) fn height_at_extended_edge(
    terrain: &SceneTerrain,
    point: ScenePlanPoint,
) -> TerrainRecipeResult<SupportElevation> {
    let half = Vec2::new(terrain.width(), terrain.depth()) * 0.5;
    let boundary = ScenePlanPoint::from_metres(point.metres().clamp(-half, half))
        .ok_or(TerrainRecipeError::NonFiniteField)?;
    Ok(terrain.required_height_at(boundary)?)
}
