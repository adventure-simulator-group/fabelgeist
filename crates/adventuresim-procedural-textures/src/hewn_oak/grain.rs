//! Periodic growth bands and fibers sharing the displacement around branch knots.

use super::{grid_hash, periodic_delta, smooth, value_noise};

const GRAIN_FILTER_GRID: u32 = 4;
const KNOT_TAPER: f32 = 0.30;
const KNOT_RING_DISTORTION: f32 = 0.12;
const KNOT_RING_COUNT: f32 = 2.0;
const LATEWOOD_SHOULDER_RATIO: f32 = 0.45;
const ANATOMICAL_MARK_DENSITY: f32 = 0.28;
const VESSEL_RADII_CELLS: [f32; 2] = [0.30, 0.44];
const RAY_RADII_CELLS: [f32; 2] = [0.43, 0.12];

const COLOR_BAND_COUNT: i32 = 17;
const COLOR_BAND_WIDTH: [f32; 2] = [0.19, 0.39];
const KNOT_COLOR_CORE: f32 = 0.65;

const RING_COUNT: f32 = 61.0;
const FIBER_COUNT: f32 = 173.0;
const RING_SPACING_WARP: f32 = 0.043;
const LATEWOOD_WIDTH: [f32; 2] = [0.12, 0.30];
const VESSEL_COLUMNS: i32 = 211;
const VESSEL_ROWS: i32 = 43;
const RAY_COLUMNS: i32 = 29;
const RAY_ROWS: i32 = 47;
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

#[derive(Default)]
pub(super) struct GrainSample {
    pub(super) dark_wood: f32,
    pub(super) latewood: f32,
    pub(super) fibers: f32,
    pub(super) vessels: f32,
    pub(super) rays: f32,
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
        // Branch cross-sections taper along their axis and lean unevenly.
        let taper = 1.0 + KNOT_TAPER * (dy / knot.radii[1]).tanh();
        let skew = dx - knot.lean * dy * (dy / knot.radii[1]).tanh();
        let radius_squared =
            (skew / (knot.radii[0] * taper)).powi(2) + (dy / knot.radii[1]).powi(2);
        let radius = radius_squared.sqrt();
        let envelope = 1.0 - smooth((radius / KNOT_INFLUENCE_RADII).clamp(0.0, 1.0));
        // Contours split around the branch intersection and converge along its axis.
        // Compact support keeps both value and slope continuous across tile boundaries.
        across -= dx / (radius_squared + KNOT_CORE_SOFTENING) * envelope;
        let core = 1.0 - smooth(((radius - 0.35) / 0.80).clamp(0.0, 1.0));
        knot_mask = knot_mask.max(core);
        let distorted_radius =
            radius + KNOT_RING_DISTORTION * (dy / knot.radii[1] + dx / knot.radii[0]).sin();
        knot_rings += (distorted_radius * std::f32::consts::TAU * KNOT_RING_COUNT).sin() * core;
    }
    (across, knot_mask, knot_rings)
}

pub(super) fn sample(u: f32, v: f32) -> GrainSample {
    let u = u.rem_euclid(1.0);
    let v = v.rem_euclid(1.0);
    let (across, knot, knot_rings) = growth_coordinate(u, v);
    let growth = across + (value_noise(u, v, 5, 9, 0x7d13) - 0.5) * RING_WANDER;
    let growth = growth + (value_noise(growth, v, 9, 2, 0x6ba1) - 0.5) * RING_SPACING_WARP;
    let phase = growth * RING_COUNT;
    let ring = phase.floor() as i32;
    let width = LATEWOOD_WIDTH[0]
        + (LATEWOOD_WIDTH[1] - LATEWOOD_WIDTH[0])
            * grid_hash(ring, 0, RING_COUNT as i32, 1, 0x481b);
    let within = phase - phase.floor();
    // Broad earlywood meets a narrow, asymmetric latewood ridge. Per-ring widths
    // and nonuniform spacing break the equally spaced sinusoidal stripe pattern.
    let latewood = smooth((within / width).clamp(0.0, 1.0))
        * (1.0 - smooth(((within - width) / (width * LATEWOOD_SHOULDER_RATIO)).clamp(0.0, 1.0)));
    let fiber_coordinate = growth + (value_noise(growth, v, 19, 17, 0x328b) - 0.5) * FIBER_WANDER;
    let fiber_band = (fiber_coordinate * FIBER_COUNT * std::f32::consts::TAU).sin();
    let interruptions =
        smooth(((value_noise(growth, v, 53, 31, 0x4ce7) - 0.34) / 0.40).clamp(0.0, 1.0));
    let fibers = ((fiber_band - 0.45) / 0.55).max(0.0).powi(2) * interruptions * (1.0 - knot);
    let color_phase = growth * COLOR_BAND_COUNT as f32;
    let color_width = COLOR_BAND_WIDTH[0]
        + (COLOR_BAND_WIDTH[1] - COLOR_BAND_WIDTH[0])
            * grid_hash(color_phase.floor() as i32, 0, COLOR_BAND_COUNT, 1, 0x942a);
    GrainSample {
        // Discrete intrinsic regions at the authored sample. Only footprint
        // integration below introduces intermediate coverage at their edges.
        dark_wood: f32::from(color_phase.rem_euclid(1.0) < color_width || knot > KNOT_COLOR_CORE),
        latewood,
        fibers,
        vessels: anatomical_marks(
            growth,
            v,
            [VESSEL_COLUMNS, VESSEL_ROWS],
            VESSEL_RADII_CELLS,
            0x67ae,
        ) * (1.0 - latewood)
            * (1.0 - knot),
        rays: anatomical_marks(growth, v, [RAY_COLUMNS, RAY_ROWS], RAY_RADII_CELLS, 0x518d)
            * (1.0 - knot),
        knot,
        knot_rings,
    }
}

