//! Immutable generated homes and transactional physical allocation.

pub use households::{BackendHouseholdPropertyOccupancy, backend_household_property_occupancies};
pub(crate) use materialization::bind_generated_households;
use materialization::{GeneratedResidentHome, generated_resident_home};

use super::occupancy::{property_occupancy_transition, property_occupancy_transition__view};
use super::*;
use crate::relationship::{Household, household};
use crate::settlement_population::settlement_resident_profile;
use adventuresim_core::{
    reputation::effective_population, settlement_property::GeneratedHomeCatalog,
};
mod households;
mod materialization;

#[derive(Clone, Debug)]
#[table(accessor = settlement_property_manifest, public)]
pub struct SettlementPropertyManifest {
    #[primary_key]
    pub settlement_id: String,
    pub digest: String,
}

/// Registered geometry is immutable. A changed generator must not silently
/// reassign existing holdings to another parcel.
#[derive(Clone, Debug)]
#[table(accessor = settlement_property, public)]
pub struct SettlementProperty {
    #[primary_key]
    pub id: String,
    #[index(btree)]
    pub settlement_id: String,
    pub building_id: u64,
    pub tier: HousingTier,
    pub resident_capacity: u32,
    pub east_metres: f32,
    pub north_metres: f32,
    pub yaw_radians: f32,
    pub width_metres: f32,
    pub depth_metres: f32,
}

/// Aggregate residents have an occupied home and household identity without
/// requiring individual character or household-member rows.
#[derive(Clone, Debug)]
#[table(accessor = household_property_occupancy)]
pub struct HouseholdPropertyOccupancy {
    #[primary_key]
    pub household_id: String,
    #[index(btree)]
    pub property_id: String,
    pub unmaterialized_residents: u32,
}

#[derive(Clone, Debug, SpacetimeType)]
pub struct AvailableResidenceProperty {
    pub property_id: String,
    pub settlement_id: String,
    pub building_id: u64,
    pub resident_capacity: u32,
    pub available_from_minute: StrategicMinute,
    pub tier: HousingTier,
    pub purchase_price: u32,
    pub rent_per_period: u32,
    pub owner_maintenance_per_period: u32,
    pub property_tax_per_period: u32,
    pub leisure_morale_basis_points: u16,
}

#[view(accessor = backend_available_residence_properties, public)]
pub fn backend_available_residence_properties(
    ctx: &ViewContext,
) -> Vec<AvailableResidenceProperty> {
    if !residence_view_is_gateway(ctx) {
        return Vec::new();
    }
    ctx.db
        .settlement_property()
        .settlement_id()
        .filter(""..)
        .filter(|property| {
            !ctx.db
                .residence_holding()
                .property_id()
                .filter(&property.id)
                .any(|holding| holding.status != ResidenceHoldingStatus::Relinquished)
                && !ctx
                    .db
                    .residence_occupant()
                    .property_id()
                    .filter(&property.id)
                    .any(|_| true)
                && !ctx
                    .db
                    .household_property_occupancy()
                    .property_id()
                    .filter(&property.id)
                    .any(|occupancy| occupancy.unmaterialized_residents > 0)
        })
        .filter_map(|property| {
            let offer = ctx
                .db
                .settlement_residence_offer()
                .id()
                .find(offer_id(&property.settlement_id, property.tier))?;
            let available_from_minute = ctx
                .db
                .residence_holding()
                .property_id()
                .filter(&property.id)
                .filter_map(|holding| holding.resolved_minute)
                .chain(
                    ctx.db
                        .property_occupancy_transition()
                        .property_id()
                        .filter(&property.id)
                        .map(|event| event.minute),
                )
                .max()
                .unwrap_or(StrategicMinute::ZERO);
            Some(AvailableResidenceProperty {
                property_id: property.id,
                settlement_id: property.settlement_id,
                building_id: property.building_id,
                resident_capacity: property.resident_capacity,
                available_from_minute,
                tier: property.tier,
                purchase_price: offer.purchase_price,
                rent_per_period: offer.rent_per_period,
                owner_maintenance_per_period: offer.owner_maintenance_per_period,
                property_tax_per_period: offer.property_tax_per_period,
                leisure_morale_basis_points: offer.leisure_morale_basis_points,
            })
        })
        .collect()
}

/// The trusted generator supplies geometry; this reducer owns registration,
/// initial household allocation and every later legal/occupancy mutation.
#[reducer]
pub fn register_settlement_properties(
    ctx: &ReducerContext,
    catalog_json: String,
) -> Result<(), String> {
    crate::strategic::require_strategic_gateway(ctx)?;
    let catalog = GeneratedHomeCatalog::parse(&catalog_json).map_err(|error| error.to_string())?;
    let settlement = ctx
        .db
        .settlement()
        .id()
        .find(&catalog.settlement_id)
        .ok_or("Settlement not found")?;
    catalog
        .validate(
            &settlement.id,
            adventuresim_core::settlement_property::ResidentCount::new(effective_population(
                settlement.population_level,
                settlement.population_estimate,
            )),
        )
        .map_err(|error| error.to_string())?;
    let digest = catalog.digest().map_err(|error| error.to_string())?;
    if let Some(existing) = ctx
        .db
        .settlement_property_manifest()
        .settlement_id()
        .find(&settlement.id)
    {
        return if existing.digest == digest {
            Ok(())
        } else {
            Err("Registered physical properties differ from the generated catalog".into())
        };
    }
    ensure_settlement_residence_offers(ctx, &settlement.id)?;
    let residents = ctx
        .db
        .settlement_resident_profile()
        .home_settlement_id()
        .filter(&settlement.id)
        .map(|resident| resident.character_id)
        .collect::<std::collections::BTreeSet<_>>();
    let mut families = ctx
        .db
        .household()
        .iter()
        .filter(|household| household.home_settlement_id == settlement.id)
        .map(|household| {
            let count = ctx
                .db
                .household_member()
                .household_id()
                .filter(&household.id)
                .filter(|member| residents.contains(&member.character_id))
                .count() as u32;
            adventuresim_core::settlement_property::HouseholdRequest {
                household_id: household.id,
                residents: adventuresim_core::settlement_property::ResidentCount::new(count),
            }
        })
        .filter(|request| request.residents.get() > 0)
        .collect::<Vec<_>>();
    families.sort_by(|left, right| left.household_id.cmp(&right.household_id));
    let allocation = catalog
        .allocate(&families)
        .map_err(|error| error.to_string())?;
    for home in catalog.homes {
        ctx.db.settlement_property().insert(SettlementProperty {
            id: home.id.as_str().to_owned(),
            settlement_id: settlement.id.clone(),
            building_id: home.building_id,
            tier: home.tier,
            resident_capacity: home.resident_capacity.get(),
            east_metres: home.east_metres,
            north_metres: home.north_metres,
            yaw_radians: home.yaw_radians,
            width_metres: home.width_metres,
            depth_metres: home.depth_metres,
        });
    }
    allocate_population(ctx, allocation, &residents, &settlement.id)?;
    ctx.db
        .settlement_property_manifest()
        .insert(SettlementPropertyManifest {
            settlement_id: settlement.id,
            digest,
        });
    Ok(())
}

