use super::super::AnimationRigScene;
use super::*;
use fabelgeist_rig::{RigJointMembership, RigJointName};

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct HumanoidBone {
    pub(crate) owner: Entity,
    pub(crate) role: BoneRole,
}

/// Every canonical MHR joint, including facial, finger, foot, and distributed
/// twist joints which do not need a semantic procedural role of their own.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct MhrBone {
    pub(crate) owner: Entity,
}

/// Cached rig topology. It changes only while an asynchronously loaded scene
/// is binding; procedural passes read it without rebuilding owner/role maps.
#[derive(Component, Debug, Clone)]
pub(crate) struct HumanoidRig {
    bones: [Option<Entity>; BoneRole::COUNT],
    rig_scene: Option<Entity>,
    sole_axes: [Option<Vec3>; 2],
    mirror_centers: Vec<Entity>,
    mirror_pairs: Vec<(Entity, Entity)>,
}

impl Default for HumanoidRig {
    fn default() -> Self {
        Self {
            bones: [None; BoneRole::COUNT],
            rig_scene: None,
            sole_axes: [None; 2],
            mirror_centers: Vec::new(),
            mirror_pairs: Vec::new(),
        }
    }
}

impl HumanoidRig {
    pub(crate) fn get(&self, role: &BoneRole) -> Option<&Entity> {
        self.bones[role.index()].as_ref()
    }

    pub(crate) fn rig_scene(&self) -> Option<Entity> {
        self.rig_scene
    }

    pub(super) fn sole_axis(&self, left: bool) -> Option<Vec3> {
        self.sole_axes[usize::from(!left)]
    }

    pub(super) fn mirror_centers(&self) -> &[Entity] {
        &self.mirror_centers
    }