/// Integrate the texture footprint before differentiating height into normals.
/// Knot compression otherwise aliases the narrow latewood shoulder into dots.
pub(super) fn filtered_sample(u: f32, v: f32) -> GrainSample {
    let mut result = GrainSample::default();
    let texel = 1.0 / super::HEWN_OAK_TEXTURE_SIZE as f32;
    let weight = 1.0 / (GRAIN_FILTER_GRID * GRAIN_FILTER_GRID) as f32;
    for y in 0..GRAIN_FILTER_GRID {
        for x in 0..GRAIN_FILTER_GRID {
            let du = ((x as f32 + 0.5) / GRAIN_FILTER_GRID as f32 - 0.5) * texel;
            let dv = ((y as f32 + 0.5) / GRAIN_FILTER_GRID as f32 - 0.5) * texel;
            let tap = sample(u + du, v + dv);
            result.dark_wood += tap.dark_wood * weight;
            result.latewood += tap.latewood * weight;
            result.fibers += tap.fibers * weight;
            result.vessels += tap.vessels * weight;
            result.rays += tap.rays * weight;
            result.knot += tap.knot * weight;
            result.knot_rings += tap.knot_rings * weight;
        }
    }
    result
}

// Sparse, jittered anatomical features in the SAME deformed coordinates as
// the growth rings. Vessel tracks run longitudinally; rays cross those tracks.
fn anatomical_marks(u: f32, v: f32, cells: [i32; 2], radius: [f32; 2], salt: u64) -> f32 {
    let x = u.rem_euclid(1.0) * cells[0] as f32;
    let y = v.rem_euclid(1.0) * cells[1] as f32;
    let mut field = 0.0_f32;
    for iy in (y.floor() as i32 - 1)..=(y.floor() as i32 + 1) {
        for ix in (x.floor() as i32 - 1)..=(x.floor() as i32 + 1) {
            let random = |seed| grid_hash(ix, iy, cells[0], cells[1], salt ^ seed);
            if random(0) < 1.0 - ANATOMICAL_MARK_DENSITY {
                continue;
            }
            let dx = (x - ix as f32 - random(0x217a)) / radius[0];
            let dy = (y - iy as f32 - random(0xe312)) / radius[1];
            let shape = (1.0 - dx * dx - dy * dy).max(0.0);
            field = field.max(shape * shape);
        }
    }
    field
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn color_regions_are_discrete_with_filtering_confined_to_boundaries() {
        let mut partial = 0;
        let mut dark = 0;
        let count = 128 * 128;
        for y in 0..128 {
            for x in 0..128 {
                let u = (x as f32 + 0.5) / 128.0;
                let v = (y as f32 + 0.5) / 128.0;
                let authored = sample(u, v).dark_wood;
                assert!(authored == 0.0 || authored == 1.0);
                let coverage = filtered_sample(u, v).dark_wood;
                partial += usize::from(coverage > 0.0 && coverage < 1.0);
                dark += usize::from(coverage > 0.5);
            }
        }
        assert!((0.001..0.15).contains(&(partial as f32 / count as f32)));
        assert!((0.10..0.50).contains(&(dark as f32 / count as f32)));
    }

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
                assert_eq!(a.dark_wood, b.dark_wood);
                assert!((a.latewood - b.latewood).abs() < 0.0002);
                assert!((a.fibers - b.fibers).abs() < 0.0002);
                assert!((a.vessels - b.vessels).abs() < 0.0002);
                assert!((a.rays - b.rays).abs() < 0.0002);
                assert!((a.knot - b.knot).abs() < 0.0002);
            }
        }
    }
}
