//! Three broad shape sections form a smooth thigh carrier. A separate measured
//! slab envelope bounds support throughout the final shaped height range.
//! Sampling uses the canonical unposed MHR frame: world Y is anatomical height;
//! X separates the left and right thighs. Animation is applied after fitting.
use crate::plate_section::PlateSection;

const SECTION_HALF_WIDTH_M: f32 = 0.025;
const ENVELOPE_STATIONS: usize = 25;
const ENVELOPE_DIRECTIONS: usize = 64;

#[derive(Clone, Copy)]
struct Section {
    center: [f32; 2],
    radii: [f32; 2],
}

impl Section {
    fn measured(points: &[[f32; 3]], y: f32) -> Self {
        let section = PlateSection::measured(points, y, SECTION_HALF_WIDTH_M);
        let mut result = Self {
            center: section.center,
            radii: [
                section.radius([1.0, 0.0]).max(section.radius([-1.0, 0.0])),
                section.radius([0.0, 1.0]).max(section.radius([0.0, -1.0])),
            ],
        };
        let factor = (0..ENVELOPE_DIRECTIONS)
            .map(|i| {
                let angle = std::f32::consts::TAU * i as f32 / ENVELOPE_DIRECTIONS as f32;
                let direction = [angle.sin(), angle.cos()];
                section.radius(direction) / result.radius(direction)
            })
            .fold(1.0, f32::max);
        result.radii = result.radii.map(|r| r * factor);
        result
    }

    fn radius(self, direction: [f32; 2]) -> f32 {
        ((direction[0] / self.radii[0]).powi(2) + (direction[1] / self.radii[1]).powi(2))
            .sqrt()
            .recip()
    }
}

pub(crate) struct TassetCarrier {
    controls: [Section; 3],
    allowance: f32,
    support: SurfaceSupport,
    heights: [f32; 2],
}

impl TassetCarrier {
    pub fn new(
        points: &[[f32; 3]],
        waist: &[[f32; 3]],
        triangles: &[[[f32; 3]; 3]],
        bottom: f32,
        top: f32,
    ) -> anyhow::Result<Self> {
        let sections = [
            Section::measured(points, bottom),
            Section::measured(points, (bottom + top) * 0.5),
            Section::measured(waist, top),
        ];
        let mut carrier = Self {
            controls: sections,
            allowance: 0.0,
            support: SurfaceSupport::new(triangles)?,
            heights: [bottom, top],
        };
        // Quadratic controls interpolate the measured middle station.
        for axis in 0..2 {
            carrier.controls[1].center[axis] = 2.0 * sections[1].center[axis]
                - (sections[0].center[axis] + sections[2].center[axis]) * 0.5;
            carrier.controls[1].radii[axis] = 2.0 * sections[1].radii[axis]
                - (sections[0].radii[axis] + sections[2].radii[axis]) * 0.5;
        }
        for station in 0..ENVELOPE_STATIONS {
            let axial = station as f32 / (ENVELOPE_STATIONS - 1) as f32;
            let measured = PlateSection::measured(
                points,
                bottom + (top - bottom) * axial,
                SECTION_HALF_WIDTH_M,
            );
            let fitted = carrier.section(axial);
            for i in 0..ENVELOPE_DIRECTIONS {
                let angle = std::f32::consts::TAU * i as f32 / ENVELOPE_DIRECTIONS as f32;
                let direction = [angle.sin(), angle.cos()];
                let r = measured.radius(direction);
                let delta: [f32; 2] = std::array::from_fn(|a| {
                    measured.center[a] + r * direction[a] - fitted.center[a]
                });
                let distance = delta[0].hypot(delta[1]);
                if distance > f32::EPSILON {
                    let ray = delta.map(|v| v / distance);
                    carrier.allowance = carrier.allowance.max(distance - fitted.radius(ray));
                }
            }
        }
        Ok(carrier)
    }

    fn section(&self, axial: f32) -> Section {
        let weights = [
            (1.0 - axial).powi(2),
            2.0 * axial * (1.0 - axial),
            axial.powi(2),
        ];
        Section {
            center: std::array::from_fn(|a| {
                (0..3)
                    .map(|i| weights[i] * self.controls[i].center[a])
                    .sum()
            }),
            radii: std::array::from_fn(|a| {
                (0..3).map(|i| weights[i] * self.controls[i].radii[a]).sum()
            }),
        }
    }

    pub fn sample(&self, height: f32, direction: [f32; 2]) -> ([f32; 2], f32) {
        let axial = (height - self.heights[0]) / (self.heights[1] - self.heights[0]);
        let section = self.section(axial);
        let formed = section.radius(direction) + self.allowance;
        let support = self.support.radius(height, section.center, direction);
        (section.center, formed.max(support))
    }

