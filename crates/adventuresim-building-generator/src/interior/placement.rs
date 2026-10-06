use crate::interior::InteriorResult as Result;
const RNG_BUILDING_FURNITURE_SIZE: fabelgeist_determinism::StreamId =
    fabelgeist_determinism::StreamId::new("building.furniture-size");
use super::budgets::{FurnitureBudget, FurniturePosition, furniture_budgets};
use super::navigation::{Navigation, Occupancy};
use super::{
    FurnitureAccessPath, InteriorLayout, InteriorLayoutError, InteriorPlacement,
    UnmetFurnitureBudget,
};
use crate::furniture::{FurnitureKey, FurnitureVariant};
use crate::{BuildingPlan, BuildingProgram, Direction, Room, StoreyIndex};
use bevy::math::Vec2;

const WALL_SETBACK_METRES: f32 = crate::WALL_THICKNESS_METRES * 0.5 + 0.06;
const CANDIDATE_STEP_METRES: f32 = 0.5;

pub fn furnish(plan: &BuildingPlan, program: &BuildingProgram) -> Result<InteriorLayout> {
    let nav = Navigation::new(plan)?;
    let mut occupancy = Occupancy::new(&nav)?;
    let mut layout = InteriorLayout::default();
    for storey in &plan.storeys {
        for room in &storey.rooms {
            for budget in furniture_budgets(program, room) {
                let mut placed = 0;
                let candidates = candidates(
                    plan,
                    program,
                    room,
                    StoreyIndex::from_serialized(storey.level),
                    budget,
                )?;
                for group in candidates {
                    if placed == budget.count {
                        break;
                    }
                    let previous = layout.placements.len();
                    layout.placements.extend(group);
                    if super::footprints::validate(plan, &nav, &layout.placements, previous)
                        .is_err()
                    {
                        layout.placements.truncate(previous);
                        continue;
                    }
                    let change = occupancy.add(&layout.placements[previous..])?;
                    let flood = occupancy.flood();
                    if nav.verify_rooms(&flood).is_err()
                        || nav.access_paths(&layout.placements, &flood).is_err()
                    {
                        occupancy.remove(change);
                        layout.placements.truncate(previous);
                        continue;
                    }
                    placed += 1;
                }
                if placed < budget.count {
                    layout.unmet_budgets.push(UnmetFurnitureBudget {
                        storey: crate::StoreyIndex::from_serialized(storey.level),
                        room_id: crate::RoomIndex::from_serialized(room.id),
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
    layout.paths = nav.access_paths(&layout.placements, &occupancy.flood())?;
    super::finishes::assign(plan, program, &mut layout.placements)?;
    Ok(layout)
}

/// Rebuild the proof from placements and authoritative architecture; serialized paths are not trusted.
pub fn validate_layout(
    plan: &BuildingPlan,
    layout: &InteriorLayout,
) -> Result<Vec<FurnitureAccessPath>> {
    let nav = Navigation::new(plan)?;
    super::footprints::validate(plan, &nav, &layout.placements, 0)?;
    if layout.placements.is_empty() {
        return Err(InteriorLayoutError::EmptyLayout);
    }
    let flood = nav.flood(&layout.placements)?;
    nav.verify_rooms(&flood)?;
    nav.access_paths(&layout.placements, &flood)
}

pub(super) fn candidates(
    plan: &BuildingPlan,
    program: &BuildingProgram,
    room: &Room,
    storey: StoreyIndex,
    budget: FurnitureBudget,
) -> Result<Vec<Vec<InteriorPlacement>>> {
    let seed = fabelgeist_determinism::StreamId::new("building.room-furniture")
        .seed(
            program.seed.into(),
            &[u64::from(room.id), u64::from(storey.serialized_ordinal()?)],
        )
        .to_u64();
    let variants = if RNG_BUILDING_FURNITURE_SIZE
        .rng(seed.into(), &[budget.kind as u64])
        .boolean()
    {
        vec![FurnitureVariant::Broad, FurnitureVariant::Compact]
    } else {
        vec![FurnitureVariant::Compact]
    };
    let mut all = Vec::new();
    for variant in variants {
        all.extend(variant_candidates(
            plan, room, storey, budget, variant, seed,
        )?);
    }
    Ok(all)
}

fn variant_candidates(
    plan: &BuildingPlan,
    room: &Room,
    storey: StoreyIndex,
    budget: FurnitureBudget,
    variant: FurnitureVariant,
    seed: u64,
) -> Result<Vec<Vec<InteriorPlacement>>> {
    let key = FurnitureKey::natural(budget.kind, variant);
    let bounds = super::geometry::RoomBounds::from_room(room, storey)?;
    let (min, max) = (bounds.min.metres(), bounds.max.metres());
    let mut choices = Vec::new();
    let preferred_facing = super::room_facing::preferred_facing(plan, room, budget.kind, bounds)?;
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
            room_id: crate::RoomIndex::from_serialized(room.id),
            storey,
            centre_metres: crate::plan_geometry::ArchitecturalPlanPoint::try_from(Vec2::ZERO)?,
            facing,
        };
        let prototype = super::composition::compose(template.clone())?;
        let mut group_min = Vec2::splat(f32::INFINITY);
        let mut group_max = Vec2::splat(f32::NEG_INFINITY);
        for placement in &prototype {
            let footprint = placement.footprint()?;
            group_min = group_min.min(footprint.centre.metres() - footprint.half.metres());
            group_max = group_max.max(footprint.centre.metres() + footprint.half.metres());
        }
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
                        crate::plan_geometry::ArchitecturalPlanPoint::from_metres(centre)?,
                        crate::plan_geometry::ArchitecturalPlanPoint::from_metres(
                            (min + max) * 0.5,
                        )?,
                    ),
                    centre_metres: crate::plan_geometry::ArchitecturalPlanPoint::try_from(centre)?,
                    ..template.clone()
                };
                if !p.footprint()?.inside_room(room)? {
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
                let tie = fabelgeist_determinism::StreamId::new("building.furniture-placement")
                    .rng(
                        seed.into(),
                        &[
                            u64::from(x),
                            u64::from(z),
                            facing as u64,
                            budget.kind as u64,
                            variant as u64,
                        ],
                    )
                    .next_u64();
                let score = super::room_facing::placement_score(plan, &p, bounds, score)?;
                if score.is_finite() {
                    choices.push((score, tie, super::composition::compose(p)?));
                }
            }
        }
    }
    choices.sort_by(|a, b| a.0.total_cmp(&b.0).then_with(|| a.1.cmp(&b.1)));
    Ok(choices.into_iter().map(|(_, _, p)| p).collect())
}
