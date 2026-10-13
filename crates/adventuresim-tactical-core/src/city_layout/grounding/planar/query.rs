//! Conservative broad phase preserves exact clipping and canonical input order.
use bevy::math::DVec2;

/// Structural branching capacity; this does not prescribe terrain resolution.
const PLANAR_QUERY_LEAF_REGIONS: usize = 8;

#[derive(Clone, Copy, Debug)]
pub(in crate::city_layout::grounding) struct PlanarBounds {
    minimum: DVec2,
    maximum: DVec2,
}

impl PlanarBounds {
    pub fn from_triangle(points: [crate::scene_coordinates::ScenePlanPoint; 3]) -> Self {
        let [first, second, third] = points.map(|point| point.metres().as_dvec2());
        Self {
            minimum: first.min(second).min(third),
            maximum: first.max(second).max(third),
        }
    }
    pub fn from_points(points: impl IntoIterator<Item = DVec2>) -> Option<Self> {
        let mut points = points.into_iter();
        let first = points.next()?;
        if !first.is_finite() {
            return None;
        }
        points.try_fold(
            Self {
                minimum: first,
                maximum: first,
            },
            |bounds, point| {
                point.is_finite().then_some(Self {
                    minimum: bounds.minimum.min(point),
                    maximum: bounds.maximum.max(point),
                })
            },
        )
    }

    pub fn expanded(self, distance: f64) -> Self {
        Self {
            minimum: self.minimum - DVec2::splat(distance),
            maximum: self.maximum + DVec2::splat(distance),
        }
    }

    fn union(self, other: Self) -> Self {
        Self {
            minimum: self.minimum.min(other.minimum),
            maximum: self.maximum.max(other.maximum),
        }
    }

    pub fn intersects(self, other: Self) -> bool {
        !self.minimum.cmpgt(other.maximum).any() && !self.maximum.cmplt(other.minimum).any()
    }
}

#[derive(Clone, Debug)]
enum QueryChildren {
    Branch { left: usize, right: usize },
    Leaf(Vec<usize>),
}

#[derive(Clone, Debug)]
struct QueryNode {
    bounds: PlanarBounds,
    children: QueryChildren,
}

/// References address the unchanged source collection. Results retain its
/// iteration order, including ties on shared edges and overlapping inputs.
/// This producer-only index is not a generated or serialized terrain product.
#[derive(Clone, Debug)]
pub(in crate::city_layout::grounding) struct PlanarQueryIndex {
    bounds: Vec<PlanarBounds>,
    nodes: Vec<QueryNode>,
}

impl PlanarQueryIndex {
    pub fn from_bounds(bounds: Vec<PlanarBounds>) -> Self {
        let mut index = Self {
            bounds,
            nodes: Vec::new(),
        };
        let mut regions: Vec<_> = (0..index.bounds.len()).collect();
        if !regions.is_empty() {
            index.append(&mut regions);
        }
        index
    }

    fn append(&mut self, regions: &mut [usize]) -> usize {
        // The root is guarded as nonempty; both recursive partitions follow
        // the leaf-capacity check and split strictly inside the slice.
        let bounds = regions[1..]
            .iter()
            .fold(self.bounds[regions[0]], |bounds, index| {
                bounds.union(self.bounds[*index])
            });
        let node = self.nodes.len();
        self.nodes.push(QueryNode {
            bounds,
            children: QueryChildren::Leaf(Vec::new()),
        });
        let children = if regions.len() <= PLANAR_QUERY_LEAF_REGIONS {
            QueryChildren::Leaf(regions.to_vec())
        } else {
            let extent = bounds.maximum - bounds.minimum;
            let axis = usize::from(extent.y > extent.x);
            let centre = |i: &usize| self.bounds[*i].minimum[axis] + self.bounds[*i].maximum[axis];
            let midpoint = regions.len() / 2;
            // Only the two membership sets are needed here. Queries restore
            // authoritative source order after visiting the matching leaves.
            regions.select_nth_unstable_by(midpoint, |a, b| {
                centre(a).total_cmp(&centre(b)).then(a.cmp(b))
            });
            let (left, right) = regions.split_at_mut(midpoint);
            QueryChildren::Branch {
                left: self.append(left),
                right: self.append(right),
            }
        };
        self.nodes[node].children = children;
        node
    }

    pub fn intersections(&self, bounds: PlanarBounds) -> Vec<usize> {
        if self.nodes.is_empty() {
            return Vec::new();
        }
        let mut pending = vec![0];
        let mut candidates = Vec::new();
        while let Some(index) = pending.pop() {
            let node = &self.nodes[index];
            if !node.bounds.intersects(bounds) {
                continue;
            }
            match &node.children {
                QueryChildren::Branch { left, right } => pending.extend([*right, *left]),
                QueryChildren::Leaf(regions) => candidates.extend(
                    regions
                        .iter()
                        .copied()
                        .filter(|i| self.bounds[*i].intersects(bounds)),
                ),
            }
        }
        candidates.sort_unstable();
        candidates
    }
}

#[cfg(test)]
mod tests;
