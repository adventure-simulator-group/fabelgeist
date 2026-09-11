//! A directional cage seats both bibs while preserving their lateral outlines.
//! Directional sections preserve lateral trim independently of render tessellation.
use std::f32::consts::TAU;

const ROWS: usize = 17;
const COLUMNS: usize = 129;

pub(super) struct BibFit([[f32; COLUMNS]; ROWS]);

impl Default for BibFit {
    fn default() -> Self {
        Self([[0.0; COLUMNS]; ROWS])
    }
}

impl BibFit {
    pub fn measure(
        point: impl Fn(f32, f32) -> [f32; 3],
        body: &[[f32; 3]],
        faces: &[[u32; 3]],
        padding: f32,
    ) -> Self {
        let measured: [[f32; COLUMNS]; ROWS] = std::array::from_fn(|row| {
            std::array::from_fn(|column| {
                if row == 0 {
                    return 0.0;
                }
                let t = row as f32 / (ROWS - 1) as f32;
                let angle = TAU * column as f32 / (COLUMNS - 1) as f32;
                let p = point(t, angle);
                let direction = direction(angle);
                let query = [p[0], p[1] * direction[2] - p[2] * direction[1]];
                let upper_height = point(0.0, angle)[1];
                let origin = p[1] * direction[1] + p[2] * direction[2];
                faces
                    .iter()
                    .filter_map(|face| {
                        let triangle = face.map(|i| {
                            let b = body[i as usize];
                            [
                                b[0],
                                b[1] * direction[2] - b[2] * direction[1],
                                b[1] * direction[1] + b[2] * direction[2],
                            ]
                        });
                        depth_at(query, triangle).filter(|depth| {
                            let hit_height = query[1] * direction[2] + depth * direction[1];
                            hit_height <= upper_height
                        })
                    })
                    .reduce(f32::max)
                    .map_or(0.0, |depth| {
                        let adjustment = depth + padding - origin;
                        if adjustment < 0.0 {
                            adjustment * t
                        } else {
                            adjustment
                        }
                    })
            })
        });
        Self(smooth_sections(measured))
    }

    pub fn offset(&self, t: f32, angle: f32) -> [f32; 3] {
        let u = angle.rem_euclid(TAU) / TAU;
        if !(0.0..=1.0).contains(&u) {
            return [0.0; 3];
        }
        let row = t.clamp(0.0, 1.0) * (ROWS - 1) as f32;
        let column = u * (COLUMNS - 1) as f32;
        let values = std::array::from_fn(|i| {
            let r = (row.floor() as isize + i as isize - 1).clamp(0, (ROWS - 1) as isize) as usize;
            cubic(
                std::array::from_fn(|j| {
                    let c = (column.floor() as isize + j as isize - 1)
                        .clamp(0, (COLUMNS - 1) as isize) as usize;
                    self.0[r][c]
                }),
                column.fract(),
            )
        });
        let distance = cubic(values, row.fract());
        direction(angle).map(|axis| axis * distance)
    }
}

/// Smooth anatomical samples while preserving the collar seam and angular periodicity.
fn smooth_sections(measured: [[f32; COLUMNS]; ROWS]) -> [[f32; COLUMNS]; ROWS] {
    std::array::from_fn(|row| {
        std::array::from_fn(|column| {
            if row == 0 {
                return 0.0;
            }
            let mut value = 0.0;
            for (dr, wr) in [(-1, 0.25), (0, 0.5), (1, 0.25)] {
                let r = (row as isize + dr).clamp(0, (ROWS - 1) as isize) as usize;
                for (dc, wc) in [(-1, 0.25), (0, 0.5), (1, 0.25)] {
                    let c = (column as isize + dc).rem_euclid((COLUMNS - 1) as isize) as usize;
                    value += measured[r][c] * wr * wc;
                }
            }
            value
        })
    })
}

