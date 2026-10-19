//! Deterministic capture tick progression and finite frame-interval admission.

use super::*;
use adventuresim_tactical_core::animation::{
    AnimationElapsed, AnimationSampleTick, AnimationValueResult,
};

pub(super) fn next_capture_simulation_tick(
    current: AnimationSampleTick,
    absolute_first_sample: bool,
) -> AnimationSampleTick {
    if absolute_first_sample {
        current
    } else {
        current.next()
    }
}

pub(super) fn capture_frame_elapsed(
    scenario_frame: usize,
) -> AnimationValueResult<AnimationElapsed> {
    let seconds = if scenario_frame == 0 {
        0.0
    } else {
        1.0 / locomotion_sample_hz()
    };
    AnimationElapsed::from_seconds(seconds)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capture_ticks_remain_unique_across_consecutive_scenario_boundaries() {
        let first = next_capture_simulation_tick(AnimationSampleTick::ZERO, true);
        let second = next_capture_simulation_tick(first, false);
        let next_scenario_frame_zero = next_capture_simulation_tick(second, false);
        assert_eq!(first, AnimationSampleTick::from(0u64));
        assert_eq!(second, AnimationSampleTick::from(1u64));
        assert_eq!(next_scenario_frame_zero, AnimationSampleTick::from(2u64));
    }
}
