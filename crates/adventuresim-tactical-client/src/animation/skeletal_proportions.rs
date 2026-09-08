//! Instance-owned skeletal proportions applied to reference poses before IK.

use super::*;
use adventuresim_core::character_proportions::{CharacterProportions, JointProportionBasis};
use bevy::gltf::GltfExtras;

/// An explicit appearance overrides the stable character-ID generated proportions.
#[derive(Component, Clone, Copy, Default)]
pub(crate) struct CharacterSkeletalProportions(pub CharacterProportions);

#[derive(Component, Clone, Copy, Default, PartialEq)]
pub(crate) struct SkeletalProportionReference(pub CharacterProportions);

#[derive(Component)]
pub(super) struct SkeletalJointBasis(JointProportionBasis);

#[derive(Component, Clone, Copy, Debug, Default, PartialEq)]
pub(super) struct SkeletalJointOffset(pub Vec3);

#[derive(serde::Deserialize)]
struct NodeExtras {
    adventuresim_proportions: Option<JointProportionBasis>,
}

pub(super) fn load_skeletal_bases(
    mut commands: Commands,
    nodes: Query<(Entity, &GltfExtras), Added<GltfExtras>>,
) {
    for (entity, extras) in &nodes {
        match serde_json::from_str::<NodeExtras>(&extras.value) {
            Ok(NodeExtras {
                adventuresim_proportions: Some(basis),
            }) if basis.is_finite() => {
                commands.entity(entity).insert(SkeletalJointBasis(basis));
            }
            Ok(NodeExtras {
                adventuresim_proportions: None,
            }) => {}
            _ => error!(?entity, "invalid exported skeletal proportion basis"),
        }
    }
}

struct BoneReference {
    entity: Entity,
    parent: Option<Entity>,
    name: Option<String>,
    bind: Transform,
    offset: Vec3,
}

impl BoneReference {
    fn global(&self, nodes: &BTreeMap<Entity, Self>, deformed: bool) -> Transform {
        let mut local = self.bind;
        if deformed {
            local.translation += self.offset;
        }
        match self.parent.and_then(|parent| nodes.get(&parent)) {
            Some(parent) => parent.global(nodes, deformed).mul_transform(local),
            None => local,
        }
    }
}

#[expect(
    clippy::type_complexity,
    reason = "reads the exported basis and reference hierarchy without changing shared rig assets"
)]
pub(super) fn sync_skeletal_proportions(
    mut commands: Commands,
    owners: Query<(
        &CharacterId,
        Option<&CharacterSkeletalProportions>,
        Option<&SkeletalProportionReference>,
    )>,
    bones: Query<(
        Entity,
        &AuthoredBindTransform,
        Option<&SkeletalJointBasis>,
        Option<&ChildOf>,
        Option<&Name>,
        Option<&SkeletalJointOffset>,
    )>,
) {
    let mut rigs = BTreeMap::<Entity, BTreeMap<Entity, BoneReference>>::new();
    for (entity, bind, basis, parent, name, _) in &bones {
        let Ok((id, explicit, reference)) = owners.get(bind.owner) else {
            continue;
        };
        if let Some(basis) = basis {
            let exported = SkeletalProportionReference(basis.0.reference);
            if reference != Some(&exported) {
                commands.entity(bind.owner).insert(exported);
            }
        }
        let proportions = explicit
            .map(|value| value.0)
            .unwrap_or_else(|| CharacterProportions::from_character_id(id.0));
        rigs.entry(bind.owner).or_default().insert(
            entity,
            BoneReference {
                entity,
                parent: parent.map(ChildOf::parent),
                name: name.map(|name| name.as_str().to_owned()),
                bind: bind.local,
                offset: basis
                    .map(|basis| Vec3::from_array(basis.0.translation(proportions)))
                    .unwrap_or_default(),
            },
        );
    }
    for rig in rigs.values_mut() {
        preserve_neutral_ground_height(rig);
        for bone in rig.values() {
            let offset = SkeletalJointOffset(bone.offset);
            if bones
                .get(bone.entity)
                .is_ok_and(|(_, _, _, _, _, current)| current != Some(&offset))
            {
                commands.entity(bone.entity).insert(offset);
            }
        }
    }
}

