//! Shapes binned into a sparse grid, and the queries on it -- on the host
//! here, and in WGSL in [`crate::wgsl`], one for one.

use rayon::prelude::*;

use crate::shape::{ALL_LAYERS, Shape};
use crate::union::smin;

/// An entry that holds no shape: a duplicate the GPU build dropped. Queries
/// skip it.
pub const EMPTY: u32 = u32::MAX;

/// No object: a [`Filter`] that leaves none out.
pub const NO_OBJECT: u32 = u32::MAX;

/// Which shapes a query looks at.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Filter {
    /// An object whose shapes are left out: the asking surface's own.
    pub exclude: u32,
    /// Only shapes on one of these layers.
    pub layers: u32,
}

impl Default for Filter {
    fn default() -> Self {
        Self::all()
    }
}

impl Filter {
    /// Every shape.
    pub fn all() -> Self {
        Self {
            exclude: NO_OBJECT,
            layers: ALL_LAYERS,
        }
    }

    pub fn excluding(mut self, object: u32) -> Self {
        self.exclude = object;
        self
    }

    pub fn on(mut self, layers: u32) -> Self {
        self.layers = layers;
        self
    }

    pub fn passes(&self, shape: &Shape) -> bool {
        shape.object != self.exclude && shape.layers() & self.layers != 0
    }
}

/// A surface a query found.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Hit {
    /// Signed distance to it.
    pub distance: f32,
    /// Its index in [`SurfaceIndex::shapes`].
    pub shape: u32,
    /// Its [`Shape::reach`].
    pub reach: f32,
    pub payload: u32,
}

/// Shader-side parameters of an index, for a uniform: how to find a
/// point's cell and bucket.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SurfaceParams {
    /// One over a cell's edge.
    pub inverse_cell: f32,
    /// Buckets in the table; a power of two, or zero for an empty index.
    pub table_size: u32,
    pub pad: [u32; 2],
}

/// Shapes binned into a sparse grid: each into every cell its surface,
/// grown by its reach, touches -- so that a point's own cell lists every
/// shape that can be found from it, and a query reads that one list. Cells
/// hash into a table of buckets; a bucket two cells share lists both, which
/// costs a query a distance or two and never an answer.
///
/// For a shader, upload [`SurfaceIndex::shapes`], [`SurfaceIndex::starts`]
/// and [`SurfaceIndex::entries`] as read-only storage bound as
/// `surface_shapes`, `surface_starts` and `surface_entries`, and
/// [`SurfaceIndex::params`] in a uniform -- or build it there
/// ([`crate::gpu::GpuSurfaceIndex`]).
#[derive(Clone, Debug, Default)]
pub struct SurfaceIndex {
    pub shapes: Vec<Shape>,
    /// Where each bucket's entries begin, and one more word: bucket `b`
    /// holds `entries[starts[b]..starts[b + 1]]`.
    pub starts: Vec<u32>,
    /// Shape indices, grouped by bucket and ascending within one.
    pub entries: Vec<u32>,
    pub params: SurfaceParams,
}

/// The bucket of a cell; the same hash as the WGSL's `surface_bucket`.
fn bucket_of(cell: [i32; 3], table_size: u32) -> u32 {
    let h = (cell[0] as u32).wrapping_mul(73_856_093)
        ^ (cell[1] as u32).wrapping_mul(19_349_663)
        ^ (cell[2] as u32).wrapping_mul(83_492_791);
    h & (table_size - 1)
}

fn cell_of(p: [f32; 3], inverse_cell: f32) -> [i32; 3] {
    p.map(|x| (x * inverse_cell).floor() as i32)
}

/// The cells a shape is binned into, lowest and highest corner; none when
/// its reach is negative -- a way to leave a shape in the buffer but out of
/// the index.
fn span(shape: &Shape, inverse_cell: f32) -> ([i32; 3], [i32; 3]) {
    if shape.reach < 0.0 {
        return ([0; 3], [-1; 3]);
    }
    let (lo, hi) = shape.bounds(shape.reach);
    (cell_of(lo, inverse_cell), cell_of(hi, inverse_cell))
}

