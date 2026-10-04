//! Durable demonstration administrations for the seeded disease cohort.

use super::*;

pub(super) fn seed(
    ctx: &ReducerContext,
    patient_id: u64,
    fixture_now: StrategicMinute,
    day_minutes: u64,
) -> Result<(), String> {
    let key_version = physiology_key(ctx)?.version;
    for (administered_at, stopped_at, dose) in [
        (
            fixture_now.saturating_sub_minutes(6 * day_minutes),
            Some(fixture_now.saturating_sub_minutes(5 * day_minutes)),
            physiology::DoseMilliunits::try_new(750).unwrap(),
        ),
        (
            fixture_now.saturating_sub_minutes(4 * day_minutes),
            Some(fixture_now.saturating_sub_minutes(3 * day_minutes)),
            physiology::DoseMilliunits::STANDARD,
        ),
        (
            fixture_now.saturating_sub_minutes(2 * day_minutes),
            None,
            physiology::DoseMilliunits::try_new(1_250).unwrap(),
        ),
    ] {
        let (sensitivity_bps, adverse_bps) = private_variation(
            ctx,
            (patient_id).into(),
            administered_at,
            "oral_rehydration_draught",
        )?;
        ctx.db
            .physiology_administration()
            .insert(PhysiologyAdministration {
                id: 0,
                patient_id,
                preparation_id: "oral_rehydration_draught".into(),
                profile_version: 1,
                route: InterventionRoute::Oral,
                dose_milliunits: dose.get(),
                region: None,
                administered_at,
                stopped_at,
                sensitivity_bps,
                adverse_bps,
                ruleset_version: physiology::PHYSIOLOGY_RULESET_VERSION,
                phenotype_key_version: key_version,
            });
    }
    Ok(())
}
