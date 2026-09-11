//! Convex anatomical sections avoid inflating an ellipse to its farthest corner.
#[derive(Clone)]
pub(super) struct PlateSection {
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
        const MINIMUM_RADIUS_M: f32 = 0.012;
        let mut radius = MINIMUM_RADIUS_M;
        for i in 0..self.boundary.len() {
            let a = sub(self.boundary[i], self.center);
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
