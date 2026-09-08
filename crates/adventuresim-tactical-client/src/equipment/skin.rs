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