    pub fn contains_height(&self, height: f32) -> bool {
        (self.support.low..=self.support.high).contains(&height)
    }
}

/// Each node encloses both neighboring intervals, including faces with no vertex
/// at the query height. Positive interpolation cannot erase their common support.
struct SurfaceSupport {
    low: f32,
    high: f32,
    sections: Vec<PlateSection>,
}

impl SurfaceSupport {
    fn new(triangles: &[[[f32; 3]; 3]]) -> anyhow::Result<Self> {
        let low = triangles
            .iter()
            .flatten()
            .map(|p| p[1])
            .fold(f32::INFINITY, f32::min);
        let high = triangles
            .iter()
            .flatten()
            .map(|p| p[1])
            .fold(f32::NEG_INFINITY, f32::max);
        anyhow::ensure!(
            low.is_finite() && high.is_finite() && high > low,
            "tasset requires a finite anatomical support span"
        );
        let intervals = ((high - low) / SECTION_HALF_WIDTH_M).ceil() as usize;
        let step = (high - low) / intervals as f32;
        let sections = (0..=intervals)
            .map(|i| {
                PlateSection::measured_surface(triangles, low + step * i as f32, step).ok_or_else(
                    || anyhow::anyhow!("tasset anatomical support contains an empty section"),
                )
            })
            .collect::<anyhow::Result<_>>()?;
        Ok(Self {
            low,
            high,
            sections,
        })
    }

    fn radius(&self, height: f32, center: [f32; 2], direction: [f32; 2]) -> f32 {
        // Height-solver brackets can leave anatomy. Their mathematical carrier
        // continuation is permitted, but final vertices must pass contains_height.
        if !(self.low..=self.high).contains(&height) {
            return 0.0;
        }
        let coordinate =
            (height - self.low) / (self.high - self.low) * (self.sections.len() - 1) as f32;
        let first = (coordinate.floor() as usize).min(self.sections.len() - 2);
        let fraction = coordinate - first as f32;
        self.sections[first].radius_from(center, direction) * (1.0 - fraction)
            + self.sections[first + 1].radius_from(center, direction) * fraction
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use adventuresim_armor_model::*;

    #[test]
    fn shaped_tasset_uses_body_support_beyond_both_nominal_ends() {
        let radius =
            |y: f32| 0.07 + 0.5 * (y - 0.6) + 0.02 * (1.0 - ((y - 0.90) / 0.04).abs()).max(0.0);
        let mut points = Vec::new();
        for row in 0..=80 {
            let y = 0.6 + row as f32 * 0.005;
            for column in 0..64 {
                let angle = column as f32 * std::f32::consts::TAU / 64.0;
                points.push([radius(y) * angle.sin(), y, radius(y) * angle.cos()]);
            }
        }
        let triangles: Vec<_> = (0..80)
            .flat_map(|row| {
                (0..64).flat_map(move |column| {
                    let a = row * 64 + column;
                    let b = row * 64 + (column + 1) % 64;
                    [[a, b, a + 64], [b, b + 64, a + 64]]
                })
            })
            .map(|face| face.map(|i| points[i]))
            .collect();
        let carrier = TassetCarrier::new(&points, &points, &triangles, 0.65, 0.85).unwrap();
        let mut design = GarmentArmorDesign::new(GarmentArmorKind::Tassets);
        design.lame_count = 8;
        design.wall_thickness = Millimeters(1);
        design.fluting = None;
        design.plate_shape = GarmentPlateShape::WrappedTassets(WrappedTassetDesign {
            upper_edge_slope: Permille(400),
            inner_cutaway: Millimeters(0),
            hem_rounding: Millimeters(0),
            section_break: 4,
            section_gap: Millimeters(8),
            ..Default::default()
        });
        let mesh = generate_wrapped_tasset(
            &design,
            TassetSide::Left,
            TassetSpan::new(0.65, 0.85).unwrap(),
            |angle, height| {
                let direction = [angle.sin(), angle.cos()];
                let (center, radius) = carrier.sample(height, direction);
                std::array::from_fn(|axis| center[axis] + (radius + 0.006) * direction[axis])
            },
        )
        .unwrap();
        assert!(mesh.positions.iter().any(|p| p[1] > 0.89));
        assert!(mesh.positions.iter().any(|p| p[1] < 0.645));
        for p in &mesh.positions {
            assert!(carrier.contains_height(p[1]));
            assert!(
                p[0].hypot(p[2]) >= radius(p[1]) + 0.001 - 1e-6,
                "final shaped shell entered the extended body support: {p:?}"
            );
        }
        assert!(!carrier.contains_height(1.1));
    }
}
