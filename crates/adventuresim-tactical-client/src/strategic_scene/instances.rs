//! Presentation-only physical descriptors; tactical observers own their assets.
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
) -> Result {
    let transform = building.transform()?;
    let datum = building.geometry_datum()?;
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
    for leaf in compile_operable_doors(&building.plan)? {
        let pose = datum.door(leaf)?;
        let door = pose.leaf();
        let centre = door.closed_centre.metres();
        commands.spawn((
            SceneDoor {
                building_id: adventuresim_tactical_core::scene_input::SceneBuildingId::from(
                    building.placement.id,
                ),
                opening_id: door.opening,
                size_metres: door.size_metres,
                doorway_centre_metres: door.closed_centre,
                tangent: door.tangent.spatial(),
                outward: door.outward.spatial(),
            },
            Transform::from_translation(centre).with_rotation(pose.native_rotation()),
            ChildOf(root),
        ));
    }
    for leaf in compile_operable_windows(&building.plan)? {
        let pose = datum.window(leaf)?;
        let window = pose.leaf();
        let centre = window.closed_centre.metres();
        commands.spawn((
            SceneWindow {
                building_id: adventuresim_tactical_core::scene_input::SceneBuildingId::from(
                    building.placement.id,
                ),
                opening_id: window.opening,
                leaf: window.leaf,
                bars: window.bars,
                size_metres: window.size_metres,
                opening_centre_metres: window.closed_centre,
                tangent: window.tangent.spatial(),
                outward: window.outward.spatial(),
            },
            Transform::from_translation(centre).with_rotation(pose.native_rotation()),
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
            id: adventuresim_tactical_core::scene_input::SceneBuildingId::from(
                building.placement.id,
            ),
            program: building.placement.program,
            orientation: building.placement.orientation,
        },
    ));
    Ok(())
}

pub(super) fn spawn_props(
    commands: &mut Commands,
    generated: &mut GeneratedTacticalScene,
    root: Entity,
) -> Result {
    for boundary in generated.boundaries.drain(..) {
        let door = boundary
            .scene
            .boundary
            .gate
            .door(boundary.scene.property_id)?;
        let pose = adventuresim_tactical_core::scene_coordinates::GateDatum::from_metres(
            boundary.elevation_metres,
        )?
        .door(door)?;
        let door = pose.leaf();
        let elevation = Vec3::Y * boundary.elevation_metres;
        let centre = door.closed_centre.metres();
        commands.spawn((
            SceneDoor {
                building_id: adventuresim_tactical_core::scene_input::SceneBuildingId::from(
                    boundary.scene.front_building_id,
                ),
                opening_id: door.opening,
                size_metres: door.size_metres,
                doorway_centre_metres: door.closed_centre,
                tangent: door.tangent.spatial(),
                outward: door.outward.spatial(),
            },
            Transform::from_translation(centre).with_rotation(pose.native_rotation()),
            ChildOf(root),
        ));
        commands.spawn((
            boundary.scene,
            Transform::from_translation(elevation),
            ChildOf(root),
        ));
    }
    for garden in generated.gardens.drain(..) {
        commands.spawn((garden, Transform::default(), ChildOf(root)));
    }
    for furniture in &generated.furniture.instances {
        commands.spawn((
            furniture.scene,
            Transform::from_translation(furniture.position_metres)
                .with_rotation(Quat::from_rotation_y(furniture.orientation.yaw_radians())),
            ChildOf(root),
        ));
    }
    Ok(())
}
