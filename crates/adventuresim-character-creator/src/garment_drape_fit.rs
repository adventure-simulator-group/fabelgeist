//! Convex cross sections for hanging cloth and fitted waist plates.
//! Fixed angular control rays bound the samples without expanding both axes
//! because a single oblique point lies outside an inscribed ellipse.
use super::local_point;
use adventuresim_armor_model::{
    GARMENT_LAME_SPACING_GAUGES, GARMENT_RING_SEGMENTS, GarmentArmorDesign, GarmentArmorKind,
    PartFrame, PartMesh,
};
use std::f32::consts::TAU;

const STATIONS: usize = 17;
const SECTION_HALF_WIDTH_M: f32 = 0.018;
const MINIMUM_SECTION_POINTS: usize = 12;
const PROFILE_SMOOTHING_PASSES: usize = 8;
const RAISED_SECTION_SOLVER_STEPS: usize = 24;

pub(crate) struct DrapeCage {
    radii: Vec<[f32; GARMENT_RING_SEGMENTS]>,
    heights: [f32; 2],
    maximum_radius: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fitted_plates_follow_raised_sections_without_hanging_their_bulk_below() {
        let frame = PartFrame {
            origin: [0.0; 3],
            axes: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
            half_extents: [0.1; 3],
        };
        let points = (0..=40)
            .flat_map(|row| {
                let y = -0.1 + row as f32 * 0.01;
                let radius = 0.1 + (y + 0.1) * 0.1;
                (0..64).map(move |column| {
                    let angle = TAU * column as f32 / 64.0;
                    [radius * angle.sin(), y, radius * angle.cos()]
                })
            })
            .collect::<Vec<_>>();
        let fitted = DrapeCage::new(&points, &frame, [-0.1, 0.3], SkirtProfile::FittedPlate);
        let hanging = DrapeCage::new(&points, &frame, [-0.1, 0.3], SkirtProfile::Hanging);
        // The raised edge extends above the nominal frame's final station.
        assert!(fitted.radius_at_angle(0.25, 0.0) > 0.13);
        assert!(fitted.radius_at_angle(-0.1, 0.0) < 0.11);
        assert!(hanging.radius_at_angle(-0.1, 0.0) > 0.135);
        // On this expanding cone, lifting the same edge increases the radius
        // needed to enclose it. A lookup at the original height is too small.
        let flat = fitted.radius_at_slope(-0.02, std::f32::consts::FRAC_PI_2, 0.0, 1.0, 0.004);
        let raised = fitted.radius_at_slope(-0.02, std::f32::consts::FRAC_PI_2, 0.3, 1.0, 0.004);
        assert!(raised > flat + 0.003);
        assert!((0.115..0.120).contains(&raised));
    }

