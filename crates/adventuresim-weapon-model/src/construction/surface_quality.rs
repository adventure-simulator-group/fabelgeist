//! Remove unresolved interior sampling edges while retaining physical boundaries.
use super::polygon::{TRIANGLE_QUALITY_RELATIVE_IMPROVEMENT, edge, shape};
use super::surface_refinement::surface_deviation;
use super::{PlanarPoint, Region};
use std::collections::{BTreeMap, BTreeSet};

const MAX_INTERIOR_COLLAPSE_ROUNDS: usize = 16;

struct Adjacency {
    edges: BTreeMap<(usize, usize), Vec<usize>>,
    stars: Vec<Vec<usize>>,
    protected: BTreeSet<usize>,
}

impl Adjacency {
    fn new<K: Eq>(region: &Region, classify: &impl Fn(PlanarPoint) -> K) -> Self {
        let mut result = Self {
            edges: BTreeMap::new(),
            stars: vec![Vec::new(); region.points.len()],
            protected: region.boundary.iter().copied().collect(),
        };
        for (index, face) in region.triangles.iter().enumerate() {
            for i in 0..3 {
                result.stars[face[i]].push(index);
                result
                    .edges
                    .entry(edge(face[i], face[(i + 1) % 3]))
                    .or_default()
                    .push(index);
            }
        }
        for (&(a, b), faces) in &result.edges {
            if faces.len() != 2
                || classify(center(region, faces[0])) != classify(center(region, faces[1]))
            {
                result.protected.extend([a, b]);
            }
        }
        result
    }
}

fn center(region: &Region, face: usize) -> PlanarPoint {
    std::array::from_fn(|axis| {
        region.triangles[face]
            .iter()
            .map(|&i| region.points[i][axis])
            .sum::<f64>()
            / 3.0
    })
}

impl Region {
    pub(crate) fn collapse_interior_sampling<K: Eq>(
        &mut self,
        classify: &impl Fn(PlanarPoint) -> K,
        height: &impl Fn(PlanarPoint) -> f64,
        maximum_deviation: f64,
        maximum_edge: f64,
    ) {
        for _ in 0..MAX_INTERIOR_COLLAPSE_ROUNDS {
            let adjacency = Adjacency::new(self, classify);
            let mut removed = BTreeSet::new();
            let mut touched = BTreeSet::<usize>::new();
            for (&(a, b), shared) in &adjacency.edges {
                if shared.len() != 2 || !shared.iter().any(|&i| self.subresolution(i)) {
                    continue;
                }
                let (remove, keep) = if !adjacency.protected.contains(&a) {
                    (a, b)
                } else if !adjacency.protected.contains(&b) {
                    (b, a)
                } else {
                    continue;
                };
                let affected = &adjacency.stars[remove];
                if affected
                    .iter()
                    .chain(&adjacency.stars[keep])
                    .any(|i| touched.contains(i))
                {
                    continue;
                }
                let neighbors = |v: usize| {
                    adjacency.stars[v]
                        .iter()
                        .flat_map(|&i| self.triangles[i])
                        .filter(|&i| i != a && i != b)
                        .collect::<BTreeSet<_>>()
                };
                if neighbors(a).intersection(&neighbors(b)).count() != 2 {
                    continue;
                }
                let key = classify(center(self, affected[0]));
                let valid = affected.iter().all(|&i| {
                    if classify(center(self, i)) != key {
                        return false;
                    }
                    if self.triangles[i].contains(&keep) {
                        return true;
                    }
                    let points =
                        self.triangles[i].map(|v| self.points[if v == remove { keep } else { v }]);
                    let centroid = std::array::from_fn(|axis| {
                        points.iter().map(|p| p[axis]).sum::<f64>() / 3.0
                    });
                    classify(centroid) == key
                        && valid_triangle(points, height, maximum_deviation, maximum_edge)
                });
                if !valid || !self.improves_star(affected, remove, keep) {
                    continue;
                }
                for &i in affected {
                    if self.triangles[i].contains(&keep) {
                        removed.insert(i);
                    } else {
                        self.triangles[i] =
                            self.triangles[i].map(|v| if v == remove { keep } else { v });
                    }
                }
                touched.extend(affected);
                touched.extend(&adjacency.stars[keep]);
            }
            if removed.is_empty() {
                break;
            }
            self.triangles = self
                .triangles
                .iter()
                .enumerate()
                .filter(|(i, _)| !removed.contains(i))
                .map(|(_, &face)| face)
                .collect();
        }
    }

    fn subresolution(&self, index: usize) -> bool {
        let face = self.triangles[index];
        let [a, b, c] = face.map(|i| self.points[i]);
        let longest = distance(a, b).max(distance(b, c)).max(distance(c, a));
        shape(&self.points, face) * longest < crate::recipe::MIN_MANUFACTURED_METRES
    }

    fn improves_star(&self, affected: &[usize], remove: usize, keep: usize) -> bool {
        let mut before = f64::INFINITY;
        let mut after = f64::INFINITY;
        for &i in affected {
            let face = self.triangles[i];
            before = before.min(shape(&self.points, face));
            if !face.contains(&keep) {
                let replaced = face.map(|v| if v == remove { keep } else { v });
                after = after.min(shape(&self.points, replaced));
            }
        }
        after > before * (1.0 + TRIANGLE_QUALITY_RELATIVE_IMPROVEMENT)
    }
}

fn distance(a: PlanarPoint, b: PlanarPoint) -> f64 {
    (a[0] - b[0]).hypot(a[1] - b[1])
}

fn valid_triangle(
    [a, b, c]: [PlanarPoint; 3],
    height: &impl Fn(PlanarPoint) -> f64,
    maximum_deviation: f64,
    maximum_edge: f64,
) -> bool {
    (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0]) > 0.0
        && [(a, b), (b, c), (c, a)]
            .into_iter()
            .all(|(p, q)| distance(p, q) <= maximum_edge)
        && surface_deviation([a, b, c], height) <= maximum_deviation / 2.0
}