fn cubic(p: [f32; 4], t: f32) -> f32 {
    let delta = p[2] - p[1];
    if delta.abs() < 1e-8 {
        return p[1];
    }
    // Limit Hermite slopes to preserve the control interval's monotonicity.
    let mut a = ((p[2] - p[0]) * 0.5 / delta).max(0.0);
    let mut b = ((p[3] - p[1]) * 0.5 / delta).max(0.0);
    let magnitude = a.hypot(b);
    if magnitude > 3.0 {
        a *= 3.0 / magnitude;
        b *= 3.0 / magnitude;
    }
    let t2 = t * t;
    let t3 = t2 * t;
    (2.0 * t3 - 3.0 * t2 + 1.0) * p[1]
        + (t3 - 2.0 * t2 + t) * a * delta
        + (-2.0 * t3 + 3.0 * t2) * p[2]
        + (t3 - t2) * b * delta
}

/// Lift over the trapezius at the sides, transitioning into anterior/posterior depth.
/// This preserves the authored lateral trim instead of inflating it to the shoulder.
fn direction(angle: f32) -> [f32; 3] {
    let up = angle.sin().powi(2);
    let back = angle.cos();
    let length = up.hypot(back);
    [0.0, up / length, back / length]
}

/// Intersect a depth ray after rotating the triangle into its directional section.
fn depth_at(p: [f32; 2], [a, b, c]: [[f32; 3]; 3]) -> Option<f32> {
    let determinant = (b[1] - c[1]) * (a[0] - c[0]) + (c[0] - b[0]) * (a[1] - c[1]);
    if determinant.abs() < 1e-10 {
        return None;
    }
    let u = ((b[1] - c[1]) * (p[0] - c[0]) + (c[0] - b[0]) * (p[1] - c[1])) / determinant;
    let v = ((c[1] - a[1]) * (p[0] - c[0]) + (a[0] - c[0]) * (p[1] - c[1])) / determinant;
    (u >= 0.0 && v >= 0.0 && u + v <= 1.0).then_some(u * a[2] + v * b[2] + (1.0 - u - v) * c[2])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn posterior_ray_interpolates_depth_and_rejects_outside_points() {
        let triangle = [[0.0, 0.0, -0.02], [0.1, 0.0, -0.04], [0.0, 0.1, -0.06]];
        assert!((depth_at([0.025, 0.025], triangle).unwrap() + 0.035).abs() < 1e-6);
        assert!(depth_at([0.1, 0.1], triangle).is_none());
        assert!(depth_at([0.0; 2], [[0.0; 3]; 3]).is_none());
    }
    #[test]
    fn shoulder_projection_ignores_the_head_above_the_collar() {
        use std::f32::consts::FRAC_PI_2;
        let carrier = |t: f32, angle: f32| {
            [
                (0.05 + 0.05 * t) * angle.sin(),
                0.1 * (1.0 - t),
                0.05 * angle.cos(),
            ]
        };
        let mut points = Vec::new();
        for height in [0.0, 0.2] {
            points.extend([
                [-1.0, height, -1.0],
                [1.0, height, -1.0],
                [1.0, height, 1.0],
                [-1.0, height, 1.0],
            ]);
        }
        let shoulder_faces = [[0, 1, 2], [0, 2, 3]];
        let all_faces = [[0, 1, 2], [0, 2, 3], [4, 5, 6], [4, 6, 7]];
        let shoulder = BibFit::measure(carrier, &points, &shoulder_faces, 0.008);
        let with_head = BibFit::measure(carrier, &points, &all_faces, 0.008);
        for t in [0.25, 0.5, 0.75] {
            let expected = shoulder.offset(t, FRAC_PI_2);
            assert_eq!(expected, with_head.offset(t, FRAC_PI_2));
            assert_eq!(
                expected[0], 0.0,
                "fitting must preserve the lateral outline"
            );
            assert!(
                expected[1] < 0.0,
                "the shoulder is below the authored carrier"
            );
        }
        assert_eq!(with_head.offset(0.0, FRAC_PI_2), [0.0; 3]);
    }
}
