use super::*;

pub(super) fn sync_procedural_equipment_skins(
    mut commands: Commands,
    parts: Query<(Entity, &ProceduralEquipmentPart, Option<&SkinnedMesh>)>,
    items: Query<(Option<&ItemOf>, Has<TacticalSceneItem>)>,
    bones: Query<(Entity, &MhrBone, &Name)>,
) {
    let mut rig_bones = HashMap::<Entity, HashMap<String, Entity>>::new();
    for (entity, bone, name) in &bones {
        rig_bones
            .entry(bone.owner)
            .or_default()
            .insert(name.as_str().to_owned(), entity);
    }
    for (entity, part, current_skin) in &parts {
        let desired_joints = items
            .get(part.item)
            .ok()
            .and_then(|(owner, scene)| (!scene).then_some(owner?.0))
            .and_then(|owner| {
                let bones = rig_bones.get(&owner)?;
                part.joint_names
                    .iter()
                    .map(|name| bones.get(name).copied())
                    .collect::<Option<Vec<_>>>()
            });
        if let Some(joints) = desired_joints {
            if current_skin.is_none_or(|skin| skin.joints != joints) {
                commands.entity(entity).insert(SkinnedMesh {
                    inverse_bindposes: part.inverse_bindposes.clone(),
                    joints,
                });
            }
        } else if current_skin.is_some() {
            commands.entity(entity).remove::<SkinnedMesh>();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;

    #[test]
    fn skeletal_proportions_follow_equipment_transfer_without_rebinding_shared_assets() {
        let mut world = World::new();
        let first = world.spawn_empty().id();
        let second = world.spawn_empty().id();
        let narrow = world
            .spawn((
                MhrBone { owner: first },
                Name::new("l_upleg"),
                Transform::from_xyz(0.05, 1.0, 0.0),
            ))
            .id();
        let wide = world
            .spawn((
                MhrBone { owner: second },
                Name::new("l_upleg"),
                Transform::from_xyz(0.15, 1.0, 0.0),
            ))
            .id();
        let item = world.spawn(ItemOf(first)).id();
        let bindposes = Handle::default();
        let part = world
            .spawn(ProceduralEquipmentPart {
                item,
                inverse_bindposes: bindposes.clone(),
                joint_names: vec!["l_upleg".into()],
            })
            .id();
        world
            .run_system_once(sync_procedural_equipment_skins)
            .unwrap();
        assert_eq!(world.get::<SkinnedMesh>(part).unwrap().joints, [narrow]);
        world.entity_mut(item).insert(ItemOf(second));
        world
            .run_system_once(sync_procedural_equipment_skins)
            .unwrap();
        let skin = world.get::<SkinnedMesh>(part).unwrap();
        assert_eq!(skin.joints, [wide]);
        assert_eq!(skin.inverse_bindposes, bindposes);
        assert_eq!(
            world
                .get::<Transform>(skin.joints[0])
                .unwrap()
                .translation
                .x,
            0.15
        );
        world.entity_mut(item).insert(TacticalSceneItem);
        world
            .run_system_once(sync_procedural_equipment_skins)
            .unwrap();
        assert!(world.get::<SkinnedMesh>(part).is_none());
    }
}
