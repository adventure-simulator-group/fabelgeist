//! A bounding volume hierarchy on the host.
//!
//! Built top-down with a binned surface-area-heuristic split, which gives a
//! better tree than the linear Morton build on the GPU does. Two jobs:
//!
//! * Host-side queries -- fitting a garment needs the body's nearest surface
//!   long before any solver runs, and that is a closest-point query on a mesh.
//! * An oracle. The GPU hierarchy in [`crate::gpu`] is checked against this
//!   one, and against brute force, because a BVH that silently misses a
//!   handful of primitives looks exactly like a BVH that works.
//!
//! Node storage is a flat array. An internal node's children are `left_first`
//! and `left_first + 1`, and both come after the parent, so a reverse pass
//! over the array refits bottom-up without any explicit ordering.

use fabelgeist_math::Vec3;

use crate::aabb::{Aabb, Ray};

/// Primitives per leaf below which splitting stops paying for itself.
const MAX_LEAF_SIZE: usize = 4;
/// Candidate split planes tried per axis.
const BIN_COUNT: usize = 12;

#[derive(Clone, Copy, Debug)]
pub struct Node {
    pub bounds: Aabb,
    /// Left child for an internal node; first primitive for a leaf.
    pub left_first: u32,
    /// Zero marks an internal node.
    pub count: u32,
}

impl Node {
    pub fn is_leaf(&self) -> bool {
        self.count > 0
    }
}

#[derive(Clone, Debug, Default)]
pub struct Bvh {
    pub nodes: Vec<Node>,
    /// Primitive indices, permuted so that a leaf's primitives are contiguous.
    pub indices: Vec<u32>,
}

