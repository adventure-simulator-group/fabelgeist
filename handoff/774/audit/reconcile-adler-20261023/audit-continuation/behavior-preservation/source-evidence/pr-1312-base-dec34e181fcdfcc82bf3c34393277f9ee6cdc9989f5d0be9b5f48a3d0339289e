use super::*;

#[expect(
    clippy::type_complexity,
    reason = "the Bevy query selects newly transformed nodes that do not yet own a captured bind transform"
)]
pub(super) fn capture_authored_bind_transforms(
    mut commands: Commands,
    nodes: Query<(Entity, &Transform), (Added<Transform>, Without<AuthoredBindTransform>)>,
    parents: Query<&ChildOf>,
    roots: Query<&AnimationRigScene>,
) {
    for (entity, transform) in &nodes {
        let mut current = entity;
        for _ in 0..64 {
            if let Ok(root) = roots.get(current) {
                commands.entity(entity).insert(AuthoredBindTransform {
                    owner: root.0,
                    local: *transform,
                });
                break;
            }
            let Ok(parent) = parents.get(current) else {
                break;
            };
            current = parent.parent();
        }
    }
}

pub(super) fn restore_authored_bind_pose(
    playbacks: Query<&AnimationPlayback>,
    mut nodes: Query<(
        &AuthoredBindTransform,
        Option<&skeletal_proportions::SkeletalJointOffset>,
        &mut Transform,
    )>,
) {
    for (bind, offset, mut transform) in &mut nodes {
        if playbacks
            .get(bind.owner)
            .is_ok_and(|playback| playback.use_authored_bind_pose)
        {
            *transform = bind.local;
            transform.translation += offset.map(|offset| offset.0).unwrap_or_default();
        }
    }
}
