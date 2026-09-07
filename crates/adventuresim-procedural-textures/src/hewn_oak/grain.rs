//! Periodic growth bands and fibers sharing the displacement around branch knots.

use super::{periodic_delta, smooth, value_noise};

const RING_COUNT: f32 = 37.0;
const FIBER_COUNT: f32 = 173.0;
const LATEWOOD_START: f32 = 0.57;
const LATEWOOD_END: f32 = 0.94;
const KNOT_INFLUENCE_RADII: f32 = 4.0;
const KNOT_CORE_SOFTENING: f32 = 0.22;
const RING_WANDER: f32 = 0.009;
const FIBER_WANDER: f32 = 0.0015;

struct Knot {
    center: [f32; 2],
    radii: [f32; 2],
    lean: f32,
}

// Authored branch intersections in the two-metre repeating timber face.
const KNOTS: [Knot; 3] = [
    Knot {
        center: [0.713, 0.367],
        radii: [0.027, 0.058],
        lean: 0.16,
    },
    Knot {
        center: [0.231, 0.786],
        radii: [0.018, 0.039],
        lean: -0.12,
    },
    Knot {
        center: [0.419, 0.124],
        radii: [0.012, 0.027],
        lean: 0.21,
    },
];

pub(super) struct GrainSample {
    pub(super) latewood: f32,
    pub(super) fibers: f32,
    pub(super) knot: f32,
    pub(super) knot_rings: f32,
}

fn growth_coordinate(u: f32, v: f32) -> (f32, f32, f32) {
    let mut across = u;
    let mut knot_mask = 0.0_f32;
    let mut knot_rings = 0.0_f32;
    for knot in &KNOTS {
        let dy = periodic_delta(v - knot.center[1]);
        let dx = periodic_delta(u - knot.center[0]) - dy * knot.lean;
        let radius_squared = (dx / knot.radii[0]).powi(2) + (dy / knot.radii[1]).powi(2);
        let radius = radius_squared.sqrt();
        let envelope = 1.0 - smooth((radius / KNOT_INFLUENCE_RADII).clamp(0.0, 1.0));
        // Contours split around the branch intersection and converge along its axis.
        // Compact support keeps both value and slope continuous across tile boundaries.
        across -= dx / (radius_squared + KNOT_CORE_SOFTENING) * envelope;
        let core = 1.0 - smooth(((radius - 0.35) / 0.80).clamp(0.0, 1.0));
        knot_mask = knot_mask.max(core);
        knot_rings += (radius * std::f32::consts::TAU * 3.0).sin() * core;
    }
    (across, knot_mask, knot_rings)
}

pub(super) fn sample(u: f32, v: f32) -> GrainSample {
    let (across, knot, knot_rings) = growth_coordinate(u, v);
    let growth = across + (value_noise(u, v, 5, 9, 0x7d13) - 0.5) * RING_WANDER;
    let phase = growth * RING_COUNT;
    let spacing = (value_noise(growth, v, 13, 3, 0x81c7) - 0.5) * 0.85;
    let band = ((phase + spacing) * std::f32::consts::TAU).sin() * 0.5 + 0.5;
    // Thin, dense latewood alternates with a broad earlywood interval.
    let latewood =
        smooth(((band - LATEWOOD_START) / (LATEWOOD_END - LATEWOOD_START)).clamp(0.0, 1.0));
    let fiber_coordinate = growth + (value_noise(growth, v, 19, 17, 0x328b) - 0.5) * FIBER_WANDER;
    let fiber_band = (fiber_coordinate * FIBER_COUNT * std::f32::consts::TAU).sin();
    let interruptions =
        smooth(((value_noise(growth, v, 53, 31, 0x4ce7) - 0.34) / 0.40).clamp(0.0, 1.0));
    let fibers = ((fiber_band - 0.45) / 0.55).max(0.0).powi(2) * interruptions * (1.0 - knot);
    GrainSample {
        latewood,
        fibers,
        knot,
        knot_rings,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grain_bends_around_both_flanks_and_recovers_beyond_the_knot() {
        let knot = &KNOTS[0];
        let displacement = |dx: f32, dy: f32| {
            let u = knot.center[0] + dx;
            growth_coordinate(u, knot.center[1] + dy).0 - u
        };
        assert!(displacement(0.035, 0.0) < -0.008);
        assert!(displacement(-0.035, 0.0) > 0.008);
        assert!(displacement(0.035, 0.20).abs() < 0.001);
    }

    #[test]
    fn growth_and_fiber_fields_match_across_the_repeat() {
        for index in 0..512 {
            let t = (index as f32 + 0.5) / 512.0;
            for (a, b) in [
                (sample(0.0, t), sample(1.0, t)),
                (sample(t, 0.0), sample(t, 1.0)),
            ] {
                assert!((a.latewood - b.latewood).abs() < 0.0002);
                assert!((a.fibers - b.fibers).abs() < 0.0002);
                assert!((a.knot - b.knot).abs() < 0.0002);
            }
        }
    }
}
