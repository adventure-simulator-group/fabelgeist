use adventuresim_tactical_core::prelude::*;
use adventuresim_tactical_netcode::bevy_replicon::prelude::Replicated;
use bevy::prelude::*;

pub(crate) fn spawn(
    commands: &mut Commands,
    obstacles: Vec<GeneratedObstacle>,
    terrain: &SceneTerrain,
    obstacle_spacing: f32,
) {
    for obstacle in obstacles {
        let (grid_x, grid_z, kind, collider, height_offset, label) = match obstacle {
            GeneratedObstacle::Tree { x, z } => (
                x,
                z,
                SceneObstacle::Tree,
                Collider::cylinder(TREE_TRUNK_RADIUS_METRES, TREE_TRUNK_HEIGHT_METRES),
                TREE_TRUNK_HEIGHT_METRES * 0.5,
                "tree trunk",
            ),
            GeneratedObstacle::Rock { x, z, recipe } => (
                x,
                z,
                SceneObstacle::Rock(recipe),
                Collider::sphere(recipe.collision_radius_metres()),
                recipe.collision_radius_metres(),
                "rock",
            ),
        };
        let x = f32::from(grid_x) * obstacle_spacing - terrain.width() * 0.5;
        let z = f32::from(grid_z) * obstacle_spacing - terrain.depth() * 0.5;
        let y = terrain.height_at(Vec2::new(x, z)).unwrap_or_default() + height_offset;
        let yaw = match kind {
            SceneObstacle::Rock(recipe) => {
                (recipe.seed >> 40) as f32 / ((1_u32 << 24) - 1) as f32 * core::f32::consts::TAU
            }
            SceneObstacle::Tree => 0.0,
        };
        commands.spawn((
            Replicated,
            Name::new(format!("Tactical scene {label}")),
            kind,
            RigidBody::Static,
            CollisionLayers::new(TACTICAL_TERRAIN_LAYER, LayerMask::ALL),
            collider,
            Transform::from_xyz(x, y, z).with_rotation(Quat::from_rotation_y(yaw)),
        ));
    }
}
