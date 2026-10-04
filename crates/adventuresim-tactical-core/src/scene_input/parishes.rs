//! Validate parish ownership against the complete near/distant building union.
use super::*;
use adventuresim_world_schema::settlement_buildings::{BuildingUse, ParishProminence};
use std::collections::{BTreeMap, BTreeSet};
#[cfg(test)]
mod tests;

struct Member {
    usage: Option<BuildingUse>,
    size: Option<adventuresim_building_generator::ServiceBuildingSize>,
    resident_capacity: u32,
    centre: bevy::math::Vec2,
}

impl Member {
    fn new(
        archetype: adventuresim_building_generator::BuildingArchetype,
        usage: Option<BuildingUse>,
        size: Option<adventuresim_building_generator::ServiceBuildingSize>,
        centre: bevy::math::Vec2,
    ) -> Self {
        let resident_capacity = crate::city_layout::CityHouseClass::ALL
            .into_iter()
            .find(|class| class.archetype() == archetype)
            .map_or(0, |class| class.resident_capacity());
        Self {
            usage,
            size,
            resident_capacity,
            centre,
        }
    }
}

pub(super) fn validate(input: &TacticalSceneInput) -> Result<(), SceneInputError> {
    if input.parishes.len() > crate::city_layout::MAX_CITY_LOTS {
        return invalid("scene exceeds parish count bound");
    }
    let buildings = input
        .buildings
        .iter()
        .map(|b| {
            (
                b.id,
                Member::new(
                    b.program.archetype,
                    b.program.usage,
                    b.program.service_size,
                    b.centre_metres,
                ),
            )
        })
        .chain(input.distant_buildings.iter().map(|b| {
            (
                b.id,
                Member::new(b.archetype, b.usage, b.service_size, b.centre_metres),
            )
        }))
        .collect::<BTreeMap<_, _>>();
    let mut ids = BTreeSet::new();
    let mut members = BTreeSet::new();
    let mut principals = 0;
    let mut schools = 0;
    for parish in &input.parishes {
        if !ids.insert(parish.programme.id) || parish.programme.population.0 == 0 {
            return invalid("parish identity or population is invalid");
        }
        let principal = parish.programme.prominence == ParishProminence::PrincipalTown;
        principals += usize::from(principal);
        schools += usize::from(parish.school_building_id.is_some());
        validate_members(parish, &buildings, &mut members)?;
    }
    if !input.parishes.is_empty() && (principals != 1 || schools > 1) {
        return invalid("town parish programme has an invalid principal or school count");
    }
    if !input.parishes.is_empty()
        && buildings.iter().any(|(id, building)| {
            matches!(
                building.usage,
                Some(
                    BuildingUse::ParishChurch
                        | BuildingUse::Rectory
                        | BuildingUse::School
                        | BuildingUse::Dwelling
                )
            ) && !members.contains(id)
        })
    {
        return invalid("settlement has a building without its parish association");
    }
    Ok(())
}

fn validate_members(
    parish: &crate::city_layout::CityParish,
    buildings: &BTreeMap<u64, Member>,
    members: &mut BTreeSet<u64>,
) -> Result<(), SceneInputError> {
    let expected_size = adventuresim_building_generator::ServiceBuildingSize::for_demand(
        adventuresim_world_schema::settlement_buildings::BuildingDemand::Parish {
            parish: parish.programme.id,
            role: adventuresim_world_schema::settlement_buildings::ParishBuildingRole::Church(
                parish.programme.church_scale,
            ),
        },
    );
    let Some(church) = buildings.get(&parish.church_building_id) else {
        return invalid("parish church is missing");
    };
    if church.usage != Some(BuildingUse::ParishChurch) || church.size != expected_size {
        return invalid("parish church recipe disagrees with its programme");
    }
    for (id, usage) in [
        (parish.church_building_id, BuildingUse::ParishChurch),
        (parish.rectory_building_id, BuildingUse::Rectory),
    ]
    .into_iter()
    .chain(
        parish
            .school_building_id
            .map(|id| (id, BuildingUse::School)),
    ) {
        if !members.insert(id) || buildings.get(&id).map(|b| b.usage) != Some(Some(usage)) {
            return invalid("parish member is missing, shared, or has the wrong use");
        }
    }
    if [Some(parish.rectory_building_id), parish.school_building_id]
        .into_iter()
        .flatten()
        .any(|id| {
            buildings[&id].centre.distance(church.centre)
                > crate::city_layout::CITY_PARISH_PRECINCT_RADIUS_METRES
        })
    {
        return invalid("parish support building is outside its precinct");
    }
    if parish.school_building_id.is_some()
        && parish.programme.prominence != ParishProminence::PrincipalTown
    {
        return invalid("town school must belong to the principal parish");
    }
    let mut residents = 0_u64;
    for allocation in &parish.residences {
        if allocation.residents.0 == 0
            || !members.insert(allocation.building_id)
            || buildings.get(&allocation.building_id).map(|b| b.usage)
                != Some(Some(BuildingUse::Dwelling))
        {
            return invalid("parish residential allocation is missing, shared, or not housing");
        }
        if allocation.residents.0 > buildings[&allocation.building_id].resident_capacity {
            return invalid("parish allocation exceeds physical housing capacity");
        }
        residents += u64::from(allocation.residents.0);
    }
    if residents != u64::from(parish.programme.population.0) {
        return invalid("parish population differs from its residential allocation");
    }
    Ok(())
}
