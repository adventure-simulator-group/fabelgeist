//! Conservative spatial candidates in source order. Callers retain their exact
//! geometry predicates and deterministic first-match policies.
use crate::{Architectural, SpatialBounds};
use bevy::math::{Quat, Vec3};

use crate::ResolvedSolid;

mod checked;
pub(crate) use checked::{try_all, try_any};

const MAX_LEAF_BOUNDS: usize = 8;

impl ResolvedSolid {
    pub(crate) fn yaw_bounds(
        &self,
    ) -> Result<SpatialBounds<Architectural>, crate::GenerationError> {
        let construct =
            || -> Result<SpatialBounds<Architectural>, crate::spatial_geometry::GeometryError> {
                let cosine = self.yaw_radians.radians().cos().abs();
                let sine = self.yaw_radians.radians().sin().abs();
                let half = Vec3::new(
                    (self.size.metres().x * cosine + self.size.metres().z * sine) * 0.5,
                    self.size.metres().y * 0.5,
                    (self.size.metres().x * sine + self.size.metres().z * cosine) * 0.5,
                );
                SpatialBounds::<Architectural>::from_metres(
                    self.centre.metres() - half,
                    self.centre.metres() + half,
                )
            };
        construct().map_err(|cause| {
            crate::CollisionError {
                source_id: self.id,
                cause,
            }
            .into()
        })
    }

    /// Covers both the yaw-only shape predicates and fully oriented cuboids.
    pub(crate) fn query_bounds(
        &self,
    ) -> Result<SpatialBounds<Architectural>, crate::GenerationError> {
        Ok(self.yaw_bounds()?.union(self.cuboid_bounds()?))
    }

    /// Separately rounded architectural envelope under the three authored rotations.
    /// Contact exclusion uses computed corners; this envelope retains its original
    /// f32 half-extent arithmetic for placement and broad-phase candidates.
    pub fn cuboid_bounds(&self) -> Result<SpatialBounds<Architectural>, crate::GenerationError> {
        let construct =
            || -> Result<SpatialBounds<Architectural>, crate::spatial_geometry::GeometryError> {
                let rotation = Quat::from_rotation_y(self.yaw_radians.radians())
                    * Quat::from_rotation_x(self.crossfall_radians.radians())
                    * Quat::from_rotation_z(self.longfall_radians.radians());
                let half = ((rotation * Vec3::X).abs() * self.size.metres().x
                    + (rotation * Vec3::Y).abs() * self.size.metres().y
                    + (rotation * Vec3::Z).abs() * self.size.metres().z)
                    * 0.5;
                SpatialBounds::<Architectural>::from_metres(
                    self.centre.metres() - half,
                    self.centre.metres() + half,
                )
            };
        construct().map_err(|cause| {
            crate::CollisionError {
                source_id: self.id,
                cause,
            }
            .into()
        })
    }
}

pub(crate) struct BoundsIndex {
    bounds: Vec<SpatialBounds<Architectural>>,
    root: Option<BoundsNode>,
}

pub(crate) fn overlapping_solids(
    solids: &[ResolvedSolid],
) -> Result<impl Iterator<Item = (&ResolvedSolid, &ResolvedSolid)>, crate::GenerationError> {
    let bounds = solids
        .iter()
        .map(ResolvedSolid::query_bounds)
        .collect::<Result<Vec<_>, _>>()?;
    let index = BoundsIndex::new(bounds)?;
    let pairs: Vec<_> = solids
        .iter()
        .enumerate()
        .flat_map(|(a, _)| {
            index
                .overlapping(index.bounds[a])
                .into_iter()
                .filter(move |&b| b > a)
                .map(move |b| (a, b))
        })
        .collect();
    Ok(pairs.into_iter().map(|(a, b)| (&solids[a], &solids[b])))
}

enum BoundsNode {
    Leaf {
        bounds: SpatialBounds<Architectural>,
        indices: Vec<usize>,
    },
    Branch {
        bounds: SpatialBounds<Architectural>,
        children: Box<[BoundsNode; 2]>,
    },
}

