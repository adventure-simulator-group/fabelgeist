use super::*;
use adventuresim_building_generator::furniture::FurnitureVariant;
use fabelgeist_determinism::mix64;

mod composition;
mod frontage;
mod market;
pub(super) use frontage::{building, group_limit};
pub(super) use market::market;

const GROUP_DOMAIN: u64 = 0x6675_726e_6772_6f75;
const KIT_GAP_METRES: f32 = 0.25;

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
    pub market: Option<crate::city_layout::CityStreetPatch>,
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
        let mut items = Vec::new();
        let mut minimum = Vec2::splat(f32::INFINITY);
        let mut maximum = Vec2::splat(f32::NEG_INFINITY);
        for kind in composition::kinds(kind, id) {
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

pub(super) fn reservation_bounds(key: FurnitureKey) -> (Vec2, Vec2) {
    let recipe = key.recipe();
    let mut min = Vec2::new(recipe.bounds.min.x, recipe.bounds.min.z);
    let mut max = Vec2::new(recipe.bounds.max.x, recipe.bounds.max.z);
    for clearance in &recipe.clearances {
        min = min.min(Vec2::new(clearance.bounds.min.x, clearance.bounds.min.z));
        max = max.max(Vec2::new(clearance.bounds.max.x, clearance.bounds.max.z));
    }
    (min, max)
}
