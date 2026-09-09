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

type EquipmentRenderPart = (
    &'static ProceduralEquipmentPart,
    &'static Mesh3d,
    &'static MeshMaterial3d<StandardMaterial>,
    Option<&'static SkinnedMesh>,
    Option<&'static MeshMorphWeights>,
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
    pub(in crate::animation_viewer) fn state(&self, item: Entity) -> EquipmentVisualState {
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
        for (part, mesh, material, skin, morphs) in
            self.parts.iter().filter(|(part, ..)| part.item == item)
        {
            count += 1;
            let Some(mesh) = self.meshes.get(&mesh.0) else {
                return EquipmentVisualState::Loading;
            };
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
        if count == 0 {
            EquipmentVisualState::Loading
        } else {
            EquipmentVisualState::Ready
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;

    #[test]
    fn missing_fallback_and_unbound_equipment_never_pass_capture_readiness() {
        let mut world = World::new();
        world.init_resource::<Assets<Mesh>>();
        world.init_resource::<Assets<StandardMaterial>>();
        let item = world.spawn_empty().id();
        let state = |world: &mut World| {
            world
                .run_system_once(move |visuals: EquipmentVisualStatus| visuals.state(item))
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
