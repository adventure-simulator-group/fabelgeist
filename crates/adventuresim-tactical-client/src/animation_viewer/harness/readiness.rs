//! Capture readiness reads the same resolved meshes and wearer skins as gameplay.
use crate::equipment::{
    ItemPlaceholder, ProceduralEquipmentFailed, ProceduralEquipmentPart,
    ProceduralEquipmentResolved,
};
use bevy::{
    ecs::system::SystemParam,
    mesh::{morph::MeshMorphWeights, skinning::SkinnedMesh},
    prelude::*,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::animation_viewer) enum EquipmentVisualState {
    Missing,
    Loading,
    Ready,
    Failed,
}

#[derive(Clone, Copy, Default)]
pub(in crate::animation_viewer) struct EquipmentVisualRequirements {
    pub names: &'static [&'static str],
    pub morph_targets: Option<usize>,
}

type EquipmentRenderPart = (
    &'static ProceduralEquipmentPart,
    &'static Mesh3d,
    &'static MeshMaterial3d<StandardMaterial>,
    Option<&'static SkinnedMesh>,
    Option<&'static MeshMorphWeights>,
    Option<&'static Name>,
);

/// Readiness requires a resolved procedural mesh, its material and a bound skin.
/// Loading cuboid proxies never qualify as captured armor.
#[derive(SystemParam)]
pub(in crate::animation_viewer) struct EquipmentVisualStatus<'w, 's> {
    roots: Query<
        'w,
        's,
        (
            &'static ItemPlaceholder,
            Has<ProceduralEquipmentResolved>,
            Has<ProceduralEquipmentFailed>,
        ),
    >,
    parts: Query<'w, 's, EquipmentRenderPart>,
    meshes: Res<'w, Assets<Mesh>>,
    materials: Res<'w, Assets<StandardMaterial>>,
}

