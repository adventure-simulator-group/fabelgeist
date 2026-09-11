//! Convex cross sections for cloth hanging around the pelvis and worn leggings.
//! Fixed angular control rays bound the samples without expanding both axes
//! because a single oblique point lies outside an inscribed ellipse.
use super::local_point;
use adventuresim_armor_model::{GARMENT_RING_SEGMENTS, PartFrame};
use std::f32::consts::TAU;

const STATIONS: usize = 17;
const SECTION_HALF_WIDTH_M: f32 = 0.018;
const MINIMUM_SECTION_POINTS: usize = 12;
const PROFILE_SMOOTHING_PASSES: usize = 8;

pub(super) struct DrapeCage {
    radii: Vec<[f32; GARMENT_RING_SEGMENTS]>,
    half_height: f32,
}

impl DrapeCage {
    pub(super) fn new(points: &[[f32; 3]], frame: &PartFrame) -> Self {
        let points = points
            .iter()
            .map(|p| local_point(frame, *p))
            .collect::<Vec<_>>();
        let mut radii = (0..STATIONS)
            .map(|row| {
                let y = frame.half_extents[1] * (2.0 * row as f32 / (STATIONS - 1) as f32 - 1.0);
                let mut nearest = points.iter().collect::<Vec<_>>();
                nearest.sort_unstable_by(|a, b| (a[1] - y).abs().total_cmp(&(b[1] - y).abs()));
                let count = nearest
                    .iter()
                    .take_while(|p| (p[1] - y).abs() < SECTION_HALF_WIDTH_M)
                    .count()
                    .max(MINIMUM_SECTION_POINTS)
                    .min(nearest.len());
                let section = &nearest[..count];
                let supports: [f32; GARMENT_RING_SEGMENTS] = std::array::from_fn(|col| {
                    let angle = TAU * col as f32 / GARMENT_RING_SEGMENTS as f32;
                    section
                        .iter()
                        .map(|p| p[0] * angle.sin() + p[2] * angle.cos())
                        .fold(0.0, f32::max)
                });
                std::array::from_fn(|col| {
                    let angle = TAU * col as f32 / GARMENT_RING_SEGMENTS as f32;
                    supports
                        .iter()
                        .enumerate()
                        .filter_map(|(normal, h)| {
                            let normal_angle = TAU * normal as f32 / GARMENT_RING_SEGMENTS as f32;
                            let facing = (angle - normal_angle).cos();
                            (facing > f32::EPSILON).then_some(h / facing)
                        })
                        .fold(f32::INFINITY, f32::min)
                })
            })
            .collect::<Vec<[f32; GARMENT_RING_SEGMENTS]>>();
        // The hem hangs beyond the hip rather than tightening around the legs.
        for row in (0..STATIONS - 1).rev() {
            let upper = radii[row + 1];
            for (radius, above) in radii[row].iter_mut().zip(upper) {
                *radius = radius.max(above);
            }
        }
        let required = radii.clone();
        for _ in 0..PROFILE_SMOOTHING_PASSES {
            let previous = radii.clone();
            for row in 1..STATIONS - 1 {
                for col in 0..GARMENT_RING_SEGMENTS {
                    radii[row][col] = (previous[row - 1][col]
                        + 2.0 * previous[row][col]
                        + previous[row + 1][col])
                        * 0.25;
                }
            }
        }
        for col in 0..GARMENT_RING_SEGMENTS {
            let shortfall = required
                .iter()
                .zip(&radii)
                .map(|(a, b)| a[col] - b[col])
                .fold(0.0, f32::max);
            for row in &mut radii {
                row[col] += shortfall;
            }
        }
        Self {
            radii,
            half_height: frame.half_extents[1],
        }
    }

    pub(super) fn radius_at_angle(&self, y: f32, angle: f32) -> f32 {
        let column = angle.rem_euclid(TAU) / TAU * GARMENT_RING_SEGMENTS as f32;
        let lower = column.floor() as usize % GARMENT_RING_SEGMENTS;
        let fraction = column.fract();
        self.radius(y, lower) * (1.0 - fraction)
            + self.radius(y, (lower + 1) % GARMENT_RING_SEGMENTS) * fraction
    }

    fn radius(&self, y: f32, col: usize) -> f32 {
        let station = ((y / self.half_height + 1.0) * 0.5 * (STATIONS - 1) as f32)
            .clamp(0.0, (STATIONS - 1) as f32);
        let lower = (station.floor() as usize).min(STATIONS - 2);
        let t = station - lower as f32;
        self.radii[lower][col] * (1.0 - t) + self.radii[lower + 1][col] * t
    }
}
