//! Moves held items into and out of the physical tactical scene.
use super::*;

const PICKUP_RANGE_M: f32 = 2.0;
/// Candidate grip distances ahead of the actor, nearest open ground first. The
/// final zero releases the item from the actor's own column, so standing
/// against a wall or bluff does not make dropping impossible.
const DROP_FORWARD_DISTANCES_M: [f32; 4] = [0.9, 1.2, 1.5, 0.0];
const DROP_GRIP_HEIGHT_M: f32 = 0.65;
const ITEM_SIGHT_HEIGHT_M: f32 = 0.6;

pub(super) fn drop_hand(
    commands: &mut Commands,
    actor: Entity,
    hand: EquipmentHand,
    players: &Query<(&Transform, &CharacterLook), With<Player>>,
    items: &Query<ItemView<'_>>,
    spatial: &SpatialQuery,
) -> bool {
    let Some(item) = hand_item(actor, hand, items) else {
        return false;
    };
    if has_children(item, items) {
        return false;
    }
    let Ok((_, _, _, _, _, physical, _, _)) = items.get(item) else {
        return false;
    };
    let Some(physical) = physical.copied().filter(|physical| physical.is_valid()) else {
        return false;
    };
    let Ok((actor_transform, _)) = players.get(actor) else {
        return false;
    };
    let shape = Collider::cuboid(
        physical.dimensions_m.x,
        physical.dimensions_m.y,
        physical.dimensions_m.z,
    );
    let Some(position) = DROP_FORWARD_DISTANCES_M.into_iter().find_map(|distance| {
        let grip = actor_transform.translation
            + *actor_transform.forward() * distance
            + Vec3::Y * DROP_GRIP_HEIGHT_M;
        let centre = item_box_center(grip, &physical);
        // Shape intersection only sees surfaces, so a box wholly behind a wall
        // or beneath a heightfield is ruled out by the sight line instead.
        let open = spatial
            .shape_intersections(
                &shape,
                centre,
                Quat::IDENTITY,
                &SpatialQueryFilter::from_excluded_entities([item, actor]),
            )
            .is_empty();
        (open && has_clear_sight(spatial, actor_transform.translation, centre)).then_some(grip)
    }) else {
        return false;
    };
    let collider = Collider::compound(vec![(-physical.anchor_offset_m, Quat::IDENTITY, shape)]);
    // A previously dropped item keeps Avian's body state after pickup, and an
    // existing `Position` would otherwise overrule the new `Transform`.
    commands
        .entity(item)
        .remove::<ItemOf>()
        .remove::<EquipSlot>()
        .insert((
            EquipmentTopology::default(),
            TacticalSceneItem,
            Transform::from_translation(position),
            Position(position),
            Rotation::default(),
            LinearVelocity::ZERO,
            AngularVelocity::ZERO,
            RigidBody::Dynamic,
            collider,
            CollisionLayers::new(TACTICAL_ITEM_LAYER, TACTICAL_TERRAIN_LAYER),
        ));
    true
}

pub(super) fn pickup(
    commands: &mut Commands,
    actor: Entity,
    hand: EquipmentHand,
    requested: Entity,
    players: &Query<(&Transform, &CharacterLook), With<Player>>,
    items: &Query<ItemView<'_>>,
    spatial: &SpatialQuery,
) -> bool {
    if hand_item(actor, hand, items).is_some() {
        return false;
    }
    let Ok((actor_transform, _)) = players.get(actor) else {
        return false;
    };
    let Ok((_, _, _, _, _, physical, scene, item_transform)) = items.get(requested) else {
        return false;
    };
    let Some((physical, item_transform)) = physical
        .filter(|physical| physical.is_valid())
        .zip(item_transform)
        .filter(|_| scene)
    else {
        return false;
    };
    let item_position = item_box_center(item_transform.translation, physical);
    if item_position.distance_squared(actor_transform.translation) > PICKUP_RANGE_M * PICKUP_RANGE_M
    {
        return false;
    }
    if !has_clear_sight(spatial, actor_transform.translation, item_position) {
        return false;
    }
    commands
        .entity(requested)
        .remove::<TacticalSceneItem>()
        .remove::<RigidBody>()
        .remove::<Collider>()
        .remove::<CollisionLayers>()
        .insert((
            ItemOf(actor),
            hand_topology(hand),
            hand.slot(),
            Transform::default(),
        ));
    true
}

fn item_box_center(grip: Vec3, physical: &TacticalEquipmentPhysical) -> Vec3 {
    grip - physical.anchor_offset_m
}

/// Terrain and support sight is evaluated separately from item pointing, so
/// item boxes do not become combat or visibility blockers.
fn has_clear_sight(spatial: &SpatialQuery, actor_position: Vec3, item_position: Vec3) -> bool {
    let origin = actor_position + Vec3::Y * ITEM_SIGHT_HEIGHT_M;
    let sight = item_position - origin;
    let Ok(direction) = Dir3::new(sight) else {
        return true;
    };
    spatial
        .cast_ray(
            origin,
            direction,
            sight.length(),
            true,
            &SpatialQueryFilter::from_mask(TACTICAL_TERRAIN_LAYER),
        )
        .is_none()
}

#[cfg(test)]
mod tests;
