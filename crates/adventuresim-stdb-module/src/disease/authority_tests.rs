//! Guarded stored disease vocabulary and malformed-row rollback acceptance.
use super::*;

#[spacetimedb::reducer]
pub fn authority_test_disease_keys(
    ctx: &ReducerContext,
    bootstrap_token: String,
) -> Result<(), String> {
    crate::strategic::require_dev_bootstrap_token(&bootstrap_token)?;
    let actor = 732088;
    crate::character::create_named_character_with_id(ctx, actor, "Disease key fixture".into())?;
    for definition in disease::STARTER_DISEASES {
        let stored = ctx.db.infection_episode().insert(InfectionEpisodeRow {
            id: 0,
            character_id: actor,
            disease_id: definition.id.stable_id().into(),
            contracted_at: StrategicMinute::ZERO,
            ruleset_version: physiology::PHYSIOLOGY_RULESET_VERSION,
            phenotype_key_version: physiology::PHENOTYPE_KEY_VERSION,
        });
        let persisted = ctx
            .db
            .infection_episode()
            .id()
            .find(stored.id)
            .ok_or("Episode missing")?;
        let resolved = InfectionEpisode::try_from(&persisted)
            .map_err(|error: EpisodeDecodeError| error.to_string())?;
        if resolved.disease_id != definition.id
            || resolved.id != stored.id
            || resolved.contracted_at != persisted.contracted_at
        {
            return Err("Stored disease identifier changed episode authority".into());
        }
        ctx.db.infection_episode().id().delete(stored.id);
    }
    let settlement_id = "authority-exposure-order".to_owned();
    for (id, start, key) in [
        ("source:later", 20, DiseaseId::Dysentery),
        ("source:earlier", 10, DiseaseId::ShroudFever),
    ] {
        ctx.db.settlement_outbreak().insert(SettlementOutbreak {
            id: id.into(),
            settlement_id: settlement_id.clone(),
            disease_id: key.stable_id().into(),
            start_minute: StrategicMinute::new(start),
            end_minute: StrategicMinute::new(30),
            intensity: 0.4,
        });
    }
    let sources = exposure_sources::SettlementExposureSource::for_settlement(ctx, &settlement_id)?;
    if sources.len() != 2
        || sources[0].id != "source:earlier"
        || sources[0].disease_id != DiseaseId::ShroudFever
        || sources[1].id != "source:later"
        || sources[1].disease_id != DiseaseId::Dysentery
        || sources
            .iter()
            .any(|source| source.scoped || source.end != StrategicMinute::new(30))
    {
        return Err("Settlement exposure ordering or key conversion changed".into());
    }
    let mut damaged = ctx
        .db
        .settlement_outbreak()
        .id()
        .find("source:earlier".to_owned())
        .ok_or("Source missing")?;
    damaged.disease_id = "ShroudFever".into();
    ctx.db.settlement_outbreak().id().update(damaged);
    if exposure_sources::SettlementExposureSource::for_settlement(ctx, &settlement_id).is_ok() {
        return Err("Malformed source vocabulary was accepted".into());
    }
    for id in ["source:earlier", "source:later"] {
        ctx.db.settlement_outbreak().id().delete(id.to_owned());
    }
    Ok(())
}

#[spacetimedb::reducer]
pub fn authority_test_invalid_disease_key(
    ctx: &ReducerContext,
    bootstrap_token: String,
    key: String,
) -> Result<(), String> {
    crate::strategic::require_dev_bootstrap_token(&bootstrap_token)?;
    let stored = ctx.db.infection_episode().insert(InfectionEpisodeRow {
        id: 732888,
        character_id: 732088,
        disease_id: key,
        contracted_at: StrategicMinute::ZERO,
        ruleset_version: physiology::PHYSIOLOGY_RULESET_VERSION,
        phenotype_key_version: physiology::PHENOTYPE_KEY_VERSION,
    });
    // The caller checks that error propagation rolls this insertion back.
    InfectionEpisode::try_from(&stored)
        .map(|_| ())
        .map_err(|error: EpisodeDecodeError| error.to_string())
}