/// Longer legs raise the pelvis instead of lowering both feet through the floor.
/// Animation still supplies its authored root motion and contact timing.
fn preserve_neutral_ground_height(rig: &mut BTreeMap<Entity, BoneReference>) {
    let named = |name: &str| rig.values().find(|bone| bone.name.as_deref() == Some(name));
    let (Some(left), Some(right), Some(root)) = (named("l_foot"), named("r_foot"), named("root"))
    else {
        return;
    };
    let correction = [left, right]
        .iter()
        .map(|foot| foot.global(rig, false).translation.y - foot.global(rig, true).translation.y)
        .sum::<f32>()
        / 2.0;
    let parent = root
        .parent
        .and_then(|entity| rig.get(&entity))
        .map(|parent| parent.global(rig, false))
        .unwrap_or_default();
    let local_correction = parent.rotation.inverse() * Vec3::Y * correction / parent.scale;
    let root_entity = root.entity;
    rig.get_mut(&root_entity).expect("located rig root").offset += local_correction;
}

#[cfg(test)]
mod tests {
    use super::*;
    use adventuresim_core::character_proportions::{BODY_PROPORTION_COUNT, BodyProportion};

    fn basis(proportion: BodyProportion, delta: Vec3) -> SkeletalJointBasis {
        let mut basis = JointProportionBasis {
            reference: CharacterProportions::default(),
            translation_metres: [[0.0; 3]; BODY_PROPORTION_COUNT],
        };
        basis.translation_metres[proportion.index()] = delta.to_array();
        SkeletalJointBasis(basis)
    }

    #[test]
    fn skeletal_proportions_keep_instances_independent_and_long_legs_grounded() {
        let mut world = World::new();
        let mut rigs = Vec::new();
        for (id, width, length) in [(42, -0.5, -0.5), (43, 0.5, 0.5)] {
            let mut proportions = CharacterProportions::default();
            proportions.set(BodyProportion::HipWidth, width).unwrap();
            proportions
                .set(BodyProportion::UpperLegLength, length)
                .unwrap();
            let owner = world
                .spawn((CharacterId(id), CharacterSkeletalProportions(proportions)))
                .id();
            let root = world
                .spawn((
                    Name::new("root"),
                    AuthoredBindTransform {
                        owner,
                        local: Transform::from_xyz(0.0, 1.0, 0.0),
                    },
                ))
                .id();
            let mut hips = Vec::new();
            for (side, sign) in [("l", 1.0), ("r", -1.0)] {
                let hip = world
                    .spawn((
                        Name::new(format!("{side}_upleg")),
                        ChildOf(root),
                        AuthoredBindTransform {
                            owner,
                            local: Transform::from_xyz(sign * 0.1, 0.0, 0.0),
                        },
                        basis(BodyProportion::HipWidth, Vec3::X * sign * 0.1),
                    ))
                    .id();
                world.spawn((
                    Name::new(format!("{side}_foot")),
                    ChildOf(hip),
                    AuthoredBindTransform {
                        owner,
                        local: Transform::from_xyz(0.0, -1.0, 0.0),
                    },
                    basis(BodyProportion::UpperLegLength, Vec3::NEG_Y * 0.1),
                ));
                hips.push(hip);
            }
            rigs.push((root, hips));
        }
        world.run_system_cached(sync_skeletal_proportions).unwrap();
        for ((root, hips), sign) in rigs.iter().zip([-1.0, 1.0]) {
            assert!(
                (world.get::<SkeletalJointOffset>(*root).unwrap().0.y - sign * 0.05).abs() < 1e-6
            );
            for (hip, side) in hips.iter().zip([1.0, -1.0]) {
                assert!(
                    (world.get::<SkeletalJointOffset>(*hip).unwrap().0.x - sign * side * 0.05)
                        .abs()
                        < 1e-6
                );
                assert_eq!(
                    world
                        .get::<AuthoredBindTransform>(*hip)
                        .unwrap()
                        .local
                        .translation
                        .x,
                    side * 0.1
                );
            }
        }
        world.run_system_cached(sync_skeletal_proportions).unwrap();
        assert!((world.get::<SkeletalJointOffset>(rigs[1].0).unwrap().0.y - 0.05).abs() < 1e-6);
    }
}
