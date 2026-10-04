//! Contact referral identity, presence and observer availability.
use super::*;

pub(super) fn public_contact_schedule_wait_minutes(
    presence: &crate::SettlementResidentPresence,
    minute: StrategicMinute,
) -> Option<u32> {
    if crate::settlement_population::npc_presence_remaining_minutes(presence, minute).is_some() {
        return Some(0);
    }
    if presence.context_suppressed || presence.health_suppressed {
        return None;
    }
    adventuresim_core::strategic_presence::DailyPresenceWindow {
        start_minute: presence.start_minute,
        end_minute: presence.end_minute,
    }
    .minutes_until_start(minute)
    .ok()
}

pub(super) fn projected_contact_schedule_wait_minutes(
    ctx: &ViewContext,
    presence: &crate::SettlementResidentPresence,
    minute: StrategicMinute,
) -> Option<u32> {
    if crate::settlement_population::npc_presence_remaining_minutes_at_view(ctx, presence, minute)
        .is_some()
    {
        return Some(0);
    }
    let suppression =
        crate::outbreak::patient_presence_suppression_at_view(ctx, presence.character_id, minute)?;
    if suppression.context_suppressed || suppression.health_suppressed {
        return None;
    }
    let mut historical_presence = presence.clone();
    historical_presence.context_suppressed = false;
    historical_presence.health_suppressed = false;
    public_contact_schedule_wait_minutes(&historical_presence, minute)
}

pub(super) fn referred_contact_target_matches(
    expected: &adventuresim_core::quest_generation::WitnessCandidate,
    current: &adventuresim_core::quest_generation::WitnessCandidate,
    settlement_id: &str,
    expected_settlement_id: &str,
) -> bool {
    let Ok(expected_settlement_id) =
        adventuresim_core::identity::SettlementId::try_new(expected_settlement_id)
    else {
        return false;
    };
    adventuresim_core::quest_generation::pattern_target_matches(
        &adventuresim_core::quest_generation::GeneratedPatternTarget {
            cohort_id: "referred-contact".into(),
            resident_character_id: expected.resident_character_id,
            demographic: expected.demographic,
            age_band: expected.age_band.clone(),
            sex: expected.sex,
            profession: expected.profession.clone(),
            expected_settlement_id,
            expected_location: expected.expected_location.clone(),
            expected_location_label: expected.expected_location_label.clone(),
            presence_version: expected.presence_version,
        },
        current,
        settlement_id,
    )
}

pub(super) fn referred_contact_is_current_view(
    ctx: &ViewContext,
    capability: &InvestigationActionCapability,
    presence: &crate::SettlementResidentPresence,
) -> bool {
    let Ok(resident_character_id) = capability.target_id.parse::<u64>() else {
        return false;
    };
    let Some((_, context_json)) = generated_authority_view(ctx, capability).ok().flatten() else {
        return false;
    };
    let Ok(context) = serde_json::from_str::<adventuresim_core::quest_generation::GenerationContext>(
        &context_json,
    ) else {
        return false;
    };
    let Some(expected) = context
        .witness_candidates
        .iter()
        .find(|candidate| candidate.resident_character_id == resident_character_id)
    else {
        return false;
    };
    let Some(npc) =
        crate::settlement_population::resolve_settlement_resident_view(ctx, resident_character_id)
    else {
        return false;
    };
    let Some(current) = (if expected.sex.is_none() {
        crate::strategic::developer_npc_witness_candidate(&npc, presence)
    } else {
        Some(adventuresim_core::quest_generation::WitnessCandidate {
            resident_character_id: npc.character_id,
            display_name: npc.name.clone(),
            demographic: crate::strategic::generated_npc_demographic(&npc),
            age_band: npc.age_band.stable_id().to_owned(),
            sex: Some(npc.sex),
            profession: npc.profession.clone(),
            visible_description: String::new(),
            expected_location: presence.location_id.clone(),
            expected_location_label: presence.location_id.clone(),
            presence_version: crate::strategic::generated_npc_presence_version(&npc, presence),
            allowed_circumstances: Default::default(),
        })
    }) else {
        return false;
    };
    referred_contact_target_matches(
        expected,
        &current,
        &presence.settlement_id,
        context.settlement_id.as_str(),
    )
}

pub(super) fn projected_contact_presence_availability(
    ctx: &ViewContext,
    capability: &InvestigationActionCapability,
    kind: action::InvestigationActionKind,
    settlement_id: Option<&str>,
    started_at: Option<StrategicMinute>,
) -> Option<ProjectedActionAvailability> {
    if kind != action::InvestigationActionKind::LocateContact
        || capability.target_kind != action::InvestigationTargetKind::Contact
    {
        return None;
    }
    let presence = capability
        .target_id
        .parse::<u64>()
        .ok()
        .and_then(|character_id| {
            ctx.db
                .settlement_resident_presence()
                .character_id()
                .find(character_id)
        });
    let Some(presence) = presence else {
        return Some(projected_target_changed_availability());
    };
    if settlement_id != Some(presence.settlement_id.as_str())
        || !referred_contact_is_current_view(ctx, capability, &presence)
    {
        return Some(projected_target_changed_availability());
    }
    match started_at
        .and_then(|minute| projected_contact_schedule_wait_minutes(ctx, &presence, minute))
    {
        Some(0) => None,
        Some(wait_minutes) => Some(ProjectedActionAvailability::unavailable(
            action::InvestigationActionUnavailableReason::ContactScheduleWindow,
            false,
            wait_minutes,
            "Wait until the referred contact's public schedule resumes.",
        )),
        None => Some(ProjectedActionAvailability::unavailable(
            action::InvestigationActionUnavailableReason::ContactNotPresent,
            false,
            0,
            "The referred contact is not currently available.",
        )),
    }
}
