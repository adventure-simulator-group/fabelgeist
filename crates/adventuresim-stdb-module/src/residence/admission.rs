//! Checked effective-dated residence capacity admission.
use super::occupancy::property_occupancy_transition;
use super::properties::household_property_occupancy;
use super::*;

struct AdmissionInterval {
    occupant: ResidenceOccupant,
    end: Option<StrategicMinute>,
}

/// A residence admission could not be prepared at its effective minute.
#[derive(Debug, thiserror::Error)]
pub(crate) enum ResidenceAdmissionError {
    #[error("Residence holding was not active at the effective minute")]
    HoldingUnavailable,
    #[error("Generated residence property not found")]
    PropertyMissing,
    #[error("Residence property has no remaining occupancy capacity")]
    CapacityExceeded,
}

/// Validate both occupants together before a shared lifecycle event writes.
pub(crate) fn preflight_residence_occupants_at(
    ctx: &ReducerContext,
    holding_id: &str,
    character_ids: [u64; 2],
    minute: StrategicMinute,
) -> Result<(), ResidenceAdmissionError> {
    let holding = ctx
        .db
        .residence_holding()
        .id()
        .find(holding_id.to_owned())
        .filter(|row| holding_active_at(ctx, &row.id, minute))
        .ok_or(ResidenceAdmissionError::HoldingUnavailable)?;
    let occupants = character_ids.map(|character_id| ResidenceOccupant {
        character_id,
        property_id: holding.property_id.clone(),
        holding_id: Some(holding.id.clone()),
        admitted_minute: minute,
    });
    require_property_room_for(ctx, &holding.property_id, &occupants, minute)
}

pub(super) fn require_property_room_for(
    ctx: &ReducerContext,
    property_id: &str,
    occupants: &[ResidenceOccupant],
    minute: StrategicMinute,
) -> Result<(), ResidenceAdmissionError> {
    let admissions: Vec<_> = occupants
        .iter()
        .filter(|occupant| {
            !occupancy::at(ctx, occupant.character_id, minute).is_some_and(|home| {
                home.property_id == occupant.property_id && home.holding_id == occupant.holding_id
            })
        })
        .map(|occupant| AdmissionInterval {
            occupant: occupant.clone(),
            end: occupancy::admission_end(
                occupant,
                minute,
                ctx.db
                    .property_occupancy_transition()
                    .character_id()
                    .filter(occupant.character_id)
                    .collect(),
            ),
        })
        .collect();
    if admissions.is_empty() {
        return Ok(());
    }
    let property = ctx
        .db
        .settlement_property()
        .id()
        .find(property_id.to_owned())
        .ok_or(ResidenceAdmissionError::PropertyMissing)?;
    let mut boundaries = std::collections::BTreeSet::from([minute]);
    boundaries.extend(admissions.iter().filter_map(|admission| admission.end));
    boundaries.extend(
        ctx.db
            .property_occupancy_transition()
            .property_id()
            .filter(&property.id)
            .filter(|event| event.minute > minute)
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
        if !admissions
            .iter()
            .any(|admission| admission.end.is_none_or(|end| boundary < end))
        {
            continue;
        }
        let mut residents: std::collections::BTreeSet<_> =
            occupancy::residents_at(ctx, &property.id, boundary)
                .into_iter()
                .collect();
        for admission in &admissions {
            residents.remove(&admission.occupant.character_id);
            if occupancy::at_after_admission(ctx, &admission.occupant, minute, boundary)
                .is_some_and(|home| home.property_id == property.id)
            {
                residents.insert(admission.occupant.character_id);
            }
        }
        if residents.len() as u64 + aggregate > u64::from(property.resident_capacity) {
            return Err(ResidenceAdmissionError::CapacityExceeded);
        }
    }
    Ok(())
}
