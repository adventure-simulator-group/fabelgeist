/// Blood exposure replay over a bounded absolute interval.
#[expect(
    clippy::too_many_arguments,
    reason = "exposure replay keeps each bounded cursor and authority input explicit"
)]
pub fn blood_exposure_attempts_through(
    ctx: &ReducerContext,
    character_id: u64,
    from: StrategicMinute,
    to: StrategicMinute,
    persist_checkpoint: bool,
    allow_healing: bool,
    plan: Option<&crate::disease::PartyDiseaseIntervalPlan>,
    max_work: u64,
) -> Result<Vec<adventuresim_core::disease::AcquisitionAttempt>, String> {
    if to <= from {
        return Ok(Vec::new());
    }
    let all_deposits = deposits(ctx, character_id)?;
    if !exposure_windows::has_active_compatible_foreign_blood(&all_deposits, from, to) {
        return Ok(Vec::new());
    }
    let routes = predicted_wound_routes(ctx, character_id, allow_healing)?;
    let mut attempts = Vec::new();
    let mut work = 0;
    for disease_id in adventuresim_core::disease::STARTER_DISEASES
        .iter()
        .filter(|definition| {
            definition.supports(adventuresim_core::disease::TransmissionVector::Blood)
        })
        .map(|definition| definition.id)
    {
        let key = format!("{character_id}:{}", disease_id.stable_id());
        let checkpoint = ctx.db.blood_exposure_checkpoint().id().find(&key);
        let start = from.saturating_add_minutes(1).max(
            checkpoint.as_ref().map_or(StrategicMinute::ZERO, |row| row.evaluated_through.saturating_add_minutes(1)),
        );
        let relevant = relevant_blood_deposits(&all_deposits, disease_id);
        let windows = exposure_windows::blood_infectious_windows(
            &relevant,
            disease_id,
            start.saturating_sub_minutes(1),
            to,
        );
        for minute in windows
            .into_iter()
            .flat_map(|(window_start, window_end)| window_start.iter_through(window_end))
        {
            adventuresim_core::disease::add_bounded_work(&mut work, 1, max_work)
                .map_err(str::to_string)?;
            let route = filth::timed_cut_exposure(
                &routes,
                minute.elapsed_since(from),
            );
            let check = plan.map_or_else(
                || crate::disease::party_physiology_check_at(ctx, character_id, minute),
                |plan| {
                    plan.check_at(
                        character_id,
                        minute,
                    )
                },
            );
            // Any infectious deposit still present here survived or preceded
            // explicit washing, so clean handling is not available for this
            // dose. Bandaging/stitching still raises the physical affordance
            // through the predicted cut-route state.
            let affordance = adventuresim_core::disease::blood_caregiving_affordance(false, route);
            let exposure = adventuresim_core::disease::residual_exposure_with_affordance(
                filth::blood_exposure(&relevant, disease_id, minute, route)
                    / MINUTES_PER_DAY as f32,
                adventuresim_core::disease::TransmissionVector::Blood,
                check,
                affordance,
            );
            if exposure <= 0.0 {
                continue;
            }
            let seed = adventuresim_core::disease::outbreak_exposure_seed(
                character_id,
                &format!("blood:{}:{minute}", disease_id.stable_id()),
            );
            attempts.push(adventuresim_core::disease::AcquisitionAttempt::exposure(
                adventuresim_core::disease::InfectionEpisode {
                    id: seed.to_u64(),
                    character_id,
                    disease_id,
                    contracted_at: minute,
                    ruleset_version: adventuresim_core::physiology::PHYSIOLOGY_RULESET_VERSION,
                    phenotype_key_version: adventuresim_core::physiology::PHENOTYPE_KEY_VERSION,
                },
                adventuresim_core::disease::definition(disease_id).base_acquisition,
                exposure,
            ));
        }
        if persist_checkpoint {
            persist_blood_checkpoint(ctx, key, character_id, disease_id, to, checkpoint.is_some());
        }
    }
    Ok(attempts)
}

fn relevant_blood_deposits(
    all_deposits: &[Deposit],
    disease_id: adventuresim_core::disease::DiseaseId,
) -> Vec<Deposit> {
    all_deposits
        .iter()
        .filter(|deposit| {
            deposit.foreign_blood()
                && deposit
                    .diseases
                    .iter()
                    .any(|snapshot| snapshot.disease_id == disease_id)
        })
        .cloned()
        .collect()
}

fn persist_blood_checkpoint(
    ctx: &ReducerContext,
    key: String,
    character_id: u64,
    disease_id: adventuresim_core::disease::DiseaseId,
    to: StrategicMinute,
    existed: bool,
) {
    let row = BloodExposureCheckpoint {
        id: key,
        character_id,
        disease_id: disease_id.stable_id().into(),
        evaluated_through: to,
    };
    if existed {
        ctx.db.blood_exposure_checkpoint().id().update(row);
    } else {
        ctx.db.blood_exposure_checkpoint().insert(row);
    }
}

#[cfg(test)]
mod source_tests {
    #[test]
    fn blood_route_receives_partial_physician_protection_after_physical_controls() {
        let source = crate::production_source(include_str!("blood_attempts.rs"));
        let exposure = source
            .split("pub fn blood_exposure_attempts_through")
            .nth(1)
            .expect("blood exposure source");
        let prevention = exposure.find("residual_exposure").unwrap();
        let physical = exposure.find("filth::blood_exposure").unwrap();
        assert!(prevention < physical);
        assert!(exposure.contains("TransmissionVector::Blood"));
        assert!(exposure.contains("timed_cut_exposure"));
        assert!(exposure.contains("blood_caregiving_affordance(false, route)"));
    }
}