impl BoundsIndex {
    pub(crate) fn new(
        bounds: impl IntoIterator<Item = SpatialBounds<Architectural>>,
    ) -> Result<Self, crate::GenerationError> {
        let bounds: Vec<_> = bounds.into_iter().collect();
        let root = (!bounds.is_empty())
            .then(|| BoundsNode::new(&mut (0..bounds.len()).collect::<Vec<_>>(), &bounds))
            .transpose()?;
        Ok(Self { bounds, root })
    }

    pub(crate) fn overlapping(&self, query: SpatialBounds<Architectural>) -> Vec<usize> {
        let mut result = Vec::new();
        if let Some(root) = &self.root {
            root.query(query, &self.bounds, &mut result);
        }
        result.sort_unstable();
        result
    }
}

impl BoundsNode {
    fn new(
        indices: &mut [usize],
        source: &[SpatialBounds<Architectural>],
    ) -> Result<Self, crate::GenerationError> {
        let (&first, rest) = indices
            .split_first()
            .ok_or(crate::spatial_geometry::GeometryError::EmptySpatialPartition)?;
        let bounds = rest
            .iter()
            .fold(source[first], |bounds, &index| bounds.union(source[index]));
        if indices.len() <= MAX_LEAF_BOUNDS {
            return Ok(Self::Leaf {
                bounds,
                indices: indices.to_vec(),
            });
        }
        // Admission checks only arithmetic the branch actually uses. A finite
        // degenerate leaf at a large coordinate need not have a finite doubled
        // centre, since leaves never sort by that representation.
        let extent = bounds.extent()?.metres();
        let axis = (1..3).fold(0, |axis, next| {
            if extent[next].total_cmp(&extent[axis]).is_ge() {
                next
            } else {
                axis
            }
        });
        for &index in indices.iter() {
            source[index].centre()?;
        }
        indices.sort_unstable_by(|&a, &b| {
            let centre = |i: usize| source[i].min().metres()[axis] + source[i].max().metres()[axis];
            centre(a).total_cmp(&centre(b)).then(a.cmp(&b))
        });
        let middle = indices.len() / 2;
        let (left, right) = indices.split_at_mut(middle);
        Ok(Self::Branch {
            bounds,
            children: Box::new([Self::new(left, source)?, Self::new(right, source)?]),
        })
    }

    fn query(
        &self,
        query: SpatialBounds<Architectural>,
        source: &[SpatialBounds<Architectural>],
        result: &mut Vec<usize>,
    ) {
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
                SpatialBounds::<Architectural>::from_metres(min, min + Vec3::new(1.0, 2.0, 3.0))
                    .unwrap()
            })
            .collect();
        let index = BoundsIndex::new(boxes.iter().copied()).unwrap();
        for query in &boxes {
            let expected: Vec<_> = boxes
                .iter()
                .enumerate()
                .filter_map(|(i, b)| b.overlaps(*query).then_some(i))
                .collect();
            assert_eq!(index.overlapping(*query), expected);
        }
        assert!(
            BoundsIndex::new([])
                .unwrap()
                .overlapping(boxes[0])
                .is_empty()
        );
    }
}

#[cfg(test)]
mod admission_tests {
    use super::*;
    #[test]
    fn partition_admission_checks_only_arithmetic_used_by_its_topology() {
        let huge = SpatialBounds::<Architectural>::at(
            crate::spatial_geometry::Position::from_metres(Vec3::splat(f32::MAX)).unwrap(),
        );
        let leaf = BoundsIndex::new([huge]).unwrap();
        assert_eq!(leaf.overlapping(huge), vec![0]);
        assert!(matches!(
            BoundsIndex::new([huge; MAX_LEAF_BOUNDS + 1]),
            Err(crate::GenerationError::Geometry(
                crate::spatial_geometry::GeometryError::NonFinite {
                    role: crate::spatial_geometry::GeometryRole::BoundsCentre,
                    ..
                }
            ))
        ));
        let wide = SpatialBounds::<Architectural>::from_metres(
            Vec3::splat(-f32::MAX),
            Vec3::splat(f32::MAX),
        )
        .unwrap();
        assert!(matches!(
            BoundsIndex::new([wide; MAX_LEAF_BOUNDS + 1]),
            Err(crate::GenerationError::Geometry(
                crate::spatial_geometry::GeometryError::NonFinite {
                    role: crate::spatial_geometry::GeometryRole::BoundsExtent,
                    ..
                }
            ))
        ));
    }
}
