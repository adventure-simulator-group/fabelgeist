//! Convex anatomical sections avoid inflating an ellipse to its farthest corner.
#[derive(Clone)]
pub(crate) struct PlateSection {
    pub center: [f32; 2],
    boundary: Vec<[f32; 2]>,
    radii: [f32; 64],
}

fn cross(a: [f32; 2], b: [f32; 2]) -> f32 {
    a[0] * b[1] - a[1] * b[0]
}
fn sub(a: [f32; 2], b: [f32; 2]) -> [f32; 2] {
    [a[0] - b[0], a[1] - b[1]]
}

impl PlateSection {
    /// Enclose complete triangle fragments through the axial measurement band.
    /// Vertex-only bands can miss an entire side of a stretched limb triangle.
    pub fn measured_surface(triangles: &[[[f32; 3]; 3]], y: f32, half_width: f32) -> Option<Self> {
        let bounds = [y - half_width, y + half_width];
        let mut samples = Vec::new();
        for triangle in triangles {
            for corner in 0..3 {
                let a = triangle[corner];
                let b = triangle[(corner + 1) % 3];
                if (bounds[0]..=bounds[1]).contains(&a[1]) {
                    samples.push([a[0], 0.0, a[2]]);
                }
                for height in bounds {
                    if (a[1] < height && b[1] > height) || (b[1] < height && a[1] > height) {
                        let t = (height - a[1]) / (b[1] - a[1]);
                        samples.push([a[0] + (b[0] - a[0]) * t, 0.0, a[2] + (b[2] - a[2]) * t]);
                    }
                }
            }
        }
        if samples.len() < 3 {
            return None;
        }
        let mut section = Self::measured(&samples, 0.0, half_width);
        // Area centroid is invariant to extra collinear vertices introduced by
        // slicing/subdivision. Averaging boundary vertices biases the center
        // toward whichever side happens to have more triangle edges.
        let mut area = 0.0_f64;
        let mut moment = [0.0_f64; 2];
        for i in 0..section.boundary.len() {
            let a = section.boundary[i].map(f64::from);
            let b = section.boundary[(i + 1) % section.boundary.len()].map(f64::from);
            let signed = a[0] * b[1] - a[1] * b[0];
            area += signed;
            for axis in 0..2 {
                moment[axis] += (a[axis] + b[axis]) * signed;
            }
        }
        if !area.is_finite() || area.abs() <= f64::EPSILON {
            return None;
        }
        section.center = moment.map(|sum| (sum / (3.0 * area)) as f32);
        section.radii = std::array::from_fn(|i| {
            let angle = i as f32 / 64.0 * std::f32::consts::TAU;
            section.raw_radius([angle.cos(), angle.sin()])
        });
        Some(section)
    }

    pub fn measured(points: &[[f32; 3]], y: f32, half_width: f32) -> Self {
        const MINIMUM_SAMPLES: usize = 16;
        let mut nearest = points.iter().collect::<Vec<_>>();
        nearest.sort_by(|a, b| (a[1] - y).abs().total_cmp(&(b[1] - y).abs()));
        let count = nearest
            .iter()
            .take_while(|p| (p[1] - y).abs() < half_width)
            .count()
            .max(MINIMUM_SAMPLES)
            .min(nearest.len());
        let mut section = nearest[..count]
            .iter()
            .map(|p| [p[0], p[2]])
            .collect::<Vec<_>>();
        section.sort_by(|a, b| a[0].total_cmp(&b[0]).then(a[1].total_cmp(&b[1])));
        section.dedup();
        let mut boundary = Vec::new();
        for indices in [
            (0..section.len()).collect::<Vec<_>>(),
            (0..section.len()).rev().collect(),
        ] {
            let start = boundary.len();
            for index in indices {
                let p = section[index];
                while boundary.len() >= start + 2 {
                    let last = boundary.len() - 1;
                    if cross(
                        sub(boundary[last], boundary[last - 1]),
                        sub(p, boundary[last]),
                    ) > 0.0
                    {
                        break;
                    }
                    boundary.pop();
                }
                boundary.push(p);
            }
            boundary.pop();
        }
        let center = std::array::from_fn(|axis| {
            boundary.iter().map(|p| p[axis]).sum::<f32>() / boundary.len() as f32
        });
        let mut section = Self {
            center,
            boundary,
            radii: [0.0; 64],
        };
        section.radii = std::array::from_fn(|i| {
            let angle = i as f32 / 64.0 * std::f32::consts::TAU;
            section.raw_radius([angle.cos(), angle.sin()])
        });
        section
    }

