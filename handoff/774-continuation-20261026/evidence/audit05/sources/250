//! Bounded household-to-home projection; membership does not imply ownership.
use super::*;
use crate::relationship::household_member__view;
use crate::residence::residence_occupant__view;

#[derive(Clone, Debug, SpacetimeType)]
pub struct BackendHouseholdPropertyOccupancy {
    pub household_id: String,
    pub property_id: String,
    pub materialized_residents: u32,
    pub unmaterialized_residents: u32,
}

#[view(accessor = backend_household_property_occupancies, public)]
pub fn backend_household_property_occupancies(
    ctx: &ViewContext,
) -> Vec<BackendHouseholdPropertyOccupancy> {
    if !residence_view_is_gateway(ctx) {
        return Vec::new();
    }
    let mut homes = std::collections::BTreeMap::new();
    for occupancy in ctx
        .db
        .household_property_occupancy()
        .property_id()
        .filter(""..)
    {
        homes.insert(
            (
                occupancy.household_id.clone(),
                occupancy.property_id.clone(),
            ),
            BackendHouseholdPropertyOccupancy {
                household_id: occupancy.household_id,
                property_id: occupancy.property_id,
                materialized_residents: 0,
                unmaterialized_residents: occupancy.unmaterialized_residents,
            },
        );
    }
    for occupant in ctx.db.residence_occupant().property_id().filter(""..) {
        let Some(member) = ctx
            .db
            .household_member()
            .character_id()
            .find(occupant.character_id)
        else {
            continue;
        };
        let key = (member.household_id.clone(), occupant.property_id.clone());
        homes
            .entry(key)
            .or_insert_with(|| BackendHouseholdPropertyOccupancy {
                household_id: member.household_id,
                property_id: occupant.property_id,
                materialized_residents: 0,
                unmaterialized_residents: 0,
            })
            .materialized_residents += 1;
    }
    homes.into_values().collect()
}
