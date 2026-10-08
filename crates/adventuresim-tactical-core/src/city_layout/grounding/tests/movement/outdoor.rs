//! Install the server's physical outdoor instances in an acceptance world.
use super::*;
use crate::scene_input::furniture::{FurnitureLayout, furniture_collider};
use crate::scene_input::{GeneratedObstacle, TacticalSceneInput};

pub(super) struct OutdoorCollision(Vec<(Name, Collider, Transform)>);

impl OutdoorCollision {
    pub(super) fn compile(
        input: &TacticalSceneInput,
        terrain: &crate::scene::SceneTerrain,
        obstacles: &[GeneratedObstacle],
        furniture: &FurnitureLayout,
    ) -> Self {
        let mut bodies = Vec::new();
        for obstacle in obstacles {
            let (x, z, shape, offset, yaw) = match *obstacle {
                GeneratedObstacle::Tree { x, z } => (
                    x,
                    z,
                    Collider::cylinder(
                        crate::scene_input::TREE_TRUNK_RADIUS_METRES,
                        crate::scene_input::TREE_TRUNK_HEIGHT_METRES,
                    ),
                    crate::scene_input::TREE_TRUNK_HEIGHT_METRES * 0.5,
                    0.0,
                ),
                GeneratedObstacle::Rock { x, z, recipe } => (
                    x,
                    z,
                    Collider::sphere(recipe.collision_radius_metres()),
                    recipe.collision_radius_metres(),
                    (recipe.seed.to_u64() >> 40) as f32 / ((1_u32 << 24) - 1) as f32
                        * core::f32::consts::TAU,
                ),
            };
            let point = Vec2::new(f32::from(x), f32::from(z)) * input.playable.spacing_metres
                - Vec2::new(terrain.width(), terrain.depth()) * 0.5;
            let height = terrain
                .height_at(point)
                .expect("physical obstacle has source support");
            bodies.push((
                Name::new(format!("Obstacle {x},{z}")),
                shape,
                Transform::from_xyz(point.x, height + offset, point.y)
                    .with_rotation(Quat::from_rotation_y(yaw)),
            ));
        }
        // Distant instances are accepted scenery without server physics.
        for instance in &furniture.instances {
            bodies.push((
                Name::new(format!("Furniture {}", instance.scene.id.0)),
                furniture_collider(instance.scene.key).unwrap(),
                Transform::from_translation(instance.position_metres.metres())
                    .with_rotation(Quat::from_rotation_y(instance.orientation.yaw_radians())),
            ));
        }
        Self(bodies)
    }

    pub(super) fn install(
        &self,
        app: &mut App,
        intersects: impl Fn(Vec3, &Collider, Quat) -> bool,
    ) {
        for (name, shape, transform) in &self.0 {
            if intersects(transform.translation, shape, transform.rotation) {
                app.world_mut()
                    .spawn((name.clone(), RigidBody::Static, shape.clone(), *transform));
            }
        }
    }
}
