// Guarded acceptance of existence-owned contact and encounter awareness.

fn contact_fixture(ctx: &ReducerContext) -> Result<(), String> {
    use crate::strategic::StrategicEncounter;
    let actor = 732061;
    let target = 732062;
    for id in [actor, target] {
        crate::character::create_named_character_with_id(ctx, id, "Contact Fixture".into())?;
    }
    let party = "authority-contact-party";
    let context = "authority-contact-context";
    let mut character = ctx.db.character().id().find(actor).ok_or("Actor missing")?;
    character.party_id = Some(party.into());
    ctx.db.character().id().update(character);
    ctx.db.strategic_encounter().insert(StrategicEncounter {
        party_id: party.into(),
        encounter_id: context.into(),
        archetype: "fixture".into(),
        enemy_count: 1,
        roll_index: 0,
        journey_movement_minute: 0,
        journey_elapsed_minute: 0,
        absolute_minute: StrategicMinute::ZERO,
        longitude_e7: 0,
        latitude_e7: 0,
        terrain: "road".into(),
        party_aware: false,
        enemy_aware: false,
        available_choices: vec!["fight".into(), "sneak".into()],
        status: StrategicEncounterStatus::AwaitingChoice,
        revision: 1,
        selected_choice: None,
        selection_explanation: String::new(),
        party_speed_m_per_minute: 1,
        enemy_speed_m_per_minute: 1,
        run_ineligibility: None,
        penalty_minutes: 0,
        loss_preview: Vec::new(),
        outcome: None,
    });
    ctx.db
        .character_context_membership()
        .insert(CharacterContextMembership {
            id: "authority-contact-membership".into(),
            context_id: context.into(),
            location_id: context.into(),
            character_id: target,
            context_kind: CharacterContextKind::StrategicEncounter,
            role: CharacterContextRole::Bystander,
            ordinal: 0,
            entered_at: StrategicMinute::ZERO,
            left_at: None,
            revision: 0,
            contact_decision: ContextualDecisionState::Allowed,
            treatment_decision: ContextualDecisionState::Unavailable,
        });
    Ok(())
}

#[reducer]
pub fn authority_test_contact_existence(
    ctx: &ReducerContext,
    bootstrap_token: String,
) -> Result<(), String> {
    crate::strategic::require_dev_bootstrap_token(&bootstrap_token)?;
    contact_fixture(ctx)?;
    let party = "authority-contact-party";
    let context = "authority-contact-context";
    if party_contacted_context(ctx, party, context) {
        return Err("Contact existed before interaction".into());
    }
    let contact = |revision, action: &str| {
        contact_context_character(ctx, 732061, 732062, context.into(), revision, action.into())
    };
    contact(1, "first")?;
    let encounter = ctx
        .db
        .strategic_encounter()
        .party_id()
        .find(party.to_owned())
        .ok_or("Encounter missing")?;
    if !party_contacted_context(ctx, party, context)
        || party_contacted_context(ctx, "other-party", context)
        || !encounter.party_aware
        || !encounter.enemy_aware
        || encounter
            .available_choices
            .iter()
            .any(|choice| choice == "sneak")
        || encounter.revision != 2
    {
        return Err(
            "Contact failed to establish scoped awareness and remove surprise choice".into(),
        );
    }
    contact(1, "first")?;
    if contact(2, "first").err().as_deref() != Some("Conflicting contextual contact retry")
        || contact(1, "stale").err().as_deref() != Some("Context contact revision is stale")
        || ctx
            .db
            .party_context_contact_authority()
            .id()
            .find(party_context_contact_id(party, context))
            .ok_or("Contact missing")?
            .revision
            != 2
        || ctx
            .db
            .contextual_contact_receipt()
            .iter()
            .filter(|receipt| receipt.actor_id == 732061)
            .count()
            != 1
    {
        return Err("Contact retry rewrote authority or accepted conflicting revision".into());
    }
    contact(2, "second")?;
    if ctx
        .db
        .party_context_contact_authority()
        .id()
        .find(party_context_contact_id(party, context))
        .ok_or("Contact missing")?
        .revision
        != 3
    {
        return Err("Repeated interaction failed to update the existing contact revision".into());
    }
    Ok(())
}