impl Bvh {
    /// Build over primitive bounding boxes. The order of `bounds` is the
    /// primitive numbering every query reports back.
    pub fn build(bounds: &[Aabb]) -> Self {
        let mut bvh = Self {
            nodes: Vec::new(),
            indices: (0..bounds.len() as u32).collect(),
        };
        if bounds.is_empty() {
            return bvh;
        }

        // An upper bound: a binary tree over n leaves of at least one
        // primitive each has at most 2n - 1 nodes.
        bvh.nodes.reserve(2 * bounds.len());
        bvh.nodes.push(Node {
            bounds: Aabb::EMPTY,
            left_first: 0,
            count: bounds.len() as u32,
        });
        bvh.subdivide(0, bounds);
        bvh
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    pub fn primitive_count(&self) -> usize {
        self.indices.len()
    }

    pub fn bounds(&self) -> Aabb {
        self.nodes.first().map(|n| n.bounds).unwrap_or(Aabb::EMPTY)
    }

    fn subdivide(&mut self, node_index: usize, bounds: &[Aabb]) {
        let (first, count) = {
            let node = &self.nodes[node_index];
            (node.left_first as usize, node.count as usize)
        };
        let slice = &self.indices[first..first + count];

        let node_bounds = slice
            .iter()
            .fold(Aabb::EMPTY, |acc, &i| acc.union(bounds[i as usize]));
        self.nodes[node_index].bounds = node_bounds;

        if count <= MAX_LEAF_SIZE {
            return;
        }

        let Some(split) = self.find_split(first, count, bounds) else {
            return;
        };

        // Partition in place around the chosen plane.
        let (axis, position) = split;
        let mut left = first;
        let mut right = first + count;
        while left < right {
            let centroid = bounds[self.indices[left] as usize].center().axis(axis);
            if centroid < position {
                left += 1;
            } else {
                right -= 1;
                self.indices.swap(left, right);
            }
        }

        let left_count = left - first;
        // Every centroid landing on one side means the split bought nothing;
        // stop rather than recurse forever on the same set.
        if left_count == 0 || left_count == count {
            return;
        }

        let left_child = self.nodes.len() as u32;
        self.nodes.push(Node {
            bounds: Aabb::EMPTY,
            left_first: first as u32,
            count: left_count as u32,
        });
        self.nodes.push(Node {
            bounds: Aabb::EMPTY,
            left_first: left as u32,
            count: (count - left_count) as u32,
        });

        self.nodes[node_index].left_first = left_child;
        self.nodes[node_index].count = 0;

        self.subdivide(left_child as usize, bounds);
        self.subdivide(left_child as usize + 1, bounds);
    }

    /// The cheapest binned split, or `None` when leaving the node whole is
    /// cheaper than any of them.
    fn find_split(&self, first: usize, count: usize, bounds: &[Aabb]) -> Option<(usize, f32)> {
        let slice = &self.indices[first..first + count];

        // Bin over the *centroid* bounds, not the node bounds: binning over
        // the latter wastes bins on the margin that large primitives add.
        let centroid_bounds = slice.iter().fold(Aabb::EMPTY, |acc, &i| {
            acc.extend(bounds[i as usize].center())
        });
        let extent = centroid_bounds.extent();

        let leaf_cost = count as f32;
        let mut best: Option<(usize, f32, f32)> = None;

        for axis in 0..3 {
            let size = extent.axis(axis);
            if size <= 0.0 {
                continue;
            }
            let scale = BIN_COUNT as f32 / size;
            let origin = centroid_bounds.min.axis(axis);

            let mut bin_bounds = [Aabb::EMPTY; BIN_COUNT];
            let mut bin_counts = [0u32; BIN_COUNT];
            for &index in slice {
                let primitive = bounds[index as usize];
                let bin = (((primitive.center().axis(axis) - origin) * scale) as usize)
                    .min(BIN_COUNT - 1);
                bin_bounds[bin] = bin_bounds[bin].union(primitive);
                bin_counts[bin] += 1;
            }

            // Sweep from both ends so each of the BIN_COUNT - 1 planes knows
            // the bounds and the population on either side of it.
            let mut left_area = [0.0f32; BIN_COUNT - 1];
            let mut right_area = [0.0f32; BIN_COUNT - 1];
            let mut left_count = [0u32; BIN_COUNT - 1];
            let mut right_count = [0u32; BIN_COUNT - 1];

            let mut accumulated = Aabb::EMPTY;
            let mut population = 0u32;
            for i in 0..BIN_COUNT - 1 {
                accumulated = accumulated.union(bin_bounds[i]);
                population += bin_counts[i];
                left_area[i] = accumulated.surface_area();
                left_count[i] = population;
            }

            accumulated = Aabb::EMPTY;
            population = 0;
            for i in (0..BIN_COUNT - 1).rev() {
                accumulated = accumulated.union(bin_bounds[i + 1]);
                population += bin_counts[i + 1];
                right_area[i] = accumulated.surface_area();
                right_count[i] = population;
            }

            for i in 0..BIN_COUNT - 1 {
                if left_count[i] == 0 || right_count[i] == 0 {
                    continue;
                }
                let cost =
                    left_count[i] as f32 * left_area[i] + right_count[i] as f32 * right_area[i];
                let position = origin + (i + 1) as f32 / scale;
                if best.is_none_or(|(_, _, best_cost)| cost < best_cost) {
                    best = Some((axis, position, cost));
                }
            }
        }

        // The parent's own area normalises the split cost into the same units
        // as `leaf_cost`.
        let parent_area = slice
            .iter()
            .fold(Aabb::EMPTY, |acc, &i| acc.union(bounds[i as usize]))
            .surface_area();
        let (axis, position, cost) = best?;
        if parent_area > 0.0 && cost / parent_area >= leaf_cost {
            return None;
        }
        Some((axis, position))
    }

    /// Recompute every node's bounds against new primitive bounds, keeping the
    /// topology. Cheap enough to run every frame on a deforming mesh, and the
    /// tree degrades gracefully as long as the deformation is not wild.
    pub fn refit(&mut self, bounds: &[Aabb]) {
        for index in (0..self.nodes.len()).rev() {
            let node = self.nodes[index];
            self.nodes[index].bounds = if node.is_leaf() {
                let first = node.left_first as usize;
                self.indices[first..first + node.count as usize]
                    .iter()
                    .fold(Aabb::EMPTY, |acc, &i| acc.union(bounds[i as usize]))
            } else {
                let left = node.left_first as usize;
                self.nodes[left].bounds.union(self.nodes[left + 1].bounds)
            };
        }
    }

    /// Call `visit` with every primitive in a leaf whose *node* box overlaps
    /// `query`.
    ///
    /// A leaf holds several primitives and its box is their union, so this
    /// over-reports: some of what it hands back does not overlap at all. That
    /// is the normal contract for a hierarchy -- it narrows the search, and
    /// the caller runs the exact test, which for a triangle or a particle is
    /// not a box test anyway. Use [`Bvh::query_aabb`] when the box test *is*
    /// the exact test.
    pub fn candidates_aabb(&self, query: &Aabb, visit: impl FnMut(u32)) {
        self.walk(|node| node.bounds.overlaps(query), visit);
    }

    /// Call `visit` with every primitive whose own box overlaps `query`.
    ///
    /// `bounds` must be the array the tree was built from.
    pub fn query_aabb(&self, bounds: &[Aabb], query: &Aabb, mut visit: impl FnMut(u32)) {
        self.candidates_aabb(query, |index| {
            if bounds[index as usize].overlaps(query) {
                visit(index);
            }
        });
    }

    /// Call `visit` with every primitive in a leaf whose node box is within
    /// `radius` of `point`. Over-reports, for the same reason as
    /// [`Bvh::candidates_aabb`].
    pub fn candidates_point(&self, point: Vec3, radius: f32, visit: impl FnMut(u32)) {
        let radius_squared = radius * radius;
        self.walk(
            |node| node.bounds.distance_squared(point) <= radius_squared,
            visit,
        );
    }

    /// Call `visit` with every primitive whose own box is within `radius` of
    /// `point`.
    pub fn query_point(
        &self,
        bounds: &[Aabb],
        point: Vec3,
        radius: f32,
        mut visit: impl FnMut(u32),
    ) {
        let radius_squared = radius * radius;
        self.candidates_point(point, radius, |index| {
            if bounds[index as usize].distance_squared(point) <= radius_squared {
                visit(index);
            }
        });
    }

    /// Nearest primitive to `point`, by whatever `distance_squared` measures.
    ///
    /// The callback returns the true squared distance to primitive `i`; the
    /// traversal prunes on box distance, so the callback is only asked about
    /// primitives that could still win.
    pub fn closest(
        &self,
        point: Vec3,
        max_distance: f32,
        mut distance_squared: impl FnMut(u32) -> f32,
    ) -> Option<(u32, f32)> {
        let mut best: Option<(u32, f32)> = None;
        // Both closures need the limit, one to read and one to tighten, so it
        // lives in a `Cell` rather than being captured mutably twice.
        let limit = std::cell::Cell::new(max_distance * max_distance);

        self.walk(
            |node| node.bounds.distance_squared(point) <= limit.get(),
            |index| {
                let d = distance_squared(index);
                if d <= limit.get() {
                    limit.set(d);
                    best = Some((index, d));
                }
            },
        );

        best.map(|(index, d)| (index, d.sqrt()))
    }

    /// Nearest ray hit. `hit` returns the distance at which the ray meets
    /// primitive `i`, if it does at all within the current limit.
    pub fn raycast(
        &self,
        ray: &Ray,
        max_distance: f32,
        mut hit: impl FnMut(u32, f32) -> Option<f32>,
    ) -> Option<(u32, f32)> {
        let mut best: Option<(u32, f32)> = None;
        let limit = std::cell::Cell::new(max_distance);

        self.walk(
            |node| ray.hits(&node.bounds, limit.get()).is_some(),
            |index| {
                if let Some(distance) = hit(index, limit.get())
                    && distance < limit.get()
                {
                    limit.set(distance);
                    best = Some((index, distance));
                }
            },
        );

        best
    }

    /// Depth-first walk with an explicit stack.
    ///
    /// `descend` decides whether a node is worth entering; `leaf` is called
    /// per primitive. `descend` is re-asked on every node, so a caller that
    /// tightens a shared limit inside `leaf` -- as `closest` and `raycast` do
    /// -- prunes the rest of the walk with it.
    fn walk(&self, mut descend: impl FnMut(&Node) -> bool, mut leaf: impl FnMut(u32)) {
        if self.nodes.is_empty() {
            return;
        }
        // 64 is well past the depth a SAH build reaches for any realistic
        // primitive count; it is a stack, not a bound on the tree.
        let mut stack: Vec<u32> = Vec::with_capacity(64);
        stack.push(0);

        while let Some(index) = stack.pop() {
            let node = &self.nodes[index as usize];
            if !descend(node) {
                continue;
            }
            if node.is_leaf() {
                let first = node.left_first as usize;
                for &primitive in &self.indices[first..first + node.count as usize] {
                    leaf(primitive);
                }
            } else {
                stack.push(node.left_first);
                stack.push(node.left_first + 1);
            }
        }
    }
}

/// A triangle mesh with a BVH over it, which is what a mesh collider is.
#[derive(Clone, Debug, Default)]
pub struct TriangleBvh {
    pub positions: Vec<Vec3>,
    pub triangles: Vec<[u32; 3]>,
    pub bvh: Bvh,
}

impl TriangleBvh {
    pub fn new(positions: Vec<Vec3>, triangles: Vec<[u32; 3]>) -> Self {
        let bounds = triangle_bounds(&positions, &triangles);
        let bvh = Bvh::build(&bounds);
        Self {
            positions,
            triangles,
            bvh,
        }
    }

