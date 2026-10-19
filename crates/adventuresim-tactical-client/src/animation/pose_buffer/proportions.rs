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
    poses: &mut RigJointTable<LocalPose>,
    entities: &RigJointTable<Option<Entity>>,
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
        const OFFSET_COMPARISON_TOLERANCE_METRES: f32 = 1e-6;
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
        let clip = BakedClip::new(
            ClipDuration::from_seconds(1.0).unwrap(),
            AnimationElapsed::from_seconds(1.0).unwrap(),
            BakedFrameCount::try_from(2).unwrap(),
            vec![BoneTrack {
                translations: vec![Vec3::new(0.1, 0.0, 0.0), Vec3::new(0.1, 0.2, 0.3)],
                rotations: vec![Quat::IDENTITY, Quat::from_rotation_z(0.4)],
                scales: vec![Vec3::ONE; 2],
                source: BoneTrackSource::Authored,
            }]
            .into(),
        )
        .unwrap();
        let offsets = state.get(&world).unwrap();
        for _ in 0..2 {
            let source = clip.sample(
                RigJointOrdinal::from(0usize),
                ClipTime::from_seconds(0.5).unwrap(),
            );
            let mut first = RigJointTable::from(vec![source]);
            let mut second = RigJointTable::from(vec![source]);
            apply_offsets(
                &mut first,
                &RigJointTable::from(vec![Some(narrow)]),
                &offsets,
            );
            apply_offsets(
                &mut second,
                &RigJointTable::from(vec![Some(wide)]),
                &offsets,
            );
            assert!(
                (second[RigJointOrdinal::from(0usize)].translation.x
                    - first[RigJointOrdinal::from(0usize)].translation.x
                    - 0.1)
                    .abs()
                    < OFFSET_COMPARISON_TOLERANCE_METRES
            );
            assert_eq!(
                second[RigJointOrdinal::from(0usize)].translation.yz(),
                source.translation.yz()
            );
            assert_eq!(
                second[RigJointOrdinal::from(0usize)].rotation,
                source.rotation
            );
            assert_eq!(
                source,
                clip.sample(
                    RigJointOrdinal::from(0usize),
                    ClipTime::from_seconds(0.5).unwrap()
                )
            );
        }
    }
}
