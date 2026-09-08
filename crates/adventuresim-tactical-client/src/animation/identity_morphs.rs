//! Per-character morph weights for asynchronously loaded body scenes.

use adventuresim_core::character_morph::CharacterMorphWeights;
use bevy::mesh::morph::MorphWeights;

use super::*;

pub(super) fn sync_character_morphs(
    roots: Query<(Entity, &AnimationRigScene)>,
    characters: Query<&CharacterId>,
    children: Query<&Children>,
    meshes: Res<Assets<Mesh>>,
    mut morphs: Query<&mut MorphWeights>,
) {
    for (root, owner) in &roots {
        let Ok(character_id) = characters.get(owner.0) else {
            continue;
        };
        let identity = CharacterMorphWeights::from_character_id(character_id.0);
        for entity in descendants_including(root, &children) {
            let Ok(mut weights) = morphs.get_mut(entity) else {
                continue;
            };
            let Some(names) = weights
                .first_mesh()
                .and_then(|mesh| meshes.get(mesh))
                .and_then(Mesh::morph_target_names)
            else {
                continue;
            };
            for (index, name) in names.iter().enumerate() {
                if let Some(value) = identity.named_weight(name)
                    && weights
                        .weights()
                        .get(index)
                        .is_some_and(|current| *current != value)
                {
                    weights.weights_mut()[index] = value;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn late_loaded_bodies_use_distinct_identity_without_mutating_shared_mesh() {
        let mut app = App::new();
        app.init_resource::<Assets<Mesh>>()
            .add_systems(Update, sync_character_morphs);
        let mut mesh = Mesh::new(bevy::mesh::PrimitiveTopology::TriangleList, default());
        mesh.set_morph_target_names(vec!["mhr_identity_00".into(), "smile".into()]);
        let mesh = app.world_mut().resource_mut::<Assets<Mesh>>().add(mesh);
        let mut bodies = Vec::new();
        for id in [42, 43] {
            let owner = app.world_mut().spawn(CharacterId(id)).id();
            let root = app.world_mut().spawn(AnimationRigScene(owner)).id();
            app.update();
            let body = app
                .world_mut()
                .spawn((
                    ChildOf(root),
                    MorphWeights::new(vec![0.0, 0.7], Some(mesh.clone())).unwrap(),
                ))
                .id();
            bodies.push(body);
        }
        app.update();
        let first = app
            .world()
            .get::<MorphWeights>(bodies[0])
            .unwrap()
            .weights()
            .to_vec();
        let second = app
            .world()
            .get::<MorphWeights>(bodies[1])
            .unwrap()
            .weights();
        assert_ne!(first[0], second[0]);
        assert_eq!(first[1], 0.7);
        app.update();
        assert_eq!(
            first,
            app.world()
                .get::<MorphWeights>(bodies[0])
                .unwrap()
                .weights()
        );
        assert_eq!(app.world().resource::<Assets<Mesh>>().len(), 1);
    }
}
