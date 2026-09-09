use super::*;
use crate::city_layout::CityStreetPatch;
use adventuresim_building_generator::furniture::{FurnitureKind, FurnitureVariant};
use adventuresim_world_schema::settlement_buildings::BuildingUse;
use fabelgeist_determinism::mix64;

const GROUP_DOMAIN: u64 = 0x6675_726e_6772_6f75;
const KIT_GAP_METRES: f32 = 0.45;
const FRONTAGE_GAP_METRES: f32 = 0.5;
const MARKET_EDGE_SLOTS: usize = 6;
const MARKET_ROWS: usize = 2;
const VENDOR_STOCK_INTERVAL: u64 = 3;
const VENDOR_STOCK_DOMAIN: u64 = 0x7665_6e64_7374_6f6b;

pub(super) struct LocalItem {
    pub key: FurnitureKey,
    pub offset: Vec2,
}

pub(super) struct Candidate {
    pub id: FurnitureGroupId,
    pub kind: FurnitureGroupKind,
    pub anchor: FurnitureAnchor,
    pub items: Vec<LocalItem>,
    pub footprint: FurnitureFootprint,
    pub market: Option<CityStreetPatch>,
}

impl Candidate {
    pub(super) fn new(
        seed: u64,
        slot: u64,
        kind: FurnitureGroupKind,
        anchor: FurnitureAnchor,
    ) -> Self {
        let (anchor_kind, anchor_id) = match anchor {
            FurnitureAnchor::Market { patch_index } => (0, u64::from(patch_index)),
            FurnitureAnchor::Building { id } => (1, id),
        };
        // Ordered tagged fields prevent swapped anchor/slot pairs from aliasing.
        let id = [anchor_kind, anchor_id, kind as u64, slot]
            .into_iter()
            .fold(mix64(seed ^ GROUP_DOMAIN), |state, field| {
                mix64(state ^ field)
            });
        let variant = if id & 1 == 0 {
            FurnitureVariant::Compact
        } else {
            FurnitureVariant::Broad
        };
        let kinds = match kind {
            FurnitureGroupKind::Vendor => [
                FurnitureKind::CanvasStall,
                if mix64(id ^ VENDOR_STOCK_DOMAIN).is_multiple_of(VENDOR_STOCK_INTERVAL) {
                    FurnitureKind::CargoStack
                } else {
                    FurnitureKind::Barrel
                },
            ],
            FurnitureGroupKind::Receiving => [FurnitureKind::CargoStack, FurnitureKind::Barrel],
            FurnitureGroupKind::HorseStop => {
                [FurnitureKind::HitchingTrough, FurnitureKind::TableBenchSet]
            }
        };
        let mut items = Vec::new();
        let mut minimum = Vec2::splat(f32::INFINITY);
        let mut maximum = Vec2::splat(f32::NEG_INFINITY);
        for kind in kinds {
            let key = FurnitureKey { kind, variant };
            let (min, max) = reservation_bounds(key);
            let offset = if items.is_empty() {
                Vec2::ZERO
            } else {
                Vec2::X * (maximum.x + KIT_GAP_METRES - min.x)
            };
            minimum = minimum.min(min + offset);
            maximum = maximum.max(max + offset);
            items.push(LocalItem { key, offset });
        }
        let centre = (minimum + maximum) * 0.5;
        for item in &mut items {
            item.offset -= centre;
        }
        Self {
            id: FurnitureGroupId(id),
            kind,
            anchor,
            items,
            market: None,
            footprint: FurnitureFootprint {
                centre_metres: Vec2::ZERO,
                half_extents_metres: (maximum - minimum) * 0.5,
                orientation: BuildingOrientation::IDENTITY,
            },
        }
    }
}

fn reservation_bounds(key: FurnitureKey) -> (Vec2, Vec2) {
    let recipe = key.recipe();
    let mut min = Vec2::new(recipe.bounds.min.x, recipe.bounds.min.z);
    let mut max = Vec2::new(recipe.bounds.max.x, recipe.bounds.max.z);
    for clearance in &recipe.clearances {
        min = min.min(Vec2::new(clearance.bounds.min.x, clearance.bounds.min.z));
        max = max.max(Vec2::new(clearance.bounds.max.x, clearance.bounds.max.z));
    }
    (min, max)
}