    pub fn radius(&self, direction: [f32; 2]) -> f32 {
        let coordinate = direction[1]
            .atan2(direction[0])
            .rem_euclid(std::f32::consts::TAU)
            / std::f32::consts::TAU
            * 64.0;
        let index = coordinate.floor() as isize;
        let t = coordinate.fract();
        let weights = [
            (1.0 - t).powi(3),
            3.0 * t.powi(3) - 6.0 * t * t + 4.0,
            -3.0 * t.powi(3) + 3.0 * t * t + 3.0 * t + 1.0,
            t.powi(3),
        ];
        weights
            .iter()
            .enumerate()
            .map(|(offset, w)| {
                w * self.radii[(index + offset as isize - 1).rem_euclid(64) as usize] / 6.0
            })
            .sum()
    }

    fn raw_radius(&self, direction: [f32; 2]) -> f32 {
        self.radius_from(self.center, direction)
    }

    /// Exact hull-ray exit from a supplied carrier center, without angular smoothing.
    pub fn radius_from(&self, center: [f32; 2], direction: [f32; 2]) -> f32 {
        const MINIMUM_RADIUS_M: f32 = 0.012;
        let mut radius = MINIMUM_RADIUS_M;
        for i in 0..self.boundary.len() {
            let a = sub(self.boundary[i], center);
            let edge = sub(
                self.boundary[(i + 1) % self.boundary.len()],
                self.boundary[i],
            );
            let determinant = cross(direction, edge);
            if determinant.abs() < 1e-8 {
                continue;
            }
            let distance = cross(a, edge) / determinant;
            let along = cross(a, direction) / determinant;
            if (0.0..=1.0).contains(&along) && distance > radius {
                radius = distance;
            }
        }
        radius
    }
}

#[cfg(test)]
mod tests {
    use super::PlateSection;
    #[test]
    fn axial_surface_sections_retain_triangles_whose_vertices_miss_the_band() {
        let mut points = Vec::new();
        for y in [-0.025, 0.025] {
            for column in 0..16 {
                let angle = column as f32 * std::f32::consts::TAU / 16.0;
                points.push([0.060 * angle.cos(), y, 0.045 * angle.sin()]);
            }
        }
        let triangles: Vec<_> = (0..16)
            .flat_map(|i| {
                let next = (i + 1) % 16;
                [
                    [points[i], points[next], points[i + 16]],
                    [points[next], points[next + 16], points[i + 16]],
                ]
            })
            .collect();
        // More densely sampled inner-facing regions must not erase a large
        // spanning face simply because its vertices lie outside the band.
        points.extend((0..16).map(|i| {
            let angle = i as f32 * std::f32::consts::TAU / 16.0;
            [0.025 * angle.cos(), 0.0, 0.025 * angle.sin()]
        }));
        let old = PlateSection::measured(&points, 0.0, 0.012);
        let surface = PlateSection::measured_surface(&triangles, 0.0, 0.012).unwrap();
        assert!(old.radius([1.0, 0.0]) < 0.03);
        assert!(surface.radius([1.0, 0.0]) > 0.058);
        assert!(surface.radius([0.0, 1.0]) > 0.043);
        let divided: Vec<_> = triangles
            .iter()
            .flat_map(|[a, b, c]| {
                let middle = std::array::from_fn(|axis| (a[axis] + b[axis]) * 0.5);
                [[*a, middle, *c], [middle, *b, *c]]
            })
            .collect();
        let resampled = PlateSection::measured_surface(&divided, 0.0, 0.012).unwrap();
        for angle in [0.0_f32, 0.7, 1.5, 2.3, 3.7] {
            let direction = [angle.cos(), angle.sin()];
            assert!((surface.radius(direction) - resampled.radius(direction)).abs() < 1e-6);
        }
        assert!(PlateSection::measured_surface(&triangles, 0.1, 0.012).is_none());
    }
    #[test]
    fn rectangular_section_keeps_flat_faces_without_corner_inflation() {
        let p = PlateSection::measured(
            &[
                [-0.08, 0.0, -0.04],
                [0.08, 0.0, -0.04],
                [0.08, 0.0, 0.04],
                [-0.08, 0.0, 0.04],
            ],
            0.0,
            0.01,
        );
        assert!((p.radius([1.0, 0.0]) - 0.08).abs() < 0.001);
        assert!((p.radius([0.0, 1.0]) - 0.04).abs() < 0.001);
        assert!((p.radius([0.8, 0.6]) - 0.04 / 0.6).abs() < 0.001);
    }
}