    #[test]
    fn plate_smoothing_encloses_a_local_bulge_without_inflating_its_attachment() {
        let frame = PartFrame {
            origin: [0.0; 3],
            axes: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
            half_extents: [0.3; 3],
        };
        let points = (0..=60)
            .flat_map(|row| {
                let y = -0.3 + row as f32 * 0.01;
                let radius = 0.1 + 0.04 * (-(y / 0.04).powi(2)).exp();
                (0..48).map(move |column| {
                    let angle = TAU * column as f32 / 48.0;
                    [radius * angle.sin(), y, radius * angle.cos()]
                })
            })
            .collect::<Vec<_>>();
        let plate = DrapeCage::new(&points, &frame, [-0.3, 0.3], SkirtProfile::FittedPlate);
        for angle in [0.0, std::f32::consts::FRAC_PI_2, std::f32::consts::PI] {
            assert!(plate.radius_at_angle(0.0, angle) >= 0.1399);
            assert!(
                (plate.radius_at_angle(0.3, angle) - 0.1).abs() < 1e-5,
                "a lower support bulge must not expand the upper attachment"
            );
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) enum SkirtProfile {
    Hanging,
    FittedPlate,
}

impl DrapeCage {
    pub(super) fn for_skirt(
        points: &[[f32; 3]],
        frame: &PartFrame,
        mesh: &PartMesh,
        design: &GarmentArmorDesign,
        gap: f32,
        chevron: f32,
    ) -> Self {
        if matches!(
            design.kind,
            GarmentArmorKind::MailSkirt | GarmentArmorKind::PaddedSkirt
        ) {
            return Self::new(
                points,
                frame,
                [-frame.half_extents[1], frame.half_extents[1]],
                SkirtProfile::Hanging,
            );
        }
        // Bound the actual raised plate domain, including the greatest
        // possible chevron rise after its supporting envelope is measured.
        let mut heights =
            mesh.positions
                .iter()
                .fold([f32::INFINITY, f32::NEG_INFINITY], |range, p| {
                    let y = local_point(frame, *p)[1];
                    [range[0].min(y), range[1].max(y)]
                });
        let radial_bound = points
            .iter()
            .map(|p| {
                let p = local_point(frame, *p);
                p[0].hypot(p[2])
            })
            .fold(0.0, f32::max)
            * (1.0 + design.flare.unit())
            + gap
            + design.wall_thickness.metres() * GARMENT_LAME_SPACING_GAUGES;
        heights[1] += chevron * radial_bound;
        Self::new(points, frame, heights, SkirtProfile::FittedPlate)
    }

    pub(crate) fn new(
        points: &[[f32; 3]],
        frame: &PartFrame,
        heights: [f32; 2],
        profile: SkirtProfile,
    ) -> Self {
        let points = points
            .iter()
            .map(|p| local_point(frame, *p))
            .collect::<Vec<_>>();
        let mut radii = (0..STATIONS)
            .map(|row| {
                let y = heights[0] + (heights[1] - heights[0]) * row as f32 / (STATIONS - 1) as f32;
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
        if matches!(profile, SkirtProfile::Hanging) {
            for row in (0..STATIONS - 1).rev() {
                let upper = radii[row + 1];
                for (radius, above) in radii[row].iter_mut().zip(upper) {
                    *radius = radius.max(above);
                }
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
                    if matches!(profile, SkirtProfile::FittedPlate) {
                        // Project each station onto its own enclosure bound.
                        // Moving a shortfall to every station would turn a
                        // local breast/hip transition into an oversized waist.
                        radii[row][col] = radii[row][col].max(required[row][col]);
                    }
                }
            }
        }
        if matches!(profile, SkirtProfile::Hanging) {
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
        }
        let maximum_radius = radii.iter().flatten().copied().fold(0.0, f32::max);
        Self {
            radii,
            heights,
            maximum_radius,
        }
    }

    pub(super) fn radius_at_angle(&self, y: f32, angle: f32) -> f32 {
        let column = angle.rem_euclid(TAU) / TAU * GARMENT_RING_SEGMENTS as f32;
        let lower = column.floor() as usize % GARMENT_RING_SEGMENTS;
        let fraction = column.fract();
        self.radius(y, lower) * (1.0 - fraction)
            + self.radius(y, (lower + 1) % GARMENT_RING_SEGMENTS) * fraction
    }

    /// Solve the chevron's height and radius together, using support at the
    /// final raised height rather than at the underlying horizontal waist.
    pub(crate) fn radius_at_slope(
        &self,
        base_height: f32,
        angle: f32,
        slope: f32,
        scale: f32,
        offset: f32,
    ) -> f32 {
        if slope == 0.0 {
            return self.radius_at_angle(base_height, angle) * scale + offset;
        }
        let mut bounds = [0.0, self.maximum_radius * scale + offset];
        for _ in 0..RAISED_SECTION_SOLVER_STEPS {
            let radius = (bounds[0] + bounds[1]) * 0.5;
            let height = base_height + slope * (radius * angle.sin()).abs();
            let required = self.radius_at_angle(height, angle) * scale + offset;
            if radius < required {
                bounds[0] = radius;
            } else {
                bounds[1] = radius;
            }
        }
        bounds[1]
    }

    fn radius(&self, y: f32, col: usize) -> f32 {
        let station = ((y - self.heights[0]) / (self.heights[1] - self.heights[0])
            * (STATIONS - 1) as f32)
            .clamp(0.0, (STATIONS - 1) as f32);
        let lower = (station.floor() as usize).min(STATIONS - 2);
        let t = station - lower as f32;
        self.radii[lower][col] * (1.0 - t) + self.radii[lower + 1][col] * t
    }
}