    pub(super) fn mirror_pairs(&self) -> &[(Entity, Entity)] {
        &self.mirror_pairs
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum BoneRole {
    Root,
    Pelvis,
    StomachOne,
    StomachTwo,
    StomachThree,
    Chest,
    NeckOne,
    Head,
    Camera,
    ClavicleLeft,
    ClavicleRight,
    ThighLeft,
    ShinLeft,
    FootLeft,
    ToeLeft,
    ThighRight,
    ShinRight,
    FootRight,
    ToeRight,
    UpperArmLeft,
    ForearmLeft,
    HandLeft,
    WeaponLeft,
    UpperArmRight,
    ForearmRight,
    HandRight,
    WeaponRight,
}

fn mhr_mirror_topology(
    names: &BTreeMap<RigJointName, Entity>,
) -> (Vec<Entity>, Vec<(Entity, Entity)>) {
    let centers = names
        .iter()
        .filter_map(|(name, &entity)| {
            (name.mhr_center_membership() == RigJointMembership::Included).then_some(entity)
        })
        .collect();
    let pairs = names
        .iter()
        .filter_map(|(name, &left)| {
            let right_name = name.mhr_right_partner()?;
            Some((left, *names.get(&right_name)?))
        })
        .collect();
    (centers, pairs)
}

impl BoneRole {
    pub(super) const COUNT: usize = 27;
    pub(super) const ALL: [Self; Self::COUNT] = [
        Self::Root,
        Self::Pelvis,
        Self::StomachOne,
        Self::StomachTwo,
        Self::StomachThree,
        Self::Chest,
        Self::NeckOne,
        Self::Head,
        Self::Camera,
        Self::ClavicleLeft,
        Self::ClavicleRight,
        Self::ThighLeft,
        Self::ShinLeft,
        Self::FootLeft,
        Self::ToeLeft,
        Self::ThighRight,
        Self::ShinRight,
        Self::FootRight,
        Self::ToeRight,
        Self::UpperArmLeft,
        Self::ForearmLeft,
        Self::HandLeft,
        Self::WeaponLeft,
        Self::UpperArmRight,
        Self::ForearmRight,
        Self::HandRight,
        Self::WeaponRight,
    ];

    pub(crate) fn index(self) -> usize {
        self as usize
    }

    fn joint_name(self) -> &'static RigJointName {
        match self {
            Self::Root => &RigJointName::BODY_WORLD,
            Self::Pelvis => &RigJointName::ROOT,
            Self::StomachOne => &RigJointName::C_SPINE0,
            Self::StomachTwo => &RigJointName::C_SPINE1,
            Self::StomachThree => &RigJointName::C_SPINE2,
            Self::Chest => &RigJointName::C_SPINE3,
            Self::NeckOne => &RigJointName::C_NECK,
            Self::Head => &RigJointName::C_HEAD,
            Self::Camera => &RigJointName::C_CAMERA,
            Self::ClavicleLeft => &RigJointName::L_CLAVICLE,
            Self::ClavicleRight => &RigJointName::R_CLAVICLE,
            Self::ThighLeft => &RigJointName::L_UPLEG,
            Self::ShinLeft => &RigJointName::L_LOWLEG,
            Self::FootLeft => &RigJointName::L_FOOT,
            Self::ToeLeft => &RigJointName::L_BALL,
            Self::ThighRight => &RigJointName::R_UPLEG,
            Self::ShinRight => &RigJointName::R_LOWLEG,
            Self::FootRight => &RigJointName::R_FOOT,
            Self::ToeRight => &RigJointName::R_BALL,
            Self::UpperArmLeft => &RigJointName::L_UPARM,
            Self::ForearmLeft => &RigJointName::L_LOWARM,
            Self::HandLeft => &RigJointName::L_WRIST,
            Self::WeaponLeft => &RigJointName::L_WEAPON,
            Self::UpperArmRight => &RigJointName::R_UPARM,
            Self::ForearmRight => &RigJointName::R_LOWARM,
            Self::HandRight => &RigJointName::R_WRIST,
            Self::WeaponRight => &RigJointName::R_WEAPON,
        }
    }

    pub(crate) fn from_name(name: &RigJointName) -> Option<Self> {
        Self::ALL.into_iter().find(|role| role.joint_name() == name)
    }
}

fn retain_stable_role(
    bones: &mut [Option<Entity>; BoneRole::COUNT],
    role: BoneRole,
    entity: Entity,
) -> Option<Entity> {
    let slot = &mut bones[role.index()];
    let duplicate = *slot;
    *slot = Some(duplicate.map_or(entity, |existing| existing.min(entity)));
    duplicate
}

#[expect(
    clippy::type_complexity,
    reason = "the Bevy query expresses the exact added-name and unbound-bone component filter"
)]
pub(crate) fn bind_humanoid_bones(
    mut commands: Commands,
    bones: Query<(Entity, &Name), (Added<Name>, Without<MhrBone>)>,
    parents: Query<&ChildOf>,
    roots: Query<&AnimationRigScene>,
) {
    for (entity, name) in &bones {
        let name = RigJointName::from(name.as_str());
        if name.mhr_membership() == RigJointMembership::Excluded {
            continue;
        }
        let mut current = entity;
        for _ in 0..64 {
            if let Ok(root) = roots.get(current) {
                let mut bone = commands.entity(entity);
                bone.insert(MhrBone { owner: root.0 });
                if let Some(role) = BoneRole::from_name(&name) {
                    bone.insert(HumanoidBone {
                        owner: root.0,
                        role,
                    });
                }
                break;
            }
            let Ok(parent) = parents.get(current) else {
                break;
            };
            current = parent.parent();
        }
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "Bevy injects each independently borrowed rig topology query as a system parameter"
)]
pub(crate) fn cache_humanoid_rigs(
    mut commands: Commands,
    all_bones: Query<(Entity, &HumanoidBone)>,
    all_mhr_bones: Query<(Entity, &MhrBone, &Name)>,
    cached: Query<(Entity, &HumanoidRig)>,
    added: Query<(), Added<HumanoidBone>>,
    added_mhr: Query<(), Added<MhrBone>>,
    mut removed: RemovedComponents<HumanoidBone>,
    mut removed_mhr: RemovedComponents<MhrBone>,
    rig_scenes: Query<(Entity, &AnimationRigScene)>,
) {
    let topology_changed = !added.is_empty()
        || !added_mhr.is_empty()
        || removed.read().next().is_some()
        || removed_mhr.read().next().is_some();
    if !topology_changed {
        return;
    }
    let mut rigs = BTreeMap::<Entity, [Option<Entity>; BoneRole::COUNT]>::new();
    for (entity, bone) in &all_bones {
        let bones = rigs.entry(bone.owner).or_insert([None; BoneRole::COUNT]);
        if retain_stable_role(bones, bone.role, entity).is_some() {
            let retained = bones[bone.role.index()].expect("duplicate role remains populated");
            warn!(
                owner = ?bone.owner,
                role = ?bone.role,
                ?retained,
                "Duplicate humanoid bone role; retaining the lowest stable entity id"
            );
        }
    }
    let rig_roots = rig_scenes
        .iter()
        .map(|(root, scene)| (scene.0, root))
        .collect::<BTreeMap<_, _>>();
    let mut named = BTreeMap::<Entity, BTreeMap<RigJointName, Entity>>::new();
    for (entity, bone, name) in &all_mhr_bones {
        named
            .entry(bone.owner)
            .or_default()
            .insert(RigJointName::from(name.as_str()), entity);
    }
    for (owner, _) in &cached {
        if !rigs.contains_key(&owner) {
            commands.entity(owner).remove::<HumanoidRig>();
        }
    }
    for (owner, bones) in rigs {
        let sole_axes = cached
            .get(owner)
            .map(|(_, rig)| rig.sole_axes)
            .unwrap_or([None; 2]);
        let names = named.remove(&owner).unwrap_or_default();
        let (mirror_centers, mirror_pairs) = mhr_mirror_topology(&names);
        if let Ok(mut entity) = commands.get_entity(owner) {
            entity.insert(HumanoidRig {
                bones,
                rig_scene: rig_roots.get(&owner).copied(),
                sole_axes,
                mirror_centers,
                mirror_pairs,
            });
        }
    }
}

