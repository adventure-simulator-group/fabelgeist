use super::super::{AnimationRigScene, AuthoredRigJoint};
use super::*;
use fabelgeist_rig::{RigJointMembership, RigJointName};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct MirroredJointPair {
    pub(super) left: Entity,
    pub(super) right: Entity,
}

#[derive(Clone, Debug, Default)]
pub(super) struct MhrMirrorTopology {
    pub(super) centers: Vec<Entity>,
    pub(super) pairs: Vec<MirroredJointPair>,
}

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
#[derive(Component, Debug, Clone, Default)]
pub(crate) struct HumanoidRig {
    bones: BTreeMap<BoneRole, Entity>,
    rig_scene: Option<Entity>,
    sole_axes: SoleAxes,
    mirror: MhrMirrorTopology,
}

#[derive(Clone, Copy, Debug, Default)]
struct SoleAxes {
    left: Option<Vec3>,
    right: Option<Vec3>,
}

#[derive(Component, Debug, Clone, Copy)]
pub(in crate::animation) struct SoleAxisCaptured;

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

impl MhrMirrorTopology {
    pub(super) fn from_names(names: &BTreeMap<RigJointName, Entity>) -> Self {
        let centers = names
            .iter()
            .filter_map(|(name, &entity)| {
                (fabelgeist_rig::mhr::center_membership(name) == RigJointMembership::Included)
                    .then_some(entity)
            })
            .collect();
        let pairs = names
            .iter()
            .filter_map(|(name, &left)| {
                let right_name = fabelgeist_rig::mhr::right_partner(name)?;
                Some(MirroredJointPair {
                    left,
                    right: *names.get(&right_name)?,
                })
            })
            .collect();
        Self { centers, pairs }
    }
}

impl HumanoidRig {
    pub(crate) fn get(&self, role: &BoneRole) -> Option<&Entity> {
        self.bones.get(role)
    }

    pub(crate) fn rig_scene(&self) -> Option<Entity> {
        self.rig_scene
    }

    pub(super) fn sole_axis(&self, foot: LeadFoot) -> Option<Vec3> {
        match foot {
            LeadFoot::Left => self.sole_axes.left,
            LeadFoot::Right => self.sole_axes.right,
        }
    }

    pub(super) fn mirror_centers(&self) -> &[Entity] {
        &self.mirror.centers
    }

    pub(super) fn mirror_pairs(&self) -> &[MirroredJointPair] {
        &self.mirror.pairs
    }

    fn retain_stable_role(&mut self, role: BoneRole, entity: Entity) -> Option<Entity> {
        let retained = self
            .get(&role)
            .map_or(entity, |existing| (*existing).min(entity));
        self.bones.insert(role, retained)
    }
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

    fn joint_name(self) -> RigJointName {
        match self {
            Self::Root => RigJointName::BODY_WORLD,
            Self::Pelvis => RigJointName::ROOT,
            Self::StomachOne => RigJointName::C_SPINE0,
            Self::StomachTwo => RigJointName::C_SPINE1,
            Self::StomachThree => RigJointName::C_SPINE2,
            Self::Chest => RigJointName::C_SPINE3,
            Self::NeckOne => RigJointName::C_NECK,
            Self::Head => RigJointName::C_HEAD,
            Self::Camera => RigJointName::C_CAMERA,
            Self::ClavicleLeft => RigJointName::L_CLAVICLE,
            Self::ClavicleRight => RigJointName::R_CLAVICLE,
            Self::ThighLeft => RigJointName::L_UPLEG,
            Self::ShinLeft => RigJointName::L_LOWLEG,
            Self::FootLeft => RigJointName::L_FOOT,
            Self::ToeLeft => RigJointName::L_BALL,
            Self::ThighRight => RigJointName::R_UPLEG,
            Self::ShinRight => RigJointName::R_LOWLEG,
            Self::FootRight => RigJointName::R_FOOT,
            Self::ToeRight => RigJointName::R_BALL,
            Self::UpperArmLeft => RigJointName::L_UPARM,
            Self::ForearmLeft => RigJointName::L_LOWARM,
            Self::HandLeft => RigJointName::L_WRIST,
            Self::WeaponLeft => RigJointName::L_WEAPON,
            Self::UpperArmRight => RigJointName::R_UPARM,
            Self::ForearmRight => RigJointName::R_LOWARM,
            Self::HandRight => RigJointName::R_WRIST,
            Self::WeaponRight => RigJointName::R_WEAPON,
        }
    }

    pub(crate) fn from_name(name: &RigJointName) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|role| &role.joint_name() == name)
    }
}

