//! Presentation-only physical descriptors; tactical observers own their assets.
use super::buildings;
use adventuresim_building_generator::{compile_operable_doors, compile_operable_windows};
use adventuresim_tactical_core::prelude::*;
use bevy::prelude::*;

pub(super) fn spawn_obstacles(
    commands: &mut Commands,
    input: &TacticalSceneInput,
    generated: &GeneratedTacticalScene,
    root: Entity,
) {
    for obstacle in &generated.obstacles {
        let (x, z, kind, offset, yaw) = match *obstacle {
            GeneratedObstacle::Tree { x, z } => (
                x,
                z,
                SceneObstacle::Tree,
                TREE_TRUNK_HEIGHT_METRES * 0.5,
                0.0,
            ),
            GeneratedObstacle::Rock { x, z, recipe } => (
                x,
                z,
                SceneObstacle::Rock(recipe),
                recipe.collision_radius_metres(),
                (recipe.seed >> 40) as f32 / ((1_u32 << 24) - 1) as f32 * std::f32::consts::TAU,
            ),
        };
        let position = Vec2::new(f32::from(x), f32::from(z)) * input.playable.spacing_metres
            - Vec2::new(generated.terrain.width(), generated.terrain.depth()) * 0.5;
        let height = generated.terrain.height_at(position).unwrap_or_default() + offset;
        commands.spawn((
            kind,
            Transform::from_xyz(position.x, height, position.y)
                .with_rotation(Quat::from_rotation_y(yaw)),
            ChildOf(root),
        ));
    }
}

pub(super) fn spawn_building(
    commands: &mut Commands,
    building: GeneratedBuilding,
    root: Entity,
    input: Option<&TacticalSceneInput>,
) {
    let transform = buildings::transform(&building);
    let origin = building.collision.bounds.centre();
    let direction = |v: Vec2| transform.rotation * Vec3::new(v.x, 0.0, v.y);
    let mut entity = commands.spawn((
        Transform::from_translation(transform.translation).with_rotation(transform.rotation),
        ChildOf(root),
    ));
    if let Some(establishment) = input.and_then(|input| {
        input
            .establishments
            .iter()
            .find(|e| e.building_id == building.placement.id)
    }) {
        entity.insert(establishment.clone());
    }
    let entity = entity.id();
    for door in compile_operable_doors(&building.plan) {
        let centre = transform.transform_point(door.closed_centre - origin);
        commands.spawn((
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
            ChildOf(root),
        ));
    }
    for window in compile_operable_windows(&building.plan) {
        let centre = transform.transform_point(window.closed_centre - origin);
        commands.spawn((
            SceneWindow {
                building_id: building.placement.id,
                opening_id: window.opening.0,
                leaf: window.leaf,
                barred: window.barred,
                size_metres: window.size_metres,
                opening_centre_metres: centre,
                tangent: direction(window.tangent),
                outward: direction(window.outward),
            },
            Transform::from_translation(centre).with_rotation(
                transform.rotation * Quat::from_rotation_y(window.closed_yaw_radians),
            ),
            ChildOf(root),
        ));
    }
    // Insert the prepared geometry together with its descriptor so the add
    // observer can consume it without regenerating the same tactical plan.
    commands.entity(entity).insert((
        crate::presentation::PreparedBuildingGeometry(
            adventuresim_tactical_core::scene_input::GeneratedBuildingRecipe {
                program: building.placement.program.clone(),
                plan: building.plan,
                collision: building.collision,
            },
        ),
        SceneBuilding {
            id: building.placement.id,
            program: building.placement.program,
            orientation: building.placement.orientation,
        },
    ));
}

pub(super) fn spawn_props(
    commands: &mut Commands,
    generated: &mut GeneratedTacticalScene,
    root: Entity,
) {
    for boundary in generated.boundaries.drain(..) {
        let door = boundary
            .scene
            .boundary
            .gate
            .door(boundary.scene.property_id);
        let elevation = Vec3::Y * boundary.elevation_metres;
        let centre = door.closed_centre + elevation;
        commands.spawn((
            SceneDoor {
                building_id: boundary.scene.front_building_id,
                opening_id: door.opening.0,
                size_metres: door.size_metres,
                doorway_centre_metres: centre,
                tangent: Vec3::new(door.tangent.x, 0.0, door.tangent.y),
                outward: Vec3::new(door.outward.x, 0.0, door.outward.y),
            },
            Transform::from_translation(centre)
                .with_rotation(Quat::from_rotation_y(door.closed_yaw_radians)),
            ChildOf(root),
        ));
        commands.spawn((
            boundary.scene,
            Transform::from_translation(elevation),
            ChildOf(root),
        ));
    }
    for garden in generated.gardens.drain(..) {
        commands.spawn((
            garden.scene,
            Transform::from_translation(Vec3::Y * garden.elevation_metres),
            ChildOf(root),
        ));
    }
    for furniture in &generated.furniture.instances {
        commands.spawn((
            furniture.scene,
            Transform::from_translation(furniture.position_metres)
                .with_rotation(Quat::from_rotation_y(furniture.orientation.yaw_radians())),
            ChildOf(root),
        ));
    }
}
