//! Follicle sampling in the look's existing random stream.
use crate::follicles::{Follicles, Growth};
use crate::rng::Rng;

pub(super) fn follicles(f: &mut Follicles, rng: &mut Rng) {
    let mut pick = rng.index(100) as i32;
    f.growth = Growth::Whorl;
    for (growth, weight) in [
        (Growth::Whorl, 55),
        (Growth::Cowlick, 12),
        (Growth::DoubleWhorl, 8),
        (Growth::Radial, 5),
        (Growth::Forward, 8),
        (Growth::Backward, 7),
        (Growth::Down, 5),
    ] {
        pick -= weight;
        if pick < 0 {
            f.growth = growth;
            break;
        }
    }
    f.whorl = [rng.range(-0.3, 0.3), rng.range(0.3, 0.8)];
    f.spiral = rng.range(0.6, 1.6);
    f.clockwise = rng.chance(0.75);
    f.angle = (f.angle * rng.range(0.7, 1.5)).clamp(0.1, 1.2);
    f.angle_spread = rng.range(0.05, 0.2);
    f.direction_spread = rng.range(0.05, 0.3);
    f.memory = rng.log_range(0.003, 0.015);
    f.radius = rng.range(25e-6, 50e-6);
    f.radius_spread = rng.range(0.15, 0.35);
    f.patchiness = if rng.chance(0.3) {
        rng.range(0.0, 0.5)
    } else {
        0.0
    };
    let h = &mut f.hairline;
    h.front = rng.range(-0.008, 0.01);
    h.sides = rng.range(-0.008, 0.008);
    h.nape = rng.range(-0.01, 0.01);
    if rng.chance(0.25) {
        h.recession += rng.range(0.0, 0.02);
    }
    h.peak = if rng.chance(0.15) {
        rng.range(0.0, 0.012)
    } else {
        0.0
    };
    h.softness = rng.range(0.002, 0.008);
}
