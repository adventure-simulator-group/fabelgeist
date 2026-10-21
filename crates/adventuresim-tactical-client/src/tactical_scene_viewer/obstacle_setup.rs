//! Native obstacle shapes and explicit membership of their real terrain scene.

use super::*;
use adventuresim_tactical_core::prelude::SceneObstacleOf;
use anyhow::{Context, Result, ensure};

const OVERLAY_RADIAL_SCALE: f32 = 1.08;
const OVERLAY_HEIGHT_SCALE: f32 = 1.02;

/// Native generated-grid and physics/mesh construction boundary.
/// Grid indices remain in the admitted playable grid until the caller applies
/// spacing in metres; y_offset is the native collider-centre height in metres.
/// Mesh handles and collider values enter their respective Bevy stores directly.
pub(super) struct ObstacleShape {
    pub grid_x: u16,
    pub grid_z: u16,
    pub kind: SceneObstacle,
    pub collider: Collider,
    pub y_offset: f32,
    pub overlay_shape: Handle<Mesh>,
    pub overlay_color: Color,
}

#[derive(Clone, Copy)]
pub(super) struct SpawnedObstacle {
    pub entity: Entity,
    pub kind: SceneObstacle,
    pub collider_frame: Transform,
}

#[derive(Resource)]
pub(super) struct CapturedObstacleBatch {
    scene: Entity,
    members: Vec<SpawnedObstacle>,
}

/// Numerical receipt for the original batch, not a replacement render layout.
#[derive(Serialize)]
pub(super) struct ObstacleBatchReceipt {
    scene_entity_bits: u64,
    trees: usize,
    rocks: usize,
    roots: Vec<ObstacleRootReceipt>,
}

#[derive(Serialize)]
struct ObstacleRootReceipt {
    entity_bits: u64,
    kind: SceneObstacle,
    collider_world_from_root: [f32; 16],
}

impl ObstacleShape {
    pub(super) fn from_generated(obstacle: GeneratedObstacle, meshes: &mut Assets<Mesh>) -> Self {
        match obstacle {
            GeneratedObstacle::Tree { x, z } => Self {
                grid_x: x,
                grid_z: z,
                kind: SceneObstacle::Tree,
                collider: Collider::cylinder(TREE_TRUNK_RADIUS_METRES, TREE_TRUNK_HEIGHT_METRES),
                y_offset: TREE_TRUNK_HEIGHT_METRES * 0.5,
                overlay_shape: meshes.add(Cylinder::new(
                    TREE_TRUNK_RADIUS_METRES * OVERLAY_RADIAL_SCALE,
                    TREE_TRUNK_HEIGHT_METRES * OVERLAY_HEIGHT_SCALE,
                )),
                overlay_color: Color::srgb(0.1, 1.0, 0.85),
            },
            GeneratedObstacle::Rock { x, z, recipe } => Self {
                grid_x: x,
                grid_z: z,
                kind: SceneObstacle::Rock(recipe),
                collider: Collider::sphere(recipe.collision_radius_metres()),
                y_offset: recipe.collision_radius_metres(),
                overlay_shape: meshes.add(Sphere::new(
                    recipe.collision_radius_metres() * OVERLAY_RADIAL_SCALE,
                )),
                overlay_color: Color::srgb(1.0, 0.08, 0.72),
            },
        }
    }
}

impl CapturedObstacleBatch {
    pub(super) fn members(&self) -> impl Iterator<Item = &SpawnedObstacle> {
        self.members.iter()
    }

    /// The actual spawner binds its exact members after terrain creation.
    pub(super) fn bind(
        commands: &mut Commands,
        scene: Entity,
        members: Vec<SpawnedObstacle>,
        scene_digest: String,
    ) {
        for member in &members {
            commands
                .entity(member.entity)
                .insert(SceneObstacleOf(scene));
        }
        commands.insert_resource(Self { scene, members });
        commands.queue(move |world: &mut World| -> bevy::prelude::Result {
            Self::check(world, &scene_digest)?;
            Ok(())
        });
    }

    /// Verify the one-scene native batch before consuming its real roots.
    /// Collider presence is observed; this does not seal collider shape bytes.
    pub(super) fn check(world: &mut World, scene_digest: &str) -> Result<ObstacleBatchReceipt> {
        let mut scenes =
            world.query_filtered::<Entity, (With<SceneTerrain>, With<SceneEnvironment>)>();
        let scene = scenes.single(world)?;
        let mut obstacles = world.query_filtered::<Entity, With<SceneObstacle>>();
        let actual_obstacles = obstacles.iter(world).count();
        let batch = world
            .get_resource::<Self>()
            .context("native obstacle spawn batch missing")?;
        ensure!(
            batch.scene == scene,
            "native obstacle batch belongs to another scene"
        );
        let environment = world
            .get::<SceneEnvironment>(scene)
            .context("native obstacle scene environment missing")?;
        ensure!(
            environment.scene_digest == scene_digest,
            "native obstacle batch scene digest mismatch"
        );
        ensure!(
            actual_obstacles == batch.members.len(),
            "native obstacle batch has missing or extra members"
        );
        let mut roots = Vec::with_capacity(batch.members.len());
        let mut unique = std::collections::HashSet::new();
        let mut trees = 0;
        let mut rocks = 0;
        for member in &batch.members {
            ensure!(
                unique.insert(member.entity),
                "native obstacle batch repeats a member"
            );
            ensure!(
                world.get::<SceneObstacleOf>(member.entity) == Some(&SceneObstacleOf(scene)),
                "native obstacle scene ownership changed"
            );
            ensure!(
                world.get::<SceneObstacle>(member.entity) == Some(&member.kind),
                "native obstacle kind changed"
            );
            ensure!(
                world.get::<ChildOf>(member.entity).is_none(),
                "native obstacle collider root became parented"
            );
            ensure!(
                world.get::<Transform>(member.entity) == Some(&member.collider_frame),
                "native obstacle collider frame changed"
            );
            ensure!(
                world.get::<RigidBody>(member.entity) == Some(&RigidBody::Static),
                "native obstacle static body changed"
            );
            ensure!(
                world.get::<CollisionLayers>(member.entity)
                    == Some(&CollisionLayers::new(
                        TACTICAL_TERRAIN_LAYER,
                        LayerMask::ALL
                    )),
                "native obstacle collision layers changed"
            );
            ensure!(
                world.get::<Collider>(member.entity).is_some(),
                "native obstacle collider missing"
            );
            match member.kind {
                SceneObstacle::Tree => trees += 1,
                SceneObstacle::Rock(_) => rocks += 1,
            }
            roots.push(ObstacleRootReceipt {
                entity_bits: member.entity.to_bits(),
                kind: member.kind,
                collider_world_from_root: member.collider_frame.to_matrix().to_cols_array(),
            });
        }
        Ok(ObstacleBatchReceipt {
            scene_entity_bits: scene.to_bits(),
            trees,
            rocks,
            roots,
        })
    }
}
