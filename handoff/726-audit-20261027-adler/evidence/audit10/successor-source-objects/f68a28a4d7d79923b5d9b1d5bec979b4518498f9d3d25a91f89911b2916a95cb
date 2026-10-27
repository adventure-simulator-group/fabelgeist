//! Landing compression and knee reserve require actual captured observations.

use super::*;
use adventuresim_tactical_core::animation::AnimationEventSpan;

struct LandingSample<'a> {
    frame: &'a FrameSample,
    compression: HeightDistance,
}

pub(super) fn landing_response_valid(
    frames: &[&FrameSample],
) -> AnimationValueResult<bool> {
    if frames.is_empty() { return Ok(true); }
    let mut maximum: Option<LandingSample<'_>> = None;
    for frame in frames {
        let Some(compression) = frame.landing_compression_metres else { return Ok(false); };
        // Keep the existing total_cmp ordering and the last sample on a tie.
        if maximum.as_ref().is_none_or(|maximum| compression.metres().total_cmp(&maximum.compression.metres()) != std::cmp::Ordering::Less) {
            maximum = Some(LandingSample { frame, compression });
        }
    }
    let Some(maximum) = maximum else { return Ok(false); };
    let knee_valid = frame_minimum_knee_flexion(maximum.frame)?.measured().is_some_and(|angle| angle.degrees() >= 10.0);
    Ok(frames.last().zip(frames.first()).is_some_and(|(last, first)| {
        last.landing_sequence.wrapping_distance_from(first.landing_sequence) == AnimationEventSpan::ONE
    }) && (0.04..=0.08).contains(&maximum.compression.metres())
        && frames.last().is_some_and(|frame| frame.landing_compression_metres.is_some_and(|height| height.metres() <= 0.005))
        && knee_valid)
}
