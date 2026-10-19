//! One diagnostic view and lighting layer; no unrelated scene is destroyed.

use super::session::Session;
use adventuresim_environment::render::{ForestGeometry, ForestViewCamera, view::ForestView};
use bevy::{camera::visibility::RenderLayers, light::CascadeShadowConfigBuilder, prelude::*};

pub(super) const SKY: Color = Color::srgb(0.62, 0.74, 0.88);
const REVIEW_LAYER: usize = 30;
const SKY_BRIGHTNESS: f32 = 500.0;
const SHADOW_REACH_METRES: f32 = 600.0;
const SUN_AZIMUTH: f32 = 0.9;
const SUN_ELEVATION: f32 = 0.75;

#[derive(Component)]
pub(super) struct ReviewCamera;
#[derive(Component)]
pub(super) struct ReviewOwned;

pub(super) fn place(world: &mut World, old: Option<Entity>, view: ForestView) -> Entity {
    if let Some(old) = old {
        world.despawn(old);
    }
    world
        .spawn((
            ReviewOwned,
            ReviewCamera,
            ForestViewCamera,
            Camera3d::default(),
            Camera {
                clear_color: ClearColorConfig::Custom(SKY),
                ..default()
            },
            view.lens(),
            view.transform(),
            AmbientLight {
                color: SKY,
                brightness: SKY_BRIGHTNESS,
                ..default()
            },
            RenderLayers::layer(REVIEW_LAYER),
        ))
        .id()
}

pub(super) fn lighting(world: &mut World) {
    let (sin_azimuth, cos_azimuth) = SUN_AZIMUTH.sin_cos();
    let (sin_elevation, cos_elevation) = SUN_ELEVATION.sin_cos();
    let towards_sun = Vec3::new(
        cos_elevation * sin_azimuth,
        sin_elevation,
        cos_elevation * cos_azimuth,
    );
    world.spawn((
        ReviewOwned,
        DirectionalLight {
            illuminance: light_consts::lux::FULL_DAYLIGHT,
            shadow_maps_enabled: true,
            ..default()
        },
        CascadeShadowConfigBuilder {
            maximum_distance: SHADOW_REACH_METRES,
            ..default()
        }
        .build(),
        Transform::default().looking_to(-towards_sun, Vec3::Y),
        RenderLayers::layer(REVIEW_LAYER),
    ));
}

pub(super) fn isolate(
    mut commands: Commands,
    session: Res<Session>,
    mut cameras: Query<&mut Camera, Without<ReviewCamera>>,
    geometry: Query<Entity, (With<ForestGeometry>, Without<ReviewOwned>)>,
) {
    if !session.is_active() {
        return;
    }
    for mut camera in &mut cameras {
        camera.is_active = false;
    }
    for entity in &geometry {
        commands
            .entity(entity)
            .insert((ReviewOwned, RenderLayers::layer(REVIEW_LAYER)));
    }
}
