//! Keep manually instantiated equipment morphs fitted to its current wearer.

use crate::animation::skeletal_proportions::{
    CharacterSkeletalProportions, SkeletalProportionReference,
};
use adventuresim_core::character_morph::CharacterMorphWeights;
use adventuresim_core::character_proportions::CharacterProportions;
use adventuresim_core::skeletal_fit::SkeletalFitMorph;
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
    characters: Query<(
        &CharacterId,
        Option<&CharacterSkeletalProportions>,
        Option<&SkeletalProportionReference>,
    )>,
) {
    for (entity, part, mesh, current) in &parts {
        let Some(names) = meshes.get(&mesh.0).and_then(Mesh::morph_target_names) else {
            continue;
        };
        if names.is_empty() {
            continue;
        }
        let appearance = items
            .get(part.item)
            .ok()
            .and_then(|(owner, scene)| (!scene).then_some(owner?.0))
            .and_then(|owner| characters.get(owner).ok())
            .map(|(id, explicit, reference)| {
                (
                    CharacterMorphWeights::from_character_id(id.0),
                    explicit
                        .map(|p| p.0)
                        .unwrap_or_else(|| CharacterProportions::from_character_id(id.0)),
                    reference.map(|p| p.0).unwrap_or_default(),
                )
            });
        let desired = names
            .iter()
            .map(|name| {
                appearance
                    .as_ref()
                    .and_then(|(weights, proportions, reference)| {
                        weights.named_weight(name).or_else(|| {
                            SkeletalFitMorph::from_name(name)
                                .map(|m| m.weight(*proportions, *reference))
                        })
                    })
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
        mesh.set_morph_target_names(vec![
            "mhr_identity_00".into(),
            SkeletalFitMorph::LongSpine.name().into(),
        ]);
        let mesh = app.world_mut().resource_mut::<Assets<Mesh>>().add(mesh);
        let first = app.world_mut().spawn(CharacterId(42)).id();
        let second = app.world_mut().spawn(CharacterId(43)).id();
        let mut longer = CharacterProportions::default();
        longer
            .set(
                adventuresim_core::character_proportions::BodyProportion::SpineLength,
                SkeletalFitMorph::LongSpine.endpoint(),
            )
            .unwrap();
        app.world_mut()
            .entity_mut(second)
            .insert(CharacterSkeletalProportions(longer));
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
        let MeshMorphWeights::Value { weights } =
            app.world().get::<MeshMorphWeights>(part).unwrap()
        else {
            panic!("instance weights");
        };
        assert_eq!(weights[1], 1.0);
        app.update();
        assert_eq!(weight(&app), 0.0);
        let MeshMorphWeights::Value { weights } =
            app.world().get::<MeshMorphWeights>(part).unwrap()
        else {
            panic!("instance weights");
        };
        assert_eq!(weights[1], 0.0);
    }
}
