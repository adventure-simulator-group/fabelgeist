//! Native global-bone capture and its serialized frame boundary.

use super::*;

impl GlobalBoneFrame {
    pub(super) fn capture(
        frame: &PlannedFrame,
        skeleton: &SkeletonState,
        subject_global: &GlobalTransform,
        subject: Entity,
        animation_bones: &Query<(
            &AnimationTargetId,
            &AuthoredBindTransform,
            Option<&Name>,
            &GlobalTransform,
        )>,
    ) -> Self {
        Self {
            scenario: frame.scenario.label().to_owned(),
            scenario_frame: frame.scenario_frame,
            time_seconds: frame.time_seconds.seconds(),
            action: skeleton.action_kind(),
            action_phase: skeleton.action_phase(),
            subject_translation: subject_global.translation().to_array(),
            subject_rotation_xyzw: subject_global.rotation().to_array(),
            bones: collect_global_bone_transforms(subject, animation_bones),
        }
    }
}

fn collect_global_bone_transforms(
    subject: Entity,
    bones: &Query<(
        &AnimationTargetId,
        &AuthoredBindTransform,
        Option<&Name>,
        &GlobalTransform,
    )>,
) -> Vec<GlobalBoneTransformSample> {
    let mut samples = bones
        .iter()
        .filter(|(_, bind, _, _)| bind.owner == subject)
        .map(|(target, _, name, transform)| {
            let (scale, rotation, translation) = transform.to_scale_rotation_translation();
            let sample = GlobalBoneTransformSample {
                name: name.map_or_else(|| "<unnamed>".to_owned(), |name| name.as_str().to_owned()),
                target_id: capture_animation_target_id(*target),
                translation: translation.to_array(),
                rotation_xyzw: rotation.to_array(),
                scale: scale.to_array(),
            };
            (*target, sample)
        })
        .collect::<Vec<_>>();
    samples.sort_by(|(left_target, left), (right_target, right)| {
        (&left.name, left_target).cmp(&(&right.name, right_target))
    });
    samples.into_iter().map(|(_, sample)| sample).collect()
}
