use crate::scene_spawn::SceneSpawnResult;
use adventuresim_building_generator::spatial_geometry::Position;
use adventuresim_tactical_core::{prelude::*, scene_coordinates::{Scene, ScenePlanPoint}};
use adventuresim_tactical_netcode::bevy_replicon::prelude::Replicated;
use bevy::prelude::*;

struct AdmittedObstacle {
    kind: SceneObstacle,
    collider: Collider,
    position: Position<Scene>,
    rotation: Quat,
}

impl AdmittedObstacle {
    fn from_generated(
        obstacle: GeneratedObstacle,
        terrain: &SceneTerrain,
        obstacle_spacing: f32,
    ) -> SceneSpawnResult<Self> {
        let (grid_x, grid_z, kind, collider, height_offset) = match obstacle {
            GeneratedObstacle::Tree { x, z } => (
                x,
                z,
                SceneObstacle::Tree,
                Collider::cylinder(TREE_TRUNK_RADIUS_METRES, TREE_TRUNK_HEIGHT_METRES),
                TREE_TRUNK_HEIGHT_METRES * 0.5,
            ),
            GeneratedObstacle::Rock { x, z, recipe } => (
                x,
                z,
                SceneObstacle::Rock(recipe),
                Collider::sphere(recipe.collision_radius_metres()),
                recipe.collision_radius_metres(),
            ),
        };
        // Decode the serialized grid ordinals into scene-local metres here;
        // preserve multiply-then-subtract rounding at this recipe boundary.
        let x = f32::from(grid_x) * obstacle_spacing - terrain.width() * 0.5;
        let z = f32::from(grid_z) * obstacle_spacing - terrain.depth() * 0.5;
        let point = ScenePlanPoint::try_from(Vec2::new(x, z))?;
        let elevation = terrain.required_height_at(point)?;
        let position = Position::from_metres(Vec3::new(x, elevation.metres() + height_offset, z))?;
        let yaw = match kind {
            SceneObstacle::Rock(recipe) => {
                (recipe.seed.to_u64() >> 40) as f32 / ((1_u32 << 24) - 1) as f32
                    * core::f32::consts::TAU
            }
            SceneObstacle::Tree => 0.0,
        };
        Ok(Self { kind, collider, position, rotation: Quat::from_rotation_y(yaw) })
    }
}

pub(crate) fn spawn(
    commands: &mut Commands,
    obstacles: Vec<GeneratedObstacle>,
    terrain: &SceneTerrain,
    obstacle_spacing: f32,
) -> SceneSpawnResult<()> {
    let admitted = obstacles.into_iter()
        .map(|obstacle| AdmittedObstacle::from_generated(obstacle, terrain, obstacle_spacing))
        .collect::<SceneSpawnResult<Vec<_>>>()?;
    for obstacle in admitted {
        let label = match obstacle.kind {
            SceneObstacle::Tree => "tree trunk",
            SceneObstacle::Rock(_) => "rock",
        };
        commands.spawn((
            Replicated,
            Name::new(format!("Tactical scene {label}")),
            obstacle.kind,
            RigidBody::Static,
            CollisionLayers::new(TACTICAL_TERRAIN_LAYER, LayerMask::ALL),
            obstacle.collider,
            Transform::from_translation(obstacle.position.metres()).with_rotation(obstacle.rotation),
        ));
    }
    Ok(())
}
