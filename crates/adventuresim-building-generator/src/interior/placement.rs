use super::budgets::{FurnitureBudget, FurniturePosition, furniture_budgets};
use super::navigation::Navigation;
use super::{
    FurnitureAccessPath, InteriorLayout, InteriorLayoutError, InteriorPlacement,
    UnmetFurnitureBudget,
};
use crate::furniture::{FurnitureKey, FurnitureVariant};
use crate::{BuildingPlan, BuildingProgram, Direction, Room};
use bevy::math::Vec2;

const WALL_SETBACK_METRES: f32 = crate::WALL_THICKNESS_METRES * 0.5 + 0.06;
const CANDIDATE_STEP_METRES: f32 = 0.5;

pub fn furnish(
    plan: &BuildingPlan,
    program: &BuildingProgram,
) -> Result<InteriorLayout, InteriorLayoutError> {
    let nav = Navigation::new(plan)?;
    let mut layout = InteriorLayout::default();
    for storey in &plan.storeys {
        for room in &storey.rooms {
            for budget in furniture_budgets(program, room) {
                let mut placed = 0;
                let candidates = candidates(plan, program, room, storey.level, budget);
                for group in candidates {
                    if placed == budget.count {
                        break;
                    }
                    let previous = layout.placements.len();
                    layout.placements.extend(group);
                    if footprints_valid(plan, &nav, &layout.placements).is_err() {
                        layout.placements.truncate(previous);
                        continue;
                    }
                    let flood = nav.flood(&layout.placements);
                    if nav.verify_rooms(&flood).is_err()
                        || nav.access_paths(&layout.placements, &flood).is_err()
                    {
                        layout.placements.truncate(previous);
                        continue;
                    }
                    placed += 1;
                }
                if placed < budget.count {
                    layout.unmet_budgets.push(UnmetFurnitureBudget {
                        storey: storey.level,
                        room_id: room.id,
                        kind: budget.kind,
                        requested: budget.count,
                        placed,
                    });
                }
            }
        }
    }
    if layout.placements.is_empty() {
        return Err(InteriorLayoutError::EmptyLayout);
    }
    layout.paths = nav.access_paths(&layout.placements, &nav.flood(&layout.placements))?;
    Ok(layout)
}

/// Rebuild the proof from placements and authoritative architecture; serialized paths are not trusted.
pub fn validate_layout(
    plan: &BuildingPlan,
    layout: &InteriorLayout,
) -> Result<Vec<FurnitureAccessPath>, InteriorLayoutError> {
    let nav = Navigation::new(plan)?;
    footprints_valid(plan, &nav, &layout.placements)?;
    if layout.placements.is_empty() {
        return Err(InteriorLayoutError::EmptyLayout);
    }
    let flood = nav.flood(&layout.placements);
    nav.verify_rooms(&flood)?;
    nav.access_paths(&layout.placements, &flood)
}

pub(super) fn footprints_valid(
    plan: &BuildingPlan,
    nav: &Navigation,
    placements: &[InteriorPlacement],
) -> Result<(), InteriorLayoutError> {
    for (index, p) in placements.iter().enumerate() {
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

pub(super) fn candidates(
    plan: &BuildingPlan,
    program: &BuildingProgram,
    room: &Room,
    storey: u16,
    budget: FurnitureBudget,
) -> Vec<Vec<InteriorPlacement>> {
    let seed = fabelgeist_determinism::mix64(
        program.seed ^ u64::from(room.id) ^ (u64::from(storey) << 32),
    );
    let variants = if seed.is_multiple_of(2) {
        vec![FurnitureVariant::Broad, FurnitureVariant::Compact]
    } else {
        vec![FurnitureVariant::Compact]
    };
    let mut all = Vec::new();
    for variant in variants {
        all.extend(variant_candidates(
            plan, room, storey, budget, variant, seed,
        ));
    }
    all
}

fn variant_candidates(
    plan: &BuildingPlan,
    room: &Room,
    storey: u16,
    budget: FurnitureBudget,
    variant: FurnitureVariant,
    seed: u64,
) -> Vec<Vec<InteriorPlacement>> {
    let key = FurnitureKey {
        kind: budget.kind,
        variant,
    };
    let (min, max) = super::geometry::room_bounds(room);
    let mut choices = Vec::new();
    let preferred_facing = super::room_facing::preferred_facing(plan, room, budget.kind, min, max);
    for facing in [
        Direction::South,
        Direction::North,
        Direction::East,
        Direction::West,
    ] {
        if preferred_facing.is_some_and(|direction| direction != facing) {
            continue;
        }
        let template = InteriorPlacement {
            key,
            room_id: room.id,
            storey,
            centre_metres: Vec2::ZERO,
            facing,
        };
        let prototype = super::composition::compose(template.clone());
        let group_min = prototype
            .iter()
            .map(|p| p.footprint().centre - p.footprint().half)
            .fold(Vec2::splat(f32::INFINITY), Vec2::min);
        let group_max = prototype
            .iter()
            .map(|p| p.footprint().centre + p.footprint().half)
            .fold(Vec2::splat(f32::NEG_INFINITY), Vec2::max);
        let start = min - group_min + Vec2::splat(WALL_SETBACK_METRES);
        let end = max - group_max - Vec2::splat(WALL_SETBACK_METRES);
        if start.cmpgt(end).any() {
            continue;
        }
        let steps = ((end - start) / CANDIDATE_STEP_METRES).ceil().as_uvec2();
        for x in 0..=steps.x {
            for z in 0..=steps.y {
                let centre = start
                    + (Vec2::new(x as f32, z as f32) * CANDIDATE_STEP_METRES).min(end - start);
                let p = InteriorPlacement {
                    facing: super::room_facing::facing_at(
                        room,
                        budget.kind,
                        facing,
                        centre,
                        (min + max) * 0.5,
                    ),
                    centre_metres: centre,
                    ..template.clone()
                };
                if !p.footprint().inside_room(room) {
                    continue;
                }
                let wall_distance = (centre + group_min - min)
                    .min(max - centre - group_max)
                    .min_element();
                let score = match budget.position {
                    FurniturePosition::Wall => wall_distance,
                    FurniturePosition::CounterRun | FurniturePosition::Centre => {
                        centre.distance((min + max) * 0.5)
                    }
                    FurniturePosition::Rows => centre.y - min.y + (centre.x - min.x) * 0.01,
                };
                let tie = fabelgeist_determinism::mix64(
                    seed ^ u64::from(x) ^ (u64::from(z) << 24) ^ (choices.len() as u64),
                );
                choices.push((score, tie, super::composition::compose(p)));
            }
        }
    }
    choices.sort_by(|a, b| a.0.total_cmp(&b.0).then_with(|| a.1.cmp(&b.1)));
    choices.into_iter().map(|(_, _, p)| p).collect()
}