#[derive(Component, Debug, Clone, Copy)]
pub(in crate::animation) struct SoleAxisCaptured;

/// Captures the foot's bind-space sole normal from the authored global bind
/// transform. The MHR foot joint has no guaranteed cardinal local sole-up axis,
/// so the authored flat bind pose defines it explicitly.
pub(crate) fn capture_humanoid_rig_axes(
    mut commands: Commands,
    feet: Query<(Entity, &HumanoidBone, &AuthoredBindTransform), Without<SoleAxisCaptured>>,
    bind_nodes: Query<(&AuthoredBindTransform, Option<&ChildOf>)>,
    mut rigs: Query<&mut HumanoidRig>,
) {
    for (entity, bone, _) in &feet {
        if !matches!(bone.role, BoneRole::FootLeft | BoneRole::FootRight) {
            continue;
        }
        let Some(bind_global) = authored_bind_global(entity, bone.owner, &bind_nodes) else {
            continue;
        };
        // Never infer the sole plane from the ankle-to-toe joint vector. That
        // vector slopes down through a normally planted foot; forcing it
        // horizontal raises the visible forefoot and produces the toe-up IK
        // pose. Sampling the live FK rotation is likewise invalid because it
        // permanently calibrates the sole from an arbitrary gait frame.
        let axis = sole_up_axis_from_bind(bind_global.rotation());
        if let Some(axis) = axis.try_normalize() {
            commands.entity(entity).insert(SoleAxisCaptured);
            if let Ok(mut rig) = rigs.get_mut(bone.owner) {
                let index = usize::from(bone.role == BoneRole::FootRight);
                rig.sole_axes[index] = Some(axis);
            }
        }
    }
}

pub(crate) fn authored_bind_global(
    entity: Entity,
    owner: Entity,
    bind_nodes: &Query<(&AuthoredBindTransform, Option<&ChildOf>)>,
) -> Option<GlobalTransform> {
    let mut current = entity;
    let mut locals = Vec::new();
    for _ in 0..64 {
        let Ok((bind, parent)) = bind_nodes.get(current) else {
            break;
        };
        if bind.owner != owner {
            break;
        }
        locals.push(bind.local);
        let Some(parent) = parent else {
            break;
        };
        current = parent.parent();
    }
    (!locals.is_empty()).then(|| {
        locals
            .into_iter()
            .rev()
            .fold(GlobalTransform::IDENTITY, |global, local| {
                global.mul_transform(local)
            })
    })
}

pub(super) fn sole_up_axis_from_bind(bind_world_rotation: Quat) -> Vec3 {
    bind_world_rotation.inverse() * Vec3::Y
}

pub(crate) fn pole_to_world(owner_rotation: Quat, owner_local_pole: Vec3) -> Vec3 {
    owner_rotation * owner_local_pole
}

