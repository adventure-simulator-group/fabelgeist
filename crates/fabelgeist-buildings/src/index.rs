//! The distance to a union of solids near a point, without visiting all of
//! them: a grid over the ground, each cell listing the solids that come
//! within a margin of it.

use crate::solid::Solid;

/// Cell edge of the grid, metres.
const CELL: f32 = 4.0;

/// Solids, and which of them each patch of ground has near it.
#[derive(Clone, Debug, Default)]
pub struct SolidIndex {
    pub solids: Vec<Solid>,
    /// How far a query looks: past it, everything is "at least this far".
    margin: f32,
    origin: [f32; 2],
    dims: [usize; 2],
    /// Each cell's run of `entries`, row by row.
    starts: Vec<u32>,
    entries: Vec<u32>,
}

impl SolidIndex {
    /// Index `solids` for queries that need distances up to `margin`.
    pub fn new(solids: Vec<Solid>, margin: f32) -> Self {
        if solids.is_empty() {
            return Self {
                margin,
                ..Self::default()
            };
        }
        let mut lo = [f32::INFINITY; 2];
        let mut hi = [f32::NEG_INFINITY; 2];
        for solid in &solids {
            let [a, b] = solid.bounds();
            lo = [lo[0].min(a[0]), lo[1].min(a[2])];
            hi = [hi[0].max(b[0]), hi[1].max(b[2])];
        }
        let origin = [lo[0] - margin, lo[1] - margin];
        let dims = [
            (((hi[0] + margin - origin[0]) / CELL).ceil() as usize).max(1),
            (((hi[1] + margin - origin[1]) / CELL).ceil() as usize).max(1),
        ];
        let mut cells = vec![Vec::new(); dims[0] * dims[1]];
        for (i, solid) in solids.iter().enumerate() {
            let [a, b] = solid.bounds();
            let span =
                |v: f32, o: f32, n: usize| (((v - o) / CELL).floor().max(0.0) as usize).min(n - 1);
            for j in
                span(a[2] - margin, origin[1], dims[1])..=span(b[2] + margin, origin[1], dims[1])
            {
                for k in span(a[0] - margin, origin[0], dims[0])
                    ..=span(b[0] + margin, origin[0], dims[0])
                {
                    cells[j * dims[0] + k].push(i as u32);
                }
            }
        }
        let mut starts = Vec::with_capacity(cells.len() + 1);
        let mut entries = Vec::new();
        for cell in cells {
            starts.push(entries.len() as u32);
            entries.extend(cell);
        }
        starts.push(entries.len() as u32);
        Self {
            solids,
            margin,
            origin,
            dims,
            starts,
            entries,
        }
    }

    pub fn margin(&self) -> f32 {
        self.margin
    }

    pub fn is_empty(&self) -> bool {
        self.solids.is_empty()
    }

    /// The solids that may come within the margin of `p`.
    pub fn near(&self, p: [f32; 3]) -> &[u32] {
        if self.solids.is_empty() {
            return &[];
        }
        let k = ((p[0] - self.origin[0]) / CELL).floor();
        let j = ((p[2] - self.origin[1]) / CELL).floor();
        if k < 0.0 || j < 0.0 || k >= self.dims[0] as f32 || j >= self.dims[1] as f32 {
            return &[];
        }
        let cell = j as usize * self.dims[0] + k as usize;
        &self.entries[self.starts[cell] as usize..self.starts[cell + 1] as usize]
    }

    /// Signed distance from `p` to the union of the solids, capped at the
    /// margin: a point further than that from all of them gets the margin.
    pub fn distance(&self, p: [f32; 3]) -> f32 {
        self.near(p)
            .iter()
            .map(|&i| self.solids[i as usize].distance(p))
            .fold(self.margin, f32::min)
    }

    /// The way out of the union at `p`, a unit vector -- the distance's
    /// gradient -- if a solid is within the margin.
    pub fn normal(&self, p: [f32; 3]) -> Option<[f32; 3]> {
        if self.distance(p) >= self.margin {
            return None;
        }
        const H: f32 = 1e-3;
        let d = |dx: f32, dy: f32, dz: f32| self.distance([p[0] + dx, p[1] + dy, p[2] + dz]);
        let g = [
            d(H, 0.0, 0.0) - d(-H, 0.0, 0.0),
            d(0.0, H, 0.0) - d(0.0, -H, 0.0),
            d(0.0, 0.0, H) - d(0.0, 0.0, -H),
        ];
        let length = (g[0] * g[0] + g[1] * g[1] + g[2] * g[2]).sqrt();
        (length > 1e-9).then(|| [g[0] / length, g[1] / length, g[2] / length])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solid::Shape;

    #[test]
    fn the_index_finds_what_brute_force_finds() {
        let solids = vec![
            Solid::new(Shape::Box, [0.0, 0.0, 0.0], [3.0, 8.0, 1.0], 0.3),
            Solid::new(Shape::Cylinder, [12.0, 1.0, -5.0], [2.5, 12.0, 0.0], 0.0),
            Solid::new(Shape::Gable, [-9.0, 4.0, 7.0], [4.0, 2.0, 3.0], 1.1),
        ];
        let index = SolidIndex::new(solids.clone(), 3.0);
        for i in 0..2000 {
            let t = i as f32;
            let p = [
                (t * 0.37).sin() * 20.0,
                (t * 0.11).cos() * 8.0 + 6.0,
                (t * 0.23).cos() * 20.0,
            ];
            let brute = solids
                .iter()
                .map(|s| s.distance(p))
                .fold(f32::INFINITY, f32::min);
            assert!(
                (index.distance(p) - brute.min(3.0)).abs() < 1e-5,
                "at {p:?}"
            );
        }
        // Beside a wall, its normal is the wall's.
        let n = index.normal([0.0, 4.0, 1.5]).unwrap();
        let expected = [0.3f32.sin(), 0.0, 0.3f32.cos()];
        assert!((0..3).all(|k| (n[k] - expected[k]).abs() < 1e-3), "{n:?}");
    }
}
