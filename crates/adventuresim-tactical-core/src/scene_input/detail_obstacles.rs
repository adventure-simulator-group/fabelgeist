//! Convert obstacle cells into their physical terrain-detail influences.
use super::{GeneratedObstacle, SceneTerrain};
use bevy::math::Vec2;

#[derive(Clone, Copy)]
pub(super) struct TerrainRockInfluence {
    pub(super) centre: Vec2,
    pub(super) radius: f32,
}

#[derive(Default)]
pub(super) struct TerrainDetailObstacles {
    pub(super) trees: Vec<Vec2>,
    pub(super) rocks: Vec<TerrainRockInfluence>,
}

impl TerrainDetailObstacles {
    pub(super) fn from_obstacles(
        terrain: &SceneTerrain,
        obstacles: &[GeneratedObstacle],
        obstacle_spacing: f32,
    ) -> Self {
        let half_extent = Vec2::new(terrain.width(), terrain.depth()) * 0.5;
        let mut influences = Self::default();
        for obstacle in obstacles {
            match *obstacle {
                GeneratedObstacle::Tree { x, z } => influences.trees.push(
                    Vec2::new(
                        f32::from(x) * obstacle_spacing,
                        f32::from(z) * obstacle_spacing,
                    ) - half_extent,
                ),
                GeneratedObstacle::Rock { x, z, recipe } => {
                    influences.rocks.push(TerrainRockInfluence {
                        centre: Vec2::new(
                            f32::from(x) * obstacle_spacing,
                            f32::from(z) * obstacle_spacing,
                        ) - half_extent,
                        radius: recipe.collision_radius_metres(),
                    });
                }
            }
        }
        influences
    }
}
