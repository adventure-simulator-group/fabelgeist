//! The arena drawn: its boxes, a sun, and the camera.

use bevy::prelude::*;

use crate::arena::{self, BlockKind};

pub(super) struct ArenaViewPlugin;

impl Plugin for ArenaViewPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ClearColor(Color::srgb(0.55, 0.62, 0.7)))
            .add_systems(Startup, draw_arena);
    }
}

fn draw_arena(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 4.0, 8.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    commands.spawn((
        DirectionalLight {
            illuminance: 9_000.0,
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_xyz(6.0, 12.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    commands.insert_resource(GlobalAmbientLight {
        brightness: 400.0,
        ..default()
    });
    let color = |kind| match kind {
        BlockKind::Floor => Color::srgb(0.42, 0.40, 0.35),
        BlockKind::Wall => Color::srgb(0.55, 0.50, 0.44),
        BlockKind::Pillar => Color::srgb(0.48, 0.45, 0.42),
    };
    for block in arena::blocks() {
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::from_size(block.size))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: color(block.kind),
                perceptual_roughness: 0.9,
                ..default()
            })),
            Transform::from_translation(block.center),
        ));
    }
}