impl EquipmentVisualStatus<'_, '_> {
    pub(in crate::animation_viewer) fn state(
        &self,
        item: Entity,
        required: EquipmentVisualRequirements,
    ) -> EquipmentVisualState {
        let Some((_, resolved, failed)) = self.roots.iter().find(|(root, _, _)| root.0 == item)
        else {
            return EquipmentVisualState::Missing;
        };
        if failed {
            return EquipmentVisualState::Failed;
        }
        if !resolved {
            return EquipmentVisualState::Loading;
        }
        let mut count = 0;
        let mut names = std::collections::BTreeSet::new();
        for (part, mesh, material, skin, morphs, name) in
            self.parts.iter().filter(|(part, ..)| part.item == item)
        {
            count += 1;
            if let Some(name) = name {
                names.insert(name.as_str());
            }
            let Some(mesh) = self.meshes.get(&mesh.0) else {
                return EquipmentVisualState::Loading;
            };
            if required.morph_targets.is_some_and(|count| {
                mesh.morph_target_names().is_none_or(|names| names.len() != count)
                    || !matches!(morphs, Some(MeshMorphWeights::Value { weights })
                        if weights.len() == count && weights.iter().all(|weight| weight.is_finite()))
            }) {
                return EquipmentVisualState::Loading;
            }
            if self.materials.get(&material.0).is_none()
                || skin.is_none_or(|skin| {
                    skin.joints.is_empty() || skin.joints.len() != part.joint_names.len()
                })
                || (mesh
                    .morph_target_names()
                    .is_some_and(|names| !names.is_empty())
                    && morphs.is_none())
            {
                return EquipmentVisualState::Loading;
            }
        }
        if count == 0
            || (!required.names.is_empty()
                && (count != required.names.len()
                    || !required.names.iter().all(|name| names.contains(name))))
        {
            EquipmentVisualState::Loading
        } else {
            EquipmentVisualState::Ready
        }
    }
    pub(in crate::animation_viewer) fn summary(&self, item: Entity) -> Vec<serde_json::Value> {
        self.parts
            .iter()
            .filter(|(part, ..)| part.item == item)
            .map(|(part, mesh, material, skin, morphs, name)| {
                let mesh = self.meshes.get(&mesh.0);
                serde_json::json!({
                    "name": name.map(Name::as_str),
                    "mesh_loaded": mesh.is_some(),
                    "material_loaded": self.materials.get(&material.0).is_some(),
                    "skin_joints": skin.map(|skin| skin.joints.len()),
                    "expected_skin_joints": part.joint_names.len(),
                    "morph_targets": mesh.and_then(Mesh::morph_target_names).map(<[String]>::len),
                    "morph_weights": match morphs {
                        Some(MeshMorphWeights::Value { weights }) => Some(weights),
                        _ => None,
                    },
                })
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;

    #[test]
    fn helmet_requires_all_three_bound_parts_with_complete_morph_weights() {
        let mut world = World::new();
        world.init_resource::<Assets<Mesh>>();
        world.init_resource::<Assets<StandardMaterial>>();
        let item = world.spawn_empty().id();
        world.spawn((ItemPlaceholder(item), ProceduralEquipmentResolved));
        let required = super::super::ArmorHarness::CloseHelmet.visual_requirements();
        let morph_count = required.morph_targets.unwrap();
        let mut mesh = Mesh::new(bevy::mesh::PrimitiveTopology::TriangleList, default());
        mesh.set_morph_target_names(
            (0..morph_count)
                .map(|index| format!("fit_{index}"))
                .collect(),
        );
        let mesh = world.resource_mut::<Assets<Mesh>>().add(mesh);
        let material = world
            .resource_mut::<Assets<StandardMaterial>>()
            .add(StandardMaterial::default());
        let mut entities = Vec::new();
        let state = |world: &mut World| {
            world
                .run_system_once(move |visuals: EquipmentVisualStatus| {
                    visuals.state(item, required)
                })
                .unwrap()
        };
        for name in required.names {
            assert_eq!(state(&mut world), EquipmentVisualState::Loading);
            entities.push(
                world
                    .spawn((
                        ProceduralEquipmentPart::new(item, default(), vec!["c_head".into()]),
                        Name::new(*name),
                        Mesh3d(mesh.clone()),
                        MeshMaterial3d(material.clone()),
                        SkinnedMesh {
                            inverse_bindposes: default(),
                            joints: vec![item],
                        },
                        MeshMorphWeights::Value {
                            weights: vec![0.0; morph_count],
                        },
                    ))
                    .id(),
            );
        }
        assert_eq!(state(&mut world), EquipmentVisualState::Ready);
        world.entity_mut(entities[2]).remove::<SkinnedMesh>();
        assert_eq!(state(&mut world), EquipmentVisualState::Loading);
        world.entity_mut(entities[2]).insert(SkinnedMesh {
            inverse_bindposes: default(),
            joints: vec![item],
        });
        world
            .entity_mut(entities[2])
            .insert(MeshMorphWeights::Value {
                weights: vec![0.0; morph_count - 1],
            });
        assert_eq!(state(&mut world), EquipmentVisualState::Loading);
        world
            .entity_mut(entities[2])
            .insert(MeshMorphWeights::Value {
                weights: vec![0.0; morph_count],
            });
        world
            .entity_mut(entities[2])
            .insert(Name::new("unrelated plate"));
        assert_eq!(state(&mut world), EquipmentVisualState::Loading);
    }

    #[test]
    fn missing_fallback_and_unbound_equipment_never_pass_capture_readiness() {
        let mut world = World::new();
        world.init_resource::<Assets<Mesh>>();
        world.init_resource::<Assets<StandardMaterial>>();
        let item = world.spawn_empty().id();
        let state = |world: &mut World| {
            world
                .run_system_once(move |visuals: EquipmentVisualStatus| {
                    visuals.state(item, EquipmentVisualRequirements::default())
                })
                .unwrap()
        };
        assert_eq!(state(&mut world), EquipmentVisualState::Missing);
        let root = world.spawn(ItemPlaceholder(item)).id();
        assert_eq!(state(&mut world), EquipmentVisualState::Loading);
        world.entity_mut(root).insert(ProceduralEquipmentResolved);
        assert_eq!(state(&mut world), EquipmentVisualState::Loading);
        world.entity_mut(root).insert(ProceduralEquipmentFailed);
        assert_eq!(state(&mut world), EquipmentVisualState::Failed);
    }
}
