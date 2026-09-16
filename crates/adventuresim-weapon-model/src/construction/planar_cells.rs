//! Retriangulation retains physical cell boundaries, not incidental diagonals.
use super::{PlanarPoint, Region, construction_budget};
use std::collections::{BTreeMap, BTreeSet};

const CELL_COLLINEAR_ROUNDOFF_ULPS: f64 = 64.0;
type DirectedEdges = BTreeMap<(usize, usize), (usize, usize)>;

impl Region {
    pub(crate) fn remesh_cells<K: Ord>(
        &mut self,
        classify: impl Fn(PlanarPoint) -> K,
    ) -> Result<(), String> {
        let mut cells = BTreeMap::<K, DirectedEdges>::new();
        for face in &self.triangles {
            let center = std::array::from_fn(|axis| {
                face.iter().map(|&i| self.points[i][axis]).sum::<f64>() / 3.0
            });
            let edges = cells.entry(classify(center)).or_default();
            for i in 0..3 {
                let [a, b] = [face[i], face[(i + 1) % 3]];
                let key = (a.min(b), a.max(b));
                if edges.remove(&key).is_none() {
                    edges.insert(key, (a, b));
                }
            }
        }
        let mut loops = Vec::new();
        for edges in cells.into_values() {
            loops.extend(boundary_loops(edges)?);
        }
        // Keep a junction whenever any incident cell turns there. Removing it
        // only from a straight neighboring cell would create a T-junction.
        let mut retained = BTreeSet::new();
        for boundary in &loops {
            for i in 0..boundary.len() {
                let points = [
                    boundary[(i + boundary.len() - 1) % boundary.len()],
                    boundary[i],
                    boundary[(i + 1) % boundary.len()],
                ]
                .map(|index| self.points[index]);
                if !collinear(points) {
                    retained.insert(boundary[i]);
                }
            }
        }
        let mut triangles = Vec::new();
        for boundary in loops {
            let indices: Vec<_> = boundary
                .into_iter()
                .filter(|i| retained.contains(i))
                .collect();
            let outline: Vec<_> = indices.iter().map(|&i| self.points[i]).collect();
            let cell = Region::triangulate(&outline, true)?;
            let mapping: Vec<_> = cell
                .points
                .iter()
                .map(|p| {
                    indices
                        .iter()
                        .copied()
                        .find(|&i| self.points[i] == *p)
                        .expect("cell triangulation retains its authored boundary vertices")
                })
                .collect();
            triangles.extend(
                cell.triangles
                    .into_iter()
                    .map(|face| face.map(|i| mapping[i])),
            );
        }
        construction_budget((triangles.len() * 2 + self.boundary.len() * 2) as f64)?;
        self.triangles = triangles;
        self.boundary.retain(|i| retained.contains(i));
        Ok(())
    }
}

fn boundary_loops(edges: DirectedEdges) -> Result<Vec<Vec<usize>>, String> {
    let mut outgoing = BTreeMap::new();
    for (a, b) in edges.into_values() {
        if outgoing.insert(a, b).is_some() {
            return Err("plate cell boundary has an unresolved junction".into());
        }
    }
    let mut loops = Vec::new();
    while let Some((&start, _)) = outgoing.first_key_value() {
        let mut boundary = Vec::new();
        let mut current = start;
        loop {
            boundary.push(current);
            current = outgoing
                .remove(&current)
                .ok_or("plate cell boundary is open")?;
            if current == start {
                break;
            }
        }
        if boundary.len() < 3 {
            return Err("plate cell boundary is degenerate".into());
        }
        loops.push(boundary);
    }
    Ok(loops)
}

fn collinear([a, b, c]: [PlanarPoint; 3]) -> bool {
    let u = [b[0] - a[0], b[1] - a[1]];
    let v = [c[0] - b[0], c[1] - b[1]];
    let subtraction =
        |a: PlanarPoint, b: PlanarPoint| [a[0].abs() + b[0].abs(), a[1].abs() + b[1].abs()];
    let du = subtraction(a, b);
    let dv = subtraction(b, c);
    let scale = du[0] * v[1].abs()
        + dv[1] * u[0].abs()
        + du[1] * v[0].abs()
        + dv[0] * u[1].abs()
        + (u[0] * v[1]).abs()
        + (u[1] * v[0]).abs();
    u[0] * v[0] + u[1] * v[1] >= 0.0
        && (u[0] * v[1] - u[1] * v[0]).abs() <= CELL_COLLINEAR_ROUNDOFF_ULPS * f64::EPSILON * scale
}
