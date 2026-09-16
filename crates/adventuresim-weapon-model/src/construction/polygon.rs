//! Conforming triangulation of simple manufactured plate outlines.
use super::PlanarPoint;
use std::collections::{BTreeMap, BTreeSet};

// Keep equivalent diagonals in stable input order across libm implementations.
pub(super) const TRIANGLE_QUALITY_TIE_TOLERANCE: f64 = 1e-12;

#[derive(Clone, Debug)]
pub(crate) struct Region {
    pub(crate) points: Vec<PlanarPoint>,
    pub(crate) triangles: Vec<[usize; 3]>,
    pub(crate) boundary: Vec<usize>,
}

fn area2(a: PlanarPoint, b: PlanarPoint, c: PlanarPoint) -> f64 {
    (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0])
}
fn distance2(a: PlanarPoint, b: PlanarPoint) -> f64 {
    (a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2)
}
pub(super) fn edge(a: usize, b: usize) -> (usize, usize) {
    (a.min(b), a.max(b))
}
pub(crate) fn signed_area(points: &[PlanarPoint]) -> f64 {
    (0..points.len())
        .map(|i| {
            let a = points[i];
            let b = points[(i + 1) % points.len()];
            a[0] * b[1] - b[0] * a[1]
        })
        .sum::<f64>()
        / 2.0
}
pub(super) fn shape(points: &[PlanarPoint], [a, b, c]: [usize; 3]) -> f64 {
    let [a, b, c] = [points[a], points[b], points[c]];
    area2(a, b, c).abs() / distance2(a, b).max(distance2(b, c)).max(distance2(c, a))
}
fn contains(p: PlanarPoint, a: PlanarPoint, b: PlanarPoint, c: PlanarPoint) -> bool {
    let signs = [area2(p, a, b), area2(p, b, c), area2(p, c, a)];
    !(signs.iter().any(|&v| v < 0.0) && signs.iter().any(|&v| v > 0.0))
}

impl Region {
    pub(crate) fn triangulate(
        input: &[PlanarPoint],
        preserve_boundary: bool,
    ) -> Result<Self, String> {
        let OutlineTolerance {
            minimum_distance,
            minimum_area,
        } = checked_outline(input)?;
        let mut points: Vec<_> = input
            .iter()
            .enumerate()
            .filter(|&(i, p)| i == 0 || distance2(*p, input[i - 1]).sqrt() > minimum_distance)
            .map(|(_, p)| *p)
            .collect();
        if points.len() > 1
            && distance2(points[0], *points.last().unwrap()).sqrt() <= minimum_distance
        {
            points.pop();
        }
        if !preserve_boundary {
            points = remove_collinear(points, minimum_area);
        }
        if signed_area(&points) < 0.0 {
            points.reverse();
        }
        let mut remaining: Vec<_> = (0..points.len()).collect();
        let mut triangles = Vec::new();
        while remaining.len() > 3 {
            let mut selected = None;
            let mut best = -1.0;
            for i in 0..remaining.len() {
                let [a, b, c] = [
                    remaining[(i + remaining.len() - 1) % remaining.len()],
                    remaining[i],
                    remaining[(i + 1) % remaining.len()],
                ];
                if area2(points[a], points[b], points[c]) <= minimum_area {
                    continue;
                }
                if remaining.iter().any(|&p| {
                    p != a
                        && p != b
                        && p != c
                        && contains(points[p], points[a], points[b], points[c])
                }) {
                    continue;
                }
                let quality = shape(&points, [a, b, c]);
                if quality > best + TRIANGLE_QUALITY_TIE_TOLERANCE {
                    selected = Some(i);
                    best = quality;
                }
            }
            let Some(i) = selected else {
                return Err("outline must be simple and nondegenerate".into());
            };
            triangles.push([
                remaining[(i + remaining.len() - 1) % remaining.len()],
                remaining[i],
                remaining[(i + 1) % remaining.len()],
            ]);
            remaining.remove(i);
        }
        if remaining.len() != 3 {
            return Err("outline collapsed during triangulation".into());
        }
        if area2(
            points[remaining[0]],
            points[remaining[1]],
            points[remaining[2]],
        ) <= minimum_area
        {
            return Err("outline has a degenerate final triangle".into());
        }
        triangles.push([remaining[0], remaining[1], remaining[2]]);
        let boundary = (0..points.len()).collect();
        let mut region = Self {
            points,
            triangles,
            boundary,
        };
        region.improve();
        Ok(region)
    }

