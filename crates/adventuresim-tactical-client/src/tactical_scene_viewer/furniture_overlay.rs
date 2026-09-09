//! Debug-only annotations of accepted placement clearances.
use super::CaptureOverlay;
use adventuresim_tactical_core::prelude::*;
use bevy::prelude::*;

const OVERLAY_LIFT_METRES: f32 = 0.12;
const OVERLAY_LINE_METRES: f32 = 0.08;

pub(super) fn annotate(
    commands: &mut Commands,
    layout: &FurnitureLayout,
    terrain: &SceneTerrain,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    let group_material = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.7, 0.1),
        unlit: true,
        ..default()
    });
    let route_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.1, 0.9, 1.0),
        unlit: true,
        ..default()
    });
    for (footprint, material) in layout
        .groups
        .iter()
        .map(|group| (group.footprint, &group_material))
        .chain(
            layout
                .reserved_routes
                .iter()
                .copied()
                .map(|route| (route, &route_material)),
        )
    {
        let corners = footprint.corners();
        for index in 0..corners.len() {
            let start = corners[index];
            let end = corners[(index + 1) % corners.len()];
            let centre = (start + end) * 0.5;
            let Some(height) = terrain.height_at(centre) else {
                continue;
            };
            let delta = end - start;
            commands.spawn((
                CaptureOverlay,
                Mesh3d(meshes.add(Cuboid::new(
                    delta.length(),
                    OVERLAY_LINE_METRES,
                    OVERLAY_LINE_METRES,
                ))),
                MeshMaterial3d(material.clone()),
                Transform::from_xyz(centre.x, height + OVERLAY_LIFT_METRES, centre.y)
                    .with_rotation(Quat::from_rotation_y(-delta.y.atan2(delta.x))),
                Visibility::Hidden,
            ));
        }
    }
}
