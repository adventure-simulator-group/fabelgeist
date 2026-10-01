//! Transfer census counts into named occupancy when the normal NPC owner
//! materializes a household. Registration itself never creates characters.
use super::*;

/// One-time census provenance; later moves do not cause a second allocation.
#[derive(Clone, Debug)]
#[table(accessor = generated_resident_home)]
pub struct GeneratedResidentHome {
    #[primary_key]
    pub character_id: u64,
    pub property_id: String,
}

pub(crate) fn bind_generated_households(
    ctx: &ReducerContext,
    settlement_id: &str,
) -> Result<(), String> {
    if ctx
        .db
        .settlement_property_manifest()
        .settlement_id()
        .find(settlement_id.to_owned())
        .is_none()
    {
        return Ok(());
    }
    let mut groups = std::collections::BTreeMap::<String, Vec<u64>>::new();
    for resident in ctx
        .db
        .settlement_resident_profile()
        .home_settlement_id()
        .filter(settlement_id)
    {
        if ctx
            .db
            .generated_resident_home()
            .character_id()
            .find(resident.character_id)
            .is_some()
        {
            continue;
        }
        let member = ctx
            .db
            .household_member()
            .character_id()
            .find(resident.character_id)
            .ok_or("Materialized resident has no authoritative household")?;
        groups
            .entry(member.household_id)
            .or_default()
            .push(resident.character_id);
    }
    for (_, members) in groups {
        let required = u32::try_from(members.len()).map_err(|error| error.to_string())?;
        let mut available = ctx
            .db
            .household_property_occupancy()
            .iter()
            .filter(|home| {
                home.unmaterialized_residents >= required
                    && ctx
                        .db
                        .settlement_property()
                        .id()
                        .find(&home.property_id)
                        .is_some_and(|property| property.settlement_id == settlement_id)
            })
            .collect::<Vec<_>>();
        available.sort_by(|left, right| left.property_id.cmp(&right.property_id));
        let mut home = available
            .into_iter()
            .next()
            .ok_or("No census home can house the materialized family")?;
        home.unmaterialized_residents -= required;
        ctx.db
            .household_property_occupancy()
            .household_id()
            .update(home.clone());
        for character_id in members {
            ctx.db
                .generated_resident_home()
                .insert(GeneratedResidentHome {
                    character_id,
                    property_id: home.property_id.clone(),
                });
            if ctx
                .db
                .residence_occupant()
                .character_id()
                .find(character_id)
                .is_none()
            {
                let occupant = ResidenceOccupant {
                    character_id,
                    property_id: home.property_id.clone(),
                    holding_id: None,
                    admitted_minute: StrategicMinute::ZERO,
                };
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
