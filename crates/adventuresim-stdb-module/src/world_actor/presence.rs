/// Context interval, occupancy, and observer visibility rules.
fn context_interval_is_well_formed(
    active: bool,
    entered_at: StrategicMinute,
    left_at: Option<StrategicMinute>,
) -> bool {
    active == left_at.is_none() && left_at.is_none_or(|left_at| left_at >= entered_at)
}

pub(crate) fn context_membership_interval_is_well_formed(row: &CharacterContextMembership) -> bool {
    context_interval_is_well_formed(
        row.active,
        row.entered_at,
        row.left_at,
    )
}

pub(crate) fn context_membership_valid_at(
    row: &CharacterContextMembership,
    minute: StrategicMinute,
) -> bool {
    context_membership_interval_is_well_formed(row)
        && row.entered_at <= minute
        && row
            .left_at
            .is_none_or(|left_at| minute < left_at)
}

fn exact_context_claim_matches(
    kind: CharacterContextKind,
    contact_ref: &str,
    expected_revision: u32,
    actual_revision: u32,
) -> bool {
    matches!(
        kind,
        CharacterContextKind::CaseSite | CharacterContextKind::HostileGroup
    ) && contact_ref == EXACT_CASE_CONTEXT_CONTACT_REF
        && expected_revision == actual_revision
}

fn exactly_one<T>(mut values: impl Iterator<Item = T>) -> Option<T> {
    let value = values.next()?;
    values.next().is_none().then_some(value)
}

fn projected_case_context_claim(
    ctx: &ReducerContext,
    observer_character_id: u64,
    membership: &CharacterContextMembership,
    minute: StrategicMinute,
) -> Option<(String, u32)> {
    context_membership_valid_at(membership, minute)
        .then(|| (membership.id.clone(), membership.revision))
        .filter(|_| {
            crate::investigation::exact_case_site_for_observer_at(
                ctx,
                observer_character_id,
                &membership.location_id,
                minute,
            )
            .is_some()
        })
}

fn character_case_site_occupancy_at_view(
    ctx: &ViewContext,
    character_id: u64,
    minute: StrategicMinute,
) -> Option<crate::investigation::CharacterCaseSiteOccupancy> {
    let mut rows = ctx
        .db
        .character_case_site_occupancy()
        .character_id()
        .filter(character_id)
        .filter(|row| {
            row.entered_at <= minute
                && row.left_at.is_none_or(|left| minute < left)
        });
    let row = rows.next()?;
    rows.next().is_none().then_some(row)
}

fn case_context_party_visible_at_view(
    ctx: &ViewContext,
    row: &CharacterContextMembership,
    party: &crate::strategic::Party,
) -> bool {
    ctx.db
        .character_time()
        .character_id()
        .find(party.leader_id)
        .is_some_and(|time| {
            let minute = time.minutes;
            context_membership_valid_at(row, minute)
                && character_case_site_occupancy_at_view(ctx, party.leader_id, minute)
                    .map(|occupancy| occupancy.case_site_id.to_place())
                    .zip(canonical_case_site_place(&row.location_id))
                    .is_some_and(|(party_place, context_place)| party_place == context_place)
                && exact_case_site_visible_to_observer_view(
                    ctx,
                    party.leader_id,
                    &row.location_id,
                    minute,
                )
                && (row.role != CharacterContextRole::Patient
                    || crate::outbreak::case_patient_visible_to_character_view(
                        ctx,
                        party.leader_id,
                        &row.context_id,
                        minute,
                    ))
        })
}

fn actor_case_presence(
    ctx: &ReducerContext,
    actor_id: u64,
) -> Option<(adventuresim_core::strategic_presence::StrategicPresence, StrategicMinute)> {
    let minute = ctx
        .db
        .character_time()
        .character_id()
        .find(actor_id)?
        .minutes;
    case_site_presence_for_observer(ctx, actor_id, actor_id, minute)
        .map(|presence| (presence, minute))
}

pub(crate) fn character_alive_at_for_view(
    ctx: &ViewContext,
    character_id: u64,
    minute: StrategicMinute,
) -> bool {
    ctx.db.character().id().find(character_id).is_some()
        && ctx
            .db
            .character_birth()
            .character_id()
            .find(character_id)
            .is_none_or(|birth| minute.is_at_or_after_signed_birth(birth.birth_minute))
        && ctx
            .db
            .character_death()
            .character_id()
            .find(character_id)
            .is_none_or(|death| death.strategic_minute > minute)
}

fn exact_case_site_visible_to_observer_view(
    ctx: &ViewContext,
    observer_character_id: u64,
    case_site_id: &str,
    minute: StrategicMinute,
) -> bool {
    let Some(place) = canonical_case_site_place(case_site_id) else {
        return false;
    };
    let Some(site) = ctx
        .db
        .case_site_authority()
        .id_key()
        .find(case_site_id.to_owned())
    else {
        return false;
    };
    if site.id.to_place() != place {
        return false;
    }
    let Some(generated_aliases) = case_site_provenance_view(ctx, &site) else {
        return false;
    };
    ctx.db
        .investigation_lead()
        .owner_character_id()
        .filter(observer_character_id)
        .any(|lead| {
            lead.recorded_at <= minute
                && canonical_case_site_place(&lead.exact_location_id).as_ref() == Some(&place)
                && (lead.case_id == site.case_id
                    || generated_aliases
                        .as_ref()
                        .is_some_and(|aliases| lead.case_id == aliases.1.as_str()))
                && lead.latitude_e7 == site.latitude_e7
                && lead.longitude_e7 == site.longitude_e7
                && lead.destination_stage.is_exact()
                && (lead.corrected_by.is_empty()
                    || ctx
                        .db
                        .investigation_lead()
                        .id()
                        .find(&lead.corrected_by)
                        .is_some_and(|correction| {
                            correction.owner_character_id == lead.owner_character_id
                                && correction.recorded_at > minute
                        }))
        })
}
