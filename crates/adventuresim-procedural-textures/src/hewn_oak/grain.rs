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

const DARK_RING_FRACTION: f32 = 0.55;

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

#[derive(serde::Serialize, serde::Deserialize, Clone, Copy)]
pub struct Knot {
    pub center: [f32; 2],
    pub radii: [f32; 2],
    pub lean: f32,
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

fn growth_coordinate(params: &crate::TextureParameters, u: f32, v: f32) -> (f32, f32, f32) {
    let mut across = u;
    let mut knot_mask = 0.0_f32;
    let mut knot_rings = 0.0_f32;
    for knot in &params.hewn_oak_grain.knots {
        let dy = periodic_delta(v - knot.center[1]);
        let dx = periodic_delta(u - knot.center[0]) - dy * knot.lean;
        // Branch cross-sections taper along their axis and lean unevenly.
        let taper = 1.0 + params.hewn_oak_grain.knot_taper * (dy / knot.radii[1]).tanh();
        let skew = dx - knot.lean * dy * (dy / knot.radii[1]).tanh();
        let radius_squared =
            (skew / (knot.radii[0] * taper)).powi(2) + (dy / knot.radii[1]).powi(2);
        let radius = radius_squared.sqrt();
        let envelope =
            1.0 - smooth((radius / params.hewn_oak_grain.knot_influence_radii).clamp(0.0, 1.0));
        // Contours split around the branch intersection and converge along its axis.
        // Compact support keeps both value and slope continuous across tile boundaries.
        across -= dx / (radius_squared + params.hewn_oak_grain.knot_core_softening)
            * envelope
            * params.hewn_oak_grain.knot_flow_strength;
        let core = 1.0
            - smooth(
                ((radius - params.hewn_oak_grain.knot_core_start)
                    / params.hewn_oak_grain.knot_core_transition)
                    .clamp(0.0, 1.0),
            );
        knot_mask = knot_mask.max(core);
        let distorted_radius = radius
            + params.hewn_oak_grain.knot_ring_distortion
                * (dy / knot.radii[1] + dx / knot.radii[0]).sin();
        knot_rings +=
            (distorted_radius * std::f32::consts::TAU * params.hewn_oak_grain.knot_ring_count)
                .sin()
                * core;
    }
    (across, knot_mask, knot_rings)
}

pub(super) fn sample(params: &crate::TextureParameters, u: f32, v: f32) -> GrainSample {
    let u = u.rem_euclid(1.0);
    let v = v.rem_euclid(1.0);
    let (across, knot, knot_rings) = growth_coordinate(params, u, v);
    let growth = across
        + (value_noise(
            params,
            u,
            v,
            params.hewn_oak_grain.growth_wander_grid[0],
            params.hewn_oak_grain.growth_wander_grid[1],
            0x7d13,
        ) - 0.5)
            * params.hewn_oak_grain.ring_wander;
    let growth = growth
        + (value_noise(
            params,
            growth,
            v,
            params.hewn_oak_grain.spacing_warp_grid[0],
            params.hewn_oak_grain.spacing_warp_grid[1],
            0x6ba1,
        ) - 0.5)
            * params.hewn_oak_grain.ring_spacing_warp;
    let phase = growth * params.hewn_oak_grain.ring_count;
    let ring = phase.floor() as i32;
    let width = params.hewn_oak_grain.latewood_width[0]
        + (params.hewn_oak_grain.latewood_width[1] - params.hewn_oak_grain.latewood_width[0])
            * grid_hash(
                params,
                ring,
                0,
                params.hewn_oak_grain.ring_count as i32,
                1,
                0x481b,
            );
    let within = phase - phase.floor();
    // Broad earlywood meets a narrow, asymmetric latewood ridge. Per-ring widths
    // and nonuniform spacing break the equally spaced sinusoidal stripe pattern.
    let latewood = smooth((within / width).clamp(0.0, 1.0))
        * (1.0
            - smooth(
                ((within - width) / (width * params.hewn_oak_grain.latewood_shoulder_ratio))
                    .clamp(0.0, 1.0),
            ));
    let fibers = fiber_response(params, growth, v, knot);
    // Select whole latewood ridges, using their exact footprint. Color follows
    // both relief flanks through the knot warp without tracing every fine feature.
    let dark_ring = grid_hash(
        params,
        ring,
        0,
        params.hewn_oak_grain.ring_count as i32,
        1,
        0x942a,
    ) < params.hewn_oak_grain.dark_ring_fraction;
    GrainSample {
        // Discrete intrinsic regions at the authored sample. Only footprint
        // integration below introduces intermediate coverage at their edges.
        dark_wood: f32::from(dark_ring && latewood > 0.0),
        latewood,
        fibers,
        vessels: anatomical_marks(
            params,
            growth,
            v,
            [
                params.hewn_oak_grain.vessel_columns,
                params.hewn_oak_grain.vessel_rows,
            ],
            params.hewn_oak_grain.vessel_radii_cells,
            0x67ae,
        ) * (1.0 - latewood)
            * (1.0 - knot),
        rays: anatomical_marks(
            params,
            growth,
            v,
            [
                params.hewn_oak_grain.ray_columns,
                params.hewn_oak_grain.ray_rows,
            ],
            params.hewn_oak_grain.ray_radii_cells,
            0x518d,
        ) * (1.0 - knot),
        knot,
        knot_rings,
    }
}

/// Integrate the texture footprint before differentiating height into normals.
/// Knot compression otherwise aliases the narrow latewood shoulder into dots.
pub(super) fn filtered_sample(params: &crate::TextureParameters, u: f32, v: f32) -> GrainSample {
    let mut result = GrainSample::default();
    let texel = 1.0 / params.size(super::HEWN_OAK_TEXTURE_SIZE) as f32;
    let weight = 1.0 / (GRAIN_FILTER_GRID * GRAIN_FILTER_GRID) as f32;
    for y in 0..GRAIN_FILTER_GRID {
        for x in 0..GRAIN_FILTER_GRID {
            let du = ((x as f32 + 0.5) / GRAIN_FILTER_GRID as f32 - 0.5) * texel;
            let dv = ((y as f32 + 0.5) / GRAIN_FILTER_GRID as f32 - 0.5) * texel;
            let tap = sample(params, u + du, v + dv);
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
fn anatomical_marks(
    params: &crate::TextureParameters,
    u: f32,
    v: f32,
    cells: [i32; 2],
    radius: [f32; 2],
    salt: u64,
) -> f32 {
    let x = u.rem_euclid(1.0) * cells[0] as f32;
    let y = v.rem_euclid(1.0) * cells[1] as f32;
    let mut field = 0.0_f32;
    for iy in (y.floor() as i32 - 1)..=(y.floor() as i32 + 1) {
        for ix in (x.floor() as i32 - 1)..=(x.floor() as i32 + 1) {
            let random = |seed| grid_hash(params, ix, iy, cells[0], cells[1], salt ^ seed);
            if random(0) < 1.0 - params.hewn_oak_grain.anatomical_mark_density {
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
        let params = &crate::TextureParameters::default();
        let mut partial = 0;
        let mut dark = 0;
        let count = 128 * 128;
        for y in 0..128 {
            for x in 0..128 {
                let u = (x as f32 + 0.5) / 128.0;
                let v = (y as f32 + 0.5) / 128.0;
                let authored = sample(params, u, v).dark_wood;
                assert!(authored == 0.0 || authored == 1.0);
                let coverage = filtered_sample(params, u, v).dark_wood;
                partial += usize::from(coverage > 0.0 && coverage < 1.0);
                dark += usize::from(coverage > 0.5);
            }
        }
        assert!((0.001..0.15).contains(&(partial as f32 / count as f32)));
        assert!((0.10..0.50).contains(&(dark as f32 / count as f32)));
    }

    #[test]
    fn dark_regions_cover_complete_relief_ridges_including_knot_flanks() {
        let params = &crate::TextureParameters::default();
        // Scan the actual warped field, including all three branch intersections.
        // Each connected latewood ridge must have one intrinsic color throughout;
        // earlywood must stay light. Independent color frequencies violate both.
        let mut dark_ridges = 0;
        let mut light_ridges = 0;
        for v in [
            0.5,
            KNOTS[0].center[1],
            KNOTS[1].center[1],
            KNOTS[2].center[1],
        ] {
            let mut ridge_color = None;
            for x in 0..8192 {
                let grain = sample(params, (x as f32 + 0.5) / 8192.0, v);
                if grain.latewood == 0.0 {
                    assert_eq!(grain.dark_wood, 0.0, "color escaped into earlywood");
                    ridge_color = None;
                } else if let Some(color) = ridge_color {
                    assert_eq!(grain.dark_wood, color, "color cuts across a relief ridge");
                } else {
                    ridge_color = Some(grain.dark_wood);
                    dark_ridges += usize::from(grain.dark_wood == 1.0);
                    light_ridges += usize::from(grain.dark_wood == 0.0);
                }
            }
        }
        assert!(dark_ridges > 20 && light_ridges > 20);
    }

    #[test]
    fn grain_bends_around_both_flanks_and_recovers_beyond_the_knot() {
        let params = &crate::TextureParameters::default();
        let knot = &KNOTS[0];
        let displacement = |dx: f32, dy: f32| {
            let u = knot.center[0] + dx;
            growth_coordinate(params, u, knot.center[1] + dy).0 - u
        };
        assert!(displacement(0.035, 0.0) < -0.008);
        assert!(displacement(-0.035, 0.0) > 0.008);
        assert!(displacement(0.035, 0.20).abs() < 0.001);
    }

    #[test]
    fn growth_and_fiber_fields_match_across_the_repeat() {
        let params = &crate::TextureParameters::default();
        for index in 0..512 {
            let t = (index as f32 + 0.5) / 512.0;
            for (a, b) in [
                (sample(params, 0.0, t), sample(params, 1.0, t)),
                (sample(params, t, 0.0), sample(params, t, 1.0)),
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

mod controls;
pub use controls::Parameters;

fn fiber_response(params: &crate::TextureParameters, growth: f32, v: f32, knot: f32) -> f32 {
    let fiber_coordinate = growth
        + (value_noise(
            params,
            growth,
            v,
            params.hewn_oak_grain.fiber_wander_grid[0],
            params.hewn_oak_grain.fiber_wander_grid[1],
            0x328b,
        ) - 0.5)
            * params.hewn_oak_grain.fiber_wander;
    let fiber_band =
        (fiber_coordinate * params.hewn_oak_grain.fiber_count * std::f32::consts::TAU).sin();
    let interruptions = smooth(
        ((value_noise(
            params,
            growth,
            v,
            params.hewn_oak_grain.fiber_interruption_grid[0],
            params.hewn_oak_grain.fiber_interruption_grid[1],
            0x4ce7,
        ) - params.hewn_oak_grain.fiber_interruption_threshold)
            / params.hewn_oak_grain.fiber_interruption_transition)
            .clamp(0.0, 1.0),
    );
    ((fiber_band - params.hewn_oak_grain.fiber_threshold) / params.hewn_oak_grain.fiber_transition)
        .max(0.0)
        .powi(2)
        * interruptions
        * (1.0 - knot)
}
