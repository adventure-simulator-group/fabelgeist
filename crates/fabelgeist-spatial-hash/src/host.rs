//! The host grid.

use crate::{CellScale, EMPTY, HashBucketCount, HashLayout, HashPointCount, bucket_of, cell_of};
use anyhow::Result;

/// Spheres (or points) hashed into a uniform grid.
///
/// Built with a counting sort over buckets, so a build is two linear passes
/// and no allocation beyond the tables themselves when reused through
/// [`HashGrid::rebuild`].
#[derive(Clone, Debug, Default)]
pub struct HashGrid {
    scale: CellScale,
    table_size: HashBucketCount,
    /// Each entry's own cell, to reject bucket collisions.
    cells: Vec<[i32; 3]>,
    /// Entry indices, grouped by bucket.
    sorted: Vec<u32>,
    /// `[begin, end)` into `sorted` per bucket; `begin == EMPTY` if empty.
    ranges: Vec<[u32; 2]>,
    buckets: Vec<u32>,
}

impl HashGrid {
    /// Hash `points` (the centres) using `scale` in their coordinate units.
    pub fn new(points: impl ExactSizeIterator<Item = [f32; 3]>, scale: CellScale) -> Result<Self> {
        let mut grid = Self::default();
        grid.rebuild(points, scale)?;
        Ok(grid)
    }

    /// Rebuild in place, reusing the tables.
    pub fn rebuild(
        &mut self,
        points: impl ExactSizeIterator<Item = [f32; 3]>,
        scale: CellScale,
    ) -> Result<()> {
        let count = points.len();
        let layout = HashLayout::try_from(HashPointCount::try_from(count)?)?;
        self.scale = scale;
        self.table_size = layout.bucket_count();
        self.cells.clear();
        self.buckets.clear();
        self.ranges.clear();
        self.ranges.resize(usize::from(self.table_size), [0, 0]);

        for point in points {
            let cell = cell_of(point, self.scale);
            let bucket = bucket_of(cell, self.table_size);
            self.cells.push(cell);
            self.buckets.push(bucket);
            self.ranges[bucket as usize][1] += 1;
        }

        // Exclusive scan of the counts into run starts.
        let mut offset = 0u32;
        for range in &mut self.ranges {
            let n = range[1];
            *range = if n == 0 {
                [EMPTY, EMPTY]
            } else {
                [offset, offset]
            };
            offset += n;
        }

        self.sorted.clear();
        self.sorted.resize(count, 0);
        for (index, &bucket) in self.buckets.iter().enumerate() {
            let range = &mut self.ranges[bucket as usize];
            self.sorted[range[1] as usize] = index as u32;
            range[1] += 1;
        }
        Ok(())
    }

    pub fn len(&self) -> usize {
        self.cells.len()
    }

    pub fn is_empty(&self) -> bool {
        self.cells.is_empty()
    }

    pub fn cell_scale(&self) -> CellScale {
        self.scale
    }

    /// Call `visit` with every entry whose cell lies within `reach` cells of
    /// the one `point` is in, along every axis. `reach` of one is the usual
    /// 27-cell neighbourhood, which covers every entry within one cell size.
    pub fn for_each_near(&self, point: [f32; 3], reach: i32, mut visit: impl FnMut(u32)) {
        if self.cells.is_empty() {
            return;
        }
        let base = cell_of(point, self.scale);
        for dz in -reach..=reach {
            for dy in -reach..=reach {
                for dx in -reach..=reach {
                    let cell = [base[0] + dx, base[1] + dy, base[2] + dz];
                    let [begin, end] = self.ranges[bucket_of(cell, self.table_size) as usize];
                    if begin == EMPTY {
                        continue;
                    }
                    for &entry in &self.sorted[begin as usize..end as usize] {
                        if self.cells[entry as usize] == cell {
                            visit(entry);
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lattice() -> Vec<[f32; 3]> {
        let mut points = Vec::new();
        let mut state = 0x9e37_79b9u32;
        for _ in 0..2000 {
            let mut next = || {
                state ^= state << 13;
                state ^= state >> 17;
                state ^= state << 5;
                (state as f32 / u32::MAX as f32) * 20.0 - 10.0
            };
            points.push([next(), next(), next()]);
        }
        points
    }

    #[test]
    fn finds_exactly_the_neighbours_brute_force_finds() {
        let points = lattice();
        let radius = 1.3;
        let grid = HashGrid::new(points.iter().copied(), CellScale::from_cell_size(radius)).unwrap();
        for (i, p) in points.iter().enumerate().step_by(37) {
            let mut found = Vec::new();
            grid.for_each_near(*p, 1, |j| {
                let q = points[j as usize];
                let d2 = (0..3).map(|k| (p[k] - q[k]).powi(2)).sum::<f32>();
                if d2 <= radius * radius {
                    found.push(j);
                }
            });
            found.sort_unstable();
            let brute: Vec<u32> = points
                .iter()
                .enumerate()
                .filter(|(_, q)| {
                    (0..3).map(|k| (p[k] - q[k]).powi(2)).sum::<f32>() <= radius * radius
                })
                .map(|(j, _)| j as u32)
                .collect();
            assert_eq!(found, brute, "point {i}");
        }
    }

    #[test]
    fn visits_each_entry_once() {
        let points = lattice();
        let grid = HashGrid::new(points.iter().copied(), CellScale::from_cell_size(3.0)).unwrap();
        let mut seen = vec![0u32; points.len()];
        grid.for_each_near([0.0; 3], 4, |j| seen[j as usize] += 1);
        assert!(seen.iter().all(|&n| n <= 1));
    }
}
