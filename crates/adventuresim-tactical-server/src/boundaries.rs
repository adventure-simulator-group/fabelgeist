//! Property enclosures use the same fixed geometry for collision and rendering.
use super::*;

pub(crate) fn on_scene_boundary_added(
    event: On<Add, SceneBoundary>,
    mut commands: Commands,
    boundaries: Query<(&SceneBoundary, &Transform)>,
) -> Result {
    let (boundary, transform) = boundaries.get(event.entity)?;
    let collider = Collider::compound(
        boundary
            .boundary
            .fixed_members()
            .iter()
            .map(|member| {
                (
                    member.centre_metres,
                    Quat::from_rotation_y(member.yaw_radians),
                    Collider::cuboid(
                        member.size_metres.x,
                        member.size_metres.y,
                        member.size_metres.z,
                    ),
                )
            })
            .collect(),
    );
    doors::spawn_door(
        &mut commands,
        event.entity,
        boundary.front_building_id,
        transform,
        Vec3::ZERO,
        boundary.boundary.gate.door(boundary.property_id),
    );
    commands.entity(event.entity).insert((
        Replicated,
        RigidBody::Static,
        CollisionLayers::new(TACTICAL_TERRAIN_LAYER, LayerMask::ALL),
        collider,
    ));
    Ok(())
}

pub(crate) fn spawn_generated_boundaries(
    commands: &mut Commands,
    boundaries: Vec<GeneratedBoundary>,
) {
    for boundary in boundaries {
        commands.spawn((
            Name::new(format!(
                "Property {} enclosure",
                boundary.scene.property_id.0
            )),
            boundary.scene,
            Transform::from_xyz(0.0, boundary.elevation_metres, 0.0),
        ));
    }
}