#[expect(
    clippy::type_complexity,
    reason = "the Bevy query expresses the admitted-identity and unbound-bone component filter"
)]
pub(crate) fn bind_humanoid_bones(
    mut commands: Commands,
    bones: Query<(Entity, &AuthoredRigJoint), (Added<AuthoredRigJoint>, Without<MhrBone>)>,
    parents: Query<&ChildOf>,
    roots: Query<&AnimationRigScene>,
) {
    for (entity, identity) in &bones {
        if fabelgeist_rig::mhr::membership(identity.name()) == RigJointMembership::Excluded {
            continue;
        }
        let mut current = entity;
        for _ in 0..64 {
            if let Ok(root) = roots.get(current) {
                let mut bone = commands.entity(entity);
                bone.insert(MhrBone { owner: root.0 });
                if let Some(role) = identity.role() {
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
    all_mhr_bones: Query<(Entity, &MhrBone, &AuthoredRigJoint)>,
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
    let mut rigs = BTreeMap::<Entity, HumanoidRig>::new();
    for (entity, bone) in &all_bones {
        let rig = rigs.entry(bone.owner).or_default();
        if let Some(previous) = rig.retain_stable_role(bone.role, entity) {
            let retained = previous.min(entity);
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
            .insert(name.name().clone(), entity);
    }
    for (owner, _) in &cached {
        if !rigs.contains_key(&owner) {
            commands.entity(owner).remove::<HumanoidRig>();
        }
    }
    for (owner, mut rig) in rigs {
        let sole_axes = cached
            .get(owner)
            .map(|(_, rig)| rig.sole_axes)
            .unwrap_or_default();
        let names = named.remove(&owner).unwrap_or_default();
        let mirror = MhrMirrorTopology::from_names(&names);
        if let Ok(mut entity) = commands.get_entity(owner) {
            rig.rig_scene = rig_roots.get(&owner).copied();
            rig.sole_axes = sole_axes;
            rig.mirror = mirror;
            entity.insert(rig);
        }
    }
}

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
        let foot = match bone.role {
            BoneRole::FootLeft => LeadFoot::Left,
            BoneRole::FootRight => LeadFoot::Right,
            _ => continue,
        };
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
                match foot {
                    LeadFoot::Left => rig.sole_axes.left = Some(axis),
                    LeadFoot::Right => rig.sole_axes.right = Some(axis),
                }
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
        let mut rig = HumanoidRig::default();
        let high = Entity::from_bits(20);
        let low = Entity::from_bits(10);
        assert_eq!(rig.retain_stable_role(BoneRole::Head, high), None);
        assert_eq!(rig.retain_stable_role(BoneRole::Head, low), Some(high));
        assert_eq!(rig.get(&BoneRole::Head).copied(), Some(low));
    }

    #[test]
    fn partial_and_empty_rigs_have_explicit_missing_chains() {
        let mut rig = HumanoidRig::default();
        assert_eq!(rig.get(&BoneRole::Head), None);
        rig.retain_stable_role(BoneRole::Head, Entity::from_bits(7));
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

        let MhrMirrorTopology { centers, pairs } = MhrMirrorTopology::from_names(&named);

        assert_eq!(centers.len(), 4);
        assert!(centers.contains(&Entity::from_bits(4)));
        assert_eq!(pairs.len(), 4);
        assert!(pairs.contains(&MirroredJointPair {
            left: Entity::from_bits(5),
            right: Entity::from_bits(6)
        }));
        assert!(pairs.contains(&MirroredJointPair {
            left: Entity::from_bits(7),
            right: Entity::from_bits(8)
        }));
        assert!(pairs.contains(&MirroredJointPair {
            left: Entity::from_bits(9),
            right: Entity::from_bits(10)
        }));
        assert!(pairs.contains(&MirroredJointPair {
            left: Entity::from_bits(11),
            right: Entity::from_bits(12)
        }));
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
            rig.retain_stable_role(BoneRole::FootLeft, foot);
            rig.retain_stable_role(BoneRole::ToeLeft, toe);
        }

        world.run_system_cached(capture_humanoid_rig_axes).unwrap();

        let bind_world_rotation = bind_root_rotation * bind_foot_rotation;
        let raw_axis = sole_up_axis_from_bind(bind_world_rotation);
        let expected = raw_axis.normalize_or_zero();
        let actual = world
            .get::<HumanoidRig>(owner)
            .unwrap()
            .sole_axis(LeadFoot::Left)
            .unwrap();
        assert!(actual.dot(expected) > 0.9999);
        assert!(world.get::<SoleAxisCaptured>(foot).is_some());
    }
}
