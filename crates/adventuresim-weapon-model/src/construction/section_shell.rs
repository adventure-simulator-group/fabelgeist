//! Hollow solids between corresponding polygonal section rings.
use super::*;
pub(crate) enum LoftEnd {
    Open,
    Closed,
}
impl Solid {
    pub(crate) fn section_shell(
        inner: &[Vec<Point>],
        outer: &[Vec<Point>],
        end: LoftEnd,
    ) -> Result<Self, String> {
        let count = inner.first().map_or(0, Vec::len);
        if inner.len() < 2
            || outer.len() < 2
            || count < 3
            || inner.iter().chain(outer).any(|ring| ring.len() != count)
        {
            return Err("shell requires matching closed section rings".into());
        }
        construction_budget(((inner.len() + outer.len()) * count * 2) as f64)?;
        let mut solid = Self::default();
        for row in 0..inner.len().max(outer.len()) - 1 {
            for side in 0..count {
                let next = (side + 1) % count;
                if row + 1 < outer.len() {
                    solid.quad(
                        outer[row][side],
                        outer[row + 1][side],
                        outer[row + 1][next],
                        outer[row][next],
                        1,
                    );
                }
                if row + 1 < inner.len() {
                    solid.quad(
                        inner[row][side],
                        inner[row][next],
                        inner[row + 1][next],
                        inner[row + 1][side],
                        2,
                    );
                }
            }
        }
        for side in 0..count {
            let next = (side + 1) % count;
            solid.quad(
                outer[0][side],
                outer[0][next],
                inner[0][next],
                inner[0][side],
                0,
            );
        }
        let last = inner.len() - 1;
        let outer_last = outer.len() - 1;
        match end {
            LoftEnd::Open => {
                for side in 0..count {
                    let next = (side + 1) % count;
                    solid.quad(
                        outer[outer_last][side],
                        inner[last][side],
                        inner[last][next],
                        outer[outer_last][next],
                        0,
                    );
                }
            }
            LoftEnd::Closed => {
                for (ring, outward) in [(&outer[outer_last], true), (&inner[last], false)] {
                    let points: Vec<_> = ring.iter().map(|p| [p[0], p[2]]).collect();
                    let region = Region::triangulate(&points, false)?;
                    for [a, b, c] in region.triangles {
                        let point =
                            |i: usize| [region.points[i][0], ring[0][1], region.points[i][1]];
                        if outward {
                            solid.triangle(point(a), point(c), point(b), 0);
                        } else {
                            solid.triangle(point(a), point(b), point(c), 0);
                        }
                    }
                }
            }
        }
        Ok(solid.positive())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn shell_volume_excludes_the_cavity_and_retains_a_distal_wall() {
        for sides in [8, 16, 32] {
            for wall in [0.001, 0.003] {
                let length = 0.3;
                let x = 0.04;
                let z = 0.008;
                let ring = |y, shrink| {
                    (0..sides)
                        .map(|i| {
                            let angle = i as f64 / sides as f64 * std::f64::consts::TAU;
                            [(x - shrink) * angle.cos(), y, (z - shrink) * angle.sin()]
                        })
                        .collect()
                };
                let area = sides as f64 / 2.0 * (std::f64::consts::TAU / sides as f64).sin();
                for end in [LoftEnd::Open, LoftEnd::Closed] {
                    let inner_length = length
                        - if matches!(end, LoftEnd::Closed) {
                            wall
                        } else {
                            0.0
                        };
                    let solid = Solid::section_shell(
                        &[ring(0.0, wall), ring(inner_length, wall)],
                        &[ring(0.0, 0.0), ring(length, 0.0)],
                        end,
                    )
                    .unwrap();
                    let expected = area * (x * z * length - (x - wall) * (z - wall) * inner_length);
                    assert!((solid.volume() - expected).abs() < 1e-12);
                }
            }
        }
    }
}