pub(super) fn market(input: &TacticalSceneInput) -> Vec<Candidate> {
    let mut candidates = Vec::new();
    for (index, patch) in input.streets.iter().enumerate() {
        let CityStreetPatch::Market {
            corners_metres: corners,
            ..
        } = *patch
        else {
            continue;
        };
        let centre = corners.into_iter().sum::<Vec2>() * 0.25;
        for edge in 0..4 {
            let start = corners[edge];
            let end = corners[(edge + 1) % 4];
            let mut tangent = (end - start).normalize_or_zero();
            if tangent == Vec2::ZERO {
                continue;
            }
            let mut inward = Vec2::new(-tangent.y, tangent.x);
            if inward.dot(centre - (start + end) * 0.5) < 0.0 {
                inward = -inward;
            }
            if Vec2::new(-tangent.y, tangent.x).dot(inward) > 0.0 {
                tangent = -tangent;
            }
            let orientation = BuildingOrientation::from_frontage_tangent(tangent).unwrap();
            let edge_clearance = reservations::market_edge_clearance(input, start, end, inward);
            for row in 0..MARKET_ROWS {
                for slot in 0..MARKET_EDGE_SLOTS {
                    let mut candidate = Candidate::new(
                        input.seed,
                        (edge * MARKET_ROWS * MARKET_EDGE_SLOTS + row * MARKET_EDGE_SLOTS + slot)
                            as u64,
                        FurnitureGroupKind::Vendor,
                        FurnitureAnchor::Market {
                            patch_index: index as u32,
                        },
                    );
                    let inset = edge_clearance
                        + FRONTAGE_GAP_METRES
                        + candidate.footprint.half_extents_metres.y * (1.0 + 2.0 * row as f32);
                    candidate.footprint.centre_metres = start
                        .lerp(end, (slot as f32 + 0.5) / MARKET_EDGE_SLOTS as f32)
                        + inward * inset;
                    candidate.footprint.orientation = orientation;
                    candidate.market = Some(*patch);
                    candidates.push(candidate);
                }
            }
        }
    }
    candidates
}

pub(super) fn building(input: &TacticalSceneInput, building: &GeneratedBuilding) -> Vec<Candidate> {
    let kind = match building.placement.program.usage {
        Some(BuildingUse::Inn | BuildingUse::Stable) => FurnitureGroupKind::HorseStop,
        Some(
            BuildingUse::Warehouse
            | BuildingUse::Granary
            | BuildingUse::Brewery
            | BuildingUse::Malthouse
            | BuildingUse::Bakehouse
            | BuildingUse::Carpenter
            | BuildingUse::TimberYard,
        ) => FurnitureGroupKind::Receiving,
        _ => return Vec::new(),
    };
    let half = building.collision.bounds.plan_half_extents();
    let mut candidates = Vec::new();
    // Candidate offsets follow each particular frontage, never a city-wide grid.
    for slot in 0..12 {
        let mut candidate = Candidate::new(
            input.seed,
            slot,
            kind,
            FurnitureAnchor::Building {
                id: building.placement.id,
            },
        );
        let size = candidate.footprint.half_extents_metres;
        let fraction = (slot % 3) as f32 * 0.5 - 0.5;
        let local = match slot / 3 {
            0 => Vec2::new(half.x * fraction, -half.y - size.y - FRONTAGE_GAP_METRES),
            1 => Vec2::new(half.x + size.x + FRONTAGE_GAP_METRES, half.y * fraction),
            2 => Vec2::new(-half.x - size.x - FRONTAGE_GAP_METRES, half.y * fraction),
            _ => Vec2::new(half.x * fraction, half.y + size.y + FRONTAGE_GAP_METRES),
        };
        candidate.footprint.centre_metres =
            building.placement.centre_metres + building.placement.orientation.local_to_world(local);
        candidate.footprint.orientation = building.placement.orientation;
        candidates.push(candidate);
    }
    candidates
}
