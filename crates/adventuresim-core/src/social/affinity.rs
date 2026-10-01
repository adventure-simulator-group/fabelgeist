//! Bounded directional affinity: decay, gains, and realized changes.

pub const AFFINITY_MIN: f32 = -100.0;

pub const AFFINITY_MAX: f32 = 100.0;

pub const AFFINITY_HALF_LIFE_MINUTES: u64 = 30 * 24 * 60;

pub fn settle_affinity(anchor: f32, elapsed_minutes: u64) -> f32 {
    if !anchor.is_finite() {
        return 0.0;
    }
    let factor = 0.5_f32.powf(elapsed_minutes as f32 / AFFINITY_HALF_LIFE_MINUTES as f32);
    let value = anchor.clamp(AFFINITY_MIN, AFFINITY_MAX) * factor;
    if value.abs() < 0.000_1 { 0.0 } else { value }
}

/// A realized morale improvement grants less affinity near the positive cap.
pub fn affinity_gain(current: f32, realized_morale_gain: f32) -> f32 {
    if !realized_morale_gain.is_finite() || realized_morale_gain <= 0.0 {
        return 0.0;
    }
    let headroom =
        (AFFINITY_MAX - current.clamp(AFFINITY_MIN, AFFINITY_MAX)) / (AFFINITY_MAX - AFFINITY_MIN);
    (realized_morale_gain * 0.8 * headroom).max(0.0)
}

pub fn realized_affinity_delta(current: f32, requested_delta: f32) -> f32 {
    (current + requested_delta).clamp(AFFINITY_MIN, AFFINITY_MAX)
        - current.clamp(AFFINITY_MIN, AFFINITY_MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decay_is_partition_independent_and_never_crosses_neutral() {
        for start in [-80.0, 80.0] {
            let once = settle_affinity(start, 40_000);
            let split = settle_affinity(settle_affinity(start, 10_000), 30_000);
            assert!((once - split).abs() < 0.0001);
            assert_eq!(once.signum(), start.signum());
            assert!(once.abs() < start.abs());
        }
    }

    #[test]
    fn realized_affinity_delta_reports_clamping() {
        assert!((realized_affinity_delta(-99.5, -2.5) + 0.5).abs() < 0.0001);
        assert_eq!(realized_affinity_delta(0.0, 0.0), 0.0);
    }

    #[test]
    fn positive_gain_requires_realized_improvement_and_diminishes() {
        assert_eq!(affinity_gain(0.0, 0.0), 0.0);
        assert_eq!(affinity_gain(0.0, -1.0), 0.0);
        assert!(affinity_gain(0.0, 5.0) > affinity_gain(90.0, 5.0));
    }
}
