//! Conservative spatial candidates in source order. Callers retain their exact
//! geometry predicates and deterministic first-match policies.
use bevy::math::{Quat, Vec3};

use crate::{ResolvedBounds, ResolvedSolid};

const MAX_LEAF_BOUNDS: usize = 8;

impl ResolvedSolid {
    pub(crate) fn yaw_bounds(&self) -> ResolvedBounds {
        let cosine = self.yaw_radians.cos().abs();
        let sine = self.yaw_radians.sin().abs();
        let half = Vec3::new(
            (self.size.x * cosine + self.size.z * sine) * 0.5,
            self.size.y * 0.5,
            (self.size.x * sine + self.size.z * cosine) * 0.5,
        );
        ResolvedBounds {
            min: self.centre - half,
            max: self.centre + half,
        }
    }

    /// Covers both the yaw-only shape predicates and fully oriented cuboids.
    pub(crate) fn query_bounds(&self) -> ResolvedBounds {
        let rotation = Quat::from_rotation_y(self.yaw_radians)
            * Quat::from_rotation_x(self.crossfall_radians)
            * Quat::from_rotation_z(self.longfall_radians);
        let half = ((rotation * Vec3::X).abs() * self.size.x
            + (rotation * Vec3::Y).abs() * self.size.y
            + (rotation * Vec3::Z).abs() * self.size.z)
            * 0.5;
        self.yaw_bounds().union(ResolvedBounds {
            min: self.centre - half,
            max: self.centre + half,
        })
    }
}

impl ResolvedBounds {
    fn union(self, other: Self) -> Self {
        Self {
            min: self.min.min(other.min),
            max: self.max.max(other.max),
        }
    }

    fn overlaps(self, other: Self) -> bool {
        self.min.cmple(other.max).all() && other.min.cmple(self.max).all()
    }

    fn conservative(self) -> Self {
        if self.min.is_finite() && self.max.is_finite() && self.min.cmple(self.max).all() {
            self
        } else {
            // Invalid geometry must still reach the authoritative audit.
            Self {
                min: Vec3::NEG_INFINITY,
                max: Vec3::INFINITY,
            }
        }
    }
}

pub(crate) struct BoundsIndex {
    bounds: Vec<ResolvedBounds>,
    root: Option<BoundsNode>,
}

pub(crate) fn overlapping_solids(
    solids: &[ResolvedSolid],
) -> impl Iterator<Item = (&ResolvedSolid, &ResolvedSolid)> {
    let index = BoundsIndex::new(solids.iter().map(ResolvedSolid::query_bounds));
    let pairs: Vec<_> = solids
        .iter()
        .enumerate()
        .flat_map(|(a, solid)| {
            index
                .overlapping(solid.query_bounds())
                .into_iter()
                .filter(move |&b| b > a)
                .map(move |b| (a, b))
        })
        .collect();
    pairs.into_iter().map(|(a, b)| (&solids[a], &solids[b]))
}

enum BoundsNode {
    Leaf {
        bounds: ResolvedBounds,
        indices: Vec<usize>,
    },
    Branch {
        bounds: ResolvedBounds,
        children: Box<[BoundsNode; 2]>,
    },
}

impl BoundsIndex {
    pub(crate) fn new(bounds: impl IntoIterator<Item = ResolvedBounds>) -> Self {
        let bounds: Vec<_> = bounds
            .into_iter()
            .map(ResolvedBounds::conservative)
            .collect();
        let root = (!bounds.is_empty())
            .then(|| BoundsNode::new(&mut (0..bounds.len()).collect::<Vec<_>>(), &bounds));
        Self { bounds, root }
    }

    pub(crate) fn overlapping(&self, query: ResolvedBounds) -> Vec<usize> {
        let mut result = Vec::new();
        if let Some(root) = &self.root {
            root.query(query.conservative(), &self.bounds, &mut result);
        }
        result.sort_unstable();
        result
    }
}

impl BoundsNode {
    fn new(indices: &mut [usize], source: &[ResolvedBounds]) -> Self {
        let bounds = indices
            .iter()
            .map(|&i| source[i])
            .reduce(ResolvedBounds::union)
            .unwrap();
        if indices.len() <= MAX_LEAF_BOUNDS {
            return Self::Leaf {
                bounds,
                indices: indices.to_vec(),
            };
        }
        let extent = bounds.max - bounds.min;
        let axis = (0..3)
            .max_by(|&a, &b| extent[a].total_cmp(&extent[b]))
            .unwrap();
        indices.sort_unstable_by(|&a, &b| {
            let centre = |i: usize| source[i].min[axis] + source[i].max[axis];
            centre(a).total_cmp(&centre(b)).then(a.cmp(&b))
        });
        let middle = indices.len() / 2;
        let (left, right) = indices.split_at_mut(middle);
        Self::Branch {
            bounds,
            children: Box::new([Self::new(left, source), Self::new(right, source)]),
        }
    }

    fn query(&self, query: ResolvedBounds, source: &[ResolvedBounds], result: &mut Vec<usize>) {
        match self {
            Self::Leaf { bounds, indices } if bounds.overlaps(query) => {
                result.extend(
                    indices
                        .iter()
                        .copied()
                        .filter(|&i| source[i].overlaps(query)),
                );
            }
            Self::Branch { bounds, children } if bounds.overlaps(query) => {
                for child in children.iter() {
                    child.query(query, source, result);
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn candidates_match_brute_force_in_source_order_including_contacts() {
        let boxes: Vec<_> = (0..100)
            .rev()
            .map(|i| {
                let min = Vec3::new((i % 13) as f32 - 7.0, (i % 7) as f32, i as f32 * 0.1);
                ResolvedBounds {
                    min,
                    max: min + Vec3::new(1.0, 2.0, 3.0),
                }
            })
            .collect();
        let index = BoundsIndex::new(boxes.iter().copied());
        for query in &boxes {
            let expected: Vec<_> = boxes
                .iter()
                .enumerate()
                .filter_map(|(i, b)| b.overlaps(*query).then_some(i))
                .collect();
            assert_eq!(index.overlapping(*query), expected);
        }
        assert!(BoundsIndex::new([]).overlapping(boxes[0]).is_empty());
    }
}
