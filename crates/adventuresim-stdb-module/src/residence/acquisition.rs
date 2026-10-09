//! Exact physical-property admission for legal residence holdings.
use super::*;

pub(super) fn acquire_residence_internal(
    ctx: &ReducerContext,
    character_id: u64,
    property_id: &str,
    tenure: ResidenceTenure,
) -> Result<String, String> {
    let now = residence_now(ctx, character_id)?;
    let property = properties::available_property(ctx, property_id, now)?;
    let settlement_id = property.settlement_id.as_str();
    let tier = property.tier;
    let character = crate::character::require_living_character(ctx, character_id)?;
    if character.current_settlement_id.as_deref() != Some(settlement_id) {
        return Err("You must be in a settlement to acquire a residence there".into());
    }
    let offer = offer(ctx, settlement_id, tier)?;
    let initial_charge = match tenure {
        ResidenceTenure::Renter => u64::from(offer.rent_per_period),
        ResidenceTenure::Owner => u64::from(offer.purchase_price),
    };
    crate::item::consume_personal_currency(ctx, character_id, initial_charge)?;
    if tenure == ResidenceTenure::Renter {
        let active_rentals: Vec<_> = ctx
            .db
            .residence_holding()
            .holder_character_id()
            .filter(character_id)
            .filter(|row| {
                row.tenure == ResidenceTenure::Renter
                    && row.status != ResidenceHoldingStatus::Relinquished
            })
            .map(|row| row.id)
            .collect();
        for active_rental in active_rentals {
            relinquish_holding_at(ctx, character_id, &active_rental, now)?;
        }
    }
    let acquired_ordinal = ctx
        .db
        .residence_holding()
        .holder_character_id()
        .filter(character_id)
        .count() as u64;
    let id = holding_id(character_id, settlement_id, tier, acquired_ordinal);
    let holding = ResidenceHolding {
        property_id: property.id,
        id: id.clone(),
        holder_character_id: character_id,
        settlement_id: settlement_id.to_owned(),
        tier,
        tenure,
        status: ResidenceHoldingStatus::Active,
        acquired_ordinal,
        acquired_minute: now,
        last_billed_minute: now,
        next_due_minute: now.saturating_add_minutes(RESIDENCE_BILLING_PERIOD_MINUTES),
        resolved_minute: None,
    };
    ctx.db.residence_holding().insert(holding.clone());
    record_transition(
        ctx,
        &holding,
        character_id,
        now,
        ResidenceTransitionKind::Acquired,
    );
    designate_holding_at(ctx, character_id, &id, now)?;
    Ok(id)
}

fn acquire_residence(
    ctx: &ReducerContext,
    character_id: u64,
    property_id: &str,
    tenure: ResidenceTenure,
) -> Result<(), String> {
    crate::strategic::require_strategic_character_authority(ctx, character_id)?;
    acquire_residence_internal(ctx, character_id, property_id, tenure).map(|_| ())
}

#[reducer]
pub fn rent_residence(
    ctx: &ReducerContext,
    character_id: u64,
    property_id: String,
) -> Result<(), String> {
    acquire_residence(ctx, character_id, &property_id, ResidenceTenure::Renter)
}

#[reducer]
pub fn buy_residence(
    ctx: &ReducerContext,
    character_id: u64,
    property_id: String,
) -> Result<(), String> {
    acquire_residence(ctx, character_id, &property_id, ResidenceTenure::Owner)
}
