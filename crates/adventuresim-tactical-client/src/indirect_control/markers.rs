//! Visual feedback for selection, current orders, and queued waypoints.

use super::*;

const SELECTION_MARKER_HEIGHT_METRES: f32 = 2.2;
const SELECTION_MARKER_RADIUS_METRES: f32 = 0.12;

#[derive(Component)]
pub(super) struct OrderMarker;

#[derive(Component)]
pub(super) struct SelectionMarker;

#[derive(Component)]
pub(super) struct WaypointMarker;

pub(super) fn update_selection_marker(
    mut commands: Commands,
    local: Res<LocalControl>,
    mut selected: ResMut<SelectedCharacter>,
    targets: Query<&GlobalTransform>,
    mut markers: Query<(Entity, &mut Transform), With<SelectionMarker>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let position = selected
        .0
        .filter(|_| local.acknowledged.mode == ControlMode::Indirect)
        .and_then(|entity| targets.get(entity).ok())
        .map(|transform| transform.translation() + Vec3::Y * SELECTION_MARKER_HEIGHT_METRES);
    let Some(position) = position else {
        for (entity, _) in &mut markers {
            commands.entity(entity).despawn();
        }
        if local.acknowledged.mode != ControlMode::Indirect {
            selected.0 = None;
        }
        return;
    };
    if let Some((_, mut transform)) = markers.iter_mut().next() {
        transform.translation = position;
        return;
    }
    commands.spawn((
        SelectionMarker,
        Mesh3d(meshes.add(Sphere::new(SELECTION_MARKER_RADIUS_METRES))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.1, 0.7, 1.0),
            unlit: true,
            ..default()
        })),
        Transform::from_translation(position),
    ));
}

#[expect(
    clippy::too_many_arguments,
    reason = "Bevy supplies local control, queue state, and rendering assets independently"
)]
pub(super) fn update_waypoint_markers(
    mut commands: Commands,
    local: Res<LocalControl>,
    queues: Query<&WaypointQueue, With<ClientPlayer>>,
    markers: Query<Entity, With<WaypointMarker>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    config: Res<TacticalCombatConfig>,
    mut previous: Local<Vec<NavigationDestination>>,
) {
    let destinations = if local.acknowledged.mode == ControlMode::Indirect {
        queues
            .iter()
            .next()
            .map_or(&[][..], WaypointQueue::destinations)
    } else {
        &[][..]
    };
    if previous.as_slice() == destinations {
        return;
    }
    for entity in &markers {
        commands.entity(entity).despawn();
    }
    *previous = destinations.to_vec();
    for destination in destinations {
        commands.spawn((
            WaypointMarker,
            Mesh3d(meshes.add(Sphere::new(
                config.ai.control.arrival_radius_metres.metres(),
            ))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(1.0, 0.8, 0.1),
                unlit: true,
                ..default()
            })),
            Transform::from_translation(destination.position.metres()),
        ));
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "Bevy supplies marker assets, local order and target transforms separately"
)]
pub(super) fn update_marker(
    mut commands: Commands,
    local: Res<LocalControl>,
    players: Query<&CharacterOrder, With<ClientPlayer>>,
    targets: Query<&GlobalTransform>,
    mut markers: Query<(Entity, &mut Transform), With<OrderMarker>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    config: Res<TacticalCombatConfig>,
    mut last: Local<Option<CharacterOrder>>,
) {
    let order = players
        .iter()
        .next()
        .copied()
        .filter(|_| local.acknowledged.mode == ControlMode::Indirect);
    if *last != order {
        for (entity, _) in &mut markers {
            commands.entity(entity).despawn();
        }
        *last = order;
    } else if !order.is_some_and(|order| {
        matches!(
            order.goal,
            CharacterGoal::Attack { .. } | CharacterGoal::Talk { .. }
        )
    }) {
        return;
    }
    let Some(order) = order else {
        return;
    };
    let point = match order.goal {
        CharacterGoal::MoveTo(destination) => destination.position.metres(),
        CharacterGoal::Hold { anchor } => anchor.metres(),
        CharacterGoal::Attack { target } | CharacterGoal::Talk { target } => {
            let Ok(target) = targets.get(target) else {
                return;
            };
            target.translation()
        }
        CharacterGoal::Idle => return,
    };
    if let Some((_, mut transform)) = markers.iter_mut().next() {
        transform.translation = point;
        return;
    }
    commands.spawn((
        OrderMarker,
        Mesh3d(meshes.add(Sphere::new(
            config.ai.control.arrival_radius_metres.metres(),
        ))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::WHITE,
            unlit: true,
            ..default()
        })),
        Transform::from_translation(point),
    ));
}