fn cells_in((lo, hi): &([i32; 3], [i32; 3])) -> usize {
    (0..3)
        .map(|k| (hi[k] - lo[k] + 1).max(0) as usize)
        .product()
}

/// Buckets for `entries`: a power of two, about two per entry, so that a
/// bucket holds a cell or two.
pub fn table_size_for(entries: u32) -> u32 {
    entries.saturating_mul(2).next_power_of_two().max(64)
}

impl SurfaceIndex {
    /// An index of `shapes` in cells of edge `cell`. A cell about the size
    /// of a typical shape's reach keeps both the lists and the number of
    /// cells a shape lands in short; [`SurfaceIndex::cell_for`] picks one.
    pub fn build(shapes: Vec<Shape>, cell: f32) -> Self {
        let inverse_cell = 1.0 / cell.max(1e-4);
        let spans: Vec<([i32; 3], [i32; 3])> =
            shapes.par_iter().map(|s| span(s, inverse_cell)).collect();
        let total: usize = spans.par_iter().map(cells_in).sum();
        if total == 0 {
            return Self {
                shapes,
                ..Self::default()
            };
        }
        let table_size = table_size_for(total as u32);
        let mut pairs: Vec<(u32, u32)> = spans
            .par_iter()
            .enumerate()
            .flat_map_iter(|(i, &(lo, hi))| {
                (lo[2]..=hi[2]).flat_map(move |z| {
                    (lo[1]..=hi[1]).flat_map(move |y| {
                        (lo[0]..=hi[0]).map(move |x| (bucket_of([x, y, z], table_size), i as u32))
                    })
                })
            })
            .collect();
        pairs.par_sort_unstable();
        // A shape two of whose cells share a bucket is listed there once:
        // no query need guard against counting a surface twice.
        pairs.dedup();
        let mut starts = vec![0u32; table_size as usize + 1];
        for &(bucket, _) in &pairs {
            starts[bucket as usize + 1] += 1;
        }
        for b in 0..table_size as usize {
            starts[b + 1] += starts[b];
        }
        let entries = pairs.into_iter().map(|(_, shape)| shape).collect();
        Self {
            shapes,
            starts,
            entries,
            params: SurfaceParams {
                inverse_cell,
                table_size,
                pad: [0; 2],
            },
        }
    }

    /// How many entries an index of `shapes` in cells of edge `cell` holds
    /// at most -- before duplicates are dropped: what a GPU build needs room
    /// for.
    pub fn entry_count(shapes: &[Shape], cell: f32) -> usize {
        let inverse_cell = 1.0 / cell.max(1e-4);
        shapes
            .par_iter()
            .map(|s| cells_in(&span(s, inverse_cell)))
            .sum()
    }

    /// A cell edge for `shapes`: the median extent of what they reach, so
    /// most land in a few cells and the few large ones in more.
    pub fn cell_for(shapes: &[Shape]) -> f32 {
        let mut extents: Vec<f32> = shapes
            .iter()
            .map(|s| {
                let (lo, hi) = s.bounds(s.reach.max(0.0));
                (0..3).map(|k| hi[k] - lo[k]).fold(0.0, f32::max)
            })
            .collect();
        if extents.is_empty() {
            return 1.0;
        }
        let middle = extents.len() / 2;
        let (_, median, _) = extents.select_nth_unstable_by(middle, f32::total_cmp);
        median.max(1e-3)
    }

    /// The shapes that may be found from `p`: its cell's list, each once.
    /// May hold a few from cells sharing the bucket.
    pub fn candidates(&self, p: [f32; 3]) -> &[u32] {
        let table_size = self.params.table_size;
        if table_size == 0 {
            return &[];
        }
        let bucket = bucket_of(cell_of(p, self.params.inverse_cell), table_size) as usize;
        &self.entries[self.starts[bucket] as usize..self.starts[bucket + 1] as usize]
    }

