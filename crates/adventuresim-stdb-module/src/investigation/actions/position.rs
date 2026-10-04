//! Admit live contact, cohort, area, site and tracking positions.

use super::*;

fn validate_tracking_action_origin(
    ctx: &ReducerContext,
    actor: &crate::Character,
    capability: &InvestigationActionCapability,
    kind: action::InvestigationActionKind,
) -> Result<(), InvestigationAdmissionError> {
    if !tracking_capability_chain_is_coherent(
        capability,
        kind,
        |id| {
            ctx.db
                .investigation_action_capability()
                .id()
                .find(id.to_owned())
        },
        |id| {
            ctx.db
                .investigation_action_attempt()
                .capability_id()
                .filter(id)
                .any(|attempt| attempt.success)
        },
    ) {
        return Err(InvestigationAdmissionError::InvalidRoute);
    }
    let predecessor = ctx
        .db
        .investigation_action_capability()
        .id()
        .find(&capability.required_action_id)
        .ok_or(InvestigationAdmissionError::InvalidRoute)?;
    validate_action_position(
        ctx,
        actor,
        &predecessor,
        predecessor.method.parse::<InvestigationActionKind>()?,
    )
}

pub(super) fn validate_action_position(
    ctx: &ReducerContext,
    actor: &crate::Character,
    capability: &InvestigationActionCapability,
    kind: action::InvestigationActionKind,
) -> Result<(), InvestigationAdmissionError> {
    match capability.target_kind {
        action::InvestigationTargetKind::Contact => {
            let resident_character_id = capability
                .target_id
                .parse::<u64>()
                .map(adventuresim_core::identity::CharacterId::from)
                .map_err(InvestigationAdmissionError::ContactIdentity)?;
            let presence = ctx
                .db
                .settlement_resident_presence()
                .character_id()
                .find(u64::from(resident_character_id))
                .ok_or(InvestigationAdmissionError::ContactPresenceMissing)?;
            if actor.current_settlement_id.as_deref() != Some(presence.settlement_id.as_str()) {
                return Err(InvestigationAdmissionError::ContactElsewhere);
            }
            if kind == action::InvestigationActionKind::LocateContact {
                let minute = character_strategic_minute(ctx, actor.id);
                if !crate::settlement_population::npc_is_present(ctx, &presence, minute) {
                    return Err(InvestigationAdmissionError::ContactNotPresent);
                }
            }
            Ok(())
        }
        action::InvestigationTargetKind::Cohort => {
            let target = ctx
                .db
                .investigation_pattern_target_authority()
                .cohort_id()
                .find(&capability.target_id)
                .ok_or(InvestigationAdmissionError::CohortAuthorityMissing)?;
            if target.case_id != capability.case_id {
                return Err(InvestigationAdmissionError::CohortCaseMismatch);
            }
            let presence = ctx
                .db
                .settlement_resident_presence()
                .character_id()
                .find(target.resident_character_id)
                .ok_or(InvestigationAdmissionError::CohortTargetUnavailable)?;
            if actor.current_settlement_id.as_deref() != Some(presence.settlement_id.as_str())
                || presence.settlement_id != target.expected_settlement_id
                || presence.location_id != target.expected_location
                || presence.settlement_id != target.expected_settlement_id
            {
                return Err(InvestigationAdmissionError::CohortMoved);
            }
            Ok(())
        }
        action::InvestigationTargetKind::Area => {
            let area = ctx
                .db
                .investigation_area_authority()
                .id()
                .find(&capability.target_id)
                .ok_or(InvestigationAdmissionError::AreaMissing)?;
            let in_origin =
                actor.current_settlement_id.as_deref() == Some(&area.origin_settlement_id);
            let at_case_site = character_case_site_id(ctx, (actor.id).into())
                .and_then(|id| ctx.db.case_site_authority().id_key().find(&id))
                .is_some_and(|site| {
                    site.case_id == area.case_id
                        && coordinate_area_contains_e7(
                            area.center_longitude_e7,
                            area.center_latitude_e7,
                            area.radius_m,
                            area.coordinates_are_geographic,
                            site.longitude_e7,
                            site.latitude_e7,
                            site.coordinates_are_geographic,
                        )
                });
            if !in_origin && !at_case_site {
                return Err(InvestigationAdmissionError::OutsideSearchArea);
            }
            Ok(())
        }
        action::InvestigationTargetKind::Site => {
            if matches!(
                kind,
                action::InvestigationActionKind::FollowTracks
                    | action::InvestigationActionKind::ReacquireTracks
            ) {
                return validate_tracking_action_origin(ctx, actor, capability, kind);
            }
            if character_case_site_id(ctx, (actor.id).into()).as_deref()
                == Some(capability.target_id.as_str())
            {
                return Ok(());
            }
            Err(InvestigationAdmissionError::SiteRequired)
        }
        action::InvestigationTargetKind::Tracks | action::InvestigationTargetKind::Route => {
            validate_tracking_action_origin(ctx, actor, capability, kind)
        }
    }
}
