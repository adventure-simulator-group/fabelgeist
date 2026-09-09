//! Fixture state only; production observers own every mesh, material and LOD.
use adventuresim_building_generator::{compile_operable_doors, compile_operable_windows};
use adventuresim_tactical_core::prelude::*;
use bevy::prelude::*;

use super::super::buildings::building_transform;

pub(in crate::tactical_scene_viewer) fn spawn_openings(
    commands: &mut Commands,
    building: &GeneratedBuilding,
) {
    let transform = building_transform(building);
    let origin = building.collision.bounds.centre();
    let direction = |v: Vec2| transform.rotation * Vec3::new(v.x, 0.0, v.y);
    for door in compile_operable_doors(&building.plan) {
        let centre = transform.transform_point(door.closed_centre - origin);
        commands.spawn((
            Name::new("Closed fixture door"),
            SceneDoor {
                building_id: building.placement.id,
                opening_id: door.opening.0,
                size_metres: door.size_metres,
                doorway_centre_metres: centre,
                tangent: direction(door.tangent),
                outward: direction(door.outward),
            },
            Transform::from_translation(centre)
                .with_rotation(transform.rotation * Quat::from_rotation_y(door.closed_yaw_radians)),
        ));
    }
    for window in compile_operable_windows(&building.plan) {
        let centre = transform.transform_point(window.closed_centre - origin);
        commands.spawn((
            Name::new("Closed fixture window"),
            SceneWindow {
                building_id: building.placement.id,
                opening_id: window.opening.0,
                size_metres: window.size_metres,
                opening_centre_metres: centre,
                tangent: direction(window.tangent),
                outward: direction(window.outward),
                barred: window.barred,
            },
            Transform::from_translation(centre).with_rotation(
                transform.rotation * Quat::from_rotation_y(window.closed_yaw_radians),
            ),
        ));
    }
}