    /// Each candidate the filter passes and that is within its reach of
    /// `p`, with its distance: what every query is made of.
    fn reachable(&self, p: [f32; 3], filter: Filter) -> impl Iterator<Item = Hit> + '_ {
        self.candidates(p).iter().filter_map(move |&i| {
            if i == EMPTY {
                return None;
            }
            let shape = &self.shapes[i as usize];
            if !filter.passes(shape) {
                return None;
            }
            let distance = shape.distance(p);
            (distance < shape.reach).then_some(Hit {
                distance,
                shape: i,
                reach: shape.reach,
                payload: shape.payload,
            })
        })
    }

    /// The nearest surface to `p` within `max` of it (and its own reach).
    pub fn nearest(&self, p: [f32; 3], filter: Filter, max: f32) -> Option<Hit> {
        self.reachable(p, filter)
            .filter(|h| h.distance < max)
            .min_by(|a, b| a.distance.total_cmp(&b.distance))
    }

    /// The four nearest surfaces within `max`, nearest first.
    pub fn nearest4(&self, p: [f32; 3], filter: Filter, max: f32) -> Vec<Hit> {
        let mut hits: Vec<Hit> = self
            .reachable(p, filter)
            .filter(|h| h.distance < max)
            .collect();
        hits.sort_by(|a, b| {
            a.distance
                .total_cmp(&b.distance)
                .then(a.shape.cmp(&b.shape))
        });
        hits.truncate(4);
        hits
    }

    /// The smooth union of every surface in reach of `p`, each blended by
    /// the smaller of `k` and its reach, folded in order of index (the
    /// smooth minimum is nearly, not exactly, independent of order; this
    /// keeps it the same from one frame to the next, and host to GPU).
    /// Infinite when none is in reach.
    pub fn union(&self, p: [f32; 3], filter: Filter, k: f32) -> f32 {
        self.reachable(p, filter).fold(f32::INFINITY, |field, hit| {
            smin(field, hit.distance, k.min(hit.reach))
        })
    }

    /// The surface `p` lies deepest inside, if any.
    pub fn inside(&self, p: [f32; 3], filter: Filter) -> Option<Hit> {
        self.reachable(p, filter)
            .filter(|h| h.distance < 0.0)
            .min_by(|a, b| a.distance.total_cmp(&b.distance))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A small deterministic generator, so the tests need no dependency.
    struct Lcg(u64);

    impl Lcg {
        fn next(&mut self) -> f32 {
            self.0 = self
                .0
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            (self.0 >> 40) as f32 / (1u64 << 24) as f32
        }

        fn point(&mut self, size: f32) -> [f32; 3] {
            [self.next() * size, self.next() * size, self.next() * size]
        }
    }

    /// Every kind of shape, sizes over two orders of magnitude, on a ground
    /// of height tiles.
    fn scene() -> Vec<Shape> {
        let mut rng = Lcg(1);
        let mut shapes = Vec::new();
        for i in 0..400u32 {
            let a = rng.point(10.0);
            let d = rng.point(2.0).map(|x| x - 1.0);
            let r = 0.02 + 0.5 * rng.next().powi(3);
            let shape = match i % 3 {
                0 => Shape::round_cone(a, r, [a[0] + d[0], a[1] + d[1], a[2] + d[2]], r * 0.5),
                1 => {
                    let q = [
                        rng.next() - 0.5,
                        rng.next() - 0.5,
                        rng.next() - 0.5,
                        rng.next(),
                    ];
                    let n = q.iter().map(|x| x * x).sum::<f32>().sqrt();
                    Shape::oriented_box(a, [r * 2.0, r, r * 3.0], q.map(|x| x / n), r * 0.2)
                }
                _ => {
                    let e = rng.point(1.0);
                    Shape::triangle(
                        a,
                        [a[0] + d[0], a[1] + d[1], a[2] + d[2]],
                        [a[0] + e[0], a[1], a[2] + e[2]],
                        r * 0.1,
                    )
                }
            };
            shapes.push(
                shape
                    .with_object(i / 4)
                    .with_reach(r * 0.8 + 0.05)
                    .with_layers(1 << (i % 3))
                    .with_payload(i),
            );
        }
        let ground = |x: f32, z: f32| 0.3 * (x * 0.7).sin() + 0.2 * (z * 1.1).cos();
        for zi in 0..10 {
            for xi in 0..10 {
                let (x0, z0) = (xi as f32, zi as f32);
                let (x1, z1) = (x0 + 1.0, z0 + 1.0);
                shapes.push(
                    Shape::height_tile(
                        [x0, z0],
                        [x1, z1],
                        [
                            ground(x0, z0),
                            ground(x1, z0),
                            ground(x0, z1),
                            ground(x1, z1),
                        ],
                        1.0,
                    )
                    .with_object(1000)
                    .with_reach(0.3)
                    .with_layers(1 << 3),
                );
            }
        }
        shapes
    }

    fn reachable_brute(shapes: &[Shape], p: [f32; 3], filter: Filter) -> Vec<Hit> {
        shapes
            .iter()
            .enumerate()
            .filter(|(_, s)| filter.passes(s))
            .map(|(i, s)| Hit {
                distance: s.distance(p),
                shape: i as u32,
                reach: s.reach,
                payload: s.payload,
            })
            .filter(|h| h.distance < h.reach)
            .collect()
    }

    #[test]
    fn every_query_finds_what_brute_force_finds() {
        let shapes = scene();
        let index = SurfaceIndex::build(shapes.clone(), SurfaceIndex::cell_for(&shapes));
        let mut rng = Lcg(2);
        let mut found = 0;
        for _ in 0..4000 {
            let p = [rng.next() * 10.0, rng.next() * 2.0 - 0.5, rng.next() * 10.0];
            let p = if rng.next() < 0.5 { p } else { rng.point(10.0) };
            let filter = Filter::all()
                .excluding((rng.next() * 100.0) as u32)
                .on(if rng.next() < 0.5 { ALL_LAYERS } else { 0b1011 });
            let max = 0.6 * rng.next();
            let mut all = reachable_brute(&shapes, p, filter);
            all.sort_by(|a, b| {
                a.distance
                    .total_cmp(&b.distance)
                    .then(a.shape.cmp(&b.shape))
            });

            let nearest = all.iter().find(|h| h.distance < max).copied();
            assert_eq!(
                index.nearest(p, filter, max).map(|h| h.shape),
                nearest.map(|h| h.shape)
            );
            let four: Vec<u32> = all
                .iter()
                .filter(|h| h.distance < max)
                .take(4)
                .map(|h| h.shape)
                .collect();
            let got: Vec<u32> = index
                .nearest4(p, filter, max)
                .iter()
                .map(|h| h.shape)
                .collect();
            assert_eq!(got, four);
            let inside = all.iter().find(|h| h.distance < 0.0).map(|h| h.shape);
            assert_eq!(index.inside(p, filter).map(|h| h.shape), inside);
            // The union, folded in the same order: a bucket lists its shapes
            // by index.
            let k = 0.1;
            let union = index.union(p, filter, k);
            let expected = reachable_brute(&shapes, p, filter)
                .iter()
                .fold(f32::INFINITY, |f, h| smin(f, h.distance, k.min(h.reach)));
            assert_eq!(union, expected);
            found += usize::from(nearest.is_some());
        }
        assert!(found > 200, "only {found} queries found anything");
    }

    #[test]
    fn a_filter_leaves_out_an_object_and_other_layers() {
        let shapes = vec![
            Shape::sphere([0.0; 3], 1.0)
                .with_object(1)
                .with_reach(0.5)
                .with_layers(1),
            Shape::sphere([0.0; 3], 0.8)
                .with_object(2)
                .with_reach(0.5)
                .with_layers(2),
        ];
        let index = SurfaceIndex::build(shapes, 1.0);
        let p = [1.2, 0.0, 0.0];
        assert_eq!(index.nearest(p, Filter::all(), 1.0).unwrap().shape, 0);
        assert_eq!(
            index
                .nearest(p, Filter::all().excluding(1), 1.0)
                .unwrap()
                .shape,
            1
        );
        assert_eq!(index.nearest(p, Filter::all().on(2), 1.0).unwrap().shape, 1);
        assert!(
            index
                .nearest(p, Filter::all().excluding(2).on(2), 1.0)
                .is_none()
        );
    }
}
