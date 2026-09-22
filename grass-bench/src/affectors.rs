//! Things that push grass aside: N small spheres circling the patch centre.
//! Their positions feed the custom material's affector uniform array (mesh
//! chunk grass) and the game grass material's single interaction slot
//! (instanced grass, first affector only, as the game only has one).

use bevy::prelude::*;

use crate::scene::{hash01, terrain_height};
use crate::settings::BenchSettings;

#[derive(Clone, Copy, Debug)]
pub struct Affector {
    pub position: Vec3,
    pub radius: f32,
    pub velocity: Vec3,
}

#[derive(Resource, Default, Clone, bevy::render::extract_resource::ExtractResource)]
pub struct Affectors {
    pub list: Vec<Affector>,
}

#[derive(Component)]
struct AffectorBall {
    index: u32,
    orbit_radius: f32,
    speed: f32,
    phase: f32,
}

pub struct AffectorsPlugin;

impl Plugin for AffectorsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Affectors>()
            .add_systems(Update, (spawn_affectors, move_affectors, feed_instanced_grass).chain());
    }
}

const RADIUS: f32 = 1.35;

fn spawn_affectors(
    mut commands: Commands,
    settings: Res<BenchSettings>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut standards: ResMut<Assets<StandardMaterial>>,
    existing: Query<Entity, With<AffectorBall>>,
    mut spawned: Local<Option<u32>>,
) {
    if *spawned == Some(settings.affector_count) {
        return;
    }
    *spawned = Some(settings.affector_count);
    for entity in &existing {
        commands.entity(entity).despawn();
    }
    if settings.affector_count == 0 {
        return;
    }
    let mesh = meshes.add(Sphere::new(0.35));
    let material = standards.add(StandardMaterial {
        base_color: Color::srgb(0.9, 0.3, 0.2),
        perceptual_roughness: 0.6,
        ..default()
    });
    for i in 0..settings.affector_count {
        commands.spawn((
            Name::new(format!("affector {i}")),
            AffectorBall {
                index: i,
                orbit_radius: 4.0 + hash01(i as u64, 31) * 12.0,
                speed: (1.2 + hash01(i as u64, 32) * 1.6) * if i % 2 == 0 { 1.0 } else { -1.0 },
                phase: hash01(i as u64, 33) * std::f32::consts::TAU,
            },
            Mesh3d(mesh.clone()),
            MeshMaterial3d(material.clone()),
            Transform::default(),
            Visibility::default(),
        ));
    }
}

fn move_affectors(
    time: Res<Time>,
    mut balls: Query<(&AffectorBall, &mut Transform)>,
    mut affectors: ResMut<Affectors>,
) {
    let t = time.elapsed_secs();
    let mut list: Vec<(u32, Affector)> = Vec::new();
    for (ball, mut transform) in &mut balls {
        let angle = ball.phase + t * ball.speed / ball.orbit_radius;
        let x = angle.cos() * ball.orbit_radius;
        let z = angle.sin() * ball.orbit_radius;
        let position = Vec3::new(x, terrain_height(x, z) + 0.35, z);
        let velocity = Vec3::new(-angle.sin(), 0.0, angle.cos()) * ball.speed;
        transform.translation = position;
        list.push((
            ball.index,
            Affector {
                position,
                radius: RADIUS,
                velocity,
            },
        ));
    }
    list.sort_by_key(|(index, _)| *index);
    let next: Vec<Affector> = list.into_iter().map(|(_, a)| a).collect();
    if next.is_empty() && affectors.list.is_empty() {
        return;
    }
    affectors.list = next;
}

/// The game's grass material has one interaction slot: the first affector
/// drives it on both instanced paths (the chunked/culled uniform and every
/// eidolon batch material) so the grass reacts there too.
pub fn feed_instanced_grass(
    affectors: Res<Affectors>,
    settings: Res<BenchSettings>,
    mut simple_params: Option<ResMut<crate::grass::simple::GrassSimpleParams>>,
    mut eidolon_materials: ResMut<Assets<crate::grass::eidolon::GrassMaterial>>,
) {
    if !affectors.is_changed() && !settings.is_changed() {
        return;
    }
    // Backlit glow rides in the spare half of the shading vector.
    let backlight = Vec2::new(settings.transmit, settings.transmit_power);
    let (interaction, motion) = match affectors.list.first() {
        Some(a) => (
            a.position.extend(RADIUS),
            a.velocity.extend(0.7),
        ),
        None => (Vec4::new(1.0e5, 0.0, 1.0e5, RADIUS), Vec4::new(0.0, 0.0, 0.0, 0.7)),
    };
    if let Some(params) = simple_params.as_mut() {
        for tier in params.tiers.iter_mut() {
            tier.interaction = interaction;
            tier.interaction_motion = motion;
            tier.shading.z = backlight.x;
            tier.shading.w = backlight.y;
        }
    }
    for (_, material) in eidolon_materials.iter_mut() {
        material.interaction = interaction;
        material.interaction_motion = motion;
        material.shading.z = backlight.x;
        material.shading.w = backlight.y;
    }
}