pub(crate) fn pole_to_owner(owner_rotation: Quat, world_pole: Vec3) -> Vec3 {
    owner_rotation.inverse() * world_pole
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn duplicate_roles_retain_the_lowest_stable_entity() {
        let mut bones = [None; BoneRole::COUNT];
        let high = Entity::from_bits(20);
        let low = Entity::from_bits(10);
        assert_eq!(retain_stable_role(&mut bones, BoneRole::Head, high), None);
        assert_eq!(
            retain_stable_role(&mut bones, BoneRole::Head, low),
            Some(high)
        );
        assert_eq!(bones[BoneRole::Head.index()], Some(low));
    }

    #[test]
    fn partial_and_empty_rigs_have_explicit_missing_chains() {
        let mut rig = HumanoidRig::default();
        assert_eq!(rig.get(&BoneRole::Head), None);
        retain_stable_role(&mut rig.bones, BoneRole::Head, Entity::from_bits(7));
        assert_eq!(rig.get(&BoneRole::Head), Some(&Entity::from_bits(7)));
        assert_eq!(rig.get(&BoneRole::NeckOne), None);
    }

    #[test]
    fn mhr_mirroring_covers_center_attachments_and_every_bilateral_joint_kind() {
        let named = [
            ("body_world", 1),
            ("root", 2),
            ("c_spine2", 3),
            ("c_camera", 4),
            ("l_uparm_twist3_proc", 5),
            ("r_uparm_twist3_proc", 6),
            ("l_index2", 7),
            ("r_index2", 8),
            ("l_subtalar", 9),
            ("r_subtalar", 10),
            ("l_weapon", 11),
            ("r_weapon", 12),
        ]
        .into_iter()
        .map(|(name, entity)| (RigJointName::from(name), Entity::from_bits(entity)))
        .collect::<BTreeMap<_, _>>();

        let (centers, pairs) = mhr_mirror_topology(&named);

        assert_eq!(centers.len(), 4);
        assert!(centers.contains(&Entity::from_bits(4)));
        assert_eq!(pairs.len(), 4);
        assert!(pairs.contains(&(Entity::from_bits(5), Entity::from_bits(6))));
        assert!(pairs.contains(&(Entity::from_bits(7), Entity::from_bits(8))));
        assert!(pairs.contains(&(Entity::from_bits(9), Entity::from_bits(10))));
        assert!(pairs.contains(&(Entity::from_bits(11), Entity::from_bits(12))));
    }

    #[test]
    fn sole_axis_is_calibrated_from_authored_bind_not_live_fk() {
        let mut world = World::new();
        let owner = world.spawn(HumanoidRig::default()).id();
        let bind_root_rotation = Quat::from_rotation_y(0.7);
        let rig_root = world
            .spawn(AuthoredBindTransform {
                owner,
                local: Transform::from_rotation(bind_root_rotation),
            })
            .id();
        let bind_foot_rotation = Quat::from_rotation_x(-0.6);
        let foot = world
            .spawn((
                HumanoidBone {
                    owner,
                    role: BoneRole::FootLeft,
                },
                AuthoredBindTransform {
                    owner,
                    local: Transform::from_rotation(bind_foot_rotation),
                },
                // This deliberately disagrees with the authored bind. A live
                // gait sample must have no effect on one-shot calibration.
                Transform::from_rotation(Quat::from_rotation_x(1.2)),
            ))
            .id();
        let toe = world
            .spawn((
                HumanoidBone {
                    owner,
                    role: BoneRole::ToeLeft,
                },
                AuthoredBindTransform {
                    owner,
                    local: Transform::from_translation(Vec3::Y * 0.14),
                },
                Transform::from_translation(Vec3::NEG_Z * 0.14),
            ))
            .id();
        world.entity_mut(rig_root).add_child(foot);
        world.entity_mut(foot).add_child(toe);
        {
            let mut rig = world.get_mut::<HumanoidRig>(owner).unwrap();
            rig.bones[BoneRole::FootLeft.index()] = Some(foot);
            rig.bones[BoneRole::ToeLeft.index()] = Some(toe);
        }

        world.run_system_cached(capture_humanoid_rig_axes).unwrap();

        let bind_world_rotation = bind_root_rotation * bind_foot_rotation;
        let raw_axis = sole_up_axis_from_bind(bind_world_rotation);
        let expected = raw_axis.normalize_or_zero();
        let actual = world
            .get::<HumanoidRig>(owner)
            .unwrap()
            .sole_axis(true)
            .unwrap();
        assert!(actual.dot(expected) > 0.9999);
        assert!(world.get::<SoleAxisCaptured>(foot).is_some());
    }
}
