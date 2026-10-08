//! Immutable broad phase for exact support triangles; no terrain resampling.
use super::*;
use bevy::prelude::Reflect;
use serde::Deserialize;
use smallvec::SmallVec;
mod transport;

/// A structural leaf capacity bounds triangle work without prescribing a
/// geographic grid spacing or altering the represented terrain resolution.
const SUPPORT_QUERY_LEAF_TRIANGLES: usize = 8;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Reflect)]
enum SupportTriangleRef {
    Natural(usize),
    Foundation { owner: usize, triangle: usize },
}

impl SupportTriangleRef {
    fn points(self, surface: &BoundedSettlementTerrain) -> [Vec3; 3] {
        match self {
            Self::Natural(index) => surface.natural_triangles[index],
            Self::Foundation { owner, triangle } => {
                let foundation = &surface.foundations[owner];
                foundation.support_triangles[triangle].map(|i| foundation.positions[i as usize])
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Reflect)]
enum QueryChildren {
    Branch { left: usize, right: usize },
    Leaf(Vec<SupportTriangleRef>),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Reflect)]
struct QueryNode {
    minimum: Vec2,
    maximum: Vec2,
    children: QueryChildren,
}

/// Compiler-local bounds avoid repeatedly resolving the same physical vertices
/// in every sorting comparison. These controls never enter the cached product.
struct TriangleQueryBounds {
    reference: SupportTriangleRef,
    minimum: Vec2,
    maximum: Vec2,
    centre_sum: Vec2,
}

impl TriangleQueryBounds {
    fn from_reference(reference: SupportTriangleRef, surface: &BoundedSettlementTerrain) -> Self {
        let points = reference.points(surface);
        Self {
            reference,
            minimum: points
                .iter()
                .fold(Vec2::splat(f32::INFINITY), |a, p| a.min(p.xz())),
            maximum: points
                .iter()
                .fold(Vec2::splat(f32::NEG_INFINITY), |a, p| a.max(p.xz())),
            centre_sum: points.iter().map(|p| p.xz()).sum(),
        }
    }
}

/// The index contains references into the canonical source/foundation geometry.
/// Serializing those references keeps immutable cached products immediately
/// usable without rebuilding a query acceleration structure during warm reuse.
#[derive(Clone, Debug, Default, PartialEq, Reflect)]
pub(super) struct SupportQueryIndex(Vec<QueryNode>);

impl SupportQueryIndex {
    pub(super) fn validate(
        &self,
        surface: &BoundedSettlementTerrain,
    ) -> Result<(), SupportGeometryIssue> {
        let mut visited = vec![false; self.0.len()];
        let mut references = std::collections::BTreeSet::new();
        let mut pending = Vec::new();
        if !self.0.is_empty() {
            pending.push(0);
        }
        while let Some(index) = pending.pop() {
            let node = self.0.get(index).ok_or(SupportGeometryIssue::Query)?;
            if visited[index]
                || !node.minimum.is_finite()
                || !node.maximum.is_finite()
                || node.minimum.cmpgt(node.maximum).any()
            {
                return Err(SupportGeometryIssue::Query);
            }
            visited[index] = true;
            match &node.children {
                QueryChildren::Branch { left, right } => {
                    for child in [*left, *right] {
                        let child_node = self.0.get(child).ok_or(SupportGeometryIssue::Query)?;
                        if child_node.minimum.cmplt(node.minimum).any()
                            || child_node.maximum.cmpgt(node.maximum).any()
                        {
                            return Err(SupportGeometryIssue::Query);
                        }
                        pending.push(child);
                    }
                }
                QueryChildren::Leaf(triangles) => {
                    for reference in triangles {
                        let exists = match reference {
                            SupportTriangleRef::Natural(i) => *i < surface.natural_triangles.len(),
                            SupportTriangleRef::Foundation { owner, triangle } => surface
                                .foundations
                                .get(*owner)
                                .is_some_and(|f| *triangle < f.support_triangles.len()),
                        };
                        if !exists || !references.insert(*reference) {
                            return Err(SupportGeometryIssue::Query);
                        }
                        let bounds = TriangleQueryBounds::from_reference(*reference, surface);
                        if bounds.minimum.cmplt(node.minimum).any()
                            || bounds.maximum.cmpgt(node.maximum).any()
                        {
                            return Err(SupportGeometryIssue::Query);
                        }
                    }
                }
            }
        }
        if visited.iter().any(|v| !v)
            || references.len()
                != surface.natural_triangles.len()
                    + surface
                        .foundations
                        .iter()
                        .map(|f| f.support_triangles.len())
                        .sum::<usize>()
        {
            return Err(SupportGeometryIssue::Query);
        }
        Ok(())
    }

    pub fn compile(surface: &BoundedSettlementTerrain) -> Self {
        let mut triangles: Vec<_> = (0..surface.natural_triangles.len())
            .map(SupportTriangleRef::Natural)
            .chain(
                surface
                    .foundations
                    .iter()
                    .enumerate()
                    .flat_map(|(owner, foundation)| {
                        (0..foundation.support_triangles.len())
                            .map(move |triangle| SupportTriangleRef::Foundation { owner, triangle })
                    }),
            )
            .map(|reference| TriangleQueryBounds::from_reference(reference, surface))
            .collect();
        let mut index = Self::default();
        if !triangles.is_empty() {
            index.append(&mut triangles, None);
        }
        index
    }

