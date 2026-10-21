//! Refinement-local obstacle admission and immutable relief preparation.
use super::{DetailNoise, GeneratedObstacle, SceneTerrain, TerrainShapeSample};
use crate::scene_coordinates::ScenePlanPoint;
use adventuresim_building_generator::spatial_geometry::{GeometryError, PositiveLength};
use bevy::math::Vec2;
use fabelgeist_determinism::Seed;
use rocks::TerrainRockInfluence;
use roots::TerrainTreeInfluence;

mod rocks;
mod roots;

#[derive(Default)]
pub(super) struct TerrainDetailObstacles {
    trees: Vec<TerrainTreeInfluence>,
    rocks: Vec<TerrainRockInfluence>,
}

impl TerrainDetailObstacles {
    /// Row-major cell ordinals and metre spacing are the obstacle-grid port.
    pub(super) fn from_obstacles(
        detail_seed: Seed,
        terrain: &SceneTerrain,
        obstacles: &[GeneratedObstacle],
        obstacle_spacing: f32,
    ) -> Result<Self, GeometryError> {
        let half_extent = Vec2::new(terrain.width(), terrain.depth()) * 0.5;
        let mut influences = Self::default();
        for obstacle in obstacles {
            match *obstacle {
                GeneratedObstacle::Tree { x, z } => {
                    let centre = ScenePlanPoint::try_from(
                        Vec2::new(
                            f32::from(x) * obstacle_spacing,
                            f32::from(z) * obstacle_spacing,
                        ) - half_extent,
                    )?;
                    influences
                        .trees
                        .push(TerrainTreeInfluence::new(detail_seed, centre)?);
                }
                GeneratedObstacle::Rock { x, z, recipe } => {
                    let centre = ScenePlanPoint::try_from(
                        Vec2::new(
                            f32::from(x) * obstacle_spacing,
                            f32::from(z) * obstacle_spacing,
                        ) - half_extent,
                    )?;
                    let radius = PositiveLength::from_metres(recipe.collision_radius_metres())?;
                    influences
                        .rocks
                        .push(TerrainRockInfluence::new(detail_seed, centre, radius)?);
                }
            }
        }
        Ok(influences)
    }

    /// Preserve native scene sampling and original obstacle accumulation order.
    pub(super) fn root_relief(&self, point: Vec2) -> f32 {
        let mut relief = 0.0;
        for tree in &self.trees {
            if let Some(value) = tree.relief(point) {
                relief += value;
            }
        }
        relief.clamp(-0.02, 0.115)
    }

    pub(super) fn rock_relief(
        &self,
        point: Vec2,
        shape: Option<TerrainShapeSample>,
        noise: &DetailNoise,
    ) -> f32 {
        let mut relief = 0.0;
        for rock in &self.rocks {
            if let Some(value) = rock.relief(point, shape, noise) {
                relief += value;
            }
        }
        relief.clamp(-0.055, 0.07)
    }
}