    fn improve(&mut self) {
        for _ in 0..16 {
            let mut edges: BTreeMap<_, Vec<_>> = BTreeMap::new();
            for (index, face) in self.triangles.iter().enumerate() {
                for i in 0..3 {
                    let a = face[i];
                    let b = face[(i + 1) % 3];
                    edges
                        .entry(edge(a, b))
                        .or_default()
                        .push((index, a, b, face[(i + 2) % 3]));
                }
            }
            let mut touched = BTreeSet::new();
            let mut changed = false;
            for entries in edges.values() {
                let [x, y] = entries.as_slice() else {
                    continue;
                };
                if touched.contains(&x.0) || touched.contains(&y.0) {
                    continue;
                }
                let candidates = [[x.3, x.1, y.3], [x.3, y.3, x.2]];
                if candidates.iter().any(|&[a, b, c]| {
                    area2(self.points[a], self.points[b], self.points[c]) <= 1e-16
                }) {
                    continue;
                }
                let before = shape(&self.points, self.triangles[x.0])
                    .min(shape(&self.points, self.triangles[y.0]));
                let after =
                    shape(&self.points, candidates[0]).min(shape(&self.points, candidates[1]));
                if after <= before * (1.0 + 1e-8) {
                    continue;
                }
                self.triangles[x.0] = candidates[0];
                self.triangles[y.0] = candidates[1];
                touched.extend([x.0, y.0]);
                changed = true;
            }
            if !changed {
                break;
            }
        }
    }

    pub(crate) fn refine(mut self, max_edge: f64) -> Self {
        let mut longest = 0.0_f64;
        for f in &self.triangles {
            for i in 0..3 {
                longest =
                    longest.max(distance2(self.points[f[i]], self.points[f[(i + 1) % 3]]).sqrt());
            }
        }
        let rounds = (longest / max_edge).log2().ceil().max(0.0) as usize + 1;
        for _ in 0..rounds {
            let mut mids = BTreeMap::new();
            for f in &self.triangles {
                for i in 0..3 {
                    let a = f[i];
                    let b = f[(i + 1) % 3];
                    let key = edge(a, b);
                    if mids.contains_key(&key)
                        || distance2(self.points[a], self.points[b]) <= max_edge * max_edge
                    {
                        continue;
                    }
                    mids.insert(key, self.points.len());
                    self.points.push(std::array::from_fn(|axis| {
                        (self.points[a][axis] + self.points[b][axis]) / 2.0
                    }));
                }
            }
            if mids.is_empty() {
                break;
            }
            self.split_edges(&mids);
            self.improve();
        }
        self.improve();
        self
    }
}

struct OutlineTolerance {
    minimum_distance: f64,
    minimum_area: f64,
}
fn checked_outline(input: &[PlanarPoint]) -> Result<OutlineTolerance, String> {
    if input.len() < 3 || input.iter().flatten().any(|v| !v.is_finite()) {
        return Err("outline needs at least three finite points".into());
    }
    let extent = (0..2)
        .map(|axis| {
            input
                .iter()
                .map(|p| p[axis])
                .fold(f64::NEG_INFINITY, f64::max)
                - input.iter().map(|p| p[axis]).fold(f64::INFINITY, f64::min)
        })
        .fold(0.0, f64::max);
    if extent == 0.0 {
        return Err("outline has no extent".into());
    }
    let minimum_distance = extent * 1e-9;
    let minimum_area = extent * extent * 1e-12;
    for i in 0..input.len() {
        let next = (i + 1) % input.len();
        for j in i + 1..input.len() {
            let other = (j + 1) % input.len();
            if j == next || other == i {
                continue;
            }
            let [a, b, c, d] = [input[i], input[next], input[j], input[other]];
            let orientation = |a, b, c| {
                let area = area2(a, b, c);
                if area.abs() <= minimum_area {
                    0_i8
                } else if area < 0.0 {
                    -1
                } else {
                    1
                }
            };
            let on_segment = |p: PlanarPoint, a: PlanarPoint, b: PlanarPoint| {
                (0..2).all(|axis| {
                    p[axis] >= a[axis].min(b[axis]) - minimum_distance
                        && p[axis] <= a[axis].max(b[axis]) + minimum_distance
                })
            };
            let [ac, ad, ca, cb] = [
                orientation(a, b, c),
                orientation(a, b, d),
                orientation(c, d, a),
                orientation(c, d, b),
            ];
            if ac * ad < 0 && ca * cb < 0
                || ac == 0 && on_segment(c, a, b)
                || ad == 0 && on_segment(d, a, b)
                || ca == 0 && on_segment(a, c, d)
                || cb == 0 && on_segment(b, c, d)
            {
                return Err("outline edges intersect or touch".into());
            }
        }
    }

    Ok(OutlineTolerance {
        minimum_distance,
        minimum_area,
    })
}

fn remove_collinear(mut points: Vec<PlanarPoint>, minimum_area: f64) -> Vec<PlanarPoint> {
    loop {
        if points.len() <= 3 {
            break;
        }
        let old = points.len();
        points = (0..old)
            .filter_map(|i| {
                let a = points[(i + old - 1) % old];
                let b = points[i];
                let c = points[(i + 1) % old];
                let between = (b[0] - a[0]) * (c[0] - b[0]) + (b[1] - a[1]) * (c[1] - b[1]) >= 0.0;
                (!(area2(a, b, c).abs() <= minimum_area && between)).then_some(b)
            })
            .collect();
        if points.len() == old {
            break;
        }
    }

    points
}
