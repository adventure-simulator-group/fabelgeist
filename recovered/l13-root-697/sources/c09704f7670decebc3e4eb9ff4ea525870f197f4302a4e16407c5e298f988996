//! A hash grid over points, for the nearest-root queries the hair nodes
//! make: the guides nearest a new strand's root, the clump guide nearest a
//! strand, whether a root is too close to one already placed.
//!
//! Points carry a group; a query sees only the groups its filter accepts,
//! which is how parting keeps strands from interpolating across a part
//! (Blender offsets each group's points far apart for the same effect).

use crate::identity::PartingGroup;
use std::collections::HashMap;

use fabelgeist_math::Vec3;

pub struct PointGrid {
    cell: f32,
    cells: HashMap<[i32; 3], Vec<u32>>,
    points: Vec<(Vec3, PartingGroup)>,
}

impl PointGrid {
    pub fn new(cell: f32) -> Self {
        Self {
            cell: cell.max(1e-6),
            cells: HashMap::new(),
            points: Vec::new(),
        }
    }

    /// A grid over `points`, sized so a cell holds a few of them.
    pub fn build(points: &[(Vec3, PartingGroup)], cell: f32) -> Self {
        let mut grid = Self::new(cell);
        for &(p, g) in points {
            grid.insert(p, g);
        }
        grid
    }

    fn key(&self, p: Vec3) -> [i32; 3] {
        [p.x, p.y, p.z].map(|v| (v / self.cell).floor() as i32)
    }

    /// Add a point; returns its index.
    pub fn insert(&mut self, p: Vec3, group: PartingGroup) -> u32 {
        let index = self.points.len() as u32;
        self.points.push((p, group));
        let key = self.key(p);
        self.cells.entry(key).or_default().push(index);
        index
    }

    pub fn len(&self) -> usize {
        self.points.len()
    }

    pub fn is_empty(&self) -> bool {
        self.points.is_empty()
    }

    pub fn point(&self, index: u32) -> Vec3 {
        self.points[index as usize].0
    }

    /// Visit every point in the cells within `ring` cells of `p`'s.
    fn visit_shell(&self, p: Vec3, ring: i32, mut visit: impl FnMut(u32)) {
        let c = self.key(p);
        for dz in -ring..=ring {
            for dy in -ring..=ring {
                for dx in -ring..=ring {
                    if dx.abs().max(dy.abs()).max(dz.abs()) != ring {
                        continue;
                    }
                    if let Some(list) = self.cells.get(&[c[0] + dx, c[1] + dy, c[2] + dz]) {
                        list.iter().for_each(|&i| visit(i));
                    }
                }
            }
        }
    }

    /// Whether any accepted point lies within `radius` of `p`.
    pub fn any_within(&self, p: Vec3, radius: f32, accept: impl Fn(PartingGroup) -> bool) -> bool {
        let rings = (radius / self.cell).ceil() as i32;
        let r2 = radius * radius;
        for ring in 0..=rings {
            let mut hit = false;
            self.visit_shell(p, ring, |i| {
                let (q, g) = self.points[i as usize];
                if !hit && accept(g) && (q - p).length_squared() < r2 {
                    hit = true;
                }
            });
            if hit {
                return true;
            }
        }
        false
    }

    /// How many points lie within `radius` of `p`.
    pub fn count_within(&self, p: Vec3, radius: f32) -> usize {
        let rings = (radius / self.cell).ceil() as i32;
        let mut count = 0;
        for ring in 0..=rings {
            self.visit_shell(p, ring, |i| {
                if (self.points[i as usize].0 - p).length() <= radius {
                    count += 1;
                }
            });
        }
        count
    }

    /// The `k` accepted points nearest `p`, nearest first, searching no
    /// further than `max_radius`. Distances are Euclidean.
    pub fn nearest_k(
        &self,
        p: Vec3,
        k: usize,
        max_radius: f32,
        accept: impl Fn(PartingGroup) -> bool,
    ) -> Vec<(u32, f32)> {
        let mut found: Vec<(u32, f32)> = Vec::with_capacity(k + 8);
        let max_rings = (max_radius / self.cell).ceil().min(1e6) as i32;
        for ring in 0..=max_rings {
            self.visit_shell(p, ring, |i| {
                let (q, g) = self.points[i as usize];
                if accept(g) {
                    found.push((i, (q - p).length()));
                }
            });
            // Every point outside the searched rings is at least `ring *
            // cell` away; once k are nearer than that, the search is done.
            if found.len() >= k {
                found.sort_by(|a, b| a.1.total_cmp(&b.1));
                if found[k - 1].1 <= ring as f32 * self.cell {
                    found.truncate(k);
                    return found;
                }
            }
        }
        found.sort_by(|a, b| a.1.total_cmp(&b.1));
        found.retain(|f| f.1 <= max_radius);
        found.truncate(k);
        found
    }

    pub fn nearest(
        &self,
        p: Vec3,
        max_radius: f32,
        accept: impl Fn(PartingGroup) -> bool,
    ) -> Option<(u32, f32)> {
        self.nearest_k(p, 1, max_radius, accept).first().copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rng::Rng;

    #[test]
    fn nearest_k_matches_brute_force() {
        let mut rng = Rng::new(3);
        let points: Vec<(Vec3, PartingGroup)> = (0..500)
            .map(|i| {
                (
                    Vec3::new(rng.next_f32(), rng.next_f32(), rng.next_f32() * 0.1),
                    PartingGroup::from_gpu_word(i % 2),
                )
            })
            .collect();
        let grid = PointGrid::build(&points, 0.05);
        for _ in 0..50 {
            let q = Vec3::new(rng.next_f32(), rng.next_f32(), 0.05);
            let got = grid.nearest_k(q, 4, 10.0, |g| g == PartingGroup::from_gpu_word(1));
            let mut brute: Vec<(u32, f32)> = points
                .iter()
                .enumerate()
                .filter(|(_, (_, g))| *g == PartingGroup::from_gpu_word(1))
                .map(|(i, (p, _))| (i as u32, (*p - q).length()))
                .collect();
            brute.sort_by(|a, b| a.1.total_cmp(&b.1));
            let got: Vec<u32> = got.iter().map(|g| g.0).collect();
            let want: Vec<u32> = brute[..4].iter().map(|g| g.0).collect();
            assert_eq!(got, want);
        }
    }
}
