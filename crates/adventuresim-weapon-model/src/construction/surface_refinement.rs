//! Conforming subdivision shared by planar regions and sampled relief surfaces.
use super::polygon::{TRIANGLE_QUALITY_TIE_TOLERANCE, edge, shape};
use super::{PlanarPoint, Region, construction_budget};
use std::collections::BTreeMap;

const MAX_SURFACE_REFINEMENT_ROUNDS: usize = 12;

impl Region {
    pub(crate) fn split_edges(&mut self, mids: &BTreeMap<(usize, usize), usize>) {
        let mut next = Vec::new();
        for &f in &self.triangles {
            let m: Vec<_> = (0..3)
                .map(|i| mids.get(&edge(f[i], f[(i + 1) % 3])).copied())
                .collect();
            match m.iter().flatten().count() {
                0 => next.push(f),
                3 => {
                    let [a, b, c] = [m[0].unwrap(), m[1].unwrap(), m[2].unwrap()];
                    next.extend([[f[0], a, c], [a, f[1], b], [c, b, f[2]], [a, b, c]]);
                }
                1 => {
                    let i = m.iter().position(Option::is_some).unwrap();
                    let [a, b, c] = [f[i], f[(i + 1) % 3], f[(i + 2) % 3]];
                    let mid = m[i].unwrap();
                    next.extend([[a, mid, c], [mid, b, c]]);
                }
                _ => {
                    let i = m.iter().position(Option::is_none).unwrap();
                    let [a, b, c] = [f[i], f[(i + 1) % 3], f[(i + 2) % 3]];
                    let bc = m[(i + 1) % 3].unwrap();
                    let ca = m[(i + 2) % 3].unwrap();
                    next.push([c, ca, bc]);
                    let options = [[[a, b, bc], [a, bc, ca]], [[a, b, ca], [b, bc, ca]]];
                    let quality = |option: [[usize; 3]; 2]| {
                        shape(&self.points, option[0]).min(shape(&self.points, option[1]))
                    };
                    next.extend(
                        if quality(options[1])
                            > quality(options[0]) + TRIANGLE_QUALITY_TIE_TOLERANCE
                        {
                            options[1]
                        } else {
                            options[0]
                        },
                    );
                }
            }
        }
        self.boundary = (0..self.boundary.len())
            .flat_map(|i| {
                let a = self.boundary[i];
                let mut points = vec![a];
                if let Some(&mid) = mids.get(&edge(a, self.boundary[(i + 1) % self.boundary.len()]))
                {
                    points.push(mid);
                }
                points
            })
            .collect();
        self.triangles = next;
    }

    /// Refine without flipping the already partitioned edges. In each cell the
    /// height must be quadratic divided by a positive affine denominator (or
    /// its continuous terminal limit). The rational Bernstein error coefficients
    /// are zero at vertices and twice the edge-midpoint errors, so half-budget
    /// midpoint checks bound the entire triangle, not merely its sampled points.
    pub(crate) fn refine_rational_surface(
        &mut self,
        height: impl Fn(PlanarPoint) -> f64,
        maximum_deviation: f64,
        maximum_edge: f64,
    ) -> Result<(), String> {
        for _ in 0..MAX_SURFACE_REFINEMENT_ROUNDS {
            let mut mids = BTreeMap::new();
            for &face in &self.triangles {
                let points = face.map(|i| self.points[i]);
                let curved = surface_deviation(points, &height) > maximum_deviation / 2.0;
                for i in 0..3 {
                    let [a, b] = [points[i], points[(i + 1) % 3]];
                    if !curved && (a[0] - b[0]).hypot(a[1] - b[1]) <= maximum_edge {
                        continue;
                    }
                    let key = edge(face[i], face[(i + 1) % 3]);
                    if mids.contains_key(&key) {
                        continue;
                    }
                    mids.insert(key, self.points.len());
                    self.points.push(std::array::from_fn(|axis| {
                        (self.points[key.0][axis] + self.points[key.1][axis]) / 2.0
                    }));
                }
            }
            if mids.is_empty() {
                return Ok(());
            }
            let faces = self
                .triangles
                .iter()
                .map(|face| {
                    1 + (0..3)
                        .filter(|&i| mids.contains_key(&edge(face[i], face[(i + 1) % 3])))
                        .count()
                })
                .sum::<usize>();
            let boundary = self.boundary.len()
                + (0..self.boundary.len())
                    .filter(|&i| {
                        mids.contains_key(&edge(
                            self.boundary[i],
                            self.boundary[(i + 1) % self.boundary.len()],
                        ))
                    })
                    .count();
            construction_budget((faces * 2 + boundary * 2) as f64)?;
            self.split_edges(&mids);
        }
        Err("plate surface exceeds its bounded refinement budget".into())
    }
}

fn surface_deviation(points: [PlanarPoint; 3], height: &impl Fn(PlanarPoint) -> f64) -> f64 {
    let heights = points.map(height);
    let center = std::array::from_fn(|axis| points.iter().map(|p| p[axis]).sum::<f64>() / 3.0);
    let mut deviation = (height(center) - heights.iter().sum::<f64>() / 3.0).abs();
    for i in 0..3 {
        let next = (i + 1) % 3;
        let midpoint = std::array::from_fn(|axis| (points[i][axis] + points[next][axis]) / 2.0);
        deviation = deviation.max((height(midpoint) - (heights[i] + heights[next]) / 2.0).abs());
    }
    deviation
}
