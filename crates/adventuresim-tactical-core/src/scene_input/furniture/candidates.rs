const RNG_FURNITURE_GROUP_SIZE: fabelgeist_determinism::StreamId =
    fabelgeist_determinism::StreamId::new("furniture.group-size");
use super::*;
use adventuresim_building_generator::furniture::FurnitureVariant;
use fabelgeist_determinism::StreamId;

mod composition;
mod frontage;
mod market;
pub(super) use frontage::{building, group_limit};
pub(super) use market::market;

const GROUP_DOMAIN: StreamId = StreamId::new("furniture.group-identity");
const KIT_GAP_METRES: f32 = 0.25;

pub(super) struct LocalItem {
    pub key: FurnitureKey,
    pub recipe: &'static adventuresim_building_generator::furniture::FurnitureRecipe,
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
    ) -> Result<Self, adventuresim_building_generator::furniture::FurnitureRecipeError> {
        let (anchor_kind, anchor_id) = match anchor {
            FurnitureAnchor::Market { patch_index } => (0, u64::from(patch_index)),
            FurnitureAnchor::Building { id } => (1, id),
        };
        let id = GROUP_DOMAIN
            .seed(seed, &[anchor_kind, anchor_id, kind as u64, slot])
            .to_u64();
        let variant = if RNG_FURNITURE_GROUP_SIZE.rng(id, &[]).boolean() {
            FurnitureVariant::Compact
        } else {
            FurnitureVariant::Broad
        };
        let mut items = Vec::new();
        let mut minimum = Vec2::splat(f32::INFINITY);
        let mut maximum = Vec2::splat(f32::NEG_INFINITY);
        for kind in composition::kinds(kind, id) {
            let key = FurnitureKey::natural(kind, variant);
            let recipe = key.recipe()?;
            let (min, max) = reservation_bounds(recipe);
            let offset = if items.is_empty() {
                Vec2::ZERO
            } else {
                Vec2::X * (maximum.x + KIT_GAP_METRES - min.x)
            };
            minimum = minimum.min(min + offset);
            maximum = maximum.max(max + offset);
            items.push(LocalItem {
                key,
                offset,
                recipe,
            });
        }
        let centre = (minimum + maximum) * 0.5;
        for item in &mut items {
            item.offset -= centre;
        }
        Ok(Self {
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
        })
    }
}

pub(super) fn reservation_bounds(
    recipe: &adventuresim_building_generator::furniture::FurnitureRecipe,
) -> (Vec2, Vec2) {
    let mut min = Vec2::new(
        recipe.bounds.min().metres().x,
        recipe.bounds.min().metres().z,
    );
    let mut max = Vec2::new(
        recipe.bounds.max().metres().x,
        recipe.bounds.max().metres().z,
    );
    for clearance in &recipe.clearances {
        min = min.min(Vec2::new(
            clearance.bounds.min().metres().x,
            clearance.bounds.min().metres().z,
        ));
        max = max.max(Vec2::new(
            clearance.bounds.max().metres().x,
            clearance.bounds.max().metres().z,
        ));
    }
    (min, max)
}
