//! Incremental checks retain the architecture proof of the accepted prefix.
use super::{InteriorLayoutError, InteriorPlacement, navigation::Navigation};
use crate::BuildingPlan;
use crate::interior::InteriorResult as Result;

/// `accepted` placements must already have passed this check against this plan.
/// Use zero when validating an untrusted or edited layout.
pub(super) fn validate(
    plan: &BuildingPlan,
    nav: &Navigation,
    placements: &[InteriorPlacement],
    accepted: usize,
) -> Result<()> {
    for (index, p) in placements.iter().enumerate() {
        // Accepted objects already satisfy static architecture and each other.
        // New objects can still obstruct their usable faces.
        if index < accepted {
            for &face in p.key.interior_spec()?.required_faces {
                let access = p.access_rect(face)?;
                if overlapping(
                    placements[accepted..]
                        .iter()
                        .filter(|q| q.storey == p.storey),
                    access,
                )? {
                    return Err(InteriorLayoutError::InaccessibleFurniture { index });
                }
            }
            continue;
        }
        let error = InteriorLayoutError::InvalidPlacement { index };
        let specification = p.key.interior_spec()?;
        if !p.centre_metres.metres().is_finite() {
            return Err(error);
        }
        let floor = nav
            .floors
            .iter()
            .find(|f| f.level == p.storey)
            .ok_or_else(|| error.clone())?;
        let room = plan
            .storeys
            .iter()
            .find(|s| crate::StoreyIndex::from_serialized(s.level) == p.storey)
            .and_then(|s| {
                s.rooms
                    .iter()
                    .find(|r| crate::RoomIndex::from_serialized(r.id) == p.room_id)
            })
            .ok_or_else(|| error.clone())?;
        let footprint = p.footprint()?;
        let elevation = floor
            .height_at(p.centre_metres)
            .ok_or_else(|| error.clone())?;
        if !floor.supports(footprint, elevation)?
            || !floor.placement_clear(
                footprint,
                elevation,
                crate::spatial_geometry::PositiveLength::from_metres(
                    specification.size_metres.metres().y,
                )?,
            )?
        {
            return Err(error);
        }
        if !footprint.inside_room(room)?
            || floor
                .obstacles
                .iter()
                .chain(&floor.reserved)
                .any(|o| o.overlaps(footprint))
            || overlapping(
                placements[..index].iter().filter(|q| q.storey == p.storey),
                footprint,
            )?
        {
            return Err(error);
        }
        for &face in specification.required_faces {
            let access = p.access_rect(face)?;
            if !access.inside_room(room)?
                || floor.obstacles.iter().any(|o| o.overlaps(access))
                || overlapping(
                    placements
                        .iter()
                        .enumerate()
                        .filter(|(j, q)| *j != index && q.storey == p.storey)
                        .map(|(_, placement)| placement),
                    access,
                )?
            {
                return Err(InteriorLayoutError::InaccessibleFurniture { index });
            }
        }
    }
    Ok(())
}

fn overlapping<'a>(
    placements: impl Iterator<Item = &'a InteriorPlacement>,
    rect: super::geometry::Rect,
) -> Result<bool> {
    for placement in placements {
        if placement.footprint()?.overlaps(rect) {
            return Ok(true);
        }
    }
    Ok(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interior::{furniture_budgets, placement::candidates};
    use crate::{BuildingArchetype, BuildingProgram};

    #[test]
    fn incremental_candidates_match_full_architectural_checks() {
        let mut accepted_candidates = 0;
        let mut rejected_candidates = 0;
        for archetype in [
            BuildingArchetype::TownHouse,
            BuildingArchetype::FachwerkMerchantHouse,
        ] {
            for seed in [42, 47] {
                let program = BuildingProgram::fixture(archetype, seed);
                let plan = crate::generate(&program).unwrap();
                let nav = Navigation::new(&plan).unwrap();
                let mut placements = Vec::new();
                for storey in &plan.storeys {
                    for room in &storey.rooms {
                        for budget in furniture_budgets(&program, room) {
                            for group in candidates(
                                &plan,
                                &program,
                                room,
                                crate::StoreyIndex::from_serialized(storey.level),
                                budget,
                            )
                            .unwrap()
                            {
                                let previous = placements.len();
                                placements.extend(group);
                                let full = validate(&plan, &nav, &placements, 0);
                                let incremental = validate(&plan, &nav, &placements, previous);
                                assert_eq!(full.is_ok(), incremental.is_ok());
                                if full.is_err() {
                                    rejected_candidates += 1;
                                    placements.truncate(previous);
                                } else {
                                    accepted_candidates += 1;
                                }
                            }
                        }
                    }
                }
            }
        }
        assert!(accepted_candidates > 0 && rejected_candidates > 0);
    }
}
