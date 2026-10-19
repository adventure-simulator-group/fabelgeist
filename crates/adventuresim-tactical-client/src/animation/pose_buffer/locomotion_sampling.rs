//! Sparse locomotion anchors and their admitted authored timeline.

use super::*;

pub(super) fn sample_locomotion_at_time(
    clip: &AnimationClip,
    joint: &RigJoint,
    duration: ClipDuration,
    time_seconds: ClipTime,
) -> AnimationValueResult<LocalPose> {
    let phase = if duration.seconds() > 0.0 {
        time_seconds.seconds() / duration.seconds()
    } else {
        0.0
    };
    sample_two_pose_locomotion_clip(clip, joint, duration, CyclePhase::from_cycles(phase)?)
}

fn sample_two_pose_locomotion_clip(
    clip: &AnimationClip,
    joint: &RigJoint,
    duration: ClipDuration,
    authored_phase: CyclePhase,
) -> AnimationValueResult<LocalPose> {
    // The source owns two unique poses: contact and passing/flight. Their
    // mirrored counterparts occupy the other half-cycle. Sample only those
    // four semantic anchors, then own every in-between here instead of asking
    // the glTF sampler (or the 30 Hz baked cache) for intermediate frames.
    let (start_index, end_index, amount) = sparse_locomotion_segment(authored_phase.cycles());
    const ANCHOR_COUNT: f32 = 4.0;
    let sample_anchor = |index: u32| {
        // Preserve the source kernel's multiply-then-divide f32 order.
        ClipTime::from_seconds(duration.seconds() * index as f32 / ANCHOR_COUNT)
            .map(|time| sample_source_clip(clip, joint, time))
    };
    Ok(sample_anchor(start_index)?.interpolate(sample_anchor(end_index)?, amount))
}

fn sparse_locomotion_segment(authored_phase: f32) -> (u32, u32, f32) {
    let coordinate = authored_phase.rem_euclid(1.0) * 4.0;
    let start = coordinate.floor() as u32;
    let t = coordinate - start as f32;
    (start, (start + 1) % 4, t * t * (3.0 - 2.0 * t))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sparse_locomotion_sampling_uses_only_semantic_quarter_cycle_anchors() {
        assert_eq!(sparse_locomotion_segment(0.0), (0, 1, 0.0));
        assert_eq!(sparse_locomotion_segment(0.25), (1, 2, 0.0));
        assert_eq!(sparse_locomotion_segment(0.5), (2, 3, 0.0));
        assert_eq!(sparse_locomotion_segment(0.75), (3, 0, 0.0));
        assert_eq!(sparse_locomotion_segment(1.0), (0, 1, 0.0));
        let (_, _, midpoint) = sparse_locomotion_segment(0.125);
        assert!((midpoint - 0.5).abs() < 0.0001);
    }
}
