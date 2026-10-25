//! Admit native obstacle-grid cells to scene geometry before terrain sampling.
use super::{GeneratedObstacle, SceneTerrain};
use crate::scene_coordinates::ScenePlanPoint;
use adventuresim_building_generator::spatial_geometry::{GeometryError, PositiveLength};
use bevy::math::Vec2;
#[derive(Clone, Copy)]
pub(super) struct TerrainRockInfluence {
    pub(super) centre: ScenePlanPoint,
    pub(super) radius: PositiveLength,
}
#[derive(Default)]
pub(super) struct TerrainDetailObstacles {
    pub(super) trees: Vec<ScenePlanPoint>,
    pub(super) rocks: Vec<TerrainRockInfluence>,
}
impl TerrainDetailObstacles {
    /// Row-major cell ordinals and metre spacing are the obstacle-grid port.
    pub(super) fn from_obstacles(
        terrain: &SceneTerrain,
        obstacles: &[GeneratedObstacle],
        obstacle_spacing: f32,
    ) -> Result<Self, GeometryError> {
        let half_extent = Vec2::new(terrain.width(), terrain.depth()) * 0.5;
        let mut influences = Self::default();
        for obstacle in obstacles {
            match *obstacle {
                GeneratedObstacle::Tree { x, z } => {
                    influences.trees.push(ScenePlanPoint::try_from(
                        Vec2::new(
                            f32::from(x) * obstacle_spacing,
                            f32::from(z) * obstacle_spacing,
                        ) - half_extent,
                    )?)
                }
                GeneratedObstacle::Rock { x, z, recipe } => {
                    influences.rocks.push(TerrainRockInfluence {
                        centre: ScenePlanPoint::try_from(
                            Vec2::new(
                                f32::from(x) * obstacle_spacing,
                                f32::from(z) * obstacle_spacing,
                            ) - half_extent,
                        )?,
                        radius: PositiveLength::from_metres(recipe.collision_radius_metres())?,
                    })
                }
            }
        }
        Ok(influences)
    }
}
