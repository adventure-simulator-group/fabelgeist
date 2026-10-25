//! Presentation attachment and visibility for worn, held, and dropped items.
use super::*;

#[expect(
    clippy::type_complexity,
    reason = "the Bevy queries describe equipment ownership, topology, rig bindings, and placeholder state exactly"
)]
pub(super) fn update_item_placeholders(
    mut commands: Commands,
    items: Query<
        (
            &Transform,
            Option<&ItemOf>,
            Option<&EquipSlot>,
            &EquipmentTopology,
            Has<TacticalSceneItem>,
        ),
        Without<ItemPlaceholder>,
    >,
    topologies: Query<
        (&EquipmentTopology, Option<&EquipmentAttachmentSockets>),
        Without<ItemPlaceholder>,
    >,
    rigs: Query<&HumanoidRig, crate::animation::AnimatedActors>,
    bind_nodes: Query<(&AuthoredBindTransform, Option<&ChildOf>)>,
    mut placeholders: Query<(
        Entity,
        &ItemPlaceholder,
        &mut Transform,
        &mut Visibility,
        Option<&ChildOf>,
        Has<ProceduralEquipmentResolved>,
        Has<ProceduralEquipmentFailed>,
    )>,
) {
    for (entity, placeholder, mut transform, mut visibility, parent, procedural, failed) in
        &mut placeholders
    {
        let Ok((item_transform, owner, slot, topology, scene)) = items.get(placeholder.0) else {
            commands.entity(entity).despawn();
            continue;
        };
        if failed {
            *visibility = Visibility::Hidden;
            commands.entity(entity).remove::<HeldWeaponConstraint>();
            continue;
        }
        if scene {
            if parent.is_some() {
                commands.entity(entity).remove::<ChildOf>();
            }
            *transform = *item_transform;
            *visibility = Visibility::Inherited;
            commands.entity(entity).remove::<HeldWeaponConstraint>();
        } else if procedural && holding_side(slot).is_none() {
            let rig_scene = resolve_character_location(topology, &topologies)
                .and(owner)
                .and_then(|owner| rigs.get(owner.0).ok())
                .and_then(HumanoidRig::rig_scene);
            if let Some(rig_scene) = rig_scene {
                if parent.is_none_or(|parent| parent.parent() != rig_scene) {
                    commands.entity(entity).insert(ChildOf(rig_scene));
                }
                *transform = Transform::IDENTITY;
                *visibility = Visibility::Inherited;
            } else {
                *visibility = Visibility::Hidden;
            }
            commands.entity(entity).remove::<HeldWeaponConstraint>();
        } else if let (Some(owner), Some(primary_hand)) = (owner, holding_side(slot)) {
            if parent.is_some() {
                commands.entity(entity).remove::<ChildOf>();
            }
            let constraint = rigs
                .get(owner.0)
                .ok()
                .and_then(|rig| held_constraint(rig, owner.0, primary_hand));
            if let Some(constraint) = constraint {
                *visibility = Visibility::Inherited;
                commands.entity(entity).insert(constraint);
            } else {
                *visibility = Visibility::Hidden;
                commands.entity(entity).remove::<HeldWeaponConstraint>();
            }
        } else if let Some((bone, correction)) = owner.and_then(|owner| {
            let rig = rigs.get(owner.0).ok()?;
            worn_attachment(topology, &topologies, rig, &bind_nodes)
        }) {
            if parent.is_none_or(|parent| parent.parent() != bone) {
                commands.entity(entity).insert(ChildOf(bone));
            }
            *transform = correction;
            *visibility = Visibility::Inherited;
            commands.entity(entity).remove::<HeldWeaponConstraint>();
        } else {
            *visibility = Visibility::Hidden;
            commands.entity(entity).remove::<HeldWeaponConstraint>();
        }
    }
}

fn held_constraint(
    rig: &HumanoidRig,
    owner: Entity,
    primary_hand: HandSide,
) -> Option<HeldWeaponConstraint> {
    let role = match primary_hand {
        HandSide::Left => BoneRole::WeaponLeft,
        HandSide::Right => BoneRole::WeaponRight,
    };
    rig.get(&role)?;
    Some(HeldWeaponConstraint {
        owner,
        primary_hand,
        secondary_grip_local: None,
    })
}

fn worn_attachment(
    topology: &EquipmentTopology,
    topologies: &Query<
        (&EquipmentTopology, Option<&EquipmentAttachmentSockets>),
        Without<ItemPlaceholder>,
    >,
    rig: &HumanoidRig,
    bind_nodes: &Query<(&AuthoredBindTransform, Option<&ChildOf>)>,
) -> Option<(Entity, Transform)> {
    let role = equipment_location_bone(resolve_character_location(topology, topologies)?);
    let bone = rig.get(&role).copied()?;
    let correction = if let Some(socket) = resolve_equipment_attachment_socket(topology, topologies)
    {
        // Generated attachment sockets are authored pelvis-local, so
        // they can be consumed directly as children of the pelvis.
        socket
    } else {
        equipment_bind_correction(role, rig, bind_nodes)?
    };
    Some((bone, correction))
}
