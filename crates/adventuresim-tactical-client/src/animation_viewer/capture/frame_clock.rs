//! Deterministic capture tick progression and finite frame-interval admission.

use super::*;
use adventuresim_tactical_core::animation::AnimationElapsed;

pub(super) fn next_capture_simulation_tick(current: u64, absolute_first_sample: bool) -> u64 {
    if absolute_first_sample {
        current
    } else {
        current.wrapping_add(1)
    }
}

pub(super) fn capture_frame_elapsed(scenario_frame: usize) -> Option<AnimationElapsed> {
    let seconds = if scenario_frame == 0 {
        0.0
    } else {
        1.0 / locomotion_sample_hz()
    };
    match AnimationElapsed::from_seconds(seconds) {
        Ok(elapsed) => Some(elapsed),
        Err(error) => {
            error!(%error, "Rejected capture frame interval");
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capture_ticks_remain_unique_across_consecutive_scenario_boundaries() {
        let first = next_capture_simulation_tick(0, true);
        let second = next_capture_simulation_tick(first, false);
        let next_scenario_frame_zero = next_capture_simulation_tick(second, false);
        assert_eq!(first, 0);
        assert_eq!(second, 1);
        assert_eq!(next_scenario_frame_zero, 2);
    }
}
