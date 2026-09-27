//! Incremental checks retain the architecture proof of the accepted prefix.
use super::{InteriorLayoutError, InteriorPlacement, navigation::Navigation};
use crate::BuildingPlan;

/// `accepted` placements must already have passed this check against this plan.
/// Use zero when validating an untrusted or edited layout.
pub(super) fn validate(
    plan: &BuildingPlan,
    nav: &Navigation,
    placements: &[InteriorPlacement],
    accepted: usize,
) -> Result<(), InteriorLayoutError> {
    for (index, p) in placements.iter().enumerate() {
        // Accepted objects already satisfy static architecture and each other.
        // New objects can still obstruct their usable faces.
        if index < accepted {
            for &face in p
                .key
                .interior_spec()
                .expect("validated prefix")
                .required_faces
            {
                let access = p.access_rect(face);
                if placements[accepted..]
                    .iter()
                    .any(|q| q.storey == p.storey && q.footprint().overlaps(access))
                {
                    return Err(InteriorLayoutError::InaccessibleFurniture { index });
                }
            }
            continue;
        }
        let error = InteriorLayoutError::InvalidPlacement { index };
        if p.key.interior_spec().is_none() || !p.centre_metres.is_finite() {
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
            .find(|s| s.level == p.storey)
            .and_then(|s| s.rooms.iter().find(|r| r.id == p.room_id))
            .ok_or_else(|| error.clone())?;
        let footprint = p.footprint();
        let elevation = floor
            .height_at(p.centre_metres)
            .ok_or_else(|| error.clone())?;
        if !floor.supports(footprint, elevation)
            || !floor.placement_clear(
                footprint,
                elevation,
                p.key.interior_spec().unwrap().size_metres.y,
            )
        {
            return Err(error);
        }
        if !footprint.inside_room(room)
            || floor
                .obstacles
                .iter()
                .chain(&floor.reserved)
                .any(|o| o.overlaps(footprint))
            || placements[..index]
                .iter()
                .any(|q| q.storey == p.storey && q.footprint().overlaps(footprint))
        {
            return Err(error);
        }
        for &face in p.key.interior_spec().unwrap().required_faces {
            let access = p.access_rect(face);
            if !access.inside_room(room)
                || floor.obstacles.iter().any(|o| o.overlaps(access))
                || placements.iter().enumerate().any(|(j, q)| {
                    j != index && q.storey == p.storey && q.footprint().overlaps(access)
                })
            {
                return Err(InteriorLayoutError::InaccessibleFurniture { index });
            }
        }
    }
    Ok(())
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
                            for group in candidates(&plan, &program, room, storey.level, budget) {
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