pub(super) fn available_property(
    ctx: &ReducerContext,
    property_id: &str,
    minute: StrategicMinute,
) -> Result<SettlementProperty, String> {
    let property = ctx
        .db
        .settlement_property()
        .id()
        .find(property_id.to_owned())
        .ok_or("Generated residence property not found")?;
    if ctx
        .db
        .residence_holding()
        .property_id()
        .filter(&property.id)
        .any(|holding| {
            holding
                .resolved_minute
                .is_none_or(|resolved| resolved > minute)
        })
        || !occupancy::residents_at(ctx, &property.id, minute).is_empty()
        || ctx
            .db
            .residence_occupant()
            .property_id()
            .filter(&property.id)
            .any(|_| true)
        || ctx
            .db
            .household_property_occupancy()
            .property_id()
            .filter(&property.id)
            .any(|occupancy| occupancy.unmaterialized_residents > 0)
    {
        return Err("Residence property is already held or occupied".into());
    }
    Ok(property)
}

pub(super) fn require_property_room_at(
    ctx: &ReducerContext,
    occupant: &ResidenceOccupant,
    minute: StrategicMinute,
) -> Result<(), String> {
    let property = ctx
        .db
        .settlement_property()
        .id()
        .find(occupant.property_id.clone())
        .ok_or("Generated residence property not found")?;
    let end = occupancy::admission_end(
        occupant,
        minute,
        ctx.db
            .property_occupancy_transition()
            .character_id()
            .filter(occupant.character_id)
            .collect(),
    );
    let mut boundaries = std::collections::BTreeSet::from([minute]);
    boundaries.extend(
        ctx.db
            .property_occupancy_transition()
            .property_id()
            .filter(&property.id)
            .filter(|event| event.minute > minute && end.is_none_or(|end| event.minute < end))
            .map(|event| event.minute),
    );
    let aggregate: u64 = ctx
        .db
        .household_property_occupancy()
        .property_id()
        .filter(&property.id)
        .map(|occupancy| u64::from(occupancy.unmaterialized_residents))
        .sum();
    for boundary in boundaries {
        let named = occupancy::residents_at(ctx, &property.id, boundary)
            .into_iter()
            .filter(|resident| *resident != occupant.character_id)
            .count() as u64;
        if named + aggregate >= u64::from(property.resident_capacity) {
            return Err("Residence property has no remaining occupancy capacity".into());
        }
    }
    Ok(())
}

fn allocate_population(
    ctx: &ReducerContext,
    allocation: Vec<adventuresim_core::settlement_property::HouseholdHome>,
    residents: &std::collections::BTreeSet<u64>,
    settlement_id: &str,
) -> Result<(), String> {
    for assignment in allocation {
        let members = ctx
            .db
            .household_member()
            .household_id()
            .filter(&assignment.household_id)
            .filter(|member| residents.contains(&member.character_id))
            .collect::<Vec<_>>();
        if members.is_empty() {
            ctx.db.household().insert(Household {
                id: assignment.household_id.clone(),
                home_settlement_id: settlement_id.to_owned(),
                created_minute: StrategicMinute::ZERO,
            });
            ctx.db
                .household_property_occupancy()
                .insert(HouseholdPropertyOccupancy {
                    household_id: assignment.household_id,
                    property_id: assignment.property_id.as_str().to_owned(),
                    unmaterialized_residents: assignment.residents.get(),
                });
        } else {
            for member in members {
                if ctx
                    .db
                    .residence_occupant()
                    .character_id()
                    .find(member.character_id)
                    .is_some()
                {
                    return Err(
                        "Seeded resident already occupies a home before property registration"
                            .into(),
                    );
                }
                let occupant = ResidenceOccupant {
                    character_id: member.character_id,
                    property_id: assignment.property_id.as_str().to_owned(),
                    holding_id: None,
                    admitted_minute: StrategicMinute::ZERO,
                };
                ctx.db
                    .generated_resident_home()
                    .insert(GeneratedResidentHome {
                        character_id: member.character_id,
                        property_id: occupant.property_id.clone(),
                    });
                occupancy::record(
                    ctx,
                    &occupant,
                    StrategicMinute::ZERO,
                    PropertyOccupancyKind::Admitted,
                );
                occupancy::store_current(ctx, occupant);
            }
        }
    }
    Ok(())
}
