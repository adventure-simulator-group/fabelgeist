use adventuresim_tactical_core::prelude::*;
use bevy::prelude::*;

/// Select each physical building once across playable and distant presentation.
pub(super) fn distant_placements(
    input: &TacticalSceneInput,
    buildings: &[GeneratedBuilding],
) -> Vec<DistantBuildingPlacement> {
    let playable: std::collections::BTreeSet<_> =
        buildings.iter().map(|b| b.placement.id).collect();
    input
        .distant_buildings
        .iter()
        .filter(|b| !playable.contains(&b.id))
        .copied()
        .collect()
}

pub(super) fn spawn_boundaries(
    commands: &mut Commands,
    boundaries: Vec<GeneratedBoundary>,
) -> Result {
    for boundary in boundaries {
        let elevation = boundary.elevation_metres();
        let scene = boundary.into_scene();
        let door = scene.boundary().gate.door(scene.property_id())?;
        let pose = adventuresim_tactical_core::scene_coordinates::GateDatum::from_metres(
            elevation.metres(),
        )?
        .door(door)?;
        let door = pose.leaf();
        let elevation = Vec3::Y * elevation.metres();
        let centre = door.closed_centre.metres();
        commands.spawn((
            SceneDoor {
                building_id: scene.front_building_id(),
                opening_id: door.opening,
                size_metres: door.size_metres,
                doorway_centre_metres: door.closed_centre,
                tangent: door.tangent.spatial(),
                outward: door.outward.spatial(),
            },
            Transform::from_translation(centre).with_rotation(pose.native_rotation()),
            super::building_review::ReviewLeafPose::from_scene(pose),
        ));
        commands.spawn((scene, Transform::from_translation(elevation)));
    }
    Ok(())
}

pub(super) fn spawn_tactical_buildings(
    commands: &mut Commands,
    buildings: Vec<GeneratedBuilding>,
) -> Result {
    for building in buildings {
        super::building_review::spawn_openings(commands, &building)?;
        let transform = building.transform()?;
        let entity = commands.spawn_empty().id();
        commands.queue(move |world: &mut World| {
            super::building_review::insert_authored_sign(world, entity, building.placement.id);
        });
        commands.entity(entity).insert((
            Name::new(format!("Tactical building {}", building.placement.id)),
            SceneBuilding {
                id: building.placement.id,
                program: building.placement.program,
                orientation: building.placement.orientation,
            },
            RigidBody::Static,
            CollisionLayers::new(TACTICAL_TERRAIN_LAYER, LayerMask::ALL),
            adventuresim_tactical_core::scene_input::compile_tactical_building_collider(
                &building.collision,
            )?,
            transform,
        ));
    }
    Ok(())
}
