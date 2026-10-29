//! Traverse the retained rig hierarchy in pose-buffer space.
//!
//! Joint ordinals remain typed across recursive sampling. Their
//! values select rig-order collections whose owner performs native indexing.

use super::*;

pub(super) fn hand_animation_joint(
    definition: &RigDefinition,
    mut joint: RigJointOrdinal,
) -> Option<ClipLayer> {
    loop {
        match definition.joints[joint].role {
            Some(BoneRole::HandRight) => return Some(ClipLayer::MainHand),
            Some(BoneRole::HandLeft) => return Some(ClipLayer::Offhand),
            _ => {}
        }
        let parent = definition.joints[joint].parent?;
        joint = parent;
    }
}

pub(super) fn local_pose_global(
    definition: &RigDefinition,
    pose: &RigJointTable<LocalPose>,
    joint: RigJointOrdinal,
    cache: &mut RigJointTable<Option<Transform>>,
) -> Transform {
    if let Some(global) = cache[joint] {
        return global;
    }
    let local = Transform {
        translation: pose[joint].translation,
        rotation: pose[joint].rotation,
        scale: pose[joint].scale,
    };
    let global = definition.joints[joint].parent.map_or(local, |parent| {
        local_pose_global(definition, pose, parent, cache).mul_transform(local)
    });
    cache[joint] = Some(global);
    global
}

pub(super) fn aim_pose_joint(
    definition: &RigDefinition,
    pose: &mut RigJointTable<LocalPose>,
    joint: RigJointOrdinal,
    from: Vec3,
    to: Vec3,
) {
    let (Some(from), Some(to)) = (from.try_normalize(), to.try_normalize()) else {
        return;
    };
    let mut cache = RigJointTable::from(vec![None; pose.len()]);
    let current = local_pose_global(definition, pose, joint, &mut cache);
    let parent_rotation = definition.joints[joint]
        .parent
        .map(|parent| local_pose_global(definition, pose, parent, &mut cache).rotation)
        .unwrap_or(Quat::IDENTITY);
    let desired_world = Quat::from_rotation_arc(from, to) * current.rotation;
    let local = parent_rotation.inverse() * desired_world;
    if local.is_finite() {
        pose[joint].rotation = local.normalize();
    }
}

pub(super) fn sampled_global_transform(
    definition: &RigDefinition,
    clip: &BakedClip,
    joint: RigJointOrdinal,
    time_seconds: ClipTime,
    cache: &mut RigJointTable<Option<Transform>>,
) -> Transform {
    if let Some(transform) = cache[joint] {
        return transform;
    }
    let pose = clip.sample(joint, time_seconds);
    let local = Transform {
        translation: pose.translation,
        rotation: pose.rotation,
        scale: pose.scale,
    };
    let global = definition.joints[joint].parent.map_or(local, |parent| {
        sampled_global_transform(definition, clip, parent, time_seconds, cache).mul_transform(local)
    });
    cache[joint] = Some(global);
    global
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hand_layer_contains_only_hand_bone_subtrees() {
        let joint = |name: &str, parent: Option<usize>| {
            let identity = AuthoredRigJoint::new(name.into());
            RigJoint {
                target: AnimationTargetId::from_name(&Name::new(name.to_owned())),
                bind: LocalPose::from_transform(Transform::IDENTITY),
                parent: parent.map(RigJointOrdinal::from),
                name: Some(identity.name().clone()),
                role: identity.role(),
                body_region: AnimationBodyRegion::Upper,
            }
        };
        let definition = RigDefinition {
            family: "test".into(),
            joints: vec![
                joint("root", None),
                joint("r_lowarm", Some(0)),
                joint("r_wrist", Some(1)),
                joint("r_index1", Some(2)),
                joint("l_wrist", Some(0)),
                joint("l_thumb1", Some(4)),
            ]
            .into(),
            name_ordinals: Default::default(),
        };

        assert_eq!(
            hand_animation_joint(&definition, RigJointOrdinal::from(0usize)),
            None
        );
        assert_eq!(
            hand_animation_joint(&definition, RigJointOrdinal::from(1usize)),
            None
        );
        assert_eq!(
            hand_animation_joint(&definition, RigJointOrdinal::from(2usize)),
            Some(ClipLayer::MainHand)
        );
        assert_eq!(
            hand_animation_joint(&definition, RigJointOrdinal::from(3usize)),
            Some(ClipLayer::MainHand)
        );
        assert_eq!(
            hand_animation_joint(&definition, RigJointOrdinal::from(4usize)),
            Some(ClipLayer::Offhand)
        );
        assert_eq!(
            hand_animation_joint(&definition, RigJointOrdinal::from(5usize)),
            Some(ClipLayer::Offhand)
        );
    }
}
