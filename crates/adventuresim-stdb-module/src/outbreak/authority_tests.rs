//! Focused, guarded patient materialization and lifecycle acceptance checks.
use super::*;
use adventuresim_core::quest_generation::{
    GenerationContext, TemplateFamily, generate, test_witnesses,
};
use adventuresim_core::strategic_presence::PresenceSuppression;

fn patient(ctx: &ReducerContext, id: &String) -> Result<OutbreakPatientAuthority, String> {
    ctx.db
        .outbreak_patient_authority()
        .id()
        .find(id)
        .ok_or("Patient missing".into())
}

fn membership(
    ctx: &ReducerContext,
    id: &String,
) -> Result<crate::world_actor::CharacterContextMembership, String> {
    ctx.db
        .character_context_membership()
        .id()
        .find(id)
        .ok_or("Membership missing".into())
}

#[spacetimedb::reducer]
pub fn authority_test_outbreak_patient_lifecycle(
    ctx: &ReducerContext,
    bootstrap_token: String,
) -> Result<(), String> {
    crate::strategic::require_dev_bootstrap_token(&bootstrap_token)?;
    let actor = 732004;
    let now = crate::time::refresh_clock(ctx)
        .map_err(|error: crate::time::WorldClockError| error.to_string())?
        .saturating_add_minutes(1_000);
    let mut generated = generate(&GenerationContext {
        seed: 0,
        observer_entropy_hi: 732,
        observer_entropy_lo: 87,
        settlement_id: adventuresim_core::identity::SettlementId::try_new("riverdale").unwrap(),
        settlement_name: "Riverdale".into(),
        scope: adventuresim_core::local_problem::Scope::Settlement {
            settlement_id: "riverdale".into(),
        },
        ordinal: 0,
        now_minute: now,
        incident_weather: adventuresim_core::weather::Precipitation::Clear,
        requested_family: Some(TemplateFamily::Outbreak),
        witness_candidates: test_witnesses(),
    })
    .map_err(|e| format!("Outbreak fixture generation failed: {e:?}"))?;
    // This fixture exercises the materialization owner with one existing resident,
    // rather than the separate full quest-generation/binding pipeline.
    let outbreak = generated.outbreak.as_mut().ok_or("Outbreak missing")?;
    outbreak.exposure_chronology.truncate(1);
    let exposure = outbreak
        .exposure_chronology
        .first_mut()
        .ok_or("Exposure missing")?;
    exposure.patient_character_id = actor;
    exposure.exposed_at = now;
    exposure.became_symptomatic_at = now;
    exposure.died_at = None;
    exposure.death_kind = None;
    let id = exposure.patient_ref.clone();
    let membership_id = format!("context:{}:patient:{actor}", generated.canonical_case_id);
    let site_id = outbreak.physical_source_site.as_str().to_owned();
    let disease = adventuresim_core::disease::definition(outbreak.disease);
    let recovery = now
        .saturating_add_minutes(disease.incubation_minutes)
        .saturating_add_minutes(disease.rise_minutes)
        .saturating_add_minutes(disease.peak_minutes)
        .saturating_add_minutes(disease.recovery_minutes);
    materialize_generated_outbreak(ctx, &generated, "riverdale", now)?;
    materialize_generated_outbreak(ctx, &generated, "riverdale", now)?;
    let open = membership(ctx, &membership_id)?;
    if !open.is_open() || !patient(ctx, &id)?.health_active {
        return Err("Materialization failed to enroll an ill patient".into());
    }
    let released = now.saturating_add_minutes(1);
    let remediation = remediation_id(&generated)?;
    for _ in 0..2 {
        commit_source_remediation(
            ctx,
            &generated.canonical_case_id,
            "party:authority-outbreak",
            "authority:outbreak-remediation",
            &remediation,
            &site_id,
            released,
        )?;
    }
    // Recovery refresh must leave the pending health checkpoint intact while ill.
    refresh_patient_context_after_time_write(ctx, (actor).into(), released);
    let closed = membership(ctx, &membership_id)?;
    let presence = ctx
        .db
        .settlement_resident_presence()
        .character_id()
        .find(actor)
        .ok_or("Presence missing")?;
    if closed.is_open()
        || closed.left_at != Some(released)
        || closed.revision != open.revision + 1
        || !patient(ctx, &id)?.health_active
        || presence.context_suppressed
        || !presence.health_suppressed
        || patient_presence_suppression_at(ctx, actor, now)
            != Some(PresenceSuppression {
                context_suppressed: true,
                health_suppressed: true,
            })
        || patient_presence_suppression_at(ctx, actor, released)
            != Some(PresenceSuppression {
                context_suppressed: false,
                health_suppressed: true,
            })
    {
        return Err("Context release erased illness or historical presence".into());
    }
    materialize_generated_outbreak(ctx, &generated, "riverdale", released)?;
    for _ in 0..2 {
        refresh_patient_context_after_time_write(ctx, (actor).into(), recovery);
    }
    let presence = ctx
        .db
        .settlement_resident_presence()
        .character_id()
        .find(actor)
        .ok_or("Presence missing")?;
    if patient(ctx, &id)?.health_active
        || membership(ctx, &membership_id)?.revision != closed.revision
        || presence.health_suppressed
        || presence.context_suppressed
        || patient_presence_suppression_at(ctx, actor, recovery)
            != Some(PresenceSuppression {
                context_suppressed: false,
                health_suppressed: false,
            })
        || patient_presence_suppression_at(ctx, actor, now)
            != Some(PresenceSuppression {
                context_suppressed: true,
                health_suppressed: true,
            })
    {
        return Err("Recovery cleanup changed closure history or trusted latest state".into());
    }
    // A second isolated patient exercises death cleanup before the recovery date.
    let dead_actor = 732087;
    crate::character::create_named_character_with_id(
        ctx,
        dead_actor,
        "Patient death fixture".into(),
    )?;
    let mut second = patient(ctx, &id)?;
    second.id = "patient:authority-death".into();
    second.patient_character_id = dead_actor;
    second.episode_id = 732087;
    second.health_active = true;
    ctx.db.outbreak_patient_authority().insert(second);
    let mut episode = ctx
        .db
        .infection_episode()
        .id()
        .find(patient(ctx, &id)?.episode_id)
        .ok_or("Episode missing")?;
    episode.id = 732087;
    episode.character_id = dead_actor;
    ctx.db.infection_episode().insert(episode);
    let dead_membership_id = format!(
        "context:{}:patient:{dead_actor}",
        generated.canonical_case_id
    );
    let mut dead_membership = open;
    dead_membership.id = dead_membership_id.clone();
    dead_membership.character_id = dead_actor;
    ctx.db
        .character_context_membership()
        .insert(dead_membership);
    let mut clock = ctx
        .db
        .character_time()
        .character_id()
        .find(dead_actor)
        .ok_or("Clock missing")?;
    clock.minutes = released;
    ctx.db.character_time().character_id().update(clock);
    crate::character::transition_character_to_dead_at(
        ctx,
        (dead_actor).into(),
        crate::character::DeathCause::DevTest,
        crate::character::DeathSource::DevTest,
        None,
        released,
    )?;
    for _ in 0..2 {
        refresh_patient_context_after_time_write(ctx, (dead_actor).into(), released);
    }
    let dead_membership = membership(ctx, &dead_membership_id)?;
    if patient(ctx, &"patient:authority-death".to_owned())?.health_active
        || dead_membership.is_open()
        || dead_membership.left_at != Some(released)
        || dead_membership.revision != 2
        || patient_presence_suppression_at(ctx, dead_actor, released)
            != Some(PresenceSuppression {
                context_suppressed: false,
                health_suppressed: true,
            })
    {
        return Err("Death cleanup or repeated closure disagrees".into());
    }
    Ok(())
}
