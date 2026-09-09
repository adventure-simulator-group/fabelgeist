//! Render layouts must match the presence of a complete wearer skin binding.
use super::*;

#[derive(Component)]
pub(crate) struct ProceduralEquipmentPart {
    pub(crate) item: Entity,
    pub(super) inverse_bindposes: Handle<SkinnedMeshInverseBindposes>,
    pub(crate) joint_names: Vec<String>,
}

impl ProceduralEquipmentPart {
    /// A loaded asset remains hidden until its wearer binding is renderable.
    pub(super) fn render_bundle(
        self,
        mesh: Handle<Mesh>,
        material: Handle<StandardMaterial>,
    ) -> impl Bundle {
        (
            Name::new("Procedural armor or clothing"),
            Visibility::Hidden,
            Mesh3d(mesh),
            MeshMaterial3d(material),
            Transform::IDENTITY,
            NoFrustumCulling,
            GrabTargetOutline(self.item),
            OutlineVolume {
                visible: false,
                colour: Color::WHITE,
                width: 4.0,
            },
            OutlineMode::FloodFlat,
            self,
        )
    }
}

#[derive(Component, Clone)]
pub(super) struct EquipmentMeshSources {
    bound: Handle<Mesh>,
    unbound: Option<Handle<Mesh>>,
}

type EquipmentRenderState = (
    Entity,
    &'static ProceduralEquipmentPart,
    &'static Mesh3d,
    Option<&'static SkinnedMesh>,
    Option<&'static EquipmentMeshSources>,
    &'static mut Visibility,
);

/// Joint attributes select Bevy's skinned pipeline even without `SkinnedMesh`.
/// Await wearer binding while hidden, and use a joint-free mesh when dropped.
pub(super) fn sync_render_bindings(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut parts: Query<EquipmentRenderState>,
    items: Query<(Option<&ItemOf>, Has<TacticalSceneItem>)>,
) {
    for (entity, part, current_mesh, skin, current_sources, mut visibility) in &mut parts {
        let worn = items
            .get(part.item)
            .is_ok_and(|(owner, scene)| owner.is_some() && !scene);
        if worn && skin.is_none() {
            visibility.set_if_neq(Visibility::Hidden);
            continue;
        }
        let mut sources = current_sources
            .cloned()
            .unwrap_or_else(|| EquipmentMeshSources {
                bound: current_mesh.0.clone(),
                unbound: None,
            });
        let mut sources_changed = current_sources.is_none();
        let desired = if skin.is_some() {
            sources.bound.clone()
        } else {
            if sources.unbound.is_none() {
                let Some(source) = meshes.get(&sources.bound) else {
                    visibility.set_if_neq(Visibility::Hidden);
                    continue;
                };
                let mut unbound = source.clone();
                unbound.remove_attribute(Mesh::ATTRIBUTE_JOINT_INDEX);
                unbound.remove_attribute(Mesh::ATTRIBUTE_JOINT_WEIGHT);
                sources.unbound = Some(meshes.add(unbound));
                sources_changed = true;
            }
            sources
                .unbound
                .as_ref()
                .expect("joint-free mesh prepared")
                .clone()
        };
        if current_mesh.0 != desired {
            commands.entity(entity).insert(Mesh3d(desired));
        }
        if sources_changed {
            commands.entity(entity).insert(sources);
        }
        visibility.set_if_neq(Visibility::Inherited);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::mesh::VertexAttributeValues;

    #[test]
    fn loading_drop_and_reequip_keep_skin_and_vertex_layout_coherent() {
        let mut app = App::new();
        app.init_resource::<Assets<Mesh>>()
            .add_systems(Update, sync_render_bindings);
        let wearer = app.world_mut().spawn_empty().id();
        let item = app.world_mut().spawn(ItemOf(wearer)).id();
        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, default());
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vec![[0.0, 0.0, 0.0]]);
        mesh.insert_attribute(
            Mesh::ATTRIBUTE_JOINT_INDEX,
            VertexAttributeValues::Uint16x4(vec![[0; 4]]),
        );
        mesh.insert_attribute(Mesh::ATTRIBUTE_JOINT_WEIGHT, vec![[1.0, 0.0, 0.0, 0.0]]);
        mesh.set_morph_target_names(vec!["body-fit".into()]);
        let original = app.world_mut().resource_mut::<Assets<Mesh>>().add(mesh);
        let part = app
            .world_mut()
            .spawn((
                ProceduralEquipmentPart {
                    item,
                    inverse_bindposes: default(),
                    joint_names: vec!["pelvis".into()],
                },
                Mesh3d(original.clone()),
                Visibility::Hidden,
            ))
            .id();
        app.update();
        assert_eq!(
            app.world().get::<Visibility>(part),
            Some(&Visibility::Hidden)
        );
        app.world_mut().entity_mut(part).insert(SkinnedMesh {
            inverse_bindposes: default(),
            joints: vec![wearer],
        });
        app.update();
        assert_eq!(
            app.world().get::<Visibility>(part),
            Some(&Visibility::Inherited)
        );
        assert_eq!(app.world().get::<Mesh3d>(part).unwrap().0, original);

        app.world_mut().entity_mut(item).insert(TacticalSceneItem);
        app.world_mut().entity_mut(part).remove::<SkinnedMesh>();
        app.update();
        let dropped = app.world().get::<Mesh3d>(part).unwrap().0.clone();
        let assets = app.world().resource::<Assets<Mesh>>();
        let dropped_mesh = assets.get(&dropped).unwrap();
        assert!(!dropped_mesh.contains_attribute(Mesh::ATTRIBUTE_JOINT_INDEX));
        assert!(!dropped_mesh.contains_attribute(Mesh::ATTRIBUTE_JOINT_WEIGHT));
        assert_eq!(
            dropped_mesh.morph_target_names(),
            assets.get(&original).unwrap().morph_target_names()
        );
        assert!(
            assets
                .get(&original)
                .unwrap()
                .contains_attribute(Mesh::ATTRIBUTE_JOINT_INDEX)
        );

        app.world_mut()
            .entity_mut(item)
            .remove::<TacticalSceneItem>();
        app.update();
        assert_eq!(
            app.world().get::<Visibility>(part),
            Some(&Visibility::Hidden)
        );
        app.world_mut().entity_mut(part).insert(SkinnedMesh {
            inverse_bindposes: default(),
            joints: vec![wearer],
        });
        app.update();
        assert_eq!(app.world().get::<Mesh3d>(part).unwrap().0, original);
        assert_eq!(
            app.world().get::<Visibility>(part),
            Some(&Visibility::Inherited)
        );
        app.world_mut().entity_mut(item).insert(TacticalSceneItem);
        app.world_mut().entity_mut(part).remove::<SkinnedMesh>();
        app.update();
        assert_eq!(app.world().get::<Mesh3d>(part).unwrap().0, dropped);
    }
}