    /// Replace the vertex positions and refit. The triangle list is unchanged,
    /// so this is what a posed body mesh does every frame.
    pub fn update_positions(&mut self, positions: Vec<Vec3>) {
        self.positions = positions;
        let bounds = triangle_bounds(&self.positions, &self.triangles);
        self.bvh.refit(&bounds);
    }

    pub fn triangle(&self, index: u32) -> (Vec3, Vec3, Vec3) {
        let [a, b, c] = self.triangles[index as usize];
        (
            self.positions[a as usize],
            self.positions[b as usize],
            self.positions[c as usize],
        )
    }

    /// Closest point on the mesh surface, with the triangle it lies on and the
    /// distance to it.
    pub fn closest_point(&self, point: Vec3, max_distance: f32) -> Option<(u32, Vec3, f32)> {
        let result = self.bvh.closest(point, max_distance, |index| {
            let (a, b, c) = self.triangle(index);
            (crate::aabb::closest_point_on_triangle(point, a, b, c) - point).length_squared()
        });
        // The traversal reports which triangle won, not where on it; one more
        // evaluation against that triangle is cheaper than carrying the point
        // through every candidate.
        result.map(|(index, distance)| {
            let (a, b, c) = self.triangle(index);
            (
                index,
                crate::aabb::closest_point_on_triangle(point, a, b, c),
                distance,
            )
        })
    }

    pub fn raycast(&self, ray: &Ray, max_distance: f32) -> Option<(u32, f32)> {
        self.bvh.raycast(ray, max_distance, |index, limit| {
            let (a, b, c) = self.triangle(index);
            crate::aabb::ray_triangle(ray, a, b, c, limit)
        })
    }
}

pub fn triangle_bounds(positions: &[Vec3], triangles: &[[u32; 3]]) -> Vec<Aabb> {
    triangles
        .iter()
        .map(|&[a, b, c]| {
            Aabb::point(positions[a as usize])
                .extend(positions[b as usize])
                .extend(positions[c as usize])
        })
        .collect()
}

#[cfg(test)]
mod tests;
