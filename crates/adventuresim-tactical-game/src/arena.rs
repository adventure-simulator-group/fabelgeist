//! The greybox arena both peers collide with: a floor ringed by low walls, and a few pillars to
//! fight around. The same on both sides, spawned at startup.

use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_netahoy::WORLD_COLLISION_LAYER;

/// Half the arena's width and depth inside its walls (m).
pub const HALF_EXTENT: f32 = 14.0;
const WALL_HEIGHT: f32 = 1.2;
const WALL_THICKNESS: f32 = 0.6;
const FLOOR_THICKNESS: f32 = 1.0;
const PILLAR_SIZE: Vec3 = Vec3::new(1.2, 3.0, 1.2);
const PILLARS: [Vec2; 4] = [
    Vec2::new(-5.0, -5.0),
    Vec2::new(5.0, -5.0),
    Vec2::new(-5.0, 5.0),
    Vec2::new(5.0, 5.0),
];

/// One box of the arena: its centre and full size.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Block {
    /// Its centre (m).
    pub center: Vec3,
    /// Its full extents (m).
    pub size: Vec3,
    /// What it is, for drawing it.
    pub kind: BlockKind,
}

/// What a [`Block`] is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlockKind {
    /// The ground.
    Floor,
    /// The ring around it.
    Wall,
    /// Something to fight around.
    Pillar,
}

/// Every box of the arena.
pub fn blocks() -> Vec<Block> {
    let span = 2.0 * (HALF_EXTENT + WALL_THICKNESS);
    let mut blocks = vec![Block {
        center: Vec3::new(0.0, -FLOOR_THICKNESS / 2.0, 0.0),
        size: Vec3::new(span, FLOOR_THICKNESS, span),
        kind: BlockKind::Floor,
    }];
    let offset = HALF_EXTENT + WALL_THICKNESS / 2.0;
    for (center, size) in [
        (Vec2::new(0.0, offset), Vec2::new(span, WALL_THICKNESS)),
        (Vec2::new(0.0, -offset), Vec2::new(span, WALL_THICKNESS)),
        (Vec2::new(offset, 0.0), Vec2::new(WALL_THICKNESS, span)),
        (Vec2::new(-offset, 0.0), Vec2::new(WALL_THICKNESS, span)),
    ] {
        blocks.push(Block {
            center: Vec3::new(center.x, WALL_HEIGHT / 2.0, center.y),
            size: Vec3::new(size.x, WALL_HEIGHT, size.y),
            kind: BlockKind::Wall,
        });
    }
    for pillar in PILLARS {
        blocks.push(Block {
            center: Vec3::new(pillar.x, PILLAR_SIZE.y / 2.0, pillar.y),
            size: PILLAR_SIZE,
            kind: BlockKind::Pillar,
        });
    }
    blocks
}

/// Where fighter `index` (0 first) enters: round a circle in the arena's middle, facing in.
pub fn spawn_point(index: u64) -> Vec3 {
    const RADIUS: f32 = 3.5;
    const SLOTS: u64 = 8;
    let angle = (index % SLOTS) as f32 / SLOTS as f32 * std::f32::consts::TAU;
    Vec3::new(RADIUS * angle.cos(), 1.0, RADIUS * angle.sin())
}

/// The arena's colliders, on the world layer the controllers collide with.
pub fn spawn_colliders(mut commands: Commands) {
    for block in blocks() {
        commands.spawn((
            Name::new(format!("arena {:?}", block.kind)),
            Transform::from_translation(block.center),
            RigidBody::Static,
            Collider::cuboid(block.size.x, block.size.y, block.size.z),
            CollisionLayers::new(WORLD_COLLISION_LAYER, LayerMask::ALL),
        ));
    }
}
