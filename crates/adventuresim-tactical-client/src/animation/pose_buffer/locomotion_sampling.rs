//! Sparse locomotion anchors and their admitted authored timeline.

use super::*;

#[derive(Debug, PartialEq)]
struct SparseLocomotionSegment {
    start: u32,
    end: u32,
    amount: UnitInterval,
}

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
    let segment = sparse_locomotion_segment(authored_phase)?;
    const ANCHOR_COUNT: f32 = 4.0;
    let sample_anchor = |index: u32| {
        // Preserve the source kernel's multiply-then-divide f32 order.
        ClipTime::from_seconds(duration.seconds() * index as f32 / ANCHOR_COUNT)
            .map(|time| sample_source_clip(clip, joint, time))
    };
    Ok(sample_anchor(segment.start)?.interpolate(sample_anchor(segment.end)?, segment.amount))
}

fn sparse_locomotion_segment(
    authored_phase: CyclePhase,
) -> AnimationValueResult<SparseLocomotionSegment> {
    let coordinate = authored_phase.cycles().rem_euclid(1.0) * 4.0;
    let start = coordinate.floor() as u32;
    let t = coordinate - start as f32;
    Ok(SparseLocomotionSegment {
        start,
        end: (start + 1) % 4,
        amount: UnitInterval::from_fraction(t * t * (3.0 - 2.0 * t))?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sparse_locomotion_sampling_uses_only_semantic_quarter_cycle_anchors() {
        assert_eq!(
            sparse_locomotion_segment(CyclePhase::from_cycles(0.0).unwrap()).unwrap(),
            SparseLocomotionSegment {
                start: 0,
                end: 1,
                amount: UnitInterval::from_fraction(0.0).unwrap()
            }
        );
        assert_eq!(
            sparse_locomotion_segment(CyclePhase::from_cycles(0.25).unwrap()).unwrap(),
            SparseLocomotionSegment {
                start: 1,
                end: 2,
                amount: UnitInterval::from_fraction(0.0).unwrap()
            }
        );
        assert_eq!(
            sparse_locomotion_segment(CyclePhase::from_cycles(0.5).unwrap()).unwrap(),
            SparseLocomotionSegment {
                start: 2,
                end: 3,
                amount: UnitInterval::from_fraction(0.0).unwrap()
            }
        );
        assert_eq!(
            sparse_locomotion_segment(CyclePhase::from_cycles(0.75).unwrap()).unwrap(),
            SparseLocomotionSegment {
                start: 3,
                end: 0,
                amount: UnitInterval::from_fraction(0.0).unwrap()
            }
        );
        assert_eq!(
            sparse_locomotion_segment(CyclePhase::from_cycles(1.0).unwrap()).unwrap(),
            SparseLocomotionSegment {
                start: 0,
                end: 1,
                amount: UnitInterval::from_fraction(0.0).unwrap()
            }
        );
        let midpoint = sparse_locomotion_segment(CyclePhase::from_cycles(0.125).unwrap())
            .unwrap()
            .amount
            .fraction();
        assert!((midpoint - 0.5).abs() < 0.0001);
    }
}
