use super::*;

pub(super) fn sample_character_plan(
    playback: &AnimationPlayback,
    rig: &PoseBufferRig,
    clips: &Assets<AnimationClip>,
    bank: &mut BakedClipBank,
    metrics: &mut PoseBufferMetrics,
    offsets: &Query<&skeletal_proportions::SkeletalJointOffset>,
) -> Option<SampledPlan> {
    let mut sampled = sample_plan(playback, &rig.definition, clips, bank, metrics)?;
    apply_offsets(&mut sampled.pose, &rig.entities, offsets);
    Some(sampled)
}

fn apply_offsets(
    poses: &mut [LocalPose],
    entities: &[Option<Entity>],
    offsets: &Query<&skeletal_proportions::SkeletalJointOffset>,
) {
    for (pose, entity) in poses.iter_mut().zip(entities) {
        if let Some(offset) = entity.and_then(|entity| offsets.get(entity).ok()) {
            pose.translation += offset.0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::SystemState;

    #[test]
    fn skeletal_proportions_preserve_shared_motion_and_do_not_accumulate() {
        let mut world = World::new();
        let narrow = world
            .spawn(skeletal_proportions::SkeletalJointOffset(
                Vec3::NEG_X * 0.05,
            ))
            .id();
        let wide = world
            .spawn(skeletal_proportions::SkeletalJointOffset(Vec3::X * 0.05))
            .id();
        let mut state =
            SystemState::<Query<&skeletal_proportions::SkeletalJointOffset>>::new(&mut world);
        let clip = BakedClip {
            duration: 1.0,
            frame_dt: 1.0,
            frames: 2,
            tracks: vec![BoneTrack {
                translations: vec![Vec3::new(0.1, 0.0, 0.0), Vec3::new(0.1, 0.2, 0.3)],
                rotations: vec![Quat::IDENTITY, Quat::from_rotation_z(0.4)],
                scales: vec![Vec3::ONE; 2],
                animated: true,
            }],
        };
        let offsets = state.get(&world).unwrap();
        for _ in 0..2 {
            let source = clip.sample(0, 0.5);
            let mut first = vec![source];
            let mut second = vec![source];
            apply_offsets(&mut first, &[Some(narrow)], &offsets);
            apply_offsets(&mut second, &[Some(wide)], &offsets);
            assert!((second[0].translation.x - first[0].translation.x - 0.1).abs() < 1e-6);
            assert_eq!(second[0].translation.yz(), source.translation.yz());
            assert_eq!(second[0].rotation, source.rotation);
            assert_eq!(source, clip.sample(0, 0.5));
        }
    }
}
