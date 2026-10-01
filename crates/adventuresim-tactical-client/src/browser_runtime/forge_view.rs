//! The forge preview shares the canvas without moving or relighting the city.
use crate::strategic_scene::protocol::{FORGE_LAYER, StrategicView};
use bevy::{camera::visibility::RenderLayers, prelude::*};
#[derive(Component)]
pub(super) struct ForgeCamera;
pub(super) fn setup(mut commands: Commands) {
    commands.spawn((
        ForgeCamera,
        Camera3d::default(),
        Camera {
            order: 128,
            is_active: false,
            ..default()
        },
        Transform::IDENTITY,
        Projection::Perspective(PerspectiveProjection {
            fov: crate::presentation::TacticalCameraSetup::default()
                .vertical_fov_degrees
                .to_radians(),
            ..default()
        }),
        RenderLayers::layer(FORGE_LAYER),
        bevy::camera::Exposure { ev100: 8.0 },
        AmbientLight {
            color: Color::srgb(0.95, 0.82, 0.66),
            brightness: 1_200.0,
            ..default()
        },
    ));
}
pub(super) fn sync(
    view: Option<Res<StrategicView>>,
    windows: Query<&Window>,
    mut cameras: Query<&mut Camera, With<ForgeCamera>>,
) {
    for mut camera in &mut cameras {
        camera.is_active = false;
        if let Some(rect) = view.as_ref().and_then(|view| view.forge)
            && let Some(window) = windows.iter().next()
        {
            rect.apply(&mut camera, window.resolution.physical_size());
        }
    }
}
