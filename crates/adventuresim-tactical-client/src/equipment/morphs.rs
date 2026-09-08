//! Keep manually instantiated equipment morphs fitted to its current wearer.

use adventuresim_core::character_morph::CharacterMorphWeights;
use bevy::mesh::morph::MeshMorphWeights;

use super::*;

pub(super) fn sync_equipment_morphs(
    mut commands: Commands,
    meshes: Res<Assets<Mesh>>,
    parts: Query<(
        Entity,
        &ProceduralEquipmentPart,
        &Mesh3d,
        Option<&MeshMorphWeights>,
    )>,
    items: Query<(Option<&ItemOf>, Has<TacticalSceneItem>)>,
    characters: Query<&CharacterId>,
) {
    for (entity, part, mesh, current) in &parts {
        let Some(names) = meshes.get(&mesh.0).and_then(Mesh::morph_target_names) else {
            continue;
        };
        if names.is_empty() {
            continue;
        }
        let identity = items
            .get(part.item)
            .ok()
            .and_then(|(owner, scene)| (!scene).then_some(owner?.0))
            .and_then(|owner| characters.get(owner).ok())
            .map(|id| CharacterMorphWeights::from_character_id(id.0));
        let desired = names
            .iter()
            .map(|name| {
                identity
                    .as_ref()
                    .and_then(|weights| weights.named_weight(name))
                    .unwrap_or(0.0)
            })
            .collect::<Vec<_>>();
        if !matches!(current, Some(MeshMorphWeights::Value { weights }) if weights == &desired) {
            commands
                .entity(entity)
                .insert(MeshMorphWeights::Value { weights: desired });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn equipment_refits_on_transfer_and_resets_when_dropped() {
        let mut app = App::new();
        app.init_resource::<Assets<Mesh>>()
            .add_systems(Update, sync_equipment_morphs);
        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, default());
        mesh.set_morph_target_names(vec!["mhr_identity_00".into()]);
        let mesh = app.world_mut().resource_mut::<Assets<Mesh>>().add(mesh);
        let first = app.world_mut().spawn(CharacterId(42)).id();
        let second = app.world_mut().spawn(CharacterId(43)).id();
        let item = app.world_mut().spawn(ItemOf(first)).id();
        let part = app
            .world_mut()
            .spawn((
                Mesh3d(mesh),
                ProceduralEquipmentPart {
                    item,
                    inverse_bindposes: default(),
                    joint_names: vec![],
                },
            ))
            .id();
        let weight = |app: &App| match app.world().get::<MeshMorphWeights>(part).unwrap() {
            MeshMorphWeights::Value { weights } => weights[0],
            MeshMorphWeights::Reference(_) => panic!("equipment owns its weights"),
        };
        app.update();
        assert_eq!(
            weight(&app),
            CharacterMorphWeights::from_character_id(42)
                .named_weight("mhr_identity_00")
                .unwrap()
        );
        app.world_mut().entity_mut(item).insert(ItemOf(second));
        app.update();
        assert_eq!(
            weight(&app),
            CharacterMorphWeights::from_character_id(43)
                .named_weight("mhr_identity_00")
                .unwrap()
        );
        app.world_mut().entity_mut(item).remove::<ItemOf>();
        app.update();
        assert_eq!(weight(&app), 0.0);
    }
}