    fn append(
        &mut self,
        triangles: &mut [TriangleQueryBounds],
        inherited_axis: Option<usize>,
    ) -> usize {
        let (minimum, maximum) = triangles.iter().fold(
            (Vec2::splat(f32::INFINITY), Vec2::splat(f32::NEG_INFINITY)),
            |(min, max), triangle| (min.min(triangle.minimum), max.max(triangle.maximum)),
        );
        let node = self.0.len();
        self.0.push(QueryNode {
            minimum,
            maximum,
            children: QueryChildren::Leaf(Vec::new()),
        });
        let children = if triangles.len() <= SUPPORT_QUERY_LEAF_TRIANGLES {
            // Retain the previous traversal and shared-edge tie order exactly.
            // A median partition does not otherwise order its two halves.
            if let Some(axis) = inherited_axis {
                triangles.sort_by(|a, b| compare_bounds(a, b, axis));
            }
            QueryChildren::Leaf(
                triangles
                    .iter()
                    .map(|triangle| triangle.reference)
                    .collect(),
            )
        } else {
            let extent = maximum - minimum;
            let axis = usize::from(extent.y > extent.x);
            let midpoint = triangles.len() / 2;
            // Only the median and the two exact membership sets are needed.
            // Sorting every branch repeats work across all ancestor levels.
            triangles.select_nth_unstable_by(midpoint, |a, b| compare_bounds(a, b, axis));
            let (left, right) = triangles.split_at_mut(midpoint);
            QueryChildren::Branch {
                left: self.append(left, Some(axis)),
                right: self.append(right, Some(axis)),
            }
        };
        self.0[node].children = children;
        node
    }

    pub fn triangles_at(
        &self,
        surface: &BoundedSettlementTerrain,
        point: Vec2,
        tolerance: f32,
    ) -> impl Iterator<Item = GroundTriangle> {
        let mut candidates =
            SmallVec::<[(SupportTriangleRef, GroundTriangle); SUPPORT_QUERY_LEAF_TRIANGLES]>::new();
        let mut owners = SmallVec::<[usize; 2]>::new();
        let rounding_tolerance = GroundTriangle::represented_point_tolerance(point);
        if !self.0.is_empty() {
            self.visit_candidates(0, point, tolerance, &mut |reference| {
                let Some(triangle) = GroundTriangle::new(reference.points(surface)) else {
                    return;
                };
                let eligible = match reference {
                    SupportTriangleRef::Natural(_) => {
                        // The same half-planes are nested by distance: testing
                        // their stricter margin is exactly the intersection.
                        triangle.contains(point, rounding_tolerance.min(tolerance))
                    }
                    SupportTriangleRef::Foundation { owner, .. } => {
                        // Ownership uses the represented point, independently
                        // of whether this face also lies in its contact margin.
                        let contact = triangle.contains(point, tolerance);
                        if !owners.contains(&owner) {
                            // Reuse only logical implications between nested
                            // margins. When ownership is stricter, a contact
                            // miss cannot own the point. When contact is
                            // stricter, its hit necessarily establishes owner.
                            let owns = if rounding_tolerance == tolerance {
                                contact
                            } else if rounding_tolerance < tolerance && !contact {
                                false
                            } else if rounding_tolerance > tolerance && contact {
                                true
                            } else {
                                triangle.contains(point, rounding_tolerance)
                            };
                            if owns {
                                owners.push(owner);
                            }
                        }
                        contact
                    }
                };
                if eligible {
                    candidates.push((reference, triangle));
                }
            });
        }
        // A property's contact margin applies only when one of its faces owns
        // the represented point. Natural faces have already passed both tests.
        // Filtering before buffering avoids retaining unrelated leaf triangles.
        candidates
            .into_iter()
            .filter_map(move |(reference, triangle)| {
                let include = match reference {
                    SupportTriangleRef::Foundation { owner, .. } => owners.contains(&owner),
                    SupportTriangleRef::Natural(_) => true,
                };
                include.then_some(triangle)
            })
    }

    fn visit_candidates(
        &self,
        index: usize,
        point: Vec2,
        tolerance: f32,
        visit: &mut impl FnMut(SupportTriangleRef),
    ) {
        // Encoded indexes need not have the compiler's balanced depth. Keep
        // traversal iterative, including for a deeply nested decoded tree.
        // A balanced compiler tree fits within address-bit depth. SmallVec
        // still grows for encoded trees, junction owners and extra candidates.
        let mut pending = SmallVec::<[usize; usize::BITS as usize]>::new();
        pending.push(index);
        while let Some(index) = pending.pop() {
            let node = &self.0[index];
            if point.cmplt(node.minimum - tolerance).any()
                || point.cmpgt(node.maximum + tolerance).any()
            {
                continue;
            }
            match &node.children {
                QueryChildren::Branch { left, right } => pending.extend([*right, *left]),
                QueryChildren::Leaf(triangles) => {
                    for reference in triangles {
                        visit(*reference);
                    }
                }
            }
        }
    }
}

fn compare_bounds(
    a: &TriangleQueryBounds,
    b: &TriangleQueryBounds,
    axis: usize,
) -> std::cmp::Ordering {
    a.centre_sum[axis]
        .total_cmp(&b.centre_sum[axis])
        .then(a.reference.cmp(&b.reference))
}

#[cfg(test)]
mod tests;
