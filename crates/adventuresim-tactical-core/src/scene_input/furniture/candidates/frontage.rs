//! Bounded local frontage searches retain door, street and neighbouring-lot access.
use super::*;
use adventuresim_world_schema::settlement_buildings::BuildingUse;

const FRONTAGE_GAP_METRES: f32 = 0.35;
const FRONTAGE_SAMPLES: u64 = 5;
const FRONTAGE_END_FRACTION: f32 = 0.8;

fn kind(building: &sites::FurnitureSite) -> Option<FurnitureGroupKind> {
    use BuildingUse::*;
    match building.placement.program.usage? {
        Inn | Stable => Some(FurnitureGroupKind::HorseStop),
        Warehouse | Granary | Brewery | Malthouse | Bakehouse | TimberYard | WoadStore | Barn
        | WeighHouse | CustomsHouse => Some(FurnitureGroupKind::Receiving),
        Dwelling | Rectory | Manor => Some(FurnitureGroupKind::Domestic),
        GeneralShop | Smithy | Weaponsmith | Armorer | Tailor | Herbalist | Bookshop | Butcher
        | Cooper | Carpenter | Wheelwright | Cobbler | Weaver | Tannery | Dyer | Ropemaker
        | Chandler | Potter | Stonecutter | Fishmonger | Apothecary | PrintingHouse | WaterMill
        | FullingMill | PaperMill | Sawmill | HorseMill | Windmill | Mint | SaltWorks | Smelter
        | AssayHouse | Brickworks | Glassworks => Some(FurnitureGroupKind::Workshop),
        _ => None,
    }
}

pub(in crate::scene_input::furniture) fn group_limit(building: &sites::FurnitureSite) -> usize {
    match kind(building) {
        Some(FurnitureGroupKind::HorseStop) => 1,
        Some(_) => 2,
        None => 0,
    }
}

pub(in crate::scene_input::furniture) fn building(
    input: &TacticalSceneInput,
    building: &sites::FurnitureSite,
) -> Result<Vec<Candidate>, crate::scene_input::SceneInputError> {
    let Some(kind) = kind(building) else {
        return Ok(Vec::new());
    };
    let half = building.half_extents.metres();
    let mut candidates = Vec::new();
    for slot in 0..FRONTAGE_SAMPLES * 4 {
        let candidate = LocalCandidate::new(
            input.seed,
            slot,
            kind,
            FurnitureAnchor::Building {
                id: building.placement.id,
            },
        )?;
        let size = candidate.half_extents.metres();
        let fraction = ((slot % FRONTAGE_SAMPLES) as f32 / (FRONTAGE_SAMPLES - 1) as f32 * 2.0
            - 1.0)
            * FRONTAGE_END_FRACTION;
        let local = match slot / FRONTAGE_SAMPLES {
            0 => Vec2::new(half.x * fraction, -half.y - size.y - FRONTAGE_GAP_METRES),
            1 => Vec2::new(half.x + size.x + FRONTAGE_GAP_METRES, half.y * fraction),
            2 => Vec2::new(-half.x - size.x - FRONTAGE_GAP_METRES, half.y * fraction),
            _ => Vec2::new(half.x * fraction, half.y + size.y + FRONTAGE_GAP_METRES),
        };
        let candidate = candidate.place(
            crate::scene_coordinates::ScenePlanPoint::try_from(
                building.placement.centre_metres.metres()
                    + building.placement.orientation.local_to_world(local),
            )?,
            building.placement.orientation,
        )?;
        candidates.push(candidate);
    }
    Ok(candidates)
}
