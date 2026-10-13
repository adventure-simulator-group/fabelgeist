//! Resolve member solids once, then conservatively prune each candidate query.
use super::*;
use crate::geometry_index::BoundsIndex;

pub(super) struct OpeningObstacles<'a> {
    members: Vec<(crate::TimberMemberId, &'a ResolvedSolid)>,
    bounds: BoundsIndex,
}

impl<'a> OpeningObstacles<'a> {
    pub(super) fn new(
        members: &[crate::TimberFrameMember],
        solids: &'a [ResolvedSolid],
    ) -> Result<Self, crate::GenerationError> {
        let mut by_id = BTreeMap::new();
        for solid in solids {
            by_id.entry(solid.id).or_insert(solid);
        }
        let members: Vec<_> = members
            .iter()
            .filter_map(|member| by_id.get(&member.solid).map(|solid| (member.id, *solid)))
            .collect();
        let bounds = BoundsIndex::new(
            members
                .iter()
                .map(|(_, solid)| solid.cuboid_bounds())
                .collect::<Result<Vec<_>, _>>()?,
        )?;
        Ok(Self { members, bounds })
    }

    pub(super) fn intersects(
        &self,
        bounds: (Vec3, Vec3),
        tie: crate::TimberMemberId,
        head: crate::TimberMemberId,
    ) -> Result<bool, crate::GenerationError> {
        Ok(self
            .bounds
            .overlapping(SpatialBounds::<Architectural>::from_metres(
                bounds.0, bounds.1,
            )?)
            .into_iter()
            .any(|index| {
                let (id, solid) = self.members[index];
                id != tie
                    && id != head
                    && crate::solid_overlap::overlaps_bounds(
                        solid,
                        bounds,
                        CONTACT_TOLERANCE_METRES,
                    )
            }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn index_preserves_exhaustive_contact_results_for_sloped_roof_members() {
        for archetype in [
            BuildingArchetype::TownHouse,
            BuildingArchetype::FachwerkMerchantHouse,
        ] {
            for seed in [42, 47, 101].map(fabelgeist_determinism::Seed::from_u64) {
                let plan = crate::generate(&BuildingProgram::fixture(archetype, seed)).unwrap();
                let frame = plan.timber_frame.as_ref().unwrap();
                let solids = &plan.resolved_geometry.solids;
                let obstacles = OpeningObstacles::new(&frame.members, solids).unwrap();
                let excluded = [frame.members[0].id, frame.members[1].id];
                let mut contacts = 0;
                let mut misses = 0;
                for member in &frame.members {
                    for offset in [
                        Vec3::ZERO,
                        Vec3::splat(0.004),
                        Vec3::new(1.3, 0.7, -0.4),
                        Vec3::splat(50.0),
                    ] {
                        let centre = (member.start.metres() + member.end.metres()) * 0.5 + offset;
                        let half = Vec3::new(0.55, 0.5, 0.2);
                        let bounds = (centre - half, centre + half);
                        let exhaustive = frame
                            .members
                            .iter()
                            .filter(|member| !excluded.contains(&member.id))
                            .any(|member| {
                                solids
                                    .iter()
                                    .find(|solid| solid.id == member.solid)
                                    .is_some_and(|solid| {
                                        crate::solid_overlap::overlaps_bounds(
                                            solid,
                                            bounds,
                                            CONTACT_TOLERANCE_METRES,
                                        )
                                    })
                            });
                        assert_eq!(
                            obstacles
                                .intersects(bounds, excluded[0], excluded[1])
                                .unwrap(),
                            exhaustive,
                            "{archetype:?} seed {seed}, candidate {bounds:?}"
                        );
                        if exhaustive {
                            contacts += 1;
                        } else {
                            misses += 1;
                        }
                    }
                }
                assert!(contacts > 0 && misses > 0);
            }
        }
    }
}
