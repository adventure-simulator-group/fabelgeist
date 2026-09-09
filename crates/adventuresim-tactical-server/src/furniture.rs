//! Static outdoor fixtures reconstruct their collider from the replicated kit key.
use adventuresim_tactical_core::prelude::*;
use adventuresim_tactical_netcode::bevy_replicon::prelude::Replicated;
use bevy::prelude::*;

pub(crate) fn on_furniture_added(
    event: On<Add, SceneFurniture>,
    furniture: Query<&SceneFurniture>,
    mut commands: Commands,
) -> Result {
    let furniture = furniture.get(event.entity)?;
    commands.entity(event.entity).insert((
        Replicated,
        RigidBody::Static,
        CollisionLayers::new(TACTICAL_TERRAIN_LAYER, LayerMask::ALL),
        furniture_collider(furniture.key),
    ));
    Ok(())
}

pub(crate) fn spawn(commands: &mut Commands, layout: FurnitureLayout) {
    for group in layout.groups {
        commands.spawn(SceneFurnitureGroup(group));
    }
    for instance in layout.instances {
        commands.spawn((
            Name::new(format!("Outdoor furniture {}", instance.scene.id.0)),
            instance.scene,
            Transform::from_translation(instance.position_metres)
                .with_rotation(Quat::from_rotation_y(instance.orientation.yaw_radians())),
        ));
    }
}

pub(crate) fn on_group_added(
    event: On<Add, SceneFurnitureGroup>,
    groups: Query<&SceneFurnitureGroup>,
    mut vista: ResMut<crate::SceneVistaBundleResource>,
) -> Result {
    let group = &groups.get(event.entity)?.0;
    if let Some(bundle) = &mut vista.0 {
        bundle.furniture_groups.retain(|old| old.id != group.id);
        bundle.furniture_groups.push(group.clone());
        bundle.furniture_groups.sort_by_key(|group| group.id);
    }
    Ok(())
}
