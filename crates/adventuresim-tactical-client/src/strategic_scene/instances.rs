//! Presentation-only physical descriptors; tactical observers own their assets.
use adventuresim_building_generator::{compile_operable_doors, compile_operable_windows};
use adventuresim_tactical_core::prelude::*;
use bevy::prelude::*;

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
                building_id: building.placement.id,
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
                building_id: building.placement.id,
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
            id: building.placement.id,
            program: building.placement.program,
            orientation: building.placement.orientation,
        },
    ));
    Ok(())
}

pub(super) fn spawn_props(
    commands: &mut Commands,
    generated: &mut crate::presentation::PreparedStrategicScene,
    root: Entity,
) {
    for furniture in &generated.furniture.instances {
        commands.spawn((
            furniture.scene,
            Transform::from_translation(furniture.position_metres.metres())
                .with_rotation(Quat::from_rotation_y(furniture.orientation.yaw_radians())),
            ChildOf(root),
        ));
    }
}
