//! Isolated database acceptance of joint effective-dated occupancy admission.
use super::*;
use crate::residence::occupancy::property_occupancy_transition;

#[reducer]
pub fn authority_test_joint_residence_capacity(
    ctx: &ReducerContext,
    bootstrap_token: String,
) -> Result<(), String> {
    crate::strategic::require_dev_bootstrap_token(&bootstrap_token)?;
    const PROPERTY: &str = "authority-joint-capacity-property";
    const HOLDING: &str = "authority-joint-capacity-holding";
    const FIRST: u64 = 726_001;
    const SECOND: u64 = 726_002;
    const OTHER: u64 = 726_003;
    ctx.db.settlement_property().insert(SettlementProperty {
        id: PROPERTY.into(),
        settlement_id: "authority-joint-capacity".into(),
        building_id: 726_001,
        tier: HousingTier::Cheap,
        resident_capacity: 1,
        east_metres: 0.0,
        north_metres: 0.0,
        yaw_radians: 0.0,
        width_metres: 1.0,
        depth_metres: 1.0,
    });
    let holding = ResidenceHolding {
        id: HOLDING.into(),
        property_id: PROPERTY.into(),
        holder_character_id: FIRST,
        settlement_id: "authority-joint-capacity".into(),
        tier: HousingTier::Cheap,
        tenure: ResidenceTenure::Owner,
        status: ResidenceHoldingStatus::Active,
        acquired_ordinal: 0,
        acquired_minute: StrategicMinute::ZERO,
        last_billed_minute: StrategicMinute::ZERO,
        next_due_minute: StrategicMinute::ZERO,
        resolved_minute: None,
    };
    ctx.db.residence_holding().insert(holding.clone());
    record_transition(
        ctx,
        &holding,
        FIRST,
        StrategicMinute::ZERO,
        ResidenceTransitionKind::Acquired,
    );
    let ceremony = StrategicMinute::new(10);
    let before = ctx.db.property_occupancy_transition().count();
    if admission::preflight_residence_occupants_at(ctx, HOLDING, [FIRST, SECOND], ceremony).is_ok()
    {
        return Err("Two new residents must not share a single remaining space".into());
    }
    if ctx.db.property_occupancy_transition().count() != before {
        return Err("Rejected preflight wrote occupancy history".into());
    }
    let mut property = ctx
        .db
        .settlement_property()
        .id()
        .find(PROPERTY.to_owned())
        .ok_or("Property missing")?;
    property.resident_capacity = 2;
    ctx.db.settlement_property().id().update(property);
    admission::preflight_residence_occupants_at(ctx, HOLDING, [FIRST, SECOND], ceremony)
        .map_err(|error| error.to_string())?;
    let previous = ResidenceOccupant {
        character_id: FIRST,
        property_id: PROPERTY.into(),
        holding_id: Some("authority-prior-holding".into()),
        admitted_minute: StrategicMinute::ZERO,
    };
    occupancy::record(
        ctx,
        &previous,
        StrategicMinute::ZERO,
        PropertyOccupancyKind::Admitted,
    );
    let replacement = ResidenceOccupant {
        holding_id: Some(HOLDING.into()),
        ..previous
    };
    let future = StrategicMinute::new(20);
    occupancy::record(ctx, &replacement, future, PropertyOccupancyKind::Removed);
    occupancy::record(
        ctx,
        &ResidenceOccupant {
            character_id: OTHER,
            property_id: PROPERTY.into(),
            holding_id: None,
            admitted_minute: future,
        },
        future,
        PropertyOccupancyKind::Admitted,
    );
    admission::preflight_residence_occupants_at(ctx, HOLDING, [FIRST, SECOND], ceremony)
        .map_err(|error| format!("Scoped removal projection rejected valid capacity: {error}"))?;
    Ok(())
}
